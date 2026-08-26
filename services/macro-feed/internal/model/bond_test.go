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
