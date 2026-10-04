# XAUUSD research model

This package builds and evaluates an offline, point-in-time model for the one-hour XAUUSD return from a 15-minute decision close. It does not connect to a broker or send orders. The currently available XAUUSD sample is short, so all results from that interval are exploratory.

## Setup

Use an isolated Python environment with Python 3.10 or newer, then install this package from this directory:

```bash
python -m venv .venv
. .venv/bin/activate
python -m pip install -e .
```

Set `TRADING_MODEL_POSTGRES_URL` and `TRADING_MODEL_CLICKHOUSE_URL` to read the existing data. Set `TRADING_MODEL_ARTIFACT_DIR` to choose a local artifact directory. Credentials in connection URLs are never included in run manifests.

## Commands

```bash
xauusd-model inspect-backbone
xauusd-model inspect-backbone --run-inference
xauusd-model report-coverage --from 2026-08-01 --to 2026-10-04 --output artifacts/xauusd-research/coverage.json
xauusd-model train --config research-config.json
xauusd-model evaluate --config research-config.json --run artifacts/xauusd-research/RUN_ID
xauusd-model predict --config research-config.json --run artifacts/xauusd-research/RUN_ID --as-of 2026-10-04T12:00:00Z
xauusd-model worker
```

`inspect-backbone` reports Chronos-2 metadata from the [official project](https://github.com/amazon-science/chronos-forecasting) and local memory. It does not download weights unless `--run-inference` is supplied. The opt-in smoke check exercises a covariate forecast and the q10/q50/q90 interface. Chronos-2 is the initial frozen candidate; its point forecasts and quantiles become numeric model inputs. The project is Apache-2.0 licensed. If inference is unavailable, the run records why and keeps the independently trained price-only candidate available; the other multimodal candidate can still use the remaining available channels.

The JSON training config should specify `from_date` and `to_date` (ISO dates); database URLs remain in environment variables. Training saves a no-change reference, price-only neural model, gradient-boosted baseline, gated multimodal model, and a run manifest. It reserves the last 20% of labeled rows as an untouched interval and purges at least four 15-minute candles before it. Chronos-2 and FinBERT weights load only when training or explicit inference is invoked.

## Data and artifacts

Options are GLD proxies, not XAUUSD options. New option and macro point-in-time records begin only after migration 029 and service deployment; current overwritten tables do not reconstruct historical values. Feature rows enforce availability timestamps, freshness limits, and explicit missing-source masks. FinBERT uses the analyzer's configured checkpoint (default `yiyanghkust/finbert-tone`) in frozen inference mode.

Run artifacts contain model/data versions, feature definitions, time windows, split boundaries, dependencies, checkpoint IDs, hashes, and metrics. They must not contain connection credentials or raw licensed news, social posts, or source documents. Check downstream data-source terms before distributing derived vectors, features, or model artifacts.

Training, forecasts, and costed evaluation are for research/paper use only. No model result is a profitability promise or a live trade instruction. Keep an untouched final interval and report Brier/log loss/calibration, return error, transaction-cost sensitivity, drawdown, turnover, exposure, period/regime metrics, and modality ablations.

Set `spread_bps`, `commission_per_side_bps`, `slippage_per_side_bps`, and `cost_sensitivity_bps` in the JSON config to match the broker/account being studied. Defaults are illustrative only. Evaluation writes fold/final metrics, prediction CSVs, model card, and artifact hashes into the run directory. Do not tune repeatedly against the untouched final interval.

## Scheduled terminal forecasts

The Compose worker loads only an evaluated run containing `run.json`, `evaluation.json`, and its `multimodal.pt` artifact. It reads the latest closed XAUUSD 15-minute candle, builds that row's point-in-time feature panel, and writes one one-hour forecast to `market.market_forecasts`. It never retrains. The unique model/symbol/decision key makes retries idempotent. Missing artifacts, database access, source features, or model weights keep the worker in an unavailable state; no placeholder forecast is written.

To provision a run, train and evaluate it with the commands above. Copy the complete run directory to the host at `var/xauusd-model/<run-id>` and point `var/xauusd-model/active` at that directory. Mounts are read-only in Compose; the separate worker cache volume stores FinBERT embeddings and downloaded model cache data. Artifacts may contain derived source features and embeddings, so follow the source licensing notes above.

Copy `infra/env/.env.trading-model.example` to `infra/env/.env.trading-model` on each deployment host. Set `TRADING_MODEL_POSTGRES_URL` to a dedicated `model_worker_user` connection and set read-only ClickHouse access in `TRADING_MODEL_CLICKHOUSE_URL`. Migration 030 creates the role and grants source reads plus forecast insert/select; a database administrator must provision its password and the connection URL. The file is optional so the app stack can start before credentials or a model are provisioned.

The protected read endpoint is `GET /api/v1/market/forecasts/XAUUSD?timeframe=15m`, routed through API Gateway with `market:read`. It returns the latest forecast with `active` status until `horizon_end`, then `expired`; no stored row returns `status: unavailable`. Return values and q10/q50/q90 are one-hour log returns, converted by the terminal to projected prices using `reference_price * exp(return)`. The terminal overlay is informational and paper-only.
