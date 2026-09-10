#!/usr/bin/env python3
"""Example: Realtime streaming with asynchronous generator interface."""

import asyncio
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from pia import AsyncPiaClient


async def main() -> None:
    api_key = os.environ.get("PIA_API_KEY", "wi_live_sample_key")

    print("Connecting to live market stream (In-Band Message Auth)...")
    async with AsyncPiaClient(api_key=api_key) as client:
        count = 0
        async for tick in client.realtime.stream(["XAUUSD", "BTCUSDT"]):
            print(f"[TICK] {tick.symbol} -> ${tick.price:.2f} (bid: {tick.bid}, ask: {tick.ask})")
            count += 1
            if count >= 5:
                print("Received 5 ticks, shutting down cleanly...")
                break


if __name__ == "__main__":
    asyncio.run(main())
