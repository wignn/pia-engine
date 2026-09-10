#!/usr/bin/env python3
"""Example: Realtime streaming with synchronous callback / thread interface."""

import os
import sys
import time

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from pia import MarketPrice, PiaClient


def main() -> None:
    api_key = os.environ.get("PIA_API_KEY", "wi_live_sample_key")
    client = PiaClient(api_key=api_key)

    client.realtime.on("connect", lambda: print("[WS] Connected to gateway. Authenticating..."))
    client.realtime.on("authenticated", lambda info: print(f"[WS] Authenticated! Tier: {info.get('plan')}"))
    client.realtime.on(
        "tick",
        lambda tick: print(f"[TICK] {tick.symbol}: ${tick.price} (bid: {tick.bid}, ask: {tick.ask})"),
    )
    client.realtime.on("error", lambda err: print(f"[WS Error] {err}"))

    # Subscribe to target pairs
    client.realtime.subscribe(["XAUUSD", "BTCUSDT"])

    print("Starting background streaming for 10 seconds...")
    client.realtime.start()

    try:
        time.sleep(10)
    finally:
        print("Stopping client...")
        client.close()


if __name__ == "__main__":
    main()
