#!/usr/bin/env python3
"""Example: Quickstart with synchronous REST API."""

import os
import sys

# Ensure local package is importable when running directly
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from pia import AuthenticationError, PiaClient, RateLimitError


def main() -> None:
    api_key = os.environ.get("PIA_API_KEY", "wi_live_sample_key")
    client = PiaClient(api_key=api_key, debug=True)

    try:
        print("1. Fetching live market prices snapshot...")
        prices = client.market.get_prices()
        print(f"Total instruments tracked: {prices.total}")

        for item in prices.items[:5]:
            print(f"[{item.symbol}] ${item.price} (bid: {item.bid}, ask: {item.ask})")

        print("\n2. Fetching historical 1-hour candles for XAUUSD...")
        candles = client.market.get_candles("XAUUSD", timeframe="1h", limit=10)
        print(f"Received {candles.count} candles for {candles.symbol}")

        telemetry = client.get_rate_limit_info()
        print(f"\nRemaining Minute Quota: {telemetry.remaining}/{telemetry.limit}")
        print(f"Remaining Daily Quota: {telemetry.daily_remaining}/{telemetry.daily_limit}")

    except AuthenticationError:
        print("Authentication error: Verify that your API key is valid.")
    except RateLimitError as e:
        print(f"Rate limited! Retry after {e.retry_after_seconds}s")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        client.close()


if __name__ == "__main__":
    main()
