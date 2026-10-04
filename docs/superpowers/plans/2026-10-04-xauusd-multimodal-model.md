# XAUUSD Multimodal Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an offline research pipeline that estimates one-hour XAUUSD return probabilities and distributions from point-in-time market and multimodal data.

**Architecture:** Add append-only history for new options and macro observations, then build a separate Python research package that creates as-of feature rows and evaluates compact baselines plus a gated multimodal model. Treat Chronos-2 as a candidate: first verify license, runtime, and covariate support; expose its forecasts as features if usable, and keep a documented price-only fallback if it is not.

**Tech Stack:** Existing PostgreSQL/ClickHouse data, Rust SQLx ingestion services, Python, PyTorch, Transformers/FinBERT, a compact gradient-boosted baseline, and local reproducible model artifacts. No new public API or live-trading integration.

**Spec:** `docs/superpowers/specs/2026-10-04-xauusd-multimodal-model-design.md`

## Global Constraints

- Decision cadence: one eligible XAUUSD 15-minute candle close; horizon: close four candles later.
- Every feature must be available at or before its decision timestamp; stale and missing inputs stay explicitly masked.
- Existing GLD options and revised macro rows are not historical point-in-time data; only newly captured history may enter historical multimodal claims.
- Keep Chronos-2 frozen initially and verify licensing, runtime, and covariate support before selecting it.
- Reuse the existing FinBERT checkpoint; do not train a language model on the short local news history.
- Evaluate against no-change, price-only, and compact tabular baselines with chronological walk-forward splits and at least four-candle purge/embargo.
- Report probability calibration, return error, costs, drawdown, turnover, exposure, periods/regimes, and modality ablations.
- Preserve an untouched final interval; track every research run and do not claim validated alpha from the current short history.
- The first release is offline/paper-only and must not submit, route, or manage live orders.

## Review Focus

- Late news enrichment: `published_at` and `processed_at` must not make a feature appear before it was available; inspect rows around decision timestamps in the generated coverage report.
- Revised macro releases: each changed value must keep its old `available_at`; inspect the as-of query before and after a revision.
- Repeated GLD option payloads: each received snapshot must have a distinct observation time and must not rewrite prior contract values; inspect a multi-snapshot chain.
- Empty or stale modality: feature output must carry missingness/age and the model must mask it; inspect a window with no recent social/options data.
- Short or discontinuous XAUUSD history: the pipeline must show coverage and mark evaluation exploratory instead of reporting a reliable validation result; inspect the run report for the current data window.

---

## File Structure

- `db/migrations/core/029_model_point_in_time_history.sql` — append-only macro-version and options-snapshot tables.
- `services/market-data/src/options.rs` — persist received summary/chain snapshots alongside existing current-state upserts.
- `services/sink-connector/src/workers/macro_worker.rs` — append a macro history row using the event's `observed_at` when a value is received or revised.
- `services/trading-model/pyproject.toml` — isolated research dependencies and `xauusd-model` CLI entry point.
- `services/trading-model/src/xauusd_model/{config,store,features,text,model,train,evaluate,artifacts,cli}.py` — focused config, data access, point-in-time feature construction, text encoding, model, training, evaluation, run metadata, and CLI modules.
- `services/trading-model/README.md` — setup, data limitations, reproducible commands, paper-only boundary, and interpretation of evaluation reports.
- `artifacts/xauusd-research/` — local run manifests, metrics, and model artifacts; keep credentials and raw licensed inputs out of artifacts.

## Task 1: Preserve new options and macro values as point-in-time history

**Files:**
- Create: `db/migrations/core/029_model_point_in_time_history.sql`
- Modify: `services/market-data/src/options.rs`
- Modify: `services/sink-connector/src/workers/macro_worker.rs`

**Interfaces:**
- Consumes: current `OptionsSummaryPayload`, `OptionsChainPayload`, and `MacroEvent` ingestion paths.
- Produces: append-only records keyed by observation/version time, queryable without depending on the mutable current-state tables.

