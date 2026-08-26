package scraper

import (
	"bytes"
	"context"
	"crypto/tls"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"regexp"
	"strings"
	"time"

	"macro-feed/internal/model"

	"github.com/PuerkitoBio/goquery"
)

var (
	nextStateRegex = regexp.MustCompile(`self\.__next_f\.push\(\[1,"(.*?)"\]\)`)
)

type NewsScraper struct {
	httpClient *http.Client
}

func NewNewsScraper(timeout time.Duration) *NewsScraper {
	if timeout <= 0 {
		timeout = 20 * time.Second
	}
	tr := &http.Transport{
		TLSClientConfig: &tls.Config{InsecureSkipVerify: true},
	}
	return &NewsScraper{
		httpClient: &http.Client{
			Timeout:   timeout,
			Transport: tr,
		},
	}
}

func (s *NewsScraper) FetchArticle(ctx context.Context, articleID, targetURL string) (model.ExtractedArticle, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, targetURL, nil)
	if err != nil {
		return model.ExtractedArticle{}, err
	}

	req.Header.Set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
	req.Header.Set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
	req.Header.Set("Accept-Language", "en-US,en;q=0.9,id;q=0.8")

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return model.ExtractedArticle{}, fmt.Errorf("fetch article failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return model.ExtractedArticle{}, fmt.Errorf("http status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(io.LimitReader(resp.Body, 10<<20)) // 10MB limit
	if err != nil {
		return model.ExtractedArticle{}, err
	}

	return s.ParseHTML(articleID, targetURL, body)
}

func (s *NewsScraper) ParseHTML(articleID, targetURL string, body []byte) (model.ExtractedArticle, error) {
	doc, err := goquery.NewDocumentFromReader(bytes.NewReader(body))
	if err != nil {
		return model.ExtractedArticle{}, err
	}

	article := model.ExtractedArticle{
		ID:        articleID,
		URL:       targetURL,
		FetchedAt: time.Now().UTC(),
	}

	baseURL, _ := url.Parse(targetURL)

	// 1. OpenGraph Meta Tags (Primary)
	if article.MediaURL == "" {
		rawImg := getMetaContent(doc, "og:image", "twitter:image", "twitter:image:src")
		if !isAvatarURL(rawImg) {
			article.MediaURL = resolveURL(baseURL, rawImg)
		}
	}

	// 2. JSON-LD Extraction
	doc.Find("script[type='application/ld+json']").Each(func(_ int, sel *goquery.Selection) {
		text := strings.TrimSpace(sel.Text())
		if text == "" {
			return
		}
		var raw interface{}
		if err := json.Unmarshal([]byte(text), &raw); err == nil {
			s.extractFromJSONLD(raw, &article, baseURL)
		}
	})

	// 3. FXStreet Next.js state extraction fallback
	if strings.Contains(targetURL, "fxstreet.com") {
		s.extractNextJSState(string(body), &article)
	}

	// 4. DOM Body Image Extractor (catches editorial.fxsstatic.com or first content img)
	if article.MediaURL == "" {
		article.MediaURL = s.extractArticleDOMImage(doc, baseURL)
	}

	if article.PublishedTime == "" {
		article.PublishedTime = getMetaContent(doc, "article:published_time", "og:updated_time")
	}

	// 5. DOM Paragraph Body Extractor
	if article.Content == "" {
		article.Content = s.extractParagraphContent(doc)
	}

	if article.Title == "" {
		article.Title = getMetaContent(doc, "og:title", "twitter:title", "title")
		if article.Title == "" {
			article.Title = strings.TrimSpace(doc.Find("title").First().Text())
		}
	}

	return article, nil
}

func (s *NewsScraper) extractFromJSONLD(data interface{}, article *model.ExtractedArticle, baseURL *url.URL) {
	switch v := data.(type) {
	case map[string]interface{}:
		if headline, ok := v["headline"].(string); ok && article.Title == "" {
			article.Title = headline
		}
		if name, ok := v["name"].(string); ok && article.Title == "" {
			article.Title = name
		}
		if datePub, ok := v["datePublished"].(string); ok && article.PublishedTime == "" {
			article.PublishedTime = datePub
		}
		if body, ok := v["articleBody"].(string); ok && article.Content == "" {
			article.Content = body
		}
		if img, ok := v["image"]; ok && article.MediaURL == "" {
			imgURL := parseImageFromJSON(img)
			if !isAvatarURL(imgURL) {
				article.MediaURL = resolveURL(baseURL, imgURL)
			}
		}
		if author, ok := v["author"]; ok && article.Author == "" {
			article.Author = parseAuthorFromJSON(author)
		}

		for _, val := range v {
			s.extractFromJSONLD(val, article, baseURL)
		}
	case []interface{}:
		for _, elem := range v {
			s.extractFromJSONLD(elem, article, baseURL)
		}
	}
}

