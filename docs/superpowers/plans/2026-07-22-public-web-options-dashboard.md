# Public Web Options Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and render an Options Analytics Dashboard in `apps/public-web` displaying KPI summary tiles (PCR, Max Pain, ATM IV, GEX), GEX distribution bar charts, and an Option Chain table with Greeks on the main landing page (`+page.svelte`).

**Architecture:** Create Svelte 5 components (`OptionsSummaryCards.svelte`, `OptionsGexChart.svelte`, `OptionChainTable.svelte`, `OptionsDashboard.svelte`) in `apps/public-web/src/lib/components/`, define Options TypeScript types in `apps/public-web/src/lib/types.ts`, and embed `OptionsDashboard` into `apps/public-web/src/routes/+page.svelte`. Data is fetched via `apiFetch` from `services/api-gateway`.

**Tech Stack:** Svelte 5 (Runes `$state`), TypeScript, Tailwind CSS v4, Lucide Svelte icons.

## Global Constraints

- File paths under `apps/public-web/`
- Standard TypeScript & Svelte syntax, no compilation/type errors (`npm run check` in `apps/public-web`)
- Full light & dark mode support via Tailwind design tokens (`bg-surface`, `border-border`, `text-text`, `text-text-muted`, `bg-accent`)

---

### Task 1: Options Data Types (`apps/public-web/src/lib/types.ts`)

**Files:**
- Modify: `apps/public-web/src/lib/types.ts`

**Interfaces:**
- Consumes: Options REST API responses (`/api/v1/options/summary`, `/api/v1/options/chain`, `/api/v1/options/gex`).
- Produces: `OptionsSnapshot`, `OptionsContract`, `OptionsGexItem` TypeScript interfaces.

- [ ] **Step 1: Append Options interfaces to types.ts**

```typescript
export interface OptionsSnapshot {
	id: string;
	symbol: string;
	underlying_price: number;
	put_call_ratio: number;
	max_pain_strike: number;
	total_open_interest: number;
	total_volume: number;
	total_gex: number;
	iv_atm: number | null;
	updated_at: string;
}

export interface OptionsContract {
	contract_symbol: string;
	symbol: string;
	option_type: 'call' | 'put' | string;
	strike: number;
	expiration_date: string;
	mark_price: number;
	bid: number | null;
	ask: number | null;
	implied_volatility: number;
	delta: number;
	gamma: number;
	theta: number;
	vega: number;
	gex: number;
	open_interest: number;
	volume: number;
	updated_at: string;
}

export interface OptionsGexItem {
	strike: number;
	call_gex: number;
	put_gex: number;
	total_gex: number;
}
```

- [ ] **Step 2: Verify type checking**

Run: `cd apps/public-web && npm run check`
Expected: PASS with 0 errors

- [ ] **Step 3: Commit**

```bash
git -C apps/public-web add src/lib/types.ts
git -C apps/public-web commit -m "feat(types): add options snapshot, contract, and gex interfaces"
```

---

### Task 2: Options UI Sub-Components (`apps/public-web/src/lib/components/`)

**Files:**
- Create: `apps/public-web/src/lib/components/OptionsSummaryCards.svelte`
- Create: `apps/public-web/src/lib/components/OptionsGexChart.svelte`
- Create: `apps/public-web/src/lib/components/OptionChainTable.svelte`

**Interfaces:**
- Consumes: `OptionsSnapshot`, `OptionsContract`, `OptionsGexItem` from `$lib/types`.
- Produces: Visual KPI cards, GEX strike distribution chart, and Option Chain table.

- [ ] **Step 1: Create OptionsSummaryCards.svelte**

Render KPI tiles for Underlying Price, Put/Call Ratio, Max Pain Strike, ATM IV, Total Open Interest & Volume, Total Dollar GEX.

- [ ] **Step 2: Create OptionsGexChart.svelte**

Render strike-by-strike GEX bar chart (Call GEX vs Put GEX).

- [ ] **Step 3: Create OptionChainTable.svelte**

Render Expiration Date filter dropdown and Option Chain table displaying contract fields & Greeks (Delta, Gamma, Theta, Vega).

- [ ] **Step 4: Verify type checking**

Run: `cd apps/public-web && npm run check`
Expected: PASS with 0 errors

- [ ] **Step 5: Commit**

```bash
git -C apps/public-web add src/lib/components/OptionsSummaryCards.svelte src/lib/components/OptionsGexChart.svelte src/lib/components/OptionChainTable.svelte
git -C apps/public-web commit -m "feat(ui): add Options summary, GEX chart, and chain table components"
```

---

### Task 3: Options Dashboard Container & Main Page Integration

**Files:**
- Create: `apps/public-web/src/lib/components/OptionsDashboard.svelte`
- Modify: `apps/public-web/src/routes/+page.svelte`

**Interfaces:**
- Consumes: Options REST API via `apiFetch`, child options components.
- Produces: Integrated Options Analytics Section on main public landing page.

- [ ] **Step 1: Create OptionsDashboard.svelte**

Manage symbol selection (`SPY`, `QQQ`, `AAPL`, `NVDA`, `BTC`, `ETH`), fetch summary, chain, and gex endpoints, handle loading/error states, and render child components.

- [ ] **Step 2: Embed OptionsDashboard in +page.svelte**

Import `OptionsDashboard` and render it as a section on the main landing page (`apps/public-web/src/routes/+page.svelte`).

- [ ] **Step 3: Run check & build in apps/public-web**

Run: `cd apps/public-web && npm run check && npm run build`
Expected: PASS with 0 errors

- [ ] **Step 4: Commit**

```bash
git -C apps/public-web add src/lib/components/OptionsDashboard.svelte src/routes/+page.svelte
git -C apps/public-web commit -m "feat(page): embed Options Dashboard in public web landing page"
```
