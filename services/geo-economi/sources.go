package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

type Source interface {
	Name() string
	Fetch(context.Context) ([]Signal, error)
}

type HTTPSource struct {
	Client    *http.Client
	UserAgent string
}

func (s HTTPSource) open(ctx context.Context, method, endpoint string, body io.Reader) (*http.Response, error) {
	client := s.Client
	if client == nil {
		client = &http.Client{Timeout: 15 * time.Second}
	}
	req, err := http.NewRequestWithContext(ctx, method, endpoint, body)
	if err != nil {
		return nil, err
	}
	if s.UserAgent == "" {
		req.Header.Set("User-Agent", "geosignals-mvp/1.0")
	} else {
		req.Header.Set("User-Agent", s.UserAgent)
	}
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode < http.StatusOK || resp.StatusCode >= http.StatusMultipleChoices {
		resp.Body.Close()
		return nil, fmt.Errorf("%s returned HTTP %s", endpoint, resp.Status)
	}
	return resp, nil
}

func (s HTTPSource) do(ctx context.Context, method, endpoint string, body io.Reader) ([]byte, error) {
	resp, err := s.open(ctx, method, endpoint, body)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	return io.ReadAll(io.LimitReader(resp.Body, 10<<20))
}

type GDELTSource struct {
	HTTPSource
	Endpoint string
}

func (s GDELTSource) Name() string { return "gdelt" }

func (s GDELTSource) Fetch(ctx context.Context) ([]Signal, error) {
	endpoint := s.Endpoint
	if endpoint == "" {
		endpoint = "https://api.gdeltproject.org/api/v2/doc/doc?query=conflict%20OR%20sanctions%20OR%20maritime%20OR%20diplomatic%20OR%20trade&mode=ArtList&format=json&maxrecords=75"
	}
	payload, err := s.do(ctx, http.MethodGet, endpoint, nil)
	if err != nil {
		return nil, err
	}
	var response struct {
		Articles []struct {
			URL           string `json:"url"`
			Title         string `json:"title"`
			SeenDate      string `json:"seendate"`
			SourceCountry string `json:"sourcecountry"`
		} `json:"articles"`
	}
	if err := json.Unmarshal(payload, &response); err != nil {
		return nil, fmt.Errorf("decode GDELT response: %w", err)
	}
	result := make([]Signal, 0, len(response.Articles))
	for _, article := range response.Articles {
		if strings.TrimSpace(article.URL) == "" || strings.TrimSpace(article.Title) == "" {
			continue
		}
		timestamp := parseGDELTTime(article.SeenDate)
		category, severity := classifyText(article.Title)
		country := strings.TrimSpace(article.SourceCountry)
		result = append(result, Signal{
			Source:     s.Name(),
			ExternalID: hashID(article.URL),
			Key:        firstNonEmpty(country, "Global"),
			Region:     regionForCountry(country),
			Category:   category,
			Scope:      firstNonEmpty(scopeForCountry(country), "global"),
			Severity:   severity,
			Confidence: 0.75,
			Timestamp:  timestamp,
			SourceURL:  article.URL,
		})
	}
	return result, nil
}

type ReliefWebSource struct {
	HTTPSource
	Endpoint string
	AppName  string
}

func (s ReliefWebSource) Name() string { return "reliefweb" }

func (s ReliefWebSource) Fetch(ctx context.Context) ([]Signal, error) {
	endpoint := s.Endpoint
	if endpoint == "" {
		endpoint = "https://api.reliefweb.int/v1/reports"
	}
	appName := s.AppName
	if appName == "" {
		appName = "geosignals-mvp"
	}
	endpoint += "?appname=" + url.QueryEscape(appName)
	requestBody := strings.NewReader(`{"limit":50,"fields":{"include":["title","url","date.created","source.name","country.name"]}}`)
	payload, err := s.do(ctx, http.MethodPost, endpoint, requestBody)
	if err != nil {
		return nil, err
	}
	var response struct {
		Data []struct {
			ID     string `json:"id"`
			Fields struct {
				Title string `json:"title"`
				URL   string `json:"url"`
				Date  struct {
					Created string `json:"created"`
				} `json:"date"`
				Source struct {
					Name string `json:"name"`
				} `json:"source"`
				Country []struct {
					Name string `json:"name"`
				} `json:"country"`
			} `json:"fields"`
		} `json:"data"`
	}
	if err := json.Unmarshal(payload, &response); err != nil {
		return nil, fmt.Errorf("decode ReliefWeb response: %w", err)
	}
	result := make([]Signal, 0, len(response.Data))
	for _, item := range response.Data {
		country := ""
		if len(item.Fields.Country) > 0 {
			country = item.Fields.Country[0].Name
		}
		category, severity := classifyText(item.Fields.Title)
		result = append(result, Signal{
			Source:     s.Name(),
			ExternalID: firstNonEmpty(item.ID, hashID(item.Fields.URL+item.Fields.Title)),
			Key:        firstNonEmpty(country, "Global"),
			Region:     regionForCountry(country),
			Category:   category,
			Scope:      firstNonEmpty(scopeForCountry(country), "global"),
			Severity:   severity,
			Confidence: 0.85,
			Timestamp:  parseTime(item.Fields.Date.Created),
			SourceURL:  item.Fields.URL,
		})
	}
	return result, nil
}

