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
```

`inspect-backbone` reports model metadata and local memory. It does not download weights unless `--run-inference` is supplied. Chronos-2 is the initial frozen candidate; its point forecasts and quantiles become numeric model inputs. The license is Apache-2.0 for the Chronos forecasting project. If inference is unavailable, the run records why and keeps the independently trained price-only candidate available; the other multimodal candidate can still use the remaining available channels.

The JSON training config should specify `from_date` and `to_date` (ISO dates); database URLs remain in environment variables. Training saves a no-change reference, price-only neural model, gradient-boosted baseline, gated multimodal model, and a run manifest. It reserves the last 20% of labeled rows as an untouched interval and purges at least four 15-minute candles before it. Chronos-2 and FinBERT weights load only when training or explicit inference is invoked.

## Data and artifacts

Options are GLD proxies, not XAUUSD options. New option and macro point-in-time records begin only after migration 029 and service deployment; current overwritten tables do not reconstruct historical values. Feature rows enforce availability timestamps, freshness limits, and explicit missing-source masks. FinBERT uses the analyzer's configured checkpoint (default `yiyanghkust/finbert-tone`) in frozen inference mode.

Run artifacts contain model/data versions, feature definitions, time windows, split boundaries, dependencies, checkpoint IDs, hashes, and metrics. They must not contain connection credentials or raw licensed news, social posts, or source documents. Check downstream data-source terms before distributing derived vectors, features, or model artifacts.

Training, forecasts, and costed evaluation are for research/paper use only. No model result is a profitability promise or a live trade instruction. Keep an untouched final interval and report Brier/log loss/calibration, return error, transaction-cost sensitivity, drawdown, turnover, exposure, period/regime metrics, and modality ablations.

Set `spread_bps`, `commission_per_side_bps`, `slippage_per_side_bps`, and `cost_sensitivity_bps` in the JSON config to match the broker/account being studied. Defaults are illustrative only. Evaluation writes fold/final metrics, prediction CSVs, model card, and artifact hashes into the run directory. Do not tune repeatedly against the untouched final interval.
