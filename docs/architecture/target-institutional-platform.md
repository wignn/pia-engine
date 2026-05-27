# ATLSD Target Institutional Platform Architecture

## Purpose

ATLSD's long-term architecture should be an institutional-grade market intelligence platform, not a single core service that owns ingestion, APIs, WebSockets, chart history, enrichment, and database access. The target architecture is event-driven, domain-separated, replayable, observable, and designed around three equal priorities:

1. **Reliability** — raw data is durable, processing is replayable, and service failures do not lose market state.
2. **Low latency** — market ticks, alerting, and realtime streams reach clients quickly without blocking on slow analytics.
3. **Data quality** — every derived output has validation, lineage, anomaly detection, and correction semantics.

## Target Topology

```text
External Sources
  ├─ Market feeds
  ├─ RSS/news
  ├─ X/social
  ├─ Economic calendar
  └─ Vendor APIs

        │
        ▼

Source Connectors / Ingestion Edge
  ├─ market-feed-connectors
  ├─ news-connectors
  ├─ calendar-connectors
  └─ social-connectors

        │ raw events
        ▼

Durable Event Backbone
NATS JetStream
with subject conventions and contract validation

        │
        ├────────────────────┬────────────────────┬────────────────────┐
        ▼                    ▼                    ▼                    ▼

Market Data Domain    News Intelligence     Intelligence Domain   Control Plane
                      Domain

        │                    │                    │                    │
        ▼                    ▼                    ▼                    ▼

ClickHouse / Redis    Postgres/Search       Factor Store          Postgres
OHLC/Ticks            NLP/News Store        WhyMove/FearGreed     Tenant/Auth

        │                    │                    │                    │
        └──────────────┬─────┴────────────┬──────┴────────────┬──────┘
                       ▼                  ▼                   ▼

                 API Gateway / BFF     Realtime Gateway     Usage Metering

                       │                  │
                       ▼                  ▼

        Desktop Trader / Public Web / Admin / Bot / Public API
```

## Core Principles

See `docs/architecture/events.md` for the concrete event envelope, naming, versioning, replay, and DLQ policy that backs this target architecture.

### Event-first, not request-first

All external data enters as raw durable events before business processing. Domain services consume events, validate them, enrich them, and publish derived events or materialized views. Avoid a model where clients call one core service and that service synchronously performs ingestion, enrichment, persistence, and broadcast.

### Separate hot path and analytical path

The realtime market path must not block on NLP, LLM, scraping, or slow analytical workloads.

Hot path:

```text
vendor tick -> connector -> event backbone -> market-data-service -> realtime-gateway -> client
```

Analytical path:

```text
news/calendar/ticks -> enrichment/intelligence -> warehouse/materialized views -> API/query layer
```

### Core is transitional

`services/core` has been decommissioned in favor of dedicated domain services and `services/api-gateway` as the public REST entrypoint.

## Market Data Domain

### Services

```text
feed-connector-finnhub
feed-connector-tiingo
feed-connector-binance
feed-connector-rsshub-market
market-normalizer
market-data-service
realtime-gateway
```

Feed connectors own external vendor sessions, reconnect/backoff, sequence tracking, and raw event publishing. They do not serve client APIs.

`market-data-service` owns canonical market state:

- latest prices
- true OHLCV candles
- tick history
- symbol/session metadata
- volatility spikes
- data quality events
- feed health
- candle correction
- market snapshots

### Event topics

```text
md.raw.finnhub.trades.v1
md.raw.tiingo.quotes.v1
md.raw.binance.trades.v1
md.normalized.trades.v1
md.normalized.quotes.v1
md.normalized.book_top.v1
md.canonical.ticks.v1
md.canonical.ohlcv.1s.v1
md.canonical.ohlcv.1m.v1
md.canonical.ohlcv.5m.v1
md.canonical.ohlcv.1h.v1
md.quality.gaps.v1
md.quality.outliers.v1
md.quality.stale_feed.v1
md.realtime.public.v1
```

Partition raw topics by `{venue}:{symbol}` and canonical topics by `{asset_class}:{symbol}`.

### Market data APIs

```text
GET /v1/market/prices
GET /v1/market/prices/{symbol}
GET /v1/market/candles/{symbol}?resolution=1m&from=&to=&limit=
GET /v1/market/trades/{symbol}
GET /v1/market/quality/{symbol}
GET /v1/market/spikes
GET /v1/market/session/{symbol}
```

### Market data storage

ClickHouse:

```text
market_ticks
market_quotes
market_book_top
market_ohlcv_1s
market_ohlcv_1m
market_ohlcv_5m
market_ohlcv_1h
market_quality_events
```

Redis:

```text
latest_price:{symbol}
latest_book:{symbol}
rolling_window:{symbol}
feed_heartbeat:{source}
```

Postgres:

```text
instruments
venues
symbol_mappings
market_sessions
quality_policies
```

Redis is hot state only. ClickHouse is the authoritative market time-series store.

## Realtime Gateway

`realtime-gateway` is separate from API/query services.

Responsibilities:

