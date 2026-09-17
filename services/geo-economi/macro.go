package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"sort"
	"strings"
	"sync"
	"time"
)

const (
	IndicatorInflation    = "inflation"
	IndicatorInterestRate = "interest_rate"
	IndicatorGDPGrowth    = "gdp_growth"
	IndicatorUnemployment = "unemployment"
	IndicatorDebtToGDP    = "debt_to_gdp"
)

type MacroMapResult struct {
	Indicator     string         `json:"indicator"`
	IndicatorName string         `json:"indicatorName"`
	Unit          string         `json:"unit"`
	Period        string         `json:"period"`
	MinValue      float64        `json:"minValue,omitempty"`
	MaxValue      float64        `json:"maxValue,omitempty"`
	Timeline      []string       `json:"timeline,omitempty"`
	Countries     []MacroCountry `json:"countries"`
	Total         int            `json:"total"`
	IsLive        bool           `json:"isLive"`
	UpdatedAt     int64          `json:"updatedAt,omitempty"`
}

type MacroCountry struct {
	ID        string             `json:"id"`
	Name      string             `json:"name"`
	Flag      string             `json:"flag,omitempty"`
	Region    string             `json:"region"`
	Ticker    string             `json:"ticker"`
	Value     *float64           `json:"value,omitempty"`
	PrevValue *float64           `json:"prevValue,omitempty"`
	Change    *float64           `json:"change,omitempty"`
	Unit      string             `json:"unit"`
	Period    string             `json:"period"`
	Rank      int                `json:"rank,omitempty"`
	History   map[string]float64 `json:"history"`
}

type MacroPoint struct {
	Source      string
	CountryCode string
	Name        string
	Region      string
	Indicator   string
	Ticker      string
	Value       float64
	Period      string
	ObservedAt  time.Time
	FetchedAt   time.Time
}

type MacroFilter struct {
	Indicator string
	Region    string
	Limit     int
}

type MacroStore struct {
	mu        sync.RWMutex
	points    map[string][]MacroPoint
	refreshed map[string]time.Time
}

func NewMacroStore() *MacroStore {
	return &MacroStore{points: make(map[string][]MacroPoint), refreshed: make(map[string]time.Time)}
}

func (s *MacroStore) UpsertBatch(points []MacroPoint) {
	s.mu.Lock()
	defer s.mu.Unlock()
	for _, point := range points {
		if point.CountryCode == "" || !validIndicator(point.Indicator) || point.Period == "" {
			continue
		}
		point.CountryCode = strings.ToUpper(point.CountryCode)
		point.Indicator = strings.ToLower(point.Indicator)
		key := point.Indicator + ":" + point.CountryCode
		rows := s.points[key]
		updated := false
		for i := range rows {
			if rows[i].Period == point.Period {
				rows[i] = point
				updated = true
				break
			}
		}
		if !updated {
			rows = append(rows, point)
		}
		sort.Slice(rows, func(i, j int) bool { return rows[i].Period > rows[j].Period })
		s.points[key] = rows
		s.refreshed[point.Indicator] = maxTime(s.refreshed[point.Indicator], point.FetchedAt)
	}
}

func (s *MacroStore) Snapshot(filter MacroFilter, now time.Time) MacroMapResult {
	indicator := strings.ToLower(strings.TrimSpace(filter.Indicator))
	if indicator == "" {
		indicator = IndicatorInflation
	}
	meta := macroMetadata(indicator)
	result := MacroMapResult{Indicator: indicator, IndicatorName: meta.name, Unit: meta.unit, Countries: []MacroCountry{}, IsLive: false}
	if !validIndicator(indicator) {
		return result
	}

	s.mu.RLock()
	defer s.mu.RUnlock()
	for key, rows := range s.points {
		if !strings.HasPrefix(key, indicator+":") || len(rows) == 0 {
			continue
		}
		point := rows[0]
		if filter.Region != "" && !strings.EqualFold(point.Region, filter.Region) {
			continue
		}
		country := MacroCountry{
			ID: point.CountryCode, Name: point.Name, Flag: flagForISO2(point.CountryCode),
			Region: point.Region, Ticker: point.Ticker, Unit: meta.unit, Period: point.Period,
			History: map[string]float64{},
		}
		for _, row := range rows {
			country.History[row.Period] = row.Value
		}
		country.Value = floatPtr(point.Value)
		if len(rows) > 1 {
			country.PrevValue = floatPtr(rows[1].Value)
			country.Change = floatPtr(point.Value - rows[1].Value)
		}
		result.Countries = append(result.Countries, country)
	}

	sort.Slice(result.Countries, func(i, j int) bool {
		return valueOf(result.Countries[i].Value) > valueOf(result.Countries[j].Value)
	})
	for i := range result.Countries {
		result.Countries[i].Rank = i + 1
		if result.Period == "" || result.Countries[i].Period > result.Period {
			result.Period = result.Countries[i].Period
		}
		for period := range result.Countries[i].History {
			if !contains(result.Timeline, period) {
				result.Timeline = append(result.Timeline, period)
			}
		}
	}
	sort.Strings(result.Timeline)
	if filter.Limit > 0 && len(result.Countries) > filter.Limit {
		result.Countries = result.Countries[:filter.Limit]
	}
	result.Total = len(result.Countries)
	if result.Total > 0 {
		result.MinValue = valueOf(result.Countries[0].Value)
		result.MaxValue = result.MinValue
		for _, country := range result.Countries {
			value := valueOf(country.Value)
			if value < result.MinValue {
				result.MinValue = value
			}
			if value > result.MaxValue {
				result.MaxValue = value
			}
		}
	}
	if refreshed := s.refreshed[indicator]; !refreshed.IsZero() {
		result.UpdatedAt = refreshed.UnixMilli()
		result.IsLive = now.Sub(refreshed) <= 48*time.Hour
	}
	return result
}