func (s *NewsScraper) extractNextJSState(html string, article *model.ExtractedArticle) {
	matches := nextStateRegex.FindAllStringSubmatch(html, -1)
	for _, match := range matches {
		if len(match) < 2 {
			continue
		}
		unescaped := strings.ReplaceAll(match[1], `\"`, `"`)
		unescaped = strings.ReplaceAll(unescaped, `\n`, "\n")
		unescaped = strings.ReplaceAll(unescaped, `\\`, `\`)

		var data interface{}
		if err := json.Unmarshal([]byte(unescaped), &data); err == nil {
			s.searchJSONKeys(data, article)
		}
	}
}

func (s *NewsScraper) searchJSONKeys(data interface{}, article *model.ExtractedArticle) {
	switch v := data.(type) {
	case map[string]interface{}:
		for k, val := range v {
			strVal, isStr := val.(string)
			if isStr {
				switch k {
				case "headline", "title":
					if article.Title == "" && len(strVal) > 5 {
						article.Title = strVal
					}
				case "articleBody", "text", "body":
					if article.Content == "" && len(strVal) > 50 {
						article.Content = strVal
					}
				case "imageUrl", "mediaUrl", "image", "src":
					if article.MediaURL == "" && strings.Contains(strVal, "fxsstatic.com") && !isAvatarURL(strVal) {
						article.MediaURL = strVal
					}
				}
			}
			s.searchJSONKeys(val, article)
		}
	case []interface{}:
		for _, elem := range v {
			s.searchJSONKeys(elem, article)
		}
	}
}

func (s *NewsScraper) extractArticleDOMImage(doc *goquery.Document, baseURL *url.URL) string {
	selectors := []string{
		"img[src*='editorial.fxsstatic.com']",
		"article img",
		".article-body img",
		".entry-content img",
		".wysiwygBlock img",
		"#article img",
		"main img",
		"img[src*='wp-content']",
	}

	for _, sel := range selectors {
		var imgURL string
		doc.Find(sel).Each(func(_ int, sel *goquery.Selection) {
			if imgURL != "" {
				return
			}
			for _, attr := range []string{"src", "data-src", "data-original"} {
				if val, exists := sel.Attr(attr); exists && strings.TrimSpace(val) != "" {
					val = strings.TrimSpace(val)
					if !isAvatarURL(val) && !strings.Contains(val, "logo") && !strings.Contains(val, "icon") {
						imgURL = resolveURL(baseURL, val)
						return
					}
				}
			}
		})
		if imgURL != "" {
			return imgURL
		}
	}
	return ""
}

func (s *NewsScraper) extractParagraphContent(doc *goquery.Document) string {
	doc.Find("script, style, nav, header, footer, iframe, noscript, svg, .advertisement, .ad-container").Remove()

	var paragraphs []string
	selectors := []string{
		"article p",
		".article-body p",
		".articlePage p",
		".entry-content p",
		".wysiwygBlock p",
		"#article p",
		".main-content p",
		"main p",
	}

	for _, sel := range selectors {
		doc.Find(sel).Each(func(_ int, s *goquery.Selection) {
			txt := strings.TrimSpace(s.Text())
			if len(txt) > 30 {
				paragraphs = append(paragraphs, txt)
			}
		})
		if len(paragraphs) > 0 {
			break
		}
	}

	if len(paragraphs) == 0 {
		doc.Find("p").Each(func(_ int, s *goquery.Selection) {
			txt := strings.TrimSpace(s.Text())
			if len(txt) > 30 {
				paragraphs = append(paragraphs, txt)
			}
		})
	}

	return strings.Join(paragraphs, "\n\n")
}

func getMetaContent(doc *goquery.Document, selectors ...string) string {
	for _, sel := range selectors {
		var content string
		doc.Find(fmt.Sprintf("meta[property='%s'], meta[name='%s']", sel, sel)).Each(func(_ int, s *goquery.Selection) {
			if val, exists := s.Attr("content"); exists && strings.TrimSpace(val) != "" {
				content = strings.TrimSpace(val)
			}
		})
		if content != "" {
			return content
		}
	}
	return ""
}

func resolveURL(base *url.URL, raw string) string {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ""
	}
	u, err := url.Parse(raw)
	if err != nil {
		return raw
	}
	if base == nil {
		return raw
	}
	return base.ResolveReference(u).String()
}

func parseImageFromJSON(data interface{}) string {
	switch v := data.(type) {
	case string:
		if !isAvatarURL(v) {
			return v
		}
	case map[string]interface{}:
		if u, ok := v["url"].(string); ok && !isAvatarURL(u) {
			return u
		}
	case []interface{}:
		if len(v) > 0 {
			return parseImageFromJSON(v[0])
		}
	}
	return ""
}

func parseAuthorFromJSON(data interface{}) string {
	switch v := data.(type) {
	case string:
		return v
	case map[string]interface{}:
		if name, ok := v["name"].(string); ok {
			return name
		}
	case []interface{}:
		if len(v) > 0 {
			return parseAuthorFromJSON(v[0])
		}
	}
	return ""
}

func isAvatarURL(raw string) bool {
	lower := strings.ToLower(raw)
	return strings.Contains(lower, "gravatar.com") || strings.Contains(lower, "/avatar") || strings.Contains(lower, "avatar/")
}
