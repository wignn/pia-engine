# Free Institutional Data Pack v1 Design

## Purpose

ATLSD needs more Bloomberg-like data coverage without relying on paid-only market data vendors. This pack adds seven free/resmi/public-source backend data domains that strengthen macro, equity intelligence, geosignals, energy, positioning, and risk-regime coverage:

1. Rates / Yield Curve
2. SEC EDGAR Filings
3. Central Bank Monitor
4. GDELT Geosignals
5. EIA Energy Data
6. CFTC COT Positioning
7. Fear & Greed / Risk Regime Index

The pack deliberately skips data that is paid-only or licensing-heavy for a production product: real-time regulated futures, OPRA options, Level 2 order books, full individual bond pricing, institutional ETF holdings feeds, and analyst ratings.

## Goals

- Add production-ready ingestion for the seven selected free data domains.
- Store raw payloads where useful for audit/debugging and normalized rows for API reads.
- Expose REST endpoints through the owning service and API gateway.
- Keep ingestion idempotent, bounded, observable, and safe to retry.
- Return cached/stale data with clear status when upstream sources fail.
- Avoid fake data. If a source is disabled or unavailable, report that status explicitly.

## Non-goals

- No new paid vendor integration.
- No new standalone service for v1.
- No options chain, futures chain, regulated order book, or full bond pricing.
- No AI summarization requirement in v1; deterministic summaries/classifiers are acceptable where useful.
- No frontend/UI implementation in this spec.

## Placement

Use existing domain services first:

```text
services/market-data
  src/rates.rs
  src/energy.rs
  src/cot.rs
  src/fear_greed.rs

services/news-service
  src/sec.rs
  src/central_bank.rs
  src/gdelt.rs
```

Rationale:

- `market-data` owns market/macro-adjacent numeric data: rates, energy, COT, and the composite fear/greed index.
- `news-service` owns document/event intelligence: SEC filings, central bank documents, GDELT/geosignals.
- A new service would add operational overhead before the source volume requires it.

## Shared source pattern

Each source follows the same simple flow:

```text
source adapter
→ fetch upstream data with timeout/user-agent/rate limit
→ parse raw payload
→ normalize rows
→ idempotent upsert
→ update source status
→ expose REST endpoint
```

Every source must record:

- `source`
- `last_success_at`
- `last_error_at`
- `last_error`
- `last_latency_ms`
- `stale` status where applicable

A source failure must not silently return an empty success response if stale data exists.

## Feature 1 — Rates / Yield Curve

### Source

Initial source: FRED, using the existing FRED integration/config.

Initial US series:

- `DGS3MO` — 3-month Treasury yield
- `DGS2` — 2-year Treasury yield
- `DGS5` — 5-year Treasury yield
- `DGS10` — 10-year Treasury yield
- `DGS30` — 30-year Treasury yield
- `DFII10` — 10-year real yield
- `T10YIE` — 10-year breakeven inflation
- `T10Y2Y` — 10-year minus 2-year spread

### Data model

```text
macro_rates
- source text
- country text
- tenor text
- date date
- value double precision
- unit text
- raw_series_id text
- created_at timestamptz
- updated_at timestamptz
unique(source, country, tenor, date)

macro_rate_spreads
- country text
- spread text
- date date
- value double precision
- created_at timestamptz
- updated_at timestamptz
unique(country, spread, date)
```

### API

```text
GET /api/v1/rates/yield-curve?country=US
GET /api/v1/rates/spreads?country=US
GET /api/v1/rates/history/{tenor}?country=US&limit=252
```

### Response requirements

Yield curve returns the latest point per tenor:

```json
{
  "country": "US",
  "source": "fred",
  "date": "2026-07-20",
  "points": [
    { "tenor": "3M", "value": 4.9, "unit": "percent" },
    { "tenor": "2Y", "value": 4.1, "unit": "percent" }
  ],
  "spreads": [
    { "spread": "2s10s", "value": -0.2, "unit": "percentage_points" }
  ],
  "stale": false,
  "updated_at": "2026-07-21T00:00:00Z"
}
```

## Feature 2 — SEC EDGAR Filings

### Source

Use official SEC APIs:

- company ticker map
- submissions endpoint per CIK

Requests must use a SEC-compliant User-Agent from config:

```text
SEC_USER_AGENT="ATLSD contact@example.com"
```

If `SEC_USER_AGENT` is missing, SEC sync is disabled and status reports configuration required.

### Data model

```text
sec_companies
- cik text primary key
- ticker text
- name text
- exchange text null
- updated_at timestamptz
unique(ticker)

sec_filings
- accession_number text primary key
- cik text
- ticker text null
- form_type text
- filing_date date
- report_date date null
- primary_document text
- document_url text
- title text
- raw_json jsonb
- created_at timestamptz
- updated_at timestamptz
unique(accession_number)
```

