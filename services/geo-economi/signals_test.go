package main

import (
	"context"
	"math"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestEffectiveSeverityAndAggregation(t *testing.T) {
	now := time.Date(2026, 9, 17, 12, 0, 0, 0, time.UTC)
	store := NewSignalStore()
	store.UpsertBatch([]Signal{
		{Source: "test", ExternalID: "1", Key: "Yemen", Region: "Middle East", Category: CategoryConflict, Severity: 0.8, Confidence: 1, Timestamp: now},
		{Source: "test", ExternalID: "2", Key: "Yemen", Region: "Middle East", Category: CategoryConflict, Severity: 0.4, Confidence: 1, Timestamp: now},
		{Source: "test", ExternalID: "old", Key: "Yemen", Category: CategoryConflict, Severity: 1, Timestamp: now.Add(-31 * 24 * time.Hour)},
	})
	response := store.Snapshot(SignalFilter{}, now)
	if response.Total != 1 || response.Items[0].SignalCount != 2 {
		t.Fatalf("unexpected aggregation: %+v", response)
	}
	if response.Items[0].MaxSeverity != 0.8 || abs(response.Items[0].AvgSeverity-0.6) > 1e-9 {
		t.Fatalf("unexpected severity: %+v", response.Items[0])
	}
}

func abs(value float64) float64 { return math.Abs(value) }

func TestGDELTSource(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"articles":[{"url":"https://example.test/a","title":"Maritime attack in Red Sea","seendate":"20260917T120000Z","sourcecountry":"Yemen"}]}`))
	}))
	defer server.Close()

	source := GDELTSource{HTTPSource: HTTPSource{Client: server.Client()}, Endpoint: server.URL}
	signals, err := source.Fetch(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if len(signals) != 1 || signals[0].Category != CategoryConflict || signals[0].Key != "Yemen" {
		t.Fatalf("unexpected signals: %+v", signals)
	}
}

func TestMapHandler(t *testing.T) {
	store := NewSignalStore()
	store.UpsertBatch([]Signal{{Source: "test", ExternalID: "1", Key: "Iran", Region: "Middle East", Category: CategorySanctions, Severity: 0.7, Timestamp: time.Now().UTC()}})
	request := httptest.NewRequest(http.MethodGet, "/api/v1/geosignals/map?min_severity=0.5", nil)
	recorder := httptest.NewRecorder()
	mapHandler(store)(recorder, request)
	if recorder.Code != http.StatusOK {
		t.Fatalf("status=%d body=%s", recorder.Code, recorder.Body.String())
	}

	request = httptest.NewRequest(http.MethodGet, "/api/v1/geosignals/map?min_severity=2", nil)
	recorder = httptest.NewRecorder()
	mapHandler(store)(recorder, request)
	if recorder.Code != http.StatusBadRequest {
		t.Fatalf("invalid query status=%d", recorder.Code)
	}
}
