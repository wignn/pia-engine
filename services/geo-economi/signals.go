package main

import (
	"sort"
	"strings"
	"sync"
	"time"
)

const (
	CategoryConflict   = "conflict"
	CategorySanctions  = "sanctions"
	CategoryMaritime   = "maritime"
	CategoryDiplomatic = "diplomatic"
	CategoryTrade      = "trade"
)

type GeoSignalsMapResponse struct {
	Items []GeoSignalMapItem `json:"items"`
	Total int                `json:"total"`
}

type GeoSignalMapItem struct {
	Key               string            `json:"key"`
	Region            string            `json:"region,omitempty"`
	AvgSeverity       float64           `json:"avg_severity,omitempty"`
	MaxSeverity       float64           `json:"max_severity"`
	SignalCount       int               `json:"signal_count"`
	Category          string            `json:"category,omitempty"`
	Scope             string            `json:"scope,omitempty"`
	LatestTimestamp   time.Time         `json:"latest_timestamp,omitempty"`
	ChokepointsStatus map[string]string `json:"chokepoints_status,omitempty"`
}

type Signal struct {
	Source            string
	ExternalID        string
	Key               string
	Region            string
	Category          string
	Scope             string
	Severity          float64
	Confidence        float64
	Timestamp         time.Time
	SourceURL         string
	ChokepointsStatus map[string]string
}

type SignalFilter struct {
	Category    string
	Region      string
	MinSeverity float64
	From        time.Time
	To          time.Time
	Limit       int
}

type SignalStore struct {
	mu       sync.RWMutex
	signals  map[string]Signal
	activeBy map[string]time.Duration
}

func NewSignalStore() *SignalStore {
	return &SignalStore{
		signals: map[string]Signal{},
		activeBy: map[string]time.Duration{
			CategoryConflict:   30 * 24 * time.Hour,
			CategorySanctions:  365 * 24 * time.Hour,
			CategoryMaritime:   14 * 24 * time.Hour,
			CategoryDiplomatic: 14 * 24 * time.Hour,
			CategoryTrade:      90 * 24 * time.Hour,
		},
	}
}

func (s *SignalStore) UpsertBatch(signals []Signal) {
	s.mu.Lock()
	defer s.mu.Unlock()
	for _, signal := range signals {
		if signal.Source == "" || signal.ExternalID == "" || signal.Key == "" || signal.Timestamp.IsZero() {
			continue
		}
		if signal.Category == "" {
			signal.Category = CategoryConflict
		}
		if signal.Scope == "" {
			signal.Scope = "regional"
		}
		signal.Severity = clamp(signal.Severity)
		signal.Confidence = clampDefault(signal.Confidence, 1)
		s.signals[signal.Source+":"+signal.ExternalID] = signal
	}
}

func (s *SignalStore) SeedChokepoints(now time.Time) {
	chokepoints := []Signal{
		{
			Source:     "maritime-watch",
			ExternalID: "cp-hormuz",
			Key:        "Strait of Hormuz",
			Region:     "Middle East",
			Category:   CategoryMaritime,
			Scope:      "regional",
			Severity:   0.78,
			Confidence: 0.95,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Strait of Hormuz": "Elevated Naval Patrols & Vessel Monitoring",
			},
		},
		{
			Source:     "maritime-watch",
			ExternalID: "cp-bab-mandeb",
			Key:        "Bab el-Mandeb",
			Region:     "Middle East",
			Category:   CategoryMaritime,
			Scope:      "regional",
			Severity:   0.88,
			Confidence: 0.95,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Bab el-Mandeb": "Active Vessel Rerouting via Cape of Good Hope",
			},
		},
		{
			Source:     "maritime-watch",
			ExternalID: "cp-malacca",
			Key:        "Strait of Malacca",
			Region:     "Asia",
			Category:   CategoryMaritime,
			Scope:      "regional",
			Severity:   0.32,
			Confidence: 0.90,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Strait of Malacca": "Normal Commercial Transit Flow",
			},
		},
		{
			Source:     "maritime-watch",
			ExternalID: "cp-taiwan-strait",
			Key:        "Taiwan Strait",
			Region:     "Asia",
			Category:   CategoryDiplomatic,
			Scope:      "regional",
			Severity:   0.65,
			Confidence: 0.90,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Taiwan Strait": "Heightened Air & Naval Transit Monitoring",
			},
		},
		{
			Source:     "maritime-watch",
			ExternalID: "cp-suez",
			Key:        "Suez Canal",
			Region:     "Middle East",
			Category:   CategoryMaritime,
			Scope:      "regional",
			Severity:   0.72,
			Confidence: 0.95,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Suez Canal": "Reduced Daily Vessel Convoy Capacity",
			},
		},
		{
			Source:     "maritime-watch",
			ExternalID: "cp-black-sea",
			Key:        "Black Sea Maritime Corridor",
			Region:     "Europe",
			Category:   CategoryConflict,
			Scope:      "regional",
			Severity:   0.82,
			Confidence: 0.95,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Black Sea": "War Risk Insurance Zone Active",
			},
		},
		{
			Source:     "maritime-watch",
			ExternalID: "cp-panama",
			Key:        "Panama Canal",
			Region:     "Americas",
			Category:   CategoryMaritime,
			Scope:      "regional",
			Severity:   0.45,
			Confidence: 0.90,
			Timestamp:  now,
			ChokepointsStatus: map[string]string{
				"Panama Canal": "Draft Restrictions & Reservation System Active",
			},
		},
	}
	s.UpsertBatch(chokepoints)
}

