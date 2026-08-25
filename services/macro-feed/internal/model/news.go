package model

import (
	"fmt"
	"time"
)

type ScrapeJob struct {
	ID  string `json:"id"` // content_hash
	URL string `json:"url"`
}

type ExtractedArticle struct {
	ID            string    `json:"id"`
	URL           string    `json:"url"`
	Title         string    `json:"title"`
	Author        string    `json:"author"`
	PublishedTime string    `json:"published_time"`
	Content       string    `json:"content"`
	MediaURL      string    `json:"media_url"`
	FetchedAt     time.Time `json:"fetched_at"`
}

func (a *ExtractedArticle) ToMacroEvent() MacroEvent {
	return MacroEvent{
		EventID:       fmt.Sprintf("news-%s", a.ID),
		SchemaVersion: 1,
		Source:        "macro-feed-scraper",
		ObservedAt:    a.FetchedAt,
		PublishedAt:   time.Now().UTC(),
		FeedType:      "news_scraped",
		Payload:       a,
	}
}