type macroIndicatorMeta struct{ name, unit, code, ticker string }

func macroMetadata(indicator string) macroIndicatorMeta {
	switch indicator {
	case IndicatorInflation:
		return macroIndicatorMeta{"Consumer Price Inflation", "%", "FP.CPI.TOTL.ZG", "CPI"}
	case IndicatorInterestRate:
		return macroIndicatorMeta{"Real Interest Rate", "%", "FR.INR.RINR", "INTR"}
	case IndicatorGDPGrowth:
		return macroIndicatorMeta{"GDP Growth", "%", "NY.GDP.MKTP.KD.ZG", "GDP"}
	case IndicatorUnemployment:
		return macroIndicatorMeta{"Unemployment Rate", "%", "SL.UEM.TOTL.ZS", "UNEMP"}
	case IndicatorDebtToGDP:
		return macroIndicatorMeta{"Central Government Debt to GDP", "%", "GC.DOD.TOTL.GD.ZS", "DEBT"}
	default:
		return macroIndicatorMeta{}
	}
}

func validIndicator(indicator string) bool {
	switch indicator {
	case IndicatorInflation, IndicatorInterestRate, IndicatorGDPGrowth, IndicatorUnemployment, IndicatorDebtToGDP:
		return true
	}
	return false
}

type worldBankRecord struct {
	CountryISO3 string `json:"countryiso3code"`
	Country     struct {
		ID       string `json:"id"`
		Value    string `json:"value"`
		ISO2Code string `json:"iso2Code"`
	} `json:"country"`
	Date  string   `json:"date"`
	Value *float64 `json:"value"`
}

type WorldBankMacroSource struct {
	HTTPSource
	Indicator string
	Endpoint  string
}

func (s WorldBankMacroSource) Name() string { return "worldbank-" + s.Indicator }

func (s WorldBankMacroSource) Fetch(ctx context.Context) ([]MacroPoint, error) {
	meta := macroMetadata(s.Indicator)
	if !validIndicator(s.Indicator) {
		return nil, fmt.Errorf("unsupported macro indicator %q", s.Indicator)
	}
	endpoint := s.Endpoint
	defaultEndpoint := endpoint == ""
	if defaultEndpoint {
		endpoint = "https://api.worldbank.org/v2/country/all/indicator/" + url.PathEscape(meta.code) + "?format=json&date=2015:2026&per_page=1000"
	}

	// World Bank paginates this endpoint. The first page contains only 1,000
	// of roughly 2,900 observations, so read every page before aggregating.
	var allRecords []worldBankRecord
	page := 1
	for {
		payload, err := s.do(ctx, http.MethodGet, worldBankPage(endpoint, page), nil)
		if err != nil {
			return nil, err
		}
		var raw []json.RawMessage
		if err := json.Unmarshal(payload, &raw); err != nil || len(raw) < 2 {
			return nil, fmt.Errorf("invalid World Bank response")
		}
		var metaPage struct {
			Pages int `json:"pages"`
		}
		if err := json.Unmarshal(raw[0], &metaPage); err != nil {
			return nil, fmt.Errorf("decode World Bank pagination: %w", err)
		}
		var records []worldBankRecord
		if err := json.Unmarshal(raw[1], &records); err != nil {
			return nil, err
		}
		allRecords = append(allRecords, records...)
		if metaPage.Pages <= page || len(records) == 0 {
			break
		}
		page++
	}

	countryCodes := map[string]string{}
	if defaultEndpoint {
		countryCodes, _ = fetchWorldBankCountryCodes(ctx, s.HTTPSource)
	}
	now := time.Now().UTC()
	points := make([]MacroPoint, 0, len(allRecords))
	seen := map[string]bool{}
	for _, record := range allRecords {
		if record.Value == nil || record.Date == "" || isAggregateCountry(record.Country.Value) {
			continue
		}
		code := iso2(record.Country.ISO2Code, countryCodes[record.CountryISO3], record.CountryISO3)
		if len(code) != 2 {
			continue
		}
		key := code + ":" + record.Date
		if seen[key] {
			continue
		}
		seen[key] = true
		points = append(points, MacroPoint{Source: s.Name(), CountryCode: code, Name: record.Country.Value, Region: macroRegion(code), Indicator: s.Indicator, Ticker: code + meta.ticker, Value: *record.Value, Period: record.Date, ObservedAt: parseYear(record.Date), FetchedAt: now})
	}
	return points, nil
}

