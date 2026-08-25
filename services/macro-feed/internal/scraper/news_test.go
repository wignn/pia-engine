package scraper_test

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"

	"macro-feed/internal/scraper"
)

func TestParseHTML_HTMLFixture(t *testing.T) {
	s := scraper.NewNewsScraper(5 * time.Second)
	sampleHTML := `
<!DOCTYPE html>
<html>
<head>
    <title>Test Gold Price Surges Past $2,400</title>
    <meta property="og:title" content="Gold Price Surges Past $2,400" />
    <meta property="og:image" content="https://example.com/images/gold.jpg" />
    <meta property="article:published_time" content="2026-08-25T10:00:00Z" />
    <script type="application/ld+json">
    {
        "@context": "https://schema.org",
        "@type": "NewsArticle",
        "headline": "Gold Price Surges Past $2,400",
        "author": {
            "@type": "Person",
            "name": "Jane Doe"
        },
        "datePublished": "2026-08-25T10:00:00Z",
        "articleBody": "Gold prices reached a new record high today driven by safe haven demand and lower real yields. Analysts expect further upside momentum as Federal Reserve monetary policy remains supportive."
    }
    </script>
</head>
<body>
    <article>
        <p>Gold prices reached a new record high today driven by safe haven demand and lower real yields.</p>
        <p>Analysts expect further upside momentum as Federal Reserve monetary policy remains supportive.</p>
    </article>
</body>
</html>`

	article, err := s.ParseHTML("hash-123", "https://example.com/news/gold-surge", []byte(sampleHTML))
	if err != nil {
		t.Fatalf("unexpected error parsing HTML: %v", err)
	}

	if article.Title != "Gold Price Surges Past $2,400" {
		t.Errorf("expected title 'Gold Price Surges Past $2,400', got '%s'", article.Title)
	}

	if article.Author != "Jane Doe" {
		t.Errorf("expected author 'Jane Doe', got '%s'", article.Author)
	}

	if article.MediaURL != "https://example.com/images/gold.jpg" {
		t.Errorf("expected media_url 'https://example.com/images/gold.jpg', got '%s'", article.MediaURL)
	}

	if !strings.Contains(article.Content, "safe haven demand") {
		t.Errorf("expected content to contain 'safe haven demand', got '%s'", article.Content)
	}

	t.Logf("\n--- FIXTURE SCRAPING OUTPUT ---\nID: %s\nTitle: %s\nAuthor: %s\nMediaURL: %s\nPublished: %s\nContent Length: %d chars\n--------------------------------\n",
		article.ID, article.Title, article.Author, article.MediaURL, article.PublishedTime, len(article.Content))
}

func TestRealNewsScraping(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	s := scraper.NewNewsScraper(12 * time.Second)

	testURLs := []struct {
		source string
		url    string
	}{
		{
			source: "FXStreet",
			url:    "https://www.fxstreet.com/news/eur-jpy-price-forecast-rises-toward-18600-after-rebounding-from-ascending-channel-bottom-202608250418",
		},
		{
			source: "InvestingLive",
			url:    "https://investinglive.com/news/what-are-the-main-events-for-today-27/",
		},
		{
			source: "ActionForex",
			url:    "https://www.actionforex.com/action-insight/market-overview/651706-aud-usd-stalls-at-resistance-as-rba-minutes-leave-real-test-to-tomorrows-cpi/",
		},
	}

	for _, tt := range testURLs {
		t.Run(tt.source, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
			defer cancel()

			article, err := s.FetchArticle(ctx, "real-test-id", tt.url)
			if err != nil {
				t.Logf("Warning: Live fetch to %s failed (network/protection): %v", tt.source, err)
				return
			}

			fmt.Printf("\n=== REAL SCRAPING RESULT: %s ===\n", tt.source)
			fmt.Printf("URL: %s\n", article.URL)
			fmt.Printf("Title: %s\n", article.Title)
			fmt.Printf("Author: %s\n", article.Author)
			fmt.Printf("Media URL: %s\n", article.MediaURL)
			fmt.Printf("Published Time: %s\n", article.PublishedTime)
			fmt.Printf("Content Length: %d chars\n", len(article.Content))
			fmt.Printf("Content Preview (first 500 chars):\n%s\n", snippet(article.Content, 500))
			fmt.Printf("===================================\n\n")

			if article.Title == "" {
				t.Errorf("expected non-empty title for %s", tt.source)
			}
		})
	}
}

func snippet(s string, maxLen int) string {
	s = strings.TrimSpace(s)
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}
