package worker

import (
	"context"
	"fmt"
	"log/slog"
	"math"
	"math/rand/v2"
	"runtime/debug"
	"strings"
	"sync"
	"time"

	"macro-feed/internal/collector"
	"macro-feed/internal/config"
	"macro-feed/internal/model"
	"macro-feed/internal/publisher"
	"macro-feed/internal/scraper"
)

type Manager struct {
	cfg        *config.Config
	scraper    *scraper.BondScraper
	collector  *collector.FredCollector
	newsWorker *NewsWorker
	publisher  *publisher.JetStreamPublisher
	logger     *slog.Logger
}

func NewManager(
	cfg *config.Config,
	s *scraper.BondScraper,
	c *collector.FredCollector,
	nw *NewsWorker,
	p *publisher.JetStreamPublisher,
	l *slog.Logger,
) *Manager {
	if l == nil {
		l = slog.Default()
	}
	return &Manager{
		cfg:        cfg,
		scraper:    s,
		collector:  c,
		newsWorker: nw,
		publisher:  p,
		logger:     l,
	}
}

func (m *Manager) Run(ctx context.Context) {
	var wg sync.WaitGroup

	// 1. Bond Scraper Routine
	wg.Add(1)
	go func() {
		defer wg.Done()
		m.runBondLoop(ctx)
	}()

	// 2. FRED Collector Routine (if API Key provided)
	if m.cfg.HasFred() && m.collector != nil {
		wg.Add(1)
		go func() {
			defer wg.Done()
			m.runFredLoop(ctx)
		}()
	} else {
		m.logger.Warn("FRED_API_KEY is not set. FRED collector routine disabled.")
	}

	// 3. News Scraper Routine
	if m.newsWorker != nil {
		wg.Add(1)
		go func() {
			defer wg.Done()
			m.runNewsConsumer(ctx)
		}()
	}

	wg.Wait()
	m.logger.Info("All worker routines stopped.")
}

func (m *Manager) runNewsConsumer(ctx context.Context) {
	m.logger.Info("Starting NATS news scrape job consumer listener")
	sub, err := m.publisher.SubscribeJobs(ctx, "SCRAPE_JOBS", "scrape.jobs", "macro-news-scraper")
	if err != nil {
		m.logger.Warn("Failed to subscribe to scrape.jobs, news consumer disabled", "error", err)
		return
	}

	for {
		select {
		case <-ctx.Done():
			return
		default:
			msgs, err := sub.Fetch(1)
			if err != nil {
				if ctx.Err() != nil {
					return
				}
				time.Sleep(500 * time.Millisecond)
				continue
			}
			for msg := range msgs.Messages() {
				m.newsWorker.ProcessJob(ctx, msg)
			}
		}
	}
}

func (m *Manager) runBondLoop(ctx context.Context) {
	m.logger.Info("Starting Bond feed loop", "interval", m.cfg.PollInterval, "subject", m.cfg.BondSubject)

	consecutiveErrors := 0
	m.safeBondExecute(ctx, &consecutiveErrors)

	for {
		sleep := calculateSleep(m.cfg.PollInterval, 5*time.Minute, consecutiveErrors)
		select {
		case <-ctx.Done():
			return
		case <-time.After(sleep):
			m.safeBondExecute(ctx, &consecutiveErrors)
		}
	}
}

func (m *Manager) safeBondExecute(ctx context.Context, errCount *int) {
	defer func() {
		if r := recover(); r != nil {
			*errCount++
			m.logger.Error("PANIC RECOVERED in bond worker",
				"error", r,
				"stack", string(debug.Stack()),
				"consecutive_errors", *errCount,
			)
		}
	}()

	data, err := m.scraper.Fetch(ctx)
	if err != nil {
		*errCount++
		m.logger.Error("Bond scraper cycle failed", "error", err, "consecutive_errors", *errCount)
		return
	}

	if err := data.Validate(); err != nil {
		*errCount++
		m.logger.Error("Bond data validation failed", "error", err)
		return
	}

	envelope := data.ToMacroEvent()
	if err := m.publisher.PublishMacro(ctx, m.cfg.BondSubject, &envelope); err != nil {
		*errCount++
		m.logger.Error("Publish bond to JetStream failed", "error", err)
		return
	}

	if *errCount > 0 {
		m.logger.Info("Bond worker recovered to healthy state")
	}
	*errCount = 0
	m.logger.Info("Bond data published successfully", "bonds_count", len(data.Bonds), "msg_id", data.MsgID())
}

func (m *Manager) runFredLoop(ctx context.Context) {
	m.logger.Info("Starting FRED rates loop", "interval", m.cfg.FredInterval, "subject", m.cfg.FredSubject)

	consecutiveErrors := 0
	m.safeFredExecute(ctx, &consecutiveErrors)

	for {
		sleep := calculateSleep(m.cfg.FredInterval, 10*time.Minute, consecutiveErrors)
		select {
		case <-ctx.Done():
			return
		case <-time.After(sleep):
			m.safeFredExecute(ctx, &consecutiveErrors)
		}
	}
}

func (m *Manager) safeFredExecute(ctx context.Context, errCount *int) {
	defer func() {
		if r := recover(); r != nil {
			*errCount++
			m.logger.Error("PANIC RECOVERED in FRED worker",
				"error", r,
				"stack", string(debug.Stack()),
				"consecutive_errors", *errCount,
			)
		}
	}()

	events, err := m.collector.FetchAll(ctx)
	if err != nil {
		*errCount++
		m.logger.Error("FRED collector cycle failed", "error", err, "consecutive_errors", *errCount)
		return
	}

	success := 0
	for _, event := range events {
		targetSubject := fmt.Sprintf("%s.%s", m.cfg.FredSubject, strings.ToLower(event.Tenor))
		envelope := event.ToMacroEvent()

		if event.SeriesID == "T10Y2Y" || event.SeriesID == "T10Y3M" {
			targetSubject = fmt.Sprintf("macro.feed.v1.spreads.%s", strings.ToLower(event.Tenor))
			envelope = event.ToMacroSpreadEvent(strings.ToLower(event.Tenor))
		}

		if err := m.publisher.PublishMacro(ctx, targetSubject, &envelope); err != nil {
			m.logger.Error("Failed to publish rate event", "series", event.SeriesID, "error", err)
			continue
		}
		success++
	}

	if *errCount > 0 {
		m.logger.Info("FRED worker recovered to healthy state")
	}
	*errCount = 0
	m.logger.Info("FRED rates published successfully", "total_events", len(events), "published", success)
}

func calculateSleep(interval, maxBackoff time.Duration, consecutiveErrors int) time.Duration {
	if consecutiveErrors == 0 {
		return interval
	}
	exponent := min(consecutiveErrors, 6)
	backoffSec := math.Pow(2, float64(exponent))
	jitter := rand.Float64() * 2.0
	delay := time.Duration(backoffSec+jitter) * time.Second
	if delay > maxBackoff {
		delay = maxBackoff
	}
	return delay
}
