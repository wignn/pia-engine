package collector

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"time"

	"macro-feed/internal/model"
)

type FredCollector struct {
	apiKey     string
	httpClient *http.Client
}

func NewFredCollector(apiKey string, timeout time.Duration) *FredCollector {
	if timeout <= 0 {
		timeout = 30 * time.Second
	}
	return &FredCollector{
		apiKey:     apiKey,
		httpClient: &http.Client{Timeout: timeout},
	}
}

func (c *FredCollector) FetchAll(ctx context.Context) ([]model.RateEvent, error) {
	if c.apiKey == "" {
		return nil, nil
	}

	var allEvents []model.RateEvent

	// 1. Fetch Rate Series
	for _, series := range model.RateSeriesList {
		events, err := c.FetchSeries(ctx, series.ID, series.Country, series.Tenor, series.Unit)
		if err != nil {
			return allEvents, fmt.Errorf("fetch series %s: %w", series.ID, err)
		}
		allEvents = append(allEvents, events...)
		time.Sleep(200 * time.Millisecond) // Polite delay for API limits
	}

	// 2. Fetch Spread Series
	for _, spread := range model.SpreadSeriesList {
		events, err := c.FetchSeries(ctx, spread.ID, spread.Country, spread.Spread, "percent")
		if err != nil {
			return allEvents, fmt.Errorf("fetch spread %s: %w", spread.ID, err)
		}
		allEvents = append(allEvents, events...)
		time.Sleep(200 * time.Millisecond)
	}

	return allEvents, nil
}

func (c *FredCollector) FetchSeries(ctx context.Context, seriesID, country, tenor, unit string) ([]model.RateEvent, error) {
	url := fmt.Sprintf(
		"https://api.stlouisfed.org/fred/series/observations?series_id=%s&api_key=%s&file_type=json&sort_order=desc&limit=100",
		seriesID, c.apiKey,
	)

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("FRED API returned HTTP %d", resp.StatusCode)
	}

	body, err := io.ReadAll(io.LimitReader(resp.Body, 8<<20))
	if err != nil {
		return nil, err
	}

	var data struct {
		Observations []struct {
			Date  string `json:"date"`
			Value string `json:"value"`
		} `json:"observations"`
	}

	if err := json.Unmarshal(body, &data); err != nil {
		return nil, fmt.Errorf("decode FRED observations: %w", err)
	}

	var events []model.RateEvent
	now := time.Now().UTC()
	for _, obs := range data.Observations {
		if obs.Value == "." || obs.Value == "" {
			continue
		}
		val, err := strconv.ParseFloat(obs.Value, 64)
		if err != nil {
			continue
		}
		events = append(events, model.RateEvent{
			Source:      "fred",
			Country:     country,
			Tenor:       tenor,
			Date:        obs.Date,
			Value:       val,
			Unit:        unit,
			SeriesID:    seriesID,
			PublishedAt: now,
		})
	}

	return events, nil
}
