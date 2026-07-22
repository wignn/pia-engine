# Public Web Options Dashboard Specification

## Overview
This specification details the frontend component additions in `apps/public-web` to render options market data (Options Summary, Option Chains with Greeks, GEX Distribution Bar Chart, Put/Call Ratio, and Max Pain) directly on the main public web page (`apps/public-web/src/routes/+page.svelte`).

---

## 1. Component Architecture & Files (`apps/public-web/src/lib/components/`)

### 1. `OptionsDashboard.svelte`
Main container component embedding symbol selector controls (`SPY`, `QQQ`, `AAPL`, `NVDA`, `BTC`, `ETH`), managing loading states, and orchestrating child components.

### 2. `OptionsSummaryCards.svelte`
KPI tile row displaying 6 key metrics:
- Underlying Price
- Put/Call Ratio (with sentiment indicator: Bullish < 0.8, Neutral 0.8-1.2, Bearish > 1.2)
- Max Pain Strike
- ATM Implied Volatility (IV)
- Aggregate Open Interest & Volume
- Total Dollar Gamma Exposure (GEX)

### 3. `OptionsGexChart.svelte`
Visual strike-level Gamma Exposure (GEX) distribution bar chart:
- Positive Call GEX vs Negative Put GEX per strike.
- Interactive tooltip displaying strike, call GEX, and put GEX.

### 4. `OptionChainTable.svelte`
Full Options Chain table:
- Filter by Expiration Date dropdown.
- Columns: Contract, Strike, Type (Call/Put), Mark Price, Bid/Ask, IV, Delta, Gamma, Theta, Vega, OI, Volume.

### 5. `src/routes/+page.svelte`
Integration of `OptionsDashboard` into the main public landing page.

---

## 2. API Data Fetching (`src/lib/api.ts`)

Endpoints consumed via `apiFetch`:
- `GET /api/v1/options/summary?symbol={symbol}`
- `GET /api/v1/options/chain?symbol={symbol}&expiration={date}`
- `GET /api/v1/options/gex?symbol={symbol}`

---

## 3. Styling & Theme

Full dark/light mode compatibility using Tailwind CSS v4 variables: `bg-surface`, `bg-bg`, `border-border`, `text-text`, `text-text-muted`, and `bg-accent`.