### API

```text
GET /api/v1/sec/filings?symbol=AAPL&form=10-K&limit=20
GET /api/v1/sec/filings/{accession_number}
GET /api/v1/sec/companies/{symbol}
```

### Behavior

- Normalize symbols uppercase.
- Validate `limit` with bounded default, e.g. default 20, max 100.
- Store SEC accession numbers without fake/generated IDs.
- Document URLs are derived from CIK/accession/primary document according to SEC URL format.

## Feature 3 — Central Bank Monitor

### Sources

Use official public pages/RSS where available:

- Federal Reserve
- ECB
- BoE
- BoJ
- RBA
- SNB
- BIS speeches where useful

### Data model

```text
central_bank_sources
- bank text
- name text
- url text
- source_type text
- active boolean
- created_at timestamptz
- updated_at timestamptz
unique(bank, url)

central_bank_documents
- id text primary key
- bank text
- document_type text
- title text
- url text
- published_at timestamptz null
- summary text null
- stance text
- confidence double precision
- raw_text text null
- raw_json jsonb null
- created_at timestamptz
- updated_at timestamptz
unique(bank, url)
```

`document_type` values:

```text
rate_decision
minutes
speech
statement
press_release
balance_sheet
```

`stance` values:

```text
hawkish
dovish
neutral
unknown
```

### API

```text
GET /api/v1/central-banks/latest?limit=50
GET /api/v1/central-banks/{bank}/documents?type=speech&limit=50
GET /api/v1/central-banks/{bank}/stance
```

### Stance classifier

Use a deterministic keyword score in v1:

- hawkish terms: inflation pressure, restrictive, higher for longer, tighten, upside risks
- dovish terms: easing, cut, downside risks, unemployment, growth slowdown, accommodative

Return `unknown` when text is missing or scores are too close.

## Feature 4 — GDELT Geosignals

### Source

Use GDELT 2.1 public APIs. The existing geosignals endpoints remain the user-facing API.

### Data model

Reuse existing `news.geosignals` for normalized events. Add raw storage if absent:

```text
geosignal_raw_events
- source text
- event_id text
- fetched_at timestamptz
- raw_json jsonb
unique(source, event_id)
```

### Normalized fields

Normalize into current geosignal shape:

- `event_id`
- `timestamp`
- `source`
- `source_url`
- `title`
- `summary`
- `category`
- `country`
- `region`
- `location_scope`
- `severity_score`
- `sentiment_score`
- `confidence_score`
- `affected_assets`
- `asset_impact`
- `freshness`

### API

Existing:

```text
GET /api/v1/geosignals
GET /api/v1/geosignals/map
GET /api/v1/geosignals/assets
```

Add:

```text
GET /api/v1/geosignals/status
```

### Asset mapping

Use a conservative rule map first:

- war/sanctions/geopolitical risk → XAUUSD, DXY, SPX
- oil supply/energy disruption → CL proxy symbols, XLE/USO if configured
- disaster affecting crop regions → agriculture commodity labels if configured
- country-specific risk → local index/currency mappings where configured

Do not invent price impact. Store affected assets as hypotheses with confidence.

## Feature 5 — EIA Energy Data

### Source

Use EIA API for public energy series. If EIA API key is required in the deployed environment and missing, disable sync with explicit status.

Config:

```text
EIA_API_KEY=optional
EIA_SYNC_SEC=21600
```

### Data model

```text
energy_series
- id text primary key
- source text
- name text
- commodity text
- unit text
- frequency text
- active boolean
- created_at timestamptz
- updated_at timestamptz

energy_observations
- series_id text
- date date
- value double precision
- raw_json jsonb null
- created_at timestamptz
- updated_at timestamptz
unique(series_id, date)
```

### Initial series categories

- crude inventories
- gasoline inventories
- distillate inventories
- refinery utilization
- crude production
- natural gas storage

### API

```text
GET /api/v1/energy/series
GET /api/v1/energy/{series_id}?limit=260
GET /api/v1/energy/dashboard
```

Dashboard returns latest values and week-over-week changes where enough history exists.

## Feature 6 — CFTC COT Positioning

### Source

Use public CFTC COT reports.

### Data model

```text
cot_reports
- market_code text
- market_name text
- report_date date
- report_type text
- commercial_long bigint null
- commercial_short bigint null
- noncommercial_long bigint null
- noncommercial_short bigint null
- nonreportable_long bigint null
- nonreportable_short bigint null
- open_interest bigint null
- created_at timestamptz
- updated_at timestamptz
unique(report_type, market_code, report_date)

cot_market_map
- market_code text primary key
- symbol text
- asset_class text
- display_name text
```

### Derived fields

Compute in API response, not stored in v1 unless needed later:

