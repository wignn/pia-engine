package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func TestMacroStoreSnapshot(t *testing.T) {
	now := time.Date(2026, 9, 17, 12, 0, 0, 0, time.UTC)
	store := NewMacroStore()
	store.UpsertBatch([]MacroPoint{
		{Source: "test", CountryCode: "ID", Name: "Indonesia", Region: "Asia", Indicator: IndicatorInflation, Ticker: "IDCPI", Value: 3.1, Period: "2026", FetchedAt: now},
		{Source: "test", CountryCode: "ID", Name: "Indonesia", Region: "Asia", Indicator: IndicatorInflation, Ticker: "IDCPI", Value: 2.8, Period: "2025", FetchedAt: now},
	})
	result := store.Snapshot(MacroFilter{Indicator: IndicatorInflation}, now)
	if result.Total != 1 || result.IndicatorName == "" || result.Countries[0].ID != "ID" {
		t.Fatalf("unexpected result: %+v", result)
	}
	if result.Countries[0].PrevValue == nil || *result.Countries[0].PrevValue != 2.8 {
		t.Fatalf("missing previous value: %+v", result.Countries[0])
	}
	if result.Countries[0].Flag != "🇮🇩" || result.Countries[0].Rank != 1 {
		t.Fatalf("unexpected country metadata: %+v", result.Countries[0])
	}
}

func TestWorldBankMacroSource(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`[{},[{
			"countryiso3code":"IDN","country":{"id":"ID","value":"Indonesia"},"date":"2026","value":3.1
		},{"countryiso3code":"IDN","country":{"id":"ID","value":"Indonesia"},"date":"2025","value":2.8
		}]]`))
	}))
	defer server.Close()
	source := WorldBankMacroSource{HTTPSource: HTTPSource{Client: server.Client()}, Indicator: IndicatorInflation, Endpoint: server.URL}
	points, err := source.Fetch(context.Background())
	if err != nil || len(points) != 2 {
		t.Fatalf("fetch error=%v points=%+v", err, points)
	}
	if points[0].CountryCode != "ID" || points[0].Region != "Asia" {
		t.Fatalf("unexpected point: %+v", points[0])
	}
}

func TestMacroMapHandler(t *testing.T) {
	store := NewMacroStore()
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/api/v1/macro/map?indicator=invalid", nil)
	macroMapHandler(store)(recorder, request)
	if recorder.Code != http.StatusBadRequest {
		t.Fatalf("status=%d", recorder.Code)
	}

	recorder = httptest.NewRecorder()
	request = httptest.NewRequest(http.MethodGet, "/api/v1/macro/map?indicator=inflation", nil)
	macroMapHandler(store)(recorder, request)
	if recorder.Code != http.StatusOK || !strings.Contains(recorder.Body.String(), `"is_live":false`) {
		t.Fatalf("status=%d body=%s", recorder.Code, recorder.Body.String())
	}
	if !strings.Contains(recorder.Body.String(), `"error_code":"MACRO_MAP_UNAVAILABLE"`) {
		t.Fatalf("missing unavailable contract: %s", recorder.Body.String())
	}
}