- client WebSocket/SSE fanout
- tenant entitlement enforcement
- symbol/channel subscription validation
- connection limits
- per-client backpressure
- snapshot-on-subscribe
- replay token for short recovery windows
- clear close codes for throttling or authorization failures

Client subscribe example:

```json
{
  "op": "subscribe",
  "channels": ["price:XAUUSD", "ohlcv.1m:XAUUSD", "news:XAUUSD", "why_move:XAUUSD"]
}
```

Outbound envelope:

```json
{
  "type": "market.price",
  "symbol": "XAUUSD",
  "ts_exchange": "2026-05-26T09:30:00.120Z",
  "ts_received": "2026-05-26T09:30:00.184Z",
  "seq": 12345,
  "data": { "price": 2368.42, "source": "tiingo" }
}
```

## News Intelligence Domain

### Services

```text
news-service
analyzer-runtime
intelligence-service
llm-narrative-service
```

### news-service

Responsibilities:

- RSS/vendor news ingestion
- social/X ingestion
- calendar ingestion coordination
- deduplication
- source reliability scoring
- entity/symbol/currency linking
- normalized article store
- raw article archive

Topics:

```text
news.raw.article.v1
news.normalized.article.v1
news.enriched.article.v1
news.cluster.updated.v1
news.high_impact.detected.v1
social.raw.post.v1
social.enriched.post.v1
calendar.raw.event.v1
calendar.normalized.event.v1
calendar.impact.updated.v1
```

Storage:

```text
Postgres: articles, news_sources, article_entities, article_symbols, news_clusters
Object storage: raw/news/{source}/{date}/{event_id}.json
Search: OpenSearch / Meilisearch / Vespa
```

### analyzer-runtime

Internal Python model runtime for:

- FinBERT sentiment
- language detection
- translation if needed
- entity/ticker extraction
- topic classification
- relevance scoring
- impact scoring

Internal APIs:

```text
AnalyzerRuntime.AnalyzeText
AnalyzerRuntime.AnalyzeArticle
AnalyzerRuntime.BatchAnalyze
AnalyzerRuntime.ScoreAssetImpact
AnalyzerRuntime.ClassifyTopic
```

Every model output includes model name, model version, input hash, confidence, and timestamp.

### intelligence-service

Owns high-value intelligence features:

- Why Did It Move
- factor board
- fear/greed score
- sentiment summary
- cross-asset explanation
- market regime
- alert explanation
- narrative generation

Input topics:

```text
market.volatility_spike.detected.v1
market.candle.closed.v1
news.enriched.article.v1
calendar.impact.updated.v1
social.enriched.post.v1
```

Output topics:

```text
intelligence.why_move.generated.v1
intelligence.factor.updated.v1
intelligence.fear_greed.updated.v1
intelligence.sentiment_summary.updated.v1
```

Client APIs:

```text
GET /v1/intelligence/why/{symbol}
GET /v1/intelligence/factors/{symbol}
GET /v1/intelligence/fear-greed
GET /v1/intelligence/sentiment/summary
GET /v1/evidence/{evidence_id}
```

## Evidence Lineage

Every derived intelligence object must be explainable and auditable.

Required lineage fields:

```text
raw_payload_id
normalized_event_id
enrichment_id
model_name
model_version
prompt_template_version
source_id
source_reliability_score
pipeline_version
created_at
```

Example `WhyMoveExplanation`:

```json
{
  "symbol": "XAUUSD",
  "window": "15m",
  "price_move": { "direction": "up", "move_pct": 0.42 },
  "primary_drivers": [
    { "name": "USD weakness", "score": 0.86, "evidence_ids": ["news_123", "calendar_456"] }
  ],
  "confidence": { "score": 0.78, "label": "high" },
  "narrative": "Gold moved higher as USD weakness and lower yields amplified demand for defensive assets.",
  "evidence_ids": ["news_123", "calendar_456", "candle_789"],
  "model_versions": { "sentiment": "finbert@1.3", "narrative": "gemini-2.5-pro@2026-05" },
  "pipeline_version": "why-move@2.1.0"
}
```

## Factor Board

Core factors:

```text
news_sentiment
social_velocity
social_sentiment
calendar_risk
macro_surprise
usd_pressure
rates_pressure
volatility_regime
liquidity_stress
cross_asset_correlation
fear_greed
```

Factor output:

```json
{
  "symbol": "XAUUSD",
  "factor": "usd_pressure",
  "score": -72,
  "confidence": 0.81,
  "lookback": "30m",
  "evidence_ids": ["news_1", "dxy_move_2"],
  "updated_at": "2026-05-26T09:30:05Z"
}
```

Score range:

```text
-100 = strongly bearish
0    = neutral
+100 = strongly bullish
```

## Platform, Security, and Tenancy

### Services

```text
api-gateway
realtime-gateway
control-plane
entitlement-service
usage-metering-service
audit-service
```

### API Gateway

Responsibilities:

- TLS termination
- WAF/rate limits
- API key/JWT validation
- request routing
- quota enforcement
- tenant context injection
- audit event emission

### Entitlement Service

