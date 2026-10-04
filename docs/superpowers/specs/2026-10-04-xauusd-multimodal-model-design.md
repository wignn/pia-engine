# XAUUSD Multimodal Forecasting Model

**Status:** Draft for user review
**Date:** 2026-10-04

## Goal

Build an advanced research model for XAUUSD that estimates the probability and distribution of the one-hour forward return from a 15-minute decision point. It should combine price history with time-aligned news, social/geopolitical signals, macro data, COT positioning, options, and other relevant stored signals.

The first release is for offline research and paper evaluation. It does not place, route, or manage live orders and it makes no profitability claim.

## Current data evidence

Read-only inspection of the running stack on 2026-10-04 found:

| Source | Observed state | Design consequence |
| --- | --- | --- |
| XAUUSD candles | 2,672 unique 15-minute buckets on 37 active days, 2026-08-21 to 2026-10-02 | Suitable for pipeline/model experiments; short for a robust financial conclusion. |
| XAUUSD raw ticks | About 49.4 million rows, 2026-09-01 to 2026-10-02; raw tick table has a 30-day TTL | Keep model training on candle data and preserve any needed raw samples outside the expiring table. |
| Forex news | 6,512 articles; 276 analyses explicitly tagged XAU/gold and 565 title/summary mentions of gold | Use event-time news features; keep relevance and missingness indicators. |
| Macro | 6,535 observations across 47 series, including Treasury yields, breakevens, policy rates, and broad dollar | Values are useful, but current rows do not preserve complete release/vintage history. |
| COT | About 12,954 rows | Align using the public release time, not the report's reference date. |
| Options | 5,628 GLD contracts and one latest snapshot per underlying; current options coverage is a GLD proxy | Existing tables overwrite contract/snapshot state. Add append-only point-in-time snapshots before training on historical options. |
| Other event features | 1,432 social posts, 6,415 geosignals, 122 central-bank documents, 423 economic-calendar rows, and 45 Fear & Greed rows | Add only features available by the decision timestamp; retain source-specific freshness. |

Counts are point-in-time observations, not guarantees of continuing coverage. PostgreSQL statistics were approximate for some tables. `GLD` options are a cross-market proxy, not XAUUSD options.

## Proposed architecture

### 1. Point-in-time dataset

Create one training example for each eligible XAUUSD 15-minute candle close. Every feature must reflect information that the system could have known at that timestamp.

Feature groups:

- **Market sequence:** multi-resolution XAUUSD returns, range, volume, volatility, gaps, and session/time features from 5m, 15m, and 1h candles.
- **Cross-market sequence:** available related instruments such as XAGUSD, DXY, rates/yields, and broad risk proxies, joined by timestamp and freshness.
- **News and text:** existing FinBERT representations plus sentiment, impact, gold relevance, source, and recency for forex/macro news. Use `published_at` and `processed_at` to prevent late-arriving analysis from appearing early.
- **Social and event signals:** sentiment, engagement, severity, affected assets, and event age for social posts and geosignals.
- **Macro and positioning:** lagged macro series, economic-calendar surprises, COT positioning, and Fear & Greed. Preserve release/availability times and forward-fill only within explicit freshness limits.
- **Options:** append-only GLD chain/summary snapshots with `observed_at`, underlying price, implied volatility, put/call activity, open interest, and gamma exposure. Do not use the current overwritten options tables as if they were historical snapshots.

The feature builder emits a source-availability mask and feature age alongside values. Missing or stale sources remain missing; they are not filled with future observations or silently treated as neutral.

### 2. Model

Implement an isolated Python research package under `services/trading-model/`, separate from the existing sentiment analyzer.

- Evaluate a pretrained multivariate time-series foundation backbone, with Chronos-2 as an initial candidate, over the cross-asset price/covariate panel. Keep the backbone frozen initially; fine-tune only if the rolling validation supports it.
- Reuse the existing FinBERT service/model for news-text embeddings rather than training a new language model from the short local news history.
- Encode numeric macro, COT, options, calendar, social, and event features with small modality-specific encoders.
- Fuse time-aligned modality representations through gated cross-attention, with explicit masks for unavailable or stale modalities.
- Add probabilistic output heads for one-hour return direction, expected return/return quantiles, and realized-volatility or uncertainty estimation.
- Train a compact gradient-boosted model and price-only neural model as benchmarks. Keep the multimodal foundation model only if it improves out-of-sample probabilistic and net-strategy metrics.

The model predicts; a separate execution system is out of scope. Diffusion is not in the initial model path. It may later generate conditional price scenarios if that use can be validated independently.

### 3. Labels and evaluation

The primary target is the XAUUSD log return from a 15-minute candle close to the close four candles later. Report the probability of an up move and the return distribution. A configurable cost buffer is applied when evaluating tradable up/flat/down decisions; broker-specific costs must be supplied for a deployment evaluation.

Evaluation uses chronological walk-forward splits with at least a four-candle purge/embargo around split boundaries. Report:

- Brier score, log loss, calibration, and return forecast error;
- net performance under explicit spread, commission, and slippage assumptions;
- drawdown, turnover, exposure, and performance by time period/regime;
- source ablations: price only, then each modality, then the complete feature set;
- comparison against naive/no-change, price-only, and compact tabular baselines.

Do not select a model by optimizing one backtest period. Preserve an untouched final time interval after research choices are frozen. Backtest overfitting is a known risk when many configurations are tried; the evaluation should track trials and avoid repeated tuning against the final interval ([Bailey et al.](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2326253)).

## Data readiness and phased scope

1. Build the point-in-time feature dataset and produce coverage/freshness reports for XAUUSD.
2. Add append-only options snapshot history and macro release/vintage capture for new observations. Establish correct COT release-time alignment.
3. Pretrain/evaluate on the available cross-asset panel, then fine-tune/evaluate XAUUSD using the existing history. Mark all current six-week results exploratory.
4. Continue collecting XAUUSD candles and point-in-time side channels. Re-run walk-forward evaluation after coverage grows; do not represent early results as validated alpha.
5. Save reproducible model artifacts and prediction reports for paper evaluation. No broker integration or automatic execution in this scope.

## Acceptance criteria

- Rebuilding a feature row for timestamp `t` cannot read records whose availability time is after `t`.
- Coverage, staleness, and missingness are reported by source and timeframe.
- Options and macro features have append-only/as-of history before they are included in historical multimodal claims.
- The trained artifact returns calibrated up/flat/down probabilities, expected return/quantiles, and uncertainty for a one-hour horizon.
- Evaluation includes chronological walk-forward results, transaction-cost sensitivity, baseline comparisons, and source ablations.
- The research interface is offline/paper-only and cannot submit a live order.
- All model runs record data window, feature version, model/backbone version, configuration, and metrics.

## Open implementation choices

- Verify Chronos-2 licensing, runtime, hardware, and covariate support in a small isolated spike before committing to it as the backbone.
- Select a minimum data-coverage gate for allowing any modality into scored backtests; missing historical options and unreleased/revised macro values must fail that gate.
- Choose the artifact format and local training interface during the implementation plan. The design does not add a public API or a production container in its first phase.
