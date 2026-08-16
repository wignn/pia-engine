# Realtime Event Distribution

`services/realtime-gateway` fans out to WebSocket clients:

- **Durable JetStream consumers** (`realtime-news`, `realtime-market`,
  `realtime-candles`) with explicit acks — a restart resumes where it left
  off instead of losing the outage window.
- **Catch-up snapshots**: a client subscribing to market streams first
  receives the latest-price snapshot from market-data, then the live stream.
- **Backpressure**: a client whose send buffer fills is disconnected
  explicitly (alerted via `atlsd_realtime_ws_send_failures_total`) instead of
  silently accumulating a stale view.
- **Auth**: API key (constant-time compare) or single-use ticket (Redis
  `GETDEL`, 30s TTL).
- Alerts and social posts remain fire-and-forget core-NATS subjects.

Broadcast routing per client uses the stream grammar in `src/streams.rs`.
