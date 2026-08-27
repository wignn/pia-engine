# Data ingestion architecture

ATLSD separates source collection, event transport, materialization, and query serving. This keeps provider-specific behavior out of the market hot path and makes failures visible at each boundary.

## Market ingestion

`services/ingestion-gateway` manages configured FX, crypto, index, and options feed sessions. It normalizes provider messages and publishes versioned raw market events to NATS JetStream through bounded per-worker queues. Queue pressure and dropped events are observable and should be treated as an ingestion incident.

`services/market-data` consumes raw market events and applies validation, deduplication, and per-symbol provider-timestamp monotonicity checks. It then:

- materializes latest prices in PostgreSQL,
- batches the market tick tape into ClickHouse,
- builds or reads OHLCV history,
- resolves market sessions and holiday rules,
- exposes market, quality, spike, and history APIs,
- publishes derived market events for downstream consumers.

Latest reference prices and historical candle data have different freshness and session semantics. A current quote can exist while a market is closed; candle writes must honor the resolved exchange session.

## Macro ingestion

`services/macro-feed` is a Go producer service. It collects:

- FRED Treasury rates and configured rate spreads,
- TradingEconomics government-bond snapshots when configured and available.

It validates collected data and publishes versioned macro events to NATS. The polling worker uses bounded retry/backoff behavior when a provider is unavailable.

`services/sink-connector` is the durable macro consumer. It validates and decodes macro event payloads, then writes:

- rates to `macro_rates`,
- spreads to `macro_rate_spreads`,
- series and observations to `macro_series` and `macro_observations`,
- bond snapshots to `macro_bonds` as raw JSON,
- scraped-news updates to the relevant news records.

The stored `macro_bonds` snapshot is not currently exposed by a dedicated public REST route. It must not be described as a queryable API until a stable response contract and route are implemented.

## News and social ingestion

`services/news-service` owns the news and calendar pipeline, including RSS/vendor sources and configured SEC, central-bank, geopolitical, and macro processing. Transactional outbox and event contracts are used where configured so database state and event publication can be reconciled.

`services/social-worker` polls configured social/X-compatible sources and publishes social events. It is an optional worker and requires its own provider configuration.

Article content extraction is not a current `services/scrapy` workspace service. Where scraping/backfill is used, follow the implementation and deployment configuration of the current news pipeline instead of relying on the old service name.

## Event transport

NATS JetStream is the durable event backbone. Consumers should:

1. validate the envelope and supported schema version,
2. reject malformed or unsupported messages explicitly,
3. process messages idempotently,
4. acknowledge only after the materialization or downstream action succeeds,
5. route invalid or repeatedly failing messages to the configured domain DLQ.

Redis is used for cache, counters, quota state, short-lived WebSocket tickets, hot state, and transitional compatibility pub/sub. It is not a replacement for the durable event stream.

## Observability and recovery

Monitor each ingestion boundary independently:

- provider connection and refresh status,
- event publish success/failure and queue drops,
- consumer lag and acknowledgements,
- database write failures,
- ClickHouse batch failures,
- data freshness and quality events,
- downstream WebSocket delivery and backpressure.

A provider outage should produce a visible stale/partial state rather than silently fabricating current data. Recovery procedures must be tested against the target environment and must not include credentials or raw production payloads in logs or documentation.

See [`events.md`](events.md) for topic/version rules and [`overview.md`](overview.md) for the current deployment topology.