- `commercial_net = commercial_long - commercial_short`
- `noncommercial_net = noncommercial_long - noncommercial_short`
- week-over-week net changes
- percentile only when enough history exists

### API

```text
GET /api/v1/cot/markets
GET /api/v1/cot/{market_code}?limit=156
GET /api/v1/cot/symbol/{symbol}?limit=156
```

## Feature 7 — Fear & Greed / Risk Regime Index

### Source

Do not depend on a proprietary Fear & Greed feed. Build ATLSD's own composite index from data ATLSD already stores or can fetch from free/public sources.

Initial components:

- market momentum from existing price/candle history
- volatility pressure from existing volatility spikes
- safe-haven pressure from DXY/XAUUSD/rates data where available
- rates stress from yield curve/spread data
- news/geosignal risk from existing news sentiment and geosignal severity
- positioning stress from CFTC COT once available

### Data model

```text
fear_greed_index
- id text primary key
- scope text
- date timestamptz
- score double precision
- label text
- components jsonb
- source_status jsonb
- created_at timestamptz
unique(scope, date)
```

`scope` values for v1:

```text
global
fx
crypto
stocks
commodities
```

`label` values:

```text
extreme_fear
fear
neutral
greed
extreme_greed
```

### API

```text
GET /api/v1/fear-greed?scope=global
GET /api/v1/fear-greed/history?scope=global&limit=365
GET /api/v1/fear-greed/components?scope=global
```

### Scoring

Use a transparent weighted score from 0 to 100:

```text
0   = extreme fear
50  = neutral
100 = extreme greed
```

V1 weights:

- momentum: 25%
- volatility/spikes: 20%
- safe-haven/rates stress: 20%
- news/geosignal risk: 20%
- positioning/COT: 15%

If a component is unavailable, re-normalize weights across available components and include the missing component in `source_status`. Do not invent component values.

## Sync scheduling

Each domain gets a small background loop inside its owning service:

```text
startup
→ sync once
→ sleep configured interval
→ repeat
```

Default intervals:

```text
RATES_SYNC_SEC=21600
SEC_SYNC_SEC=21600
CENTRAL_BANK_SYNC_SEC=1800
GDELT_SYNC_SEC=900
EIA_SYNC_SEC=21600
COT_SYNC_SEC=21600
FEAR_GREED_SYNC_SEC=3600
```

Sync loops must:

- use configured HTTP timeouts
- avoid panics on malformed upstream data
- skip malformed records with warning and continue when safe
- update source status on success/failure
- be safe to run repeatedly

## API gateway routes

Add protected proxy routes:

```text
/api/v1/rates/*
/api/v1/sec/*
/api/v1/central-banks/*
/api/v1/energy/*
/api/v1/cot/*
/api/v1/fear-greed*
/api/v1/geosignals/status
```

## Error handling

- Invalid query parameters return a structured error with non-200 status.
- Upstream failures during sync do not break service startup.
- API reads return stale data with metadata when available.
- If no cached data exists, return a clear source-unavailable response.
- Optional sources disabled by missing config report disabled status instead of failing startup.

## Testing

Each module needs one small runnable check for parsing/normalization:

- rates: FRED observation normalization and spread calculation
- SEC: accession/document URL normalization
- central bank: stance classifier
- GDELT: category/asset mapping normalization
- EIA: observation parsing and WoW dashboard change
- COT: CSV/fixed-format row parsing and net position calculation
- fear/greed: component normalization, missing-component weight rebalancing, and label mapping

No broad framework changes are required.

## Verification

Before shipping implementation:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Manual smoke checks after services run:

```bash
curl http://localhost:8000/api/v1/rates/yield-curve
curl http://localhost:8000/api/v1/sec/filings?symbol=AAPL
curl http://localhost:8000/api/v1/central-banks/latest
curl http://localhost:8000/api/v1/geosignals/status
curl http://localhost:8000/api/v1/energy/dashboard
curl http://localhost:8000/api/v1/cot/markets
```

## Rollout order

Implement in this order:

1. Rates / Yield Curve
2. SEC EDGAR Filings
3. Central Bank Monitor
4. GDELT Geosignals status + ingestion
5. EIA Energy Data
6. CFTC COT Positioning
7. Fear & Greed / Risk Regime Index

Rates comes first because ATLSD already has FRED integration. SEC and central bank follow because they are official public sources and add high-value document intelligence. GDELT, EIA, and COT then deepen geosignal, energy, and positioning coverage. Fear/greed comes after the source components exist so it can be computed from real inputs instead of placeholders.

## Deliberate skips

Skipped for v1:

- real-time CME/regulated futures
- OPRA options chain
- Level 2 order book / DOM
- full individual bond quotes
- institutional ETF holdings and NAV feeds
- analyst ratings and price targets
- AIS/flight/satellite-derived commercial data

Add these only when a paid provider contract, licensing constraints, and target customer need are clear.
