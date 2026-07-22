import asyncio
import json
import logging
import math
from datetime import datetime, timezone
from typing import Any

import nats
import yfinance as yf

logger = logging.getLogger("options-worker")
SYMBOLS = ["SPY", "QQQ", "AAPL", "MSFT", "TSLA", "NVDA"]
CHAIN_SUBJECT = "md.raw.options.chain.v1"


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
        return max(0, int(value))
    except (TypeError, ValueError):
        return default


def _expiry_from_contract(symbol: str, contract_symbol: str) -> str:
    suffix = contract_symbol.removeprefix(symbol).removesuffix("C").removesuffix("P")
    return f"20{suffix[:2]}-{suffix[2:4]}-{suffix[4:6]}" if len(suffix) >= 6 else ""


def _contract(row: Any, symbol: str, option_type: str, underlying_price: float) -> dict[str, Any]:
    contract_symbol = str(row.get("contractSymbol", ""))
    strike = _as_float(row.get("strike"))
    mark = _as_float(row.get("lastPrice"))
    iv = _as_float(row.get("impliedVolatility"), 0.2)
    oi = _as_int(row.get("openInterest"))
    volume = _as_int(row.get("volume"))
    gamma = 0.0

    return {
        "contract_symbol": contract_symbol,
        "symbol": symbol,
        "option_type": option_type,
        "strike": strike,
        "expiration_date": _expiry_from_contract(symbol, contract_symbol),
        "mark_price": mark,
        "bid": _as_float(row.get("bid")),
        "ask": _as_float(row.get("ask")),
        "implied_volatility": iv,
        "delta": 0.0,
        "gamma": gamma,
        "theta": 0.0,
        "vega": 0.0,
        "gex": gamma * underlying_price * underlying_price * 100.0 * oi * (1 if option_type == "call" else -1),
        "open_interest": oi,
        "volume": volume,
    }


def _payloads(symbol: str) -> dict[str, Any]:
    ticker = yf.Ticker(symbol)
    expiration = ticker.options[0]
    chain = ticker.option_chain(expiration)
    price = _as_float(ticker.fast_info.get("last_price")) or _as_float(ticker.info.get("regularMarketPrice"))

    contracts = [
        _contract(row, symbol, "call", price)
        for row in chain.calls.to_dict("records")
    ] + [
        _contract(row, symbol, "put", price)
        for row in chain.puts.to_dict("records")
    ]

    updated_at = datetime.now(timezone.utc).isoformat()
    return {"symbol": symbol, "underlying_price": price, "contracts": contracts, "updated_at": updated_at}


async def start_options_worker(nats_url: str, poll_interval: int) -> None:
    client = await nats.connect(nats_url)
    logger.info("options worker connected to NATS at %s", nats_url)

    while True:
        for symbol in SYMBOLS:
            try:
                chain = await asyncio.to_thread(_payloads, symbol)
                await client.publish(CHAIN_SUBJECT, json.dumps(chain, allow_nan=False).encode())
                logger.info("published Yahoo options payloads for %s", symbol)
            except Exception as exc:
                logger.warning("failed to publish Yahoo options for %s: %s", symbol, exc)
        await asyncio.sleep(poll_interval)
