import asyncio
import json
import logging
import math
import os
from datetime import datetime, timedelta, timezone
from typing import Any
from zoneinfo import ZoneInfo

import nats
import requests
from alpaca.data.historical.option import OptionHistoricalDataClient
from alpaca.data.requests import OptionSnapshotRequest
from alpaca.trading.client import TradingClient
from alpaca.trading.enums import AssetStatus, ExerciseStyle
from alpaca.trading.requests import GetOptionContractsRequest

logger = logging.getLogger("options-worker")
CHAIN_SUBJECT = "md.raw.options.chain.v1"
DEFAULT_SYMBOLS = "SPY,QQQ,AAPL,MSFT,TSLA,NVDA,GLD"


def _as_float(value: Any, default: float = 0.0) -> float:
    try:
        if value is None:
            return default
        parsed = float(value)
        return parsed if math.isfinite(parsed) else default
    except (TypeError, ValueError):
        return default


def _as_int(value: Any, default: int = 0) -> int:
    try:
        if value is None:
            return default
        return max(0, int(float(value)))
    except (TypeError, ValueError):
        return default


def _get(value: Any, *keys: str) -> Any:
    for key in keys:
        if isinstance(value, dict) and key in value:
            return value[key]
        if hasattr(value, key):
            return getattr(value, key)
    return None


def _enum_value(value: Any) -> str:
    return str(getattr(value, "value", value) or "").lower()


def _symbols() -> list[str]:
    raw = os.getenv("OPTIONS_SYMBOLS", DEFAULT_SYMBOLS)
    return [symbol.strip().upper() for symbol in raw.split(",") if symbol.strip()]


def _underlying_price(api_key: str, secret_key: str, symbol: str) -> float:
    res = requests.get(
        f"https://data.alpaca.markets/v2/stocks/{symbol}/trades/latest",
        headers={"APCA-API-KEY-ID": api_key, "APCA-API-SECRET-KEY": secret_key},
        timeout=10,
    )
    res.raise_for_status()
    return _as_float(res.json().get("trade", {}).get("p"))


def _snapshots(option_client: OptionHistoricalDataClient, symbols: list[str]) -> dict[str, Any]:
    if not symbols:
        return {}
    result = option_client.get_option_snapshot(OptionSnapshotRequest(symbol_or_symbols=symbols))
    return result if isinstance(result, dict) else getattr(result, "data", {}) or {}


def _contract(contract: Any, snapshot: Any, symbol: str) -> dict[str, Any]:
    quote = _get(snapshot, "latest_quote")
    trade = _get(snapshot, "latest_trade")
    greeks = _get(snapshot, "greeks")
    bid = _as_float(_get(quote, "bid_price", "bp"))
    ask = _as_float(_get(quote, "ask_price", "ap"))
    close_price = _as_float(_get(contract, "close_price"))
    mark = (bid + ask) / 2.0 if bid > 0.0 and ask > 0.0 else _as_float(_get(trade, "price", "p"), close_price)

    return {
        "contract_symbol": str(_get(contract, "symbol") or ""),
        "symbol": symbol,
        "option_type": _enum_value(_get(contract, "type")),
        "strike": _as_float(_get(contract, "strike_price")),
        "expiration_date": str(_get(contract, "expiration_date") or ""),
        "mark_price": mark,
        "bid": bid,
        "ask": ask,
        "implied_volatility": _as_float(_get(snapshot, "implied_volatility"), 0.2),
        "delta": _as_float(_get(greeks, "delta")),
        "gamma": _as_float(_get(greeks, "gamma")),
        "theta": _as_float(_get(greeks, "theta")),
        "vega": _as_float(_get(greeks, "vega")),
        "gex": 0.0,
        "open_interest": _as_int(_get(contract, "open_interest")),
        "volume": _as_int(_get(trade, "size", "s")),
    }


def _payloads(
    trade_client: TradingClient,
    option_client: OptionHistoricalDataClient,
    api_key: str,
    secret_key: str,
    symbol: str,
) -> dict[str, Any]:
    now = datetime.now(tz=ZoneInfo("America/New_York"))
    limit = max(1, int(os.getenv("OPTIONS_CONTRACT_LIMIT", "200")))
    days = max(1, int(os.getenv("OPTIONS_EXPIRATION_DAYS", "60")))
    contracts_res = trade_client.get_option_contracts(
        GetOptionContractsRequest(
            underlying_symbols=[symbol],
            status=AssetStatus.ACTIVE,
            expiration_date_gte=(now + timedelta(days=1)).date(),
            expiration_date_lte=(now + timedelta(days=days)).date(),
            style=ExerciseStyle.AMERICAN,
            limit=limit,
        )
    )
    contracts = list(getattr(contracts_res, "option_contracts", []) or [])
    contract_symbols = [str(_get(contract, "symbol")) for contract in contracts if _get(contract, "symbol")]

    try:
        snapshots = _snapshots(option_client, contract_symbols)
    except Exception as exc:
        logger.warning("failed to fetch Alpaca option snapshots for %s: %s", symbol, exc)
        snapshots = {}

    price = _underlying_price(api_key, secret_key, symbol)
    rows = [_contract(contract, snapshots.get(str(_get(contract, "symbol"))), symbol) for contract in contracts]
    updated_at = datetime.now(timezone.utc).isoformat()
    return {"symbol": symbol, "underlying_price": price, "contracts": rows, "updated_at": updated_at}


async def start_options_worker(nats_url: str, poll_interval: int) -> None:
    api_key = os.getenv("ALPACA_API_KEY", "").strip()
    secret_key = os.getenv("ALPACA_SECRET_KEY", "").strip()
    if not api_key or not secret_key:
        logger.warning("Alpaca options worker disabled; ALPACA_API_KEY/ALPACA_SECRET_KEY not set")
        return

    trade_client = TradingClient(
        api_key=api_key,
        secret_key=secret_key,
        paper=os.getenv("ALPACA_PAPER", "true").lower() != "false",
        url_override=os.getenv("ALPACA_TRADE_API_URL", "https://paper-api.alpaca.markets"),
    )
    option_client = OptionHistoricalDataClient(api_key, secret_key)
    nats_client = await nats.connect(nats_url)
    symbol_delay = max(0, int(os.getenv("OPTIONS_SYMBOL_DELAY_SEC", "5")))
    logger.info("options worker connected to NATS at %s", nats_url)

    while True:
        for symbol in _symbols():
            try:
                chain = await asyncio.to_thread(_payloads, trade_client, option_client, api_key, secret_key, symbol)
                await nats_client.publish(CHAIN_SUBJECT, json.dumps(chain, allow_nan=False).encode())
                logger.info("published Alpaca options payload for %s (%s contracts)", symbol, len(chain["contracts"]))
            except Exception as exc:
                logger.warning("failed to publish Alpaca options for %s: %s", symbol, exc)
            if symbol_delay:
                await asyncio.sleep(symbol_delay)
        await asyncio.sleep(poll_interval)
