package scraper_test

import (
	"context"
	"testing"
	"time"

	"macro-feed/internal/scraper"
)

func TestBondScraperInit(t *testing.T) {
	s := scraper.NewBondScraper("https://tradingeconomics.com/united-states/government-bond-yield", "api", 5*time.Second)
	if s == nil {
		t.Fatal("expected non-nil scraper instance")
	}

	_, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)

	defer cancel()
}
