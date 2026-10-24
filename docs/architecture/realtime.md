# Realtime Event Distribution

`services/realtime-gateway` fans out to WebSocket clients. NATS is the one
broadcast path for market ticks: ingestion publishes raw events to JetStream,
`market-data` materializes them and publishes deduplicated tick events, and
each realtime instance has its own Core NATS subscriptions to the broadcast
subjects. Core subscriptions copy each live event to every gateway instance;
they must not be replaced with one shared durable consumer, which would split
events between instances. The Unix socket tick path has been removed.

- **Durable raw market consumer** (`market-data-raw-v1`) with explicit ack
  after processing. A restart resumes from its last acknowledged raw event
  within the 24-hour market stream retention window.
- **Live WebSocket fanout** uses independent NATS subscriptions per instance;
  WebSocket broadcasts are ephemeral and reconnecting clients use snapshots
  for current market state.
- **Catch-up snapshots**: a client subscribing to market streams first
  receives the latest-price snapshot from market-data, then the live stream.
- **Backpressure**: a client whose send buffer fills is disconnected
  explicitly (alerted via `atlsd_realtime_ws_send_failures_total`) instead of
  silently accumulating a stale view.
- **Auth**: API key (constant-time compare) or single-use ticket (Redis
  `GETDEL`, 30s TTL).
- Alerts, news and social posts use the same per-instance Core NATS fanout.

Broadcast routing per client uses the stream grammar in `src/streams.rs`.
