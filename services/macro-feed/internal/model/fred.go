package model

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"time"
)

type MacroEvent struct {
	EventID       string      `json:"event_id"`
	SchemaVersion int         `json:"schema_version"`
	Source        string      `json:"source"`
	ObservedAt    time.Time   `json:"observed_at"`
	PublishedAt   time.Time   `json:"published_at"`
	FeedType      string      `json:"feed_type"`
	Payload       interface{} `json:"payload"`
}

type RateEvent struct {
	Source      string    `json:"source"`
	Country     string    `json:"country"`
	Tenor       string    `json:"tenor"`
	Date        string    `json:"date"`
	Value       float64   `json:"value"`
	Unit        string    `json:"unit"`
	SeriesID    string    `json:"series_id"`
	PublishedAt time.Time `json:"published_at"`
}

func (r *RateEvent) ToMacroEvent() MacroEvent {
	observedAt, err := time.Parse("2006-01-02", r.Date)
	if err != nil {
		observedAt = r.PublishedAt
	}
	return MacroEvent{
		EventID:       r.MsgID(),
		SchemaVersion: 1,
		Source:        r.Source,
		ObservedAt:    observedAt.UTC(),
		PublishedAt:   time.Now().UTC(),
		FeedType:      "rate",
		Payload:       r,
	}
}

func (r *RateEvent) ToMacroSpreadEvent(spread string) MacroEvent {
	event := r.ToMacroEvent()
	event.FeedType = "spread"
	event.Payload = struct {
		Country  string  `json:"country"`
		Spread   string  `json:"spread"`
		Date     string  `json:"date"`
		Value    float64 `json:"value"`
		SeriesID string  `json:"series_id"`
	}{r.Country, spread, r.Date, r.Value, r.SeriesID}
	return event
}

func (r *RateEvent) MsgID() string {
	raw := fmt.Sprintf("%s:%s:%s:%s", r.Source, r.Country, r.Tenor, r.Date)
	hash := sha256.Sum256([]byte(raw))
	return fmt.Sprintf("rate-%s", hex.EncodeToString(hash[:8]))
}

type RateSeriesConfig struct {
	ID      string
	Country string
	Tenor   string
	Unit    string
}

type SpreadSeriesConfig struct {
	ID      string
	Country string
	Spread  string
}

var RateSeriesList = []RateSeriesConfig{
	{ID: "DGS3MO", Country: "US", Tenor: "3M", Unit: "percent"},
	{ID: "DGS2", Country: "US", Tenor: "2Y", Unit: "percent"},
	{ID: "DGS5", Country: "US", Tenor: "5Y", Unit: "percent"},
	{ID: "DGS10", Country: "US", Tenor: "10Y", Unit: "percent"},
	{ID: "DGS30", Country: "US", Tenor: "30Y", Unit: "percent"},
	{ID: "DFII10", Country: "US", Tenor: "10Y_REAL", Unit: "percent"},
	{ID: "T10YIE", Country: "US", Tenor: "10Y_BREAKEVEN", Unit: "percent"},
}

var SpreadSeriesList = []SpreadSeriesConfig{
	{ID: "T10Y2Y", Country: "US", Spread: "2s10s"},
	{ID: "T10Y3M", Country: "US", Spread: "3m10y"},
}
