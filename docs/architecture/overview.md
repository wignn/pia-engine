# ATLSD Architecture Overview

ATLSD is an event-driven, multi-tenant market intelligence platform. The
authoritative plan for ongoing work is `docs/architecture/REMEDIATION_PLAN.md`;
the target-state design lives in `target-institutional-platform.md`.

## Architecture Layers

1. **Frontend Applications (`apps/`)** — both are git submodules:
   - `pia`: Next.js dashboard for SaaS tenant control.
   - `public-web`: SvelteKit public marketing site.
2. **Rust services (`services/`)**:
   - `ingestion-gateway`: vendor feeds (FX/crypto/index/options) → normalized
     `md.raw.*` events on NATS JetStream.
   - `market-data`: consumes raw ticks, dedup + monotonic guard, persists to
     Postgres (latest prices) and ClickHouse (tick tape), builds candles
     (in-memory engine + ClickHouse materialized views), serves market REST.
   - `news-service`: RSS/vendor news pipeline, economic calendar, scrapes via
     `scrapy`, publishes domain events through the transactional outbox.
   - `intelligence-service`: sentiment, Why-Did-It-Move, factor analysis;
     calls the Python `analyzer` runtime.
   - `realtime-gateway`: WebSocket fanout to clients; durable JetStream
     consumers, catch-up snapshots, Redis-backed single-use tickets.
   - `api-gateway`: public REST entrypoint (API keys, quotas, proxy to domain
     services with the internal shared secret).
   - `control-plane`: SaaS users, plans, API keys, OAuth.
   - `scrapy`: article extraction worker (NATS `scrape.jobs` → `scrape.results`).
   - `bot`: Discord delivery.
   - `db-migrate`: versioned migration runner (Postgres + ClickHouse) with
     schema history, checksums, and baselining.
3. **Python services**:
   - `analyzer`: FinBERT/sentiment runtime (internal, called by intelligence-service).
   - `social-worker`: Twitter/TruthSocial poller → `social.posts`.
4. **Shared crates (`crates/`)**: `atlsd-contracts` (event contracts),
   `atlsd-eventbus` (JetStream publishers/streams/consumers), `atlsd-auth`
   (internal auth, JWT, API keys), `atlsd-common`, `atlsd-domain`,
   `atlsd-observability` (tracing + Prometheus metrics registry).

## Datastores

- **PostgreSQL** (`core`): SaaS state, news, market latest prices, outbox,
  dead-letter batches.
- **ClickHouse** (`market`): tick tape (`price_ticks`), OHLCV materialized
  from ticks (`ohlcv_1m` + 5m/15m/1h rollups).
- **NATS JetStream**: durable domain events (market raw, market dedup +
  candles, news, intelligence, platform).
- **Redis**: cache, counters, WS tickets, compatibility pub/sub.

## Entry points

Public REST: api-gateway (`:8000`) · WebSocket: realtime-gateway (`:8020`) ·
Admin: control-plane (`:8081`) · Public site: public-web (`:5173`).
All other services and datastores are internal-only.
