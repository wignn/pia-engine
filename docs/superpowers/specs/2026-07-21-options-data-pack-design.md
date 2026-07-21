# Options Data Pack Specification

## Overview
The Options Data Pack adds support for ingesting, calculating analytics for, and serving institutional-grade Options market data (Call/Put Option Chains, Implied Volatility, Greeks, GEX/Gamma Exposure, Put/Call Ratio, and Max Pain) across both Crypto (Deribit) and US Stock/Index markets (Yahoo Finance).

All external HTTP polling is centralized within `services/ingestion-gateway`, keeping feed ingestion isolated behind a single gateway. Ingested payloads are calculated, normalized, and published over EventBus (`redis` / `nats`), then ingested and served via REST API by `services/market-data` and proxied through `services/api-gateway`.

---

## 1. Database Schema (`db/migrations/core/013_options_data_pack.sql`)

### `options_snapshots`
Stores aggregated summary statistics for a given underlying asset symbol.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | `TEXT` | `PRIMARY KEY` | Unique ID format `{symbol}` (e.g. `SPY`, `BTC`) |
| `symbol` | `TEXT` | `NOT NULL` | Normalized underlying symbol |
| `underlying_price` | `DOUBLE PRECISION` | `NOT NULL` | Current price of underlying |
| `put_call_ratio` | `DOUBLE PRECISION` | `NOT NULL` | Total Put Volume / Total Call Volume |
| `max_pain_strike` | `DOUBLE PRECISION` | `NOT NULL` | Strike with minimum option holder payout |
| `total_open_interest` | `BIGINT` | `NOT NULL` | Sum of open interest across calls & puts |
| `total_volume` | `BIGINT` | `NOT NULL` | Sum of volume across calls & puts |
| `total_gex` | `DOUBLE PRECISION` | `NOT NULL` | Aggregate dollar Gamma Exposure |
| `iv_atm` | `DOUBLE PRECISION` | | At-The-Money Implied Volatility |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | Timestamp of last update |

### `options_contracts`
Stores individual contract parameters, Greeks, GEX, and market data.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `contract_symbol` | `TEXT` | `PRIMARY KEY` | Canonical symbol (e.g., `BTC-26JUL26-65000-C` or `SPY260721C00550000`) |
| `symbol` | `TEXT` | `NOT NULL` | Underlying symbol |
| `option_type` | `TEXT` | `NOT NULL` | `call` or `put` |
| `strike` | `DOUBLE PRECISION` | `NOT NULL` | Option strike price |
| `expiration_date` | `DATE` | `NOT NULL` | Expiration date |
| `mark_price` | `DOUBLE PRECISION` | `NOT NULL` | Mark price / last price |
| `bid` | `DOUBLE PRECISION` | | Current bid price |
| `ask` | `DOUBLE PRECISION` | | Current ask price |
| `implied_volatility` | `DOUBLE PRECISION` | `NOT NULL` | Implied volatility (decimal, e.g., 0.25 = 25%) |
| `delta` | `DOUBLE PRECISION` | `NOT NULL` | Black-Scholes Delta (-1.0 to 1.0) |
| `gamma` | `DOUBLE PRECISION` | `NOT NULL` | Black-Scholes Gamma |
| `theta` | `DOUBLE PRECISION` | `NOT NULL` | Black-Scholes Theta |
| `vega` | `DOUBLE PRECISION` | `NOT NULL` | Black-Scholes Vega |
| `gex` | `DOUBLE PRECISION` | `NOT NULL` | Dollar Gamma Exposure ($) |
| `open_interest` | `BIGINT` | `NOT NULL` | Open interest contract count |
| `volume` | `BIGINT` | `NOT NULL` | Traded volume contract count |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | Timestamp of last update |

---

## 2. Ingestion & Analytical Calculations (`services/ingestion-gateway`)

### Providers
1. **Deribit (Crypto Options)**:
   - Endpoint: `https://www.deribit.com/api/v2/public/get_book_summary_by_currency?currency={CURRENCY}&kind=option`
   - Public REST, no API key required.
   - Assets: `BTC`, `ETH`.

2. **Yahoo Finance (Stock/Index Options)**:
   - Endpoint: `https://query2.finance.yahoo.com/v7/finance/options/{SYMBOL}`
   - Public REST with custom User-Agent.
   - Assets: `SPY`, `QQQ`, `AAPL`, `MSFT`, `TSLA`, `NVDA`.

### Financial Analytics Engines
- **Black-Scholes Greeks (Yahoo Finance fallback)**:
  - $d_1 = \frac{\ln(S / K) + (r + \frac{\sigma^2}{2}) T}{\sigma \sqrt{T}}$
  - $d_2 = d_1 - \sigma \sqrt{T}$
  - $\Delta_{\text{call}} = N(d_1), \quad \Delta_{\text{put}} = N(d_1) - 1$
  - $\Gamma = \frac{N'(d_1)}{S \sigma \sqrt{T}}$
  - $Vega = S N'(d_1) \sqrt{T}$
- **Gamma Exposure (GEX)**:
  - $GEX_{\text{call}} = \Gamma \times S \times 100 \times OI \times S$
  - $GEX_{\text{put}} = -\Gamma \times S \times 100 \times OI \times S$
- **Max Pain**:
  - Strike price $K$ that minimizes total cumulative intrinsic value payout:
    $\sum_{\text{calls}} \max(0, S_i - K) \times OI + \sum_{\text{puts}} \max(0, K - S_i) \times OI$
- **Put/Call Ratio**:
  - $PCR = \frac{\sum Volume_{\text{put}}}{\sum Volume_{\text{call}}}$

---

## 3. Storage & REST API Endpoints (`services/market-data`)

- `GET /api/v1/options/summary?symbol=SPY`
  - Returns `options_snapshots` for given underlying symbol.
- `GET /api/v1/options/chain?symbol=SPY&expiration=2026-07-25`
  - Returns list of `options_contracts` filtered by underlying symbol and optional expiration date.
- `GET /api/v1/options/gex?symbol=SPY`
  - Returns strike-level aggregated GEX breakdown for visualization.

---

## 4. Gateway Routing (`services/api-gateway`)

Protected routes added:
- `/api/v1/options/summary` -> proxies to `market-data`
- `/api/v1/options/chain` -> proxies to `market-data`
- `/api/v1/options/chain/{symbol}` -> proxies to `market-data`
- `/api/v1/options/gex` -> proxies to `market-data`
- `/api/v1/options/gex/{symbol}` -> proxies to `market-data`
