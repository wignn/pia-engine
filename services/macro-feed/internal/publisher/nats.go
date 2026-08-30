package publisher

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"time"

	"macro-feed/internal/model"

	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"
)

type JetStreamPublisher struct {
	nc     *nats.Conn
	js     jetstream.JetStream
	logger *slog.Logger
}

func NewJetStreamPublisher(natsURL string, logger *slog.Logger) (*JetStreamPublisher, error) {
	if logger == nil {
		logger = slog.Default()
	}

	opts := []nats.Option{
		nats.Timeout(10 * time.Second),
		nats.ReconnectWait(2 * time.Second),
		nats.MaxReconnects(-1),
		nats.DisconnectErrHandler(func(_ *nats.Conn, err error) {
			logger.Warn("NATS connection disconnected", "error", err)
		}),
		nats.ReconnectHandler(func(nc *nats.Conn) {
			logger.Info("NATS connection reconnected", "url", nc.ConnectedUrl())
		}),
		nats.ClosedHandler(func(_ *nats.Conn) {
			logger.Error("NATS connection closed permanently")
		}),
	}

	nc, err := nats.Connect(natsURL, opts...)
	if err != nil {
		return nil, fmt.Errorf("nats connect: %w", err)
	}

	js, err := jetstream.New(nc)
	if err != nil {
		nc.Close()
		return nil, fmt.Errorf("jetstream init: %w", err)
	}

	return &JetStreamPublisher{
		nc:     nc,
		js:     js,
		logger: logger,
	}, nil
}

func (p *JetStreamPublisher) EnsureStream(ctx context.Context, streamName string, subjects []string) error {
	cfg := jetstream.StreamConfig{
		Name:        streamName,
		Description: "Macro Economic Feeds",
		Subjects:    subjects,
		Storage:     jetstream.FileStorage,
		Retention:   jetstream.LimitsPolicy,
		MaxAge:      7 * 24 * time.Hour,
		Duplicates:  5 * time.Minute,
	}

	_, err := p.js.CreateOrUpdateStream(ctx, cfg)
	return err
}

func (p *JetStreamPublisher) SubscribeJobs(ctx context.Context, streamName, filterSubject, durableName string) (jetstream.Consumer, error) {
	stream, err := p.js.Stream(ctx, streamName)
	if err != nil {
		return nil, fmt.Errorf("get stream %s: %w", streamName, err)
	}

	cfg := jetstream.ConsumerConfig{
		Durable:       durableName,
		FilterSubject: filterSubject,
		AckPolicy:     jetstream.AckExplicitPolicy,
	}

	consumer, err := stream.CreateOrUpdateConsumer(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("create consumer %s: %w", durableName, err)
	}

	return consumer, nil
}

func (p *JetStreamPublisher) PublishBond(ctx context.Context, subject string, data *model.DashboardData) error {
	if data == nil {
		return errors.New("cannot publish nil bond data")
	}
	return p.publishWithMsgID(ctx, subject, data.MsgID(), data)
}

func (p *JetStreamPublisher) PublishMacro(ctx context.Context, subject string, data *model.MacroEvent) error {
	if data == nil {
		return errors.New("cannot publish nil macro event")
	}
	return p.publishWithMsgID(ctx, subject, data.EventID, data)
}

func (p *JetStreamPublisher) PublishRate(ctx context.Context, subject string, data *model.RateEvent) error {
	if data == nil {
		return errors.New("cannot publish nil rate data")
	}
	return p.publishWithMsgID(ctx, subject, data.MsgID(), data)
}

func (p *JetStreamPublisher) publishWithMsgID(ctx context.Context, subject, msgID string, data any) error {
	bytes, err := json.Marshal(data)
	if err != nil {
		return fmt.Errorf("marshal data: %w", err)
	}

	msg := &nats.Msg{
		Subject: subject,
		Data:    bytes,
		Header:  nats.Header{},
	}
	if msgID != "" {
		msg.Header.Set(jetstream.MsgIDHeader, msgID)
	}

	ack, err := p.js.PublishMsg(ctx, msg)
	if err != nil {
		return fmt.Errorf("publish to %s: %w", subject, err)
	}

	p.logger.Debug("Message published to JetStream",
		"subject", subject,
		"stream", ack.Stream,
		"seq", ack.Sequence,
		"duplicate", ack.Duplicate,
	)
	return nil
}

func (p *JetStreamPublisher) Close() {
	if p.nc != nil {
		p.nc.Drain()
	}
}
