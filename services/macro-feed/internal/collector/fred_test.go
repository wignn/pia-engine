package collector_test

import (
	"context"
	"testing"
	"time"

	"macro-feed/internal/collector"
)

func TestFredCollectorInit(t *testing.T) {
	c := collector.NewFredCollector("", 5*time.Second)
	if c == nil {
		t.Fatal("expected non-nil FRED collector")
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)
	defer cancel()

	// Empty API key should return nil without error
	events, err := c.FetchAll(ctx)
	if err != nil {
		t.Errorf("expected no error with empty key, got %v", err)
	}
	if len(events) != 0 {
		t.Errorf("expected 0 events, got %d", len(events))
	}
}
