package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/signal"
	"syscall"

	"macro-feed/internal/collector"
	"macro-feed/internal/config"
	"macro-feed/internal/publisher"
	"macro-feed/internal/scraper"
	"macro-feed/internal/worker"
)

func main() {
	// 1. Structured Logging
	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))
	slog.SetDefault(logger)

	// 2. Load Config
	cfg, err := config.Load()
	if err != nil {
		logger.Error("Failed to load environment config", "error", err)
		os.Exit(1)
	}

	logger.Info("Starting Macro Feed Worker Manager...",
		"bond_subject", cfg.BondSubject,
		"fred_subject", cfg.FredSubject,
	)

	// 3. Graceful Shutdown Context
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	// 4. Initialize NATS JetStream Publisher
	jsPub, err := publisher.NewJetStreamPublisher(cfg.NatsURL, logger)
	if err != nil {
		logger.Error("Failed to connect to NATS JetStream", "error", err)
		os.Exit(1)
	}
	defer jsPub.Close()

	// Ensure Stream provisioning
	streamSubjects := []string{fmt.Sprintf("%s.*", cfg.StreamName), "macro.feed.v1.>"}
	if err := jsPub.EnsureStream(ctx, cfg.StreamName, streamSubjects); err != nil {
		logger.Warn("Warning: failed to ensure stream", "stream", cfg.StreamName, "error", err)
	}

	// 5. Initialize Components
	bondScraper := scraper.NewBondScraper(cfg.BondURL, cfg.TEApiKey, cfg.RequestTimeout)
	newsScraper := scraper.NewNewsScraper(cfg.RequestTimeout)
	newsWorker := worker.NewNewsWorker(newsScraper, jsPub, logger)
	var fredCollector *collector.FredCollector
	if cfg.HasFred() {
		fredCollector = collector.NewFredCollector(cfg.FredAPIKey, cfg.RequestTimeout)
	}

	// 6. Initialize Worker Manager
	mgr := worker.NewManager(cfg, bondScraper, fredCollector, newsWorker, jsPub, logger)

	// 7. Run Worker Manager (blocking until context canceled)
	mgr.Run(ctx)

	logger.Info("Macro feed worker stopped gracefully")
}
