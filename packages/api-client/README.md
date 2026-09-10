# @piaa/sdk (Official TypeScript / JavaScript SDK)

The official, production-ready SDK for the **PIA Market Intelligence & Realtime Financial Platform**.

Designed for institutional algorithmic traders, fintech dashboards, and quantitative applications across **Node.js 18+**, **Bun**, **Deno**, and modern **Browsers**.

---

## Features

- ⚡ **Zero-Fuss Sensible Defaults**: Connect in 3 lines of code with pre-configured endpoints and sensible timeout/retry defaults.
- 🔁 **Enterprise Resiliency**: Automatic exponential backoff with full jitter for transient 5xx/429 network hiccups and retry-after headers.
- 🛡️ **Typed Error Hierarchy**: Clear, actionable, strongly-typed errors (`AuthenticationError`, `RateLimitError`, `TimeoutError`, `ValidationError`, `NetworkError`).
- 🔒 **Zero Sensitive Data Leaks**: Automatic redaction of API keys, bearer tokens, and secrets from error logs and stack traces.
- 📡 **Cross-Platform Realtime Streaming**: Built-in resilient WebSocket client with **In-Band Message Authentication**, ping/pong keep-alives, and automatic re-subscription on reconnect.
- 📊 **Rate Limit Telemetry**: Real-time inspection of RFC 6585 and daily quota headers (`X-RateLimit-*`, `X-DailyQuota-*`).

---

## Installation

```bash
# Using bun
bun add @piaa/sdk

# Using npm
npm install @piaa/sdk

# Using pnpm / yarn
pnpm add @piaa/sdk
```

---

## Quickstart

### 1. REST API Usage

```typescript
import { PiaClient, RateLimitError, AuthenticationError } from "@piaa/sdk";

// Automatically picks up process.env.PIA_API_KEY if omitted
const client = new PiaClient({
  apiKey: "wi_live_your_api_key",
});

async function run() {
  try {
    // 1. Fetch live multi-asset snapshot (105+ symbols)
    const prices = await client.market.getPrices();
    console.log(`Tracked assets: ${prices.total}`);
    for (const item of prices.items.slice(0, 5)) {
      console.log(`[${item.symbol}] $${item.price} (${item.asset_type})`);
    }

    // 2. Fetch historical OHLCV candlesticks
    const candles = await client.market.getCandles("XAUUSD", {
      timeframe: "1h",
      limit: 100,
    });
    console.log(`Fetched ${candles.count} bars for ${candles.symbol}`);

    // 3. Inspect rate limit telemetry
    const quota = client.getRateLimitInfo();
    console.log(`Remaining daily hits: ${quota.dailyRemaining}/${quota.dailyLimit}`);
  } catch (err) {
    if (err instanceof AuthenticationError) {
      console.error("Invalid or expired API key.");
    } else if (err instanceof RateLimitError) {
      console.error(`Rate limited! Retry after ${err.retryAfterSeconds} seconds.`);
    } else {
      console.error("API error:", err);
    }
  }
}

run();
```

---

### 2. Realtime WebSocket Streaming (In-Band Message Auth)

Cross-platform streaming without query-string leakage:

```typescript
import { PiaClient } from "@piaa/sdk";

const client = new PiaClient({ apiKey: "wi_live_..." });

// Listen for connection events
client.realtime.on("connect", () => {
  console.log("WebSocket connected. Authenticating...");
});

// Fired once In-Band authentication is accepted by the server
client.realtime.on("authenticated", (info) => {
  console.log("Authenticated! Tier:", info.plan);

  // Subscribe to market symbols
  client.realtime.subscribe(["XAUUSD", "BTCUSDT", "EURUSD"]);
});

// Handle real-time price ticks
client.realtime.on("tick", (tick) => {
  console.log(`[TICK] ${tick.symbol} -> $${tick.price} (bid: ${tick.bid}, ask: ${tick.ask})`);
});

// Handle disconnections (auto-reconnect is handled automatically)
client.realtime.on("disconnect", ({ code, reason }) => {
  console.warn(`WebSocket closed (${code}): ${reason}`);
});

// Start streaming
client.realtime.connect();

// To stop and prevent auto-reconnection:
// client.realtime.disconnect();
```

---

## Configuration Reference

```typescript
const client = new PiaClient({
  // Secret API key. Falls back to process.env.PIA_API_KEY
  apiKey: "wi_live_...",

  // Unified REST gateway (Default: "https://api-engine.wign.dev")
  baseUrl: "https://api-engine.wign.dev",

  // Realtime streaming gateway (Default: "wss://api-engine.wign.dev/api/v1/ws")
  wsUrl: "wss://api-engine.wign.dev/api/v1/ws",

  // Request timeout in milliseconds (Default: 15,000)
  timeoutMs: 10_000,

  // Maximum retry attempts for transient errors (Default: 3)
  maxRetries: 3,

  // Initial retry delay for exponential backoff (Default: 500ms)
  retryDelayMs: 500,

  // Custom HTTP headers injected into all requests
  headers: {
    "X-Client-App": "my-trading-bot",
  },

  // Custom fetch implementation (useful for mocking or custom proxies)
  fetch: customFetch,

  // Custom WebSocket implementation (e.g. 'ws' in Node.js)
  WebSocket: customWebSocket,

  // Enable verbose debug logs (Default: false)
  debug: false,
});
```

---

## Error Handling

All errors thrown by the SDK inherit from `PiaError`:

| Error Class | HTTP Status | Description | Properties |
|---|---|---|---|
| `ConfigurationError` | N/A | Missing or invalid client configuration | `message` |
| `ValidationError` | N/A | Invalid method arguments (e.g. empty symbol) | `paramName` |
| `AuthenticationError` | 401 | Invalid, expired, or revoked API key | `statusCode`, `endpoint` |
| `PermissionError` | 403 | API key lacks required scope (e.g. `realtime:ws`) | `requiredScope` |
| `RateLimitError` | 429 | Rate limit or daily quota exceeded | `retryAfterSeconds`, `dailyRemaining`, `minuteRemaining` |
| `TimeoutError` | 408 / N/A | Request exceeded configured `timeoutMs` | `timeoutMs`, `endpoint` |
| `NetworkError` | N/A | Network drop, DNS resolution failure | `causeError` |
| `ParseError` | N/A | Response was not valid JSON | `rawText` |
| `ApiError` | Other | General HTTP errors | `statusCode`, `rawBody` |

---

## Testing

Run the test suite using Bun:

```bash
bun test
```