func worldBankPage(endpoint string, page int) string {
	separator := "?"
	if strings.Contains(endpoint, "?") {
		separator = "&"
	}
	return endpoint + separator + "page=" + fmt.Sprint(page)
}

func fetchWorldBankCountryCodes(ctx context.Context, source HTTPSource) (map[string]string, error) {
	payload, err := source.do(ctx, http.MethodGet, "https://api.worldbank.org/v2/country?format=json&per_page=400", nil)
	if err != nil {
		return nil, err
	}
	var raw []json.RawMessage
	if err := json.Unmarshal(payload, &raw); err != nil || len(raw) < 2 {
		return nil, fmt.Errorf("invalid World Bank country response")
	}
	var records []struct {
		ID   string `json:"id"`
		ISO2 string `json:"iso2Code"`
	}
	if err := json.Unmarshal(raw[1], &records); err != nil {
		return nil, err
	}
	codes := make(map[string]string, len(records))
	for _, record := range records {
		if len(record.ID) == 3 && len(record.ISO2) == 2 {
			codes[strings.ToUpper(record.ID)] = strings.ToUpper(record.ISO2)
		}
	}
	return codes, nil
}

func iso2(value, mapped, iso3 string) string {
	value = strings.ToUpper(strings.TrimSpace(value))
	if len(value) == 2 {
		return value
	}
	mapped = strings.ToUpper(strings.TrimSpace(mapped))
	if len(mapped) == 2 {
		return mapped
	}
	return map[string]string{"USA": "US", "IDN": "ID", "GBR": "GB", "JPN": "JP", "CHN": "CN", "IND": "IN", "DEU": "DE", "FRA": "FR", "ITA": "IT", "CAN": "CA", "BRA": "BR", "RUS": "RU", "AUS": "AU", "KOR": "KR", "ZAF": "ZA", "MEX": "MX", "TUR": "TR", "SAU": "SA", "IRN": "IR", "ISR": "IL", "UKR": "UA"}[strings.ToUpper(iso3)]
}

func parseYear(value string) time.Time { t, _ := time.Parse("2006", value); return t.UTC() }
func maxTime(a, b time.Time) time.Time {
	if b.After(a) {
		return b
	}
	return a
}
func floatPtr(value float64) *float64 { return &value }
func valueOf(value *float64) float64 {
	if value == nil {
		return 0
	}
	return *value
}
func contains(values []string, value string) bool {
	for _, item := range values {
		if item == value {
			return true
		}
	}
	return false
}

func isAggregateCountry(name string) bool {
	lower := strings.ToLower(name)
	for _, marker := range []string{"income", "world", "europe & central asia", "latin america", "middle east", "south asia", "sub-saharan", "east asia & pacific", "north america", "euro area", "oecd"} {
		if strings.Contains(lower, marker) {
			return true
		}
	}
	return false
}

func macroRegion(code string) string {
	code = strings.ToUpper(code)
	switch {
	case code == "US" || code == "CA" || code == "GB" || code == "FR" || code == "DE" || code == "IT" || code == "JP":
		return map[string]string{"US": "G7", "CA": "G7", "GB": "G7", "FR": "G7", "DE": "G7", "IT": "G7", "JP": "G7"}[code]
	case contains([]string{"BR", "RU", "IN", "CN", "ZA"}, code):
		return "BRICS"
	case contains([]string{"BE", "DK", "FI", "IE", "NL", "NO", "PL", "PT", "ES", "SE", "CH", "AT", "GR"}, code):
		return "Europe"
	case contains([]string{"MX", "AR", "CL", "CO", "PE", "US", "CA"}, code):
		return "Americas"
	case contains([]string{"ID", "KR", "SG", "TH", "MY", "VN", "PH", "AU", "NZ", "TR", "SA", "IR", "IL"}, code):
		return "Asia"
	case contains([]string{"NG", "EG", "KE", "GH", "ET", "MA", "DZ", "TN"}, code):
		return "Africa"
	case contains([]string{"FJ", "PG"}, code):
		return "Oceania"
	default:
		return "Unknown"
	}
}

func flagForISO2(code string) string {
	code = strings.ToUpper(code)
	if len(code) != 2 {
		return ""
	}
	return string([]rune{rune(0x1F1E6 + int(code[0]-'A')), rune(0x1F1E6 + int(code[1]-'A'))})
}
