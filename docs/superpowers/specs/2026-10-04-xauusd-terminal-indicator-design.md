# XAUUSD Model Forecast Indicator for PIA Terminal

**Status:** Draft for user review
**Date:** 2026-10-04

## Goal

Expose the existing offline XAUUSD one-hour forecast in `apps/terminal` (deployed as the `pia-terminal` image) as a research indicator. The current product target is XAUUSD on a 15-minute chart. One forecast is produced for each completed 15-minute decision candle and expires at its one-hour horizon.

The feature displays probabilities, estimated return/price range, uncertainty, and freshness. It is informational and paper-only: it must not create, route, or manage orders.

## Current state and constraints

- The research package is `services/trading-model`; it has a local CLI for training, evaluation, and as-of prediction but no scheduled worker, shared forecast store, or application API.
- `apps/terminal` already loads OHLCV through its Next.js API routes, uses `lightweight-charts`, and has indicator state and chart overlays.
- The `terminal` Compose service builds the `pia-terminal` image from this terminal application. There is no second trading terminal source in the current workspace.
- The research model has not been trained in this environment. No forecast artifact is currently available for the UI.
- Preserve existing uncommitted changes in the primary checkout. Implementation belongs in the active `codex/xauusd-multimodal-model` worktree.

## Architecture

1. **Scheduled Python worker:** Load a configured, already-trained model run; do not retrain on each candle. On each newly closed XAUUSD 15-minute candle, build the current as-of panel, produce one forecast, and append it to PostgreSQL. A unique `(model_version, symbol, decision_at)` key makes retries idempotent. The worker has no public port.
2. **Forecast history:** Add a migration for an append-only XAUUSD forecast table. Store symbol, decision and horizon timestamps, reference price, up/flat/down probabilities, expected one-hour log return, q10/q50/q90 log returns, uncertainty, model/data/feature versions, and source availability/age metadata. Restrict writes to the model worker and reads to market-data/API roles.
3. **Read API:** Add a protected `market-data` endpoint for the latest forecast and route it through the existing API Gateway, reusing the existing `market:read` authentication. Return the forecast timestamps and a server-computed status (`active` until `horizon_end`, otherwise `expired`). Return an explicit empty/not-found result when no forecast exists; do not invent a fallback forecast.
4. **Terminal data path:** Add a server-side Next.js proxy route that uses the existing server-held core API key. A small client hook requests the latest forecast only for XAUUSD at 15m and refreshes periodically; it must not expose service credentials to the browser.
5. **Chart indicator:** Add a persisted/toggleable AI forecast indicator. Mark the decision candle, show a compact forecast panel with probabilities, expected return, projected q10/q50/q90 prices, uncertainty, model version, decision time, horizon, and active/expired/unavailable status. Convert log-return estimates to price levels using `reference_price * exp(return)`. Do not draw an intermediate price path because the model predicts only the one-hour endpoint distribution.

## Freshness and failure behavior

- Use only the latest closed 15-minute XAUUSD candle as a forecast decision point.
- Do not display a forecast for another symbol or timeframe as if it applies to the active chart.
- An expired forecast remains inspectable but is visually marked expired and does not appear as a current signal.
- Missing model artifacts, unavailable source data, worker errors, API failures, and no historical forecast produce an explicit unavailable state. The UI must not substitute hard-coded or synthetic predictions.
- Worker retries may not create duplicate records for the same model/version and decision timestamp. Worker/API logs must omit database credentials and licensed raw news.

## Scope boundaries

- No automatic retraining, fine-tuning, broker connection, order entry, portfolio sizing, or profitability claim.
- No forecast overlays on instruments other than XAUUSD or timeframes other than 15m in the first version.
- No second PIA Portal dashboard; the confirmed target is `apps/terminal` / `pia-terminal`.
- Training and weight acquisition remain explicit operator actions. The indicator can report “model unavailable” until a trained artifact is configured.

## Acceptance criteria

- A newly completed 15-minute XAUUSD candle can result in at most one stored forecast per model version.
- The protected API returns the latest forecast with correct decision/horizon timestamps, versions, values, and active/expired state.
- The terminal displays the forecast only for XAUUSD/15m and clearly handles missing, expired, and API-error states.
- q10/q50/q90 price levels are derived from the forecast reference price and corresponding log-return quantiles; the UI does not imply a predicted intrahour path.
- Existing chart data, indicators, and API-key handling continue to use their current paths.
- No order-submission code path is added.

## Deployment dependency

The worker cannot publish meaningful forecasts until an evaluated model artifact is trained and made available at its configured path, required point-in-time data history is present, and the forecast-table migration is deployed. The feature must remain visibly unavailable until those requirements are met.
