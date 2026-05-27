# ATLSD Event Architecture

## Purpose

ATLSD uses versioned events as the backbone for institutional market intelligence. Raw data is persisted before processing, derived state is built by consumers, and important outputs can be replayed by topic, symbol, tenant, source, and time range.

## Event Envelope

All events use the shared `atlsd_contracts::EventEnvelope<T>` shape:

```json
{
  "event_id": "018f4fd3-6c21-7420-9c7a-33e18b2a10a1",
  "event_type": "md.canonical.ticks.v1",
  "schema_version": 1,
  "occurred_at": "2026-05-26T09:30:00.120Z",
  "published_at": "2026-05-26T09:30:00.184Z",
  "source": "market-data-service",
  "partition_key": "commodity:XAUUSD",
  "metadata": {
    "tenant_id": null,
    "trace": {
      "correlation_id": "018f4fd3-6c21-7420-9c7a-33e18b2a10a1",
      "causation_id": null,
      "raw_event_id": "018f4fd3-6c21-7420-9c7a-33e18b2a10a0",
      "pipeline_version": "market-normalizer@1.0.0"
    },
    "quality_flags": [],
    "replayed": false
  },
  "payload": {}
}
```

## Naming

Use this pattern:

```text
<domain>.<category>.<name>.v<major>
```

Examples:

```text
md.raw.finnhub.trades.v1
md.canonical.ticks.v1
news.enriched.article.v1
intelligence.why_move.generated.v1
tenant.entitlement.changed.v1
usage.api.requested.v1
```

## Partition Keys

Market data:

```text
{asset_class}:{symbol}
```

Tenant/platform events:

```text
{tenant_id}
```

Raw vendor events:

```text
{venue}:{symbol}
```

## Versioning

- Additive fields keep the same major version.
- Field removals, type changes, or semantic changes require a new topic version.
- Consumers must reject event versions they do not explicitly support.
- Schema registry compatibility must be checked before deployment.


## Event Backbone

ATLSD's preferred durable event backbone is NATS JetStream. Subjects use the same dot-separated names as event types, for example `md.canonical.ticks.v1` and `tenant.entitlement.changed.v1`.

Redis remains transitional for cache, counters, and compatibility pub/sub while services dual-publish to NATS during migration.

Suggested JetStream streams:

```text
ATLSD_MARKET       -> md.>
ATLSD_NEWS         -> news.>, social.>
ATLSD_INTELLIGENCE -> intelligence.>
ATLSD_PLATFORM     -> tenant.>, usage.>, audit.>, platform.>
```
## Replay

Replay must be explicit and observable:

- replayed events set `metadata.replayed = true`
- replay jobs use isolated consumer groups first
- replay scope must include topic, time range, and partition selector
- replay output must not overwrite production materialized views until validated

## Dead Letter Queues

Invalid or unprocessable events go to domain-specific DLQs:

```text
md.deadletter.events.v1
news.deadletter.events.v1
intelligence.deadletter.events.v1
platform.deadletter.events.v1
```

DLQ events include the original envelope, failure reason, processor name, processor version, and failure timestamp.