Control-plane remains source of truth, but runtime services should use entitlement snapshots/cache.

Flow:

```text
control-plane updates plan/api key
  -> emits entitlement.changed.v1
  -> entitlement cache updates
  -> api-gateway/realtime-gateway enforce
```

Example entitlement snapshot:

```json
{
  "tenant_id": "tenant_institutional_alpha",
  "allowed_symbols": ["XAUUSD", "EURUSD"],
  "allowed_channels": ["market.price", "news", "why_move"],
  "max_ws_connections": 10,
  "history_depth_days": 365,
  "can_use_llm": true
}
```

### Usage Metering

Metering must cover more than REST requests:

```text
API request count
WebSocket connection duration
WebSocket messages delivered
Subscribed symbols
Historical candle query volume
LLM/intelligence usage
News search usage
```

Events:

```text
usage.api.requested.v1
usage.ws.connected.v1
usage.ws.message_delivered.v1
usage.intelligence.generated.v1
usage.quota.exceeded.v1
```

## Observability

Required metrics:

```text
ingestion_lag_ms
event_bus_consumer_lag
tick_to_client_latency_ms
feed_reconnect_count
feed_gap_count
candle_close_lag_ms
websocket_connections
websocket_dropped_messages
api_latency_p95/p99
news_enrichment_latency
why_move_generation_latency
data_quality_score
tenant_quota_denials
```

SLO examples:

```text
Market tick p95 end-to-end latency < 500ms
Latest price REST p95 < 50ms
OHLC query p95 < 500ms
Candle close emitted < 2s after bucket end
WebSocket reconnect recovery < 5s
Raw event durability 99.99%
API availability 99.95%
Audit event loss target: zero
```

## Reliability Model

Required guarantees:

- raw event persisted before processing
- at-least-once processing
- idempotent consumers
- deterministic candle generation
- dead-letter queues
- replay by topic/source/symbol/time range
- late tick correction
- consumer lag monitoring
- disaster recovery snapshots

Candle lifecycle topics:

```text
market.candle.closed.v1
market.candle.corrected.v1
```

Late ticks should emit correction events instead of silently rewriting historical candles.

## Deployment Topology

```text
Edge/load balancer
  ├─ api-gateway replicas
  ├─ realtime-gateway replicas
  └─ admin/control-plane replicas

Internal services
  ├─ market-data-service replicas
  ├─ news-service workers
  ├─ intelligence-service workers
  ├─ calendar-service workers
  ├─ usage-metering workers
  └─ analyzer/LLM workers

Data plane
  ├─ NATS JetStream
  ├─ Redis cluster
  ├─ ClickHouse cluster
  ├─ Postgres HA
  └─ Object storage
```

## Migration Roadmap

### Phase 1 — Contracts First

- Define event schemas.
- Define canonical topic names.
- Define service boundaries.
- Keep current runtime mostly intact.

Deliverables:

```text
crates/atlsd-contracts
docs/architecture/events.md
docs/architecture/target-institutional-platform.md
```

### Phase 2 — Market Data Extraction

- Create `services/market-data`.
- Move latest prices, OHLC, spikes, sessions, and data-quality out of core.
- Add true candle API.
- Move ClickHouse writes behind market-data service.
- Desktop uses market-data APIs.

### Phase 3 — Realtime Gateway

- Create `services/realtime-gateway`.
- Move WebSocket fanout from core.
- Add tenant entitlement enforcement.
- Support snapshot + delta + replay token.

### Phase 4 — News and Intelligence Split

- Create `services/news-service`.
- Create `services/intelligence-service`.
- Analyzer becomes internal model runtime.
- Why Move, factors, fear/greed, and sentiment summary leave core.

### Phase 5 — Platform Hardening

- Add API gateway.
- Add entitlement-service.
- Add usage-metering-service.
- Add audit-service.
- Add schema registry.

### Phase 6 — Institutional Backbone

- Move from Redis Streams/pubsub to NATS JetStream for durable domain events while keeping Redis for cache/counter workloads.
- Add replay tooling.
- Add disaster recovery strategy.
- Add multi-region edge for realtime.

## Recommended Final Service Map

```text
services/
  api-gateway/
  realtime-gateway/
  market-data/
  market-connectors/
    finnhub/
    tiingo/
    binance/
  news-service/
  intelligence-service/
  analyzer-runtime/
  calendar-service/
  control-plane/
  entitlement-service/
  usage-metering/
  audit-service/
  bot/
```

Shared crates:

```text
crates/
  atlsd-contracts
  atlsd-auth
  atlsd-domain
  atlsd-observability
  atlsd-eventbus
  atlsd-market
  atlsd-tenant
```

## Final Direction

ATLSD should evolve into an **event-driven institutional market intelligence platform with separate market-data, realtime, news, intelligence, and control-plane domains**.

The key shift is:

```text
Core Service as the platform
```

becomes:

```text
Event backbone + domain services as the platform
```

Every new feature should be designed against that target: clear domain ownership, versioned event contracts, replayable data, deterministic materialized views, explicit evidence lineage, and tenant enforcement at API/realtime boundaries.


