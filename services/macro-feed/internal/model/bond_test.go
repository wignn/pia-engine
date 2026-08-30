package model_test

import (
	"testing"
	"time"

	"macro-feed/internal/model"
)

func TestDashboardDataValidate(t *testing.T) {
	// Empty data should fail validation
	emptyData := &model.DashboardData{}
	if err := emptyData.Validate(); err == nil {
		t.Error("expected error for empty dashboard data, got nil")
	}

	// Valid data
	validData := &model.DashboardData{
		Source:    "test",
		FetchedAt: time.Now().UTC(),
		Bonds: []model.Bond{
			{
				Symbol: "USGG10YR:IND",
				Yield:  4.25,
			},
		},
	}
	if err := validData.Validate(); err != nil {
		t.Errorf("expected valid data, got error: %v", err)
	}

	// Deterministic MsgID check
	msgID1 := validData.MsgID()
	msgID2 := validData.MsgID()
	if msgID1 == "" || msgID1 != msgID2 {
		t.Errorf("expected deterministic msgID, got %s and %s", msgID1, msgID2)
	}
}

func TestDashboardDataToMacroEventWrapsSnapshot(t *testing.T) {
	data := &model.DashboardData{
		Source:    "test-source",
		FetchedAt: "2026-08-27T12:30:00Z",
		Bonds: []model.Bond{{
			Symbol: "USGG10YR:IND",
			Name:   "United States 10 Year Bond Yield",
			Yield:  4.25,
		}},
		HistoryAvailable: true,
		HistoryKind:      "provider",
		Histories: map[string][]model.HistoryPoint{
			"USGG10YR:IND": []model.HistoryPoint{{Date: "2026-08-27", Value: 4.25}},
		},
	}

	event := data.ToMacroEvent()
	payload, ok := event.Payload.(model.BondSnapshotPayload)
	if !ok {
		t.Fatalf("expected bond snapshot payload, got %T", event.Payload)
	}
	if payload.Country != "US" || payload.AsOf != "2026-08-27" {
		t.Fatalf("unexpected snapshot metadata: %+v", payload)
	}
	if payload.Raw != data || payload.Raw.HistoryKind != "provider" {
		t.Fatal("expected raw dashboard data to be preserved")
	}
}
