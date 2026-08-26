package model

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"time"
)

type Quote struct {
	Actual       float64 `json:"actual"`
	DailyChange  float64 `json:"dailyChange"`
	DailyPercent float64 `json:"dailyPercent"`
	Monthly      float64 `json:"monthly"`
	Yearly       float64 `json:"yearly"`
	Forecast     float64 `json:"forecast"`
}

type Bond struct {
	Symbol      string  `json:"symbol"`
	Name        string  `json:"name"`
	Yield       float64 `json:"yield"`
	DayChange   float64 `json:"dayChange"`
	MonthChange float64 `json:"monthChange"`
	YearChange  float64 `json:"yearChange"`
	Date        string  `json:"date"`
}

type Related struct {
	Name     string  `json:"name"`
	Last     float64 `json:"last"`
	Previous float64 `json:"previous"`
	Unit     string  `json:"unit"`
	Date     string  `json:"date"`
}

type HistoryPoint struct {
	Date  string  `json:"date"`
	Value float64 `json:"value"`
}

type DashboardData struct {
	Source           string                    `json:"source"`
	FetchedAt        string                    `json:"fetchedAt"`
	Quote            Quote                     `json:"quote"`
	Bonds            []Bond                    `json:"bonds"`
	Related          []Related                 `json:"related"`
	Forecast         []float64                 `json:"forecast"`
	History          []HistoryPoint            `json:"history"`
	Histories        map[string][]HistoryPoint `json:"histories,omitempty"`
	HistoryAvailable bool                      `json:"historyAvailable"`
	HistoryMessage   string                    `json:"historyMessage,omitempty"`
}

// Validate checks for data sanity and non-empty bond records
func (d *DashboardData) Validate() error {
	if len(d.Bonds) == 0 {
		return errors.New("sanity check failed: bonds list is empty")
	}

	validBonds := 0
	for _, b := range d.Bonds {
		if b.Symbol != "" && b.Yield != 0 {
			validBonds++
		}
	}
	if validBonds == 0 {
		return errors.New("sanity check failed: all bonds have zero or empty values")
	}
	return nil
}

// MsgID produces a deterministic hash for NATS JetStream deduplication window
func (d *DashboardData) ToMacroEvent() MacroEvent {
	return MacroEvent{
		EventID:       d.MsgID(),
		SchemaVersion: 1,
		Source:        d.Source,
		ObservedAt:    parseObservedAt(d.FetchedAt),
		PublishedAt:   time.Now().UTC(),
		FeedType:      "bond",
		Payload:       d,
	}
}

func parseObservedAt(value string) time.Time {
	parsed, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return time.Now().UTC()
	}
	return parsed
}

func (d *DashboardData) MsgID() string {
	raw, _ := json.Marshal(struct {
		Actual float64 `json:"a"`
		Bonds  []Bond  `json:"b"`
		Date   string  `json:"d"`
	}{
		Actual: d.Quote.Actual,
		Bonds:  d.Bonds,
		Date:   time.Now().UTC().Format("2006-01-02T15:04"),
	})
	hash := sha256.Sum256(raw)
	return fmt.Sprintf("bond-us-%s", hex.EncodeToString(hash[:8]))
}
