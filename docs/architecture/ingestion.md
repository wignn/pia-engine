# Data Ingestion Engine

Vendor feeds enter through `services/ingestion-gateway` (FX primary/secondary,
crypto, index, options). Ticks are normalized and published as `md.raw.*`
events to NATS JetStream (stream `ATLSD_MARKET`, dedup window 120s) via a
bounded per-worker publish queue (drops are counted and alerted on).

`services/market-data` consumes `md.raw.*`, applies the per-symbol monotonic
provider-timestamp guard, then: persists latest prices to Postgres, batches
the tick tape to ClickHouse (retry + dead-letter on failure), and feeds the
candle engine. News ingestion (RSS/GDELT/Finnhub/SEC/central banks) lives in
`services/news-service`; article extraction is delegated to `services/scrapy`
via the `scrape.jobs` / `scrape.results` contract in `atlsd-contracts`.

See `events.md` for the topic grammar and `REMEDIATION_PLAN.md` Fase 1 for
the durability guarantees.