- [ ] Add `options_snapshot_history` with generated snapshot ID, symbol, `observed_at`, underlying price, summary metrics, and JSONB source payload; index `(symbol, observed_at)`.
- [ ] Add `options_contract_history` keyed by snapshot ID and contract symbol, with the contract fields needed for IV, open interest, volume, and Greeks; store each incoming chain as a new snapshot before/with the existing upsert.
- [ ] Tag option snapshot history rows as `summary` or `chain` so point-in-time joins can select the correct payload without ambiguous same-time rows.
- [ ] Add `macro_observation_history` with source/type, series key, observation date, value/raw value, and `available_at`; index the as-of lookup keys. Append a row for new or revised macro rates, spreads, and series observations using `event.observed_at`.
- [ ] Keep current-state upsert behavior intact; ensure a history-write failure is surfaced as an ingestion error rather than silently claiming that the point-in-time record was saved.
- [ ] Manually inspect migration SQL and the existing ingest call paths to confirm the history tables preserve multiple versions and do not alter the public/current-state query shape.

## Task 2: Create the isolated research package and validate the backbone candidate

**Files:**
- Create: `services/trading-model/pyproject.toml`
- Create: `services/trading-model/src/xauusd_model/config.py`
- Create: `services/trading-model/src/xauusd_model/cli.py`
- Create: `services/trading-model/README.md`

**Interfaces:**
- Produces: `xauusd-model inspect-backbone` and `xauusd-model report-coverage --from ... --to ...`; shared settings for database URLs, date windows, seed, and artifact directory.

- [ ] Pin a minimal Python dependency set in the research package without changing the analyzer service's requirements.
- [ ] Implement `inspect-backbone` to record Chronos-2 model identifier, license/source, covariate support, CPU/GPU memory needs, and a small inference runtime observation; do not download/load model weights until the user runs the command in their environment.
- [ ] Select Chronos-2 only if its license and inference interface support the planned use; otherwise configure a documented price-only sequence fallback and record the reason in the run manifest.
- [ ] Add README commands and a data-handling note that artifacts exclude credentials and raw inputs whose license does not permit redistribution.

## Task 3: Build an as-of multimodal panel and coverage report

**Files:**
- Create: `services/trading-model/src/xauusd_model/store.py`
- Create: `services/trading-model/src/xauusd_model/features.py`
- Modify: `services/trading-model/src/xauusd_model/cli.py`

**Interfaces:**
- `store.load_market_panel(config: ResearchConfig, start: str, end: str) -> DataFrame`
- `store.load_side_channels(config: ResearchConfig, start: str, end: str) -> dict[str, DataFrame]`
- `features.build_asof_panel(market: DataFrame, side_channels: dict[str, DataFrame], freshness: dict[str, timedelta] | None = None) -> DataFrame`
- `features.coverage_report(panel: DataFrame) -> dict`
- Panel rows use UTC `decision_at`, `target_end`, per-source values, `{source}_age_seconds`, and `{source}_available` columns.

- [ ] Read XAUUSD 5m/15m/1h candles and available related market series, plus news, social/geosignals, calendar, COT, macro history, Fear & Greed, and new options history from their existing stores.
- [ ] Use source event/availability timestamps in as-of joins; for news analysis require both publication and processing to be no later than `decision_at`.
- [ ] Keep option/macro historical rows unavailable before their new history capture begins; never backfill overwritten current-state values into past feature rows.
- [ ] Create market returns/range/volatility/session features and numeric modality summaries with explicit source age and availability masks; forward-fill only inside source-specific freshness limits.
- [ ] Implement coverage output by source, timeframe, date range, available fraction, freshness distribution, and first valid point-in-time-history timestamp; make `report-coverage` write a human-readable JSON/Markdown report.
- [ ] Inspect one early and one late decision timestamp manually to verify no source row later than the decision timestamp enters its features.

## Task 4: Encode text and train probabilistic model candidates

