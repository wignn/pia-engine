package worker

import (
	"context"
	"encoding/json"
	"log/slog"

	"macro-feed/internal/model"
	"macro-feed/internal/publisher"
	"macro-feed/internal/scraper"

	"github.com/nats-io/nats.go/jetstream"
)

type NewsWorker struct {
	scraper   *scraper.NewsScraper
	publisher *publisher.JetStreamPublisher
	logger    *slog.Logger
}

func NewNewsWorker(s *scraper.NewsScraper, p *publisher.JetStreamPublisher, l *slog.Logger) *NewsWorker {
	if l == nil {
		l = slog.Default()
	}
	return &NewsWorker{
		scraper:   s,
		publisher: p,
		logger:    l,
	}
}

func (w *NewsWorker) ProcessJob(ctx context.Context, msg jetstream.Msg) error {
	var job model.ScrapeJob
	if err := json.Unmarshal(msg.Data(), &job); err != nil {
		w.logger.Error("Failed to unmarshal scrape job", "error", err)
		msg.Ack()
		return err
	}

	if job.URL == "" {
		msg.Ack()
		return nil
	}

	w.logger.Info("Scraping article for job", "id", job.ID, "url", job.URL)
	article, err := w.scraper.FetchArticle(ctx, job.ID, job.URL)
	if err != nil {
		w.logger.Warn("Failed to scrape article URL", "url", job.URL, "error", err)
		// Ack to prevent infinite loop on un-scrapable URLs
		msg.Ack()
		return err
	}

	event := article.ToMacroEvent()
	targetSubject := "macro.feed.v1.news.scraped"
	if err := w.publisher.PublishMacro(ctx, targetSubject, &event); err != nil {
		w.logger.Error("Failed to publish news scraped event to JetStream", "id", job.ID, "error", err)
		return err
	}

	w.logger.Info("Article scraped and published successfully", "id", job.ID, "title", article.Title, "media_url", article.MediaURL)
	msg.Ack()
	return nil
}