type SanctionsSource struct {
	HTTPSource
	NameValue string
	Endpoint  string
}

func (s SanctionsSource) Name() string { return firstNonEmpty(s.NameValue, "sanctions") }

func (s SanctionsSource) Fetch(ctx context.Context) ([]Signal, error) {
	if s.Endpoint == "" {
		return nil, fmt.Errorf("sanctions endpoint is not configured")
	}
	payload, err := s.do(ctx, http.MethodGet, s.Endpoint, nil)
	if err != nil {
		return nil, err
	}
	decoder := xml.NewDecoder(strings.NewReader(string(payload)))
	var names []string
	for {
		token, err := decoder.Token()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, fmt.Errorf("decode sanctions XML: %w", err)
		}
		start, ok := token.(xml.StartElement)
		if !ok {
			continue
		}
		if strings.EqualFold(start.Name.Local, "firstName") || strings.EqualFold(start.Name.Local, "lastName") || strings.EqualFold(start.Name.Local, "wholeName") {
			var value string
			if err := decoder.DecodeElement(&value, &start); err == nil && strings.TrimSpace(value) != "" {
				names = append(names, strings.TrimSpace(value))
			}
		}
	}
	now := time.Now().UTC()
	result := make([]Signal, 0, len(names))
	for _, name := range names {
		result = append(result, Signal{Source: s.Name(), ExternalID: hashID(name), Key: name, Category: CategorySanctions, Scope: "country", Severity: 0.65, Confidence: 0.95, Timestamp: now, SourceURL: s.Endpoint})
	}
	return result, nil
}

func parseGDELTTime(value string) time.Time {
	for _, layout := range []string{"20060102T150405Z", "20060102150405"} {
		if parsed, err := time.Parse(layout, strings.TrimSpace(value)); err == nil {
			return parsed.UTC()
		}
	}
	return time.Now().UTC()
}

func parseTime(value string) time.Time {
	parsed, err := time.Parse(time.RFC3339, strings.TrimSpace(value))
	if err != nil {
		return time.Now().UTC()
	}
	return parsed.UTC()
}

func classifyText(text string) (string, float64) {
	lower := strings.ToLower(text)
	switch {
	case strings.Contains(lower, "sanction"):
		return CategorySanctions, 0.65
	case strings.Contains(lower, "attack") || strings.Contains(lower, "war") || strings.Contains(lower, "conflict") || strings.Contains(lower, "missile") || strings.Contains(lower, "strike"):
		return CategoryConflict, 0.85
	case strings.Contains(lower, "maritime") || strings.Contains(lower, "vessel") || strings.Contains(lower, "shipping") || strings.Contains(lower, "red sea"):
		return CategoryMaritime, 0.75
	case strings.Contains(lower, "diplomatic") || strings.Contains(lower, "ambassador") || strings.Contains(lower, "ceasefire"):
		return CategoryDiplomatic, 0.55
	case strings.Contains(lower, "trade") || strings.Contains(lower, "tariff") || strings.Contains(lower, "export"):
		return CategoryTrade, 0.45
	default:
		return CategoryConflict, 0.35
	}
}

func hashID(value string) string {
	digest := sha256.Sum256([]byte(value))
	return hex.EncodeToString(digest[:16])
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return strings.TrimSpace(value)
		}
	}
	return ""
}

func scopeForCountry(country string) string {
	if strings.TrimSpace(country) == "" {
		return ""
	}
	return "country"
}

func regionForCountry(country string) string {
	lower := strings.ToLower(strings.TrimSpace(country))
	switch {
	case strings.Contains(lower, "yemen") || strings.Contains(lower, "saudi") || strings.Contains(lower, "iran") || strings.Contains(lower, "iraq") || strings.Contains(lower, "israel") || strings.Contains(lower, "syria"):
		return "Middle East"
	case strings.Contains(lower, "ukraine") || strings.Contains(lower, "russia") || strings.Contains(lower, "germany") || strings.Contains(lower, "france") || strings.Contains(lower, "united kingdom"):
		return "Europe"
	case strings.Contains(lower, "nigeria") || strings.Contains(lower, "somalia") || strings.Contains(lower, "sudan") || strings.Contains(lower, "ethiopia"):
		return "Africa"
	case strings.Contains(lower, "china") || strings.Contains(lower, "taiwan") || strings.Contains(lower, "japan") || strings.Contains(lower, "india"):
		return "Asia-Pacific"
	case strings.Contains(lower, "united states") || strings.Contains(lower, "canada") || strings.Contains(lower, "mexico"):
		return "North America"
	default:
		return ""
	}
}