**Files:**
- Create: `services/trading-model/src/xauusd_model/text.py`
- Create: `services/trading-model/src/xauusd_model/model.py`
- Create: `services/trading-model/src/xauusd_model/train.py`
- Modify: `services/trading-model/src/xauusd_model/cli.py`

**Interfaces:**
- `text.encode_news(rows: DataFrame, checkpoint: str, batch_size: int) -> DataFrame` returns timestamped FinBERT embeddings plus existing sentiment/relevance metadata.
- `model.MultimodalForecaster.forward(batch) -> dict[str, Tensor]` returns `direction_logits`, `return_quantiles`, and `uncertainty`.
- `train.fit_candidates(panel, config) -> dict[str, Path]` writes candidate artifacts and run metadata.

- [ ] Load the existing FinBERT checkpoint through Transformers to encode only articles available by each decision time; cache embeddings by article/version and never train FinBERT on this local corpus.
- [ ] Add benchmark candidates: no-change, price-only compact neural sequence model, and gradient-boosted tabular model.
- [ ] Add modality-specific numeric encoders and a gated cross-attention fusion block with masks for unavailable/stale sources; consume Chronos-2 forecasts as a frozen model feature when the Task 2 spike selects it.
- [ ] Train direction, one-hour return quantile, and uncertainty outputs. Use seeded chronological train/calibration/final splits, purge at least four candles at each boundary, and do not fit or tune against the untouched final interval.
- [ ] Store feature version, requested decision window, source windows, split boundaries, seed, dependencies, checkpoint IDs, and data coverage in each run manifest.

## Task 5: Evaluate, report, and keep the interface paper-only

**Files:**
- Create: `services/trading-model/src/xauusd_model/evaluate.py`
- Create: `services/trading-model/src/xauusd_model/artifacts.py`
- Modify: `services/trading-model/src/xauusd_model/cli.py`
- Modify: `services/trading-model/README.md`

**Interfaces:**
- `evaluate.walk_forward(panel, config) -> EvaluationReport`
- `evaluate.score_predictions(predictions, costs) -> dict`
- CLI: `xauusd-model train --config ...`, `xauusd-model evaluate --run ...`, and `xauusd-model predict --run ... --as-of ...`.
- Prediction output contains calibrated up/flat/down probabilities, expected return, configured return quantiles, uncertainty, source masks/ages, and model/data version IDs.

- [ ] Evaluate walk-forward folds and an untouched final interval using Brier score, log loss, calibration, return forecast error, and net metrics under explicit spread/commission/slippage scenarios.
- [ ] Report drawdown, turnover, exposure, results by period/regime, and ablations for price only, each supported modality, and the complete available feature set.
- [ ] Track attempted model/configuration runs so repeated tuning is visible; label results exploratory when sample size or source coverage is inadequate.
- [ ] Save a compact model card and run report with known data gaps, GLD-as-proxy limitations, time range, metrics, calibration, intended paper-only use, and artifact hashes.
- [ ] Keep prediction CLI local and offline; verify its command path has no broker/order client imports or order-submission behavior.

## Self-review notes

- Spec coverage: point-in-time feature alignment and masks (Task 3); append-only options/macro capture (Task 1); Chronos-2 check and FinBERT reuse (Tasks 2 and 4); multimodal model and benchmarks (Task 4); labels, walk-forward, costs, ablations, final interval, run tracking (Tasks 4 and 5); offline artifacts and paper-only boundary (Task 5).
- Interface consistency: CLI verbs are defined in Tasks 2 and 5; store/panel interfaces are defined in Task 3; model and training outputs are defined in Task 4; evaluation consumes the same feature panel and returns the report persisted in Task 5.
- Scope: one research subsystem with its ingestion history prerequisite; no public endpoint, deployment container, or live execution path.
- Limitation: software test suites are not included in this plan; validation is through migration/ingestion inspection, point-in-time coverage reports, and the requested model evaluation workflow.