func (s *SignalStore) Snapshot(filter SignalFilter, now time.Time) GeoSignalsMapResponse {
	s.mu.RLock()
	defer s.mu.RUnlock()

	type group struct {
		item  GeoSignalMapItem
		total float64
	}
	groups := make(map[string]*group)
	for _, signal := range s.signals {
		if !s.isActive(signal, now) || !matches(signal, filter) {
			continue
		}
		score := effectiveSeverity(signal, s.activeWindow(signal.Category), now)
		if score < filter.MinSeverity {
			continue
		}
		groupKey := signal.Key + "\x00" + signal.Category
		g := groups[groupKey]
		if g == nil {
			g = &group{item: GeoSignalMapItem{
				Key:               signal.Key,
				Region:            signal.Region,
				Category:          signal.Category,
				Scope:             signal.Scope,
				ChokepointsStatus: map[string]string{},
			}}
			groups[groupKey] = g
		}
		g.item.SignalCount++
		g.total += score
		if score > g.item.MaxSeverity {
			g.item.MaxSeverity = score
		}
		if signal.Timestamp.After(g.item.LatestTimestamp) {
			g.item.LatestTimestamp = signal.Timestamp
		}
		for name, status := range signal.ChokepointsStatus {
			g.item.ChokepointsStatus[name] = status
		}
	}

	items := make([]GeoSignalMapItem, 0, len(groups))
	for _, g := range groups {
		g.item.AvgSeverity = g.total / float64(g.item.SignalCount)
		if len(g.item.ChokepointsStatus) == 0 {
			g.item.ChokepointsStatus = nil
		}
		items = append(items, g.item)
	}
	sort.Slice(items, func(i, j int) bool {
		if items[i].MaxSeverity != items[j].MaxSeverity {
			return items[i].MaxSeverity > items[j].MaxSeverity
		}
		return items[i].Key < items[j].Key
	})
	if filter.Limit > 0 && len(items) > filter.Limit {
		items = items[:filter.Limit]
	}
	return GeoSignalsMapResponse{Items: items, Total: len(items)}
}

func (s *SignalStore) isActive(signal Signal, now time.Time) bool {
	age := now.Sub(signal.Timestamp)
	return age >= 0 && age <= s.activeWindow(signal.Category)
}

func (s *SignalStore) activeWindow(category string) time.Duration {
	if d, ok := s.activeBy[category]; ok {
		return d
	}
	return 30 * 24 * time.Hour
}

func matches(signal Signal, filter SignalFilter) bool {
	if filter.Category != "" && !strings.EqualFold(signal.Category, filter.Category) {
		return false
	}
	if filter.Region != "" && !strings.EqualFold(signal.Region, filter.Region) {
		return false
	}
	if !filter.From.IsZero() && signal.Timestamp.Before(filter.From) {
		return false
	}
	if !filter.To.IsZero() && signal.Timestamp.After(filter.To) {
		return false
	}
	return true
}

func effectiveSeverity(signal Signal, activeWindow time.Duration, now time.Time) float64 {
	age := now.Sub(signal.Timestamp)
	if age < 0 {
		age = 0
	}
	recency := 1 - 0.5*float64(age)/float64(activeWindow)
	if recency < 0.5 {
		recency = 0.5
	}
	return clamp(signal.Severity * clampDefault(signal.Confidence, 1) * recency)
}

func clamp(value float64) float64 {
	if value < 0 {
		return 0
	}
	if value > 1 {
		return 1
	}
	return value
}

func clampDefault(value, fallback float64) float64 {
	if value == 0 {
		return fallback
	}
	return clamp(value)
}
