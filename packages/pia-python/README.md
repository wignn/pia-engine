# piaa-sdk (Official Python SDK)

Official, production-grade Python SDK for the **PIA Market Intelligence & Realtime Financial Platform**.

Designed for institutional quantitative trading bots, financial analytics, fintech backends, and data science workflows.

---

## Key Features

- ⚡ **Simple, Intuitive API**: Start fetching market data in under 3 lines of code with zero unnecessary ceremony.
- 🔁 **Enterprise Resiliency**: Automatic exponential backoff with full jitter on transient network errors (`5xx`, `408`, `429`) and respect for `Retry-After` headers.
- 📡 **Cross-Platform Realtime Streaming**: Resilient WebSocket client featuring **In-Band Message Authentication**, ping/pong keep-alives, auto-reconnect, and dynamic symbol subscriptions.
- 🛡️ **Typed Exception Hierarchy**: Actionable exceptions (`AuthenticationError`, `RateLimitError`, `TimeoutError`, `ValidationError`, `NetworkError`) with detailed quota telemetry attributes.
- 🔒 **Zero Sensitive Data Leaks**: Automatic regex redaction of `wi_live_...` API keys and Bearer tokens in error strings and logs.
- 🚀 **Dual Sync & Async Support**: Both synchronous (`PiaClient`) and modern asyncio (`AsyncPiaClient`) interfaces available.
- 🏷️ **Type Hinting**: Fully typed with PEP 561 `py.typed` marker for flawless IDE autocompletion and MyPy verification.

---

## Installation

```bash
# Core REST client
pip install piaa-sdk

# With WebSocket realtime streaming support
pip install "piaa-sdk[realtime]"
```

---

## Quickstart

### 1. Synchronous REST API

```python
from pia import PiaClient, RateLimitError, AuthenticationError

# Automatically picks up os.environ["PIA_API_KEY"] if omitted
client = PiaClient(api_key="wi_live_your_key")

try:
    # 1. Fetch live multi-asset snapshot (105+ symbols)
    prices = client.market.get_prices()
    print(f"Total instruments tracked: {prices.total}")

    for item in prices.items[:5]:
        print(f"[{item.symbol}] ${item.price:.2f} (bid: {item.bid}, ask: {item.ask})")

    # 2. Fetch historical candlestick bars
    candles = client.market.get_candles("XAUUSD", timeframe="1h", limit=100)
    print(f"Fetched {candles.count} candles for {candles.symbol}")

    # 3. Check rate limit telemetry
    quota = client.get_rate_limit_info()
    print(f"Remaining daily requests: {quota.daily_remaining}/{quota.daily_limit}")

except AuthenticationError:
    print("Invalid or expired API key.")
except RateLimitError as e:
    print(f"Rate limited! Retry after {e.retry_after_seconds} seconds.")
except Exception as e:
    print(f"Unexpected error: {e}")
finally:
    client.close()
```

---

### 2. Asynchronous REST API (`asyncio` / FastAPI)

```python
import asyncio
from pia import AsyncPiaClient

async def main():
    async with AsyncPiaClient(api_key="wi_live_...") as client:
        prices = await client.market.get_prices()
        print(f"Total instruments: {prices.total}")

asyncio.run(main())
```

---

### 3. Realtime WebSocket Streaming (In-Band Message Auth)

Cross-platform streaming without URL query-token leaks.

#### Asynchronous Streaming (Async Generator):

```python
import asyncio
from pia import AsyncPiaClient

async def stream_prices():
    async with AsyncPiaClient(api_key="wi_live_...") as client:
        print("Connecting to live ticker feed...")
        async for tick in client.realtime.stream(["XAUUSD", "BTCUSDT"]):
            print(f"[TICK] {tick.symbol} -> ${tick.price:.2f} (bid: {tick.bid}, ask: {tick.ask})")

asyncio.run(stream_prices())
```

#### Synchronous / Callback-based (Background Thread):

```python
from pia import PiaClient

client = PiaClient(api_key="wi_live_...")

# Register event callbacks
client.realtime.on("connect", lambda: print("WebSocket TCP connected."))
client.realtime.on("authenticated", lambda info: print("Authenticated successfully:", info))
client.realtime.on("tick", lambda tick: print(f"[TICK] {tick.symbol} -> {tick.price}"))
client.realtime.on("error", lambda err: print(f"Error: {err}"))

# Subscribe to target instruments
client.realtime.subscribe(["XAUUSD", "BTCUSDT"])

# Run in foreground (or call client.realtime.start() for non-blocking background thread)
client.realtime.run_forever()
```

---

## Configuration Reference

```python
from pia import PiaClient

client = PiaClient(
    # API key (defaults to os.environ["PIA_API_KEY"])
    api_key="wi_live_...",

    # Unified REST gateway (Default: "https://api-engine.wign.dev")
    base_url="https://api-engine.wign.dev",

    # Realtime WebSocket stream (Default: "wss://api-engine.wign.dev/api/v1/ws")
    ws_url="wss://api-engine.wign.dev/api/v1/ws",

    # Request timeout in seconds (Default: 15.0)
    timeout=10.0,

    # Maximum retry attempts on 5xx / 429 errors (Default: 3)
    max_retries=3,

    # Base retry delay for exponential backoff (Default: 0.5s)
    retry_delay=0.5,

    # Custom headers injected into all requests
    headers={"X-Trader-Id": "bot-algo-alpha"},

    # Enable debug logging (Default: False)
    debug=False,
)
```

---

## Typed Exception Hierarchy

All exceptions thrown by the SDK inherit from `PiaError`:

| Exception Class | HTTP Code | Description | Key Attributes |
|---|---|---|---|
| `ConfigurationError` | N/A | Missing or invalid client options | `message` |
| `ValidationError` | N/A | Invalid input argument (e.g. empty symbol) | `param_name` |
| `AuthenticationError` | 401 | Invalid, missing, or revoked API key | `status_code`, `endpoint` |
| `PermissionError` | 403 | Missing required scope/tier | `required_scope` |
| `RateLimitError` | 429 | Minute rate limit or daily quota exhausted | `retry_after_seconds`, `daily_remaining`, `minute_remaining` |
| `TimeoutError` | 408 / N/A | Request exceeded configured timeout | `timeout_seconds` |
| `NetworkError` | N/A | Connection refused, DNS failure, drop | `cause` |
| `ParseError` | N/A | Malformed non-JSON server response | `raw_text` |
| `ApiError` | Other | Unexpected HTTP status code | `status_code`, `raw_response` |

---

## Running Unit Tests

```bash
python3 -m unittest discover -s tests -p "*_test.py"
```
