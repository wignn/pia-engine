"use client";

import { useEffect, useState } from "react";
import { accountApi, type KeyInfo, type User } from "@/src/lib/api/account";

type SnippetLang = "curl" | "python" | "typescript" | "go";

const BASE_API_URL = process.env.NEXT_PUBLIC_API_URL || "https://api-engine.wign.dev";
const BASE_WS_URL = process.env.NEXT_PUBLIC_WS_URL || "wss://api-engine.wign.dev/api/v1/ws";

const REST_ENDPOINTS = [
  {
    method: "GET",
    path: "/api/v1/market/prices",
    title: "Multi-Asset Price Snapshot",
    desc: "Returns latest prices, bid/ask spreads, and active exchange session status for 105+ global assets.",
    params: "None",
    sampleResponse: `{
  "items": [
    {
      "symbol": "XAUUSD",
      "price": 4414.06,
      "asset_type": "forex",
      "session": { "state": "open", "exchange": "FX", "is_open": true },
      "received_at": "2026-09-10T04:07:13Z"
    },
    {
      "symbol": "BTCUSDT",
      "price": 78347.10,
      "asset_type": "crypto",
      "session": { "state": "open", "exchange": "CRYPTO", "is_open": true },
      "received_at": "2026-09-10T04:07:15Z"
    }
  ],
  "total": 105
}`,
  },
  {
    method: "GET",
    path: "/api/v1/market/candles?symbol=XAUUSD&tf=1m",
    title: "Historical OHLCV Candlesticks",
    desc: "Pre-aggregated 1-minute rollup candlestick series from ClickHouse with sub-millisecond query latency.",
    params: "symbol (required, e.g. XAUUSD), tf (optional: 1m, 5m, 15m, 1h, 1d)",
    sampleResponse: `{
  "symbol": "XAUUSD",
  "timeframe": "1m",
  "candles": [
    {
      "time": 1789013160,
      "open": 4412.50,
      "high": 4415.20,
      "low": 4411.80,
      "close": 4414.06,
      "volume": 128.4
    }
  ]
}`,
  },
  {
    method: "GET",
    path: "/api/v1/options/summary?symbol=SPY",
    title: "Options Sentiment & Max Pain",
    desc: "Aggregated options market indicators including Max Pain price, Put/Call volume ratio, and implied volatility skew.",
    params: "symbol (required, e.g. SPY, QQQ, GLD)",
    sampleResponse: `{
  "symbol": "SPY",
  "spot_price": 542.80,
  "max_pain": 540.00,
  "put_call_ratio": 0.84,
  "total_call_volume": 1420500,
  "total_put_volume": 1193220,
  "updated_at": "2026-09-10T04:00:00Z"
}`,
  },
  {
    method: "GET",
    path: "/api/v1/options/gex?symbol=SPY",
    title: "Gamma Exposure Profile (GEX)",
    desc: "Net dealer gamma exposure breakdown across strike prices for market maker positioning analysis.",
    params: "symbol (required, e.g. SPY)",
    sampleResponse: `{
  "symbol": "SPY",
  "net_gamma": 425100000,
  "regime": "positive_gamma",
  "volatility_drag": "dampening",
  "strikes": [
    { "strike": 535.0, "call_gamma": 850000, "put_gamma": -1200000, "net": -350000 },
    { "strike": 540.0, "call_gamma": 2400000, "put_gamma": -450000, "net": 1950000 }
  ]
}`,
  },
  {
    method: "GET",
    path: "/api/v1/social/posts?limit=10",
    title: "Real-Time Financial Social Pulse",
    desc: "Curated institutional social feeds from verified financial analysts, economists, and central banks via NATS.",
    params: "limit (optional, default 20, max 100), offset (optional cursor)",
    sampleResponse: `{
  "posts": [
    {
      "id": "183348123891",
      "author_handle": "financialjuice",
      "content": "US Initial Jobless Claims actual 218K vs 225K forecast.",
      "published_at": "2026-09-10T04:05:00Z"
    }
  ]
}`,
  },
];

export default function DocsPage() {
  const [user, setUser] = useState<User | null>(null);
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [selectedKey, setSelectedKey] = useState<string>("wi_live_YOUR_API_KEY");
  const [activeLang, setActiveLang] = useState<SnippetLang>("python");
  const [copiedSection, setCopiedSection] = useState<string | null>(null);

  useEffect(() => {
    accountApi.me().then((res) => {
      if (res?.user) {
        setUser(res.user);
        accountApi.keys().then((kList) => {
          if (kList && kList.length > 0) {
            setKeys(kList);
            setSelectedKey(`${kList[0].key_prefix}••••••••••••`);
          }
        });
      }
    }).catch(() => {
      // unauthenticated
    });
  }, []);

  const handleCopy = (text: string, sectionId: string) => {
    navigator.clipboard.writeText(text);
    setCopiedSection(sectionId);
    setTimeout(() => setCopiedSection(null), 2000);
  };

  const getPythonSnippet = () => `# pip install piaa-sdk
from pia import PiaClient, RateLimitError, AuthenticationError

client = PiaClient(api_key="${selectedKey}")

try:
    # 1. Fetch live multi-asset snapshot (105+ instruments)
    prices = client.market.get_prices()
    print(f"Total assets tracked: {prices.total}")
    for item in prices.items[:5]:
        print(f"[{item.symbol}] Price: \${item.price} ({item.asset_type})")

    # 2. Fetch historical candlesticks
    candles = client.market.get_candles("XAUUSD", timeframe="1m", limit=5)
    print(f"Retrieved {candles.count} candles for {candles.symbol}")

    # 3. Check rate limit telemetry
    quota = client.get_rate_limit_info()
    print(f"Remaining daily quota: {quota.daily_remaining}/{quota.daily_limit}")

    # 4. Realtime WebSocket Streaming (batteries-included)
    from pia import AsyncPiaClient
    import asyncio

    async def stream():
        async_client = AsyncPiaClient(api_key="${selectedKey}")
        async for tick in async_client.realtime.stream(["XAUUSD", "BTCUSDT"]):
            print(f"[TICK] {tick.symbol} -> {tick.price}")

    # asyncio.run(stream())

except RateLimitError as e:
    print(f"Rate limit exceeded! Retry after {e.retry_after_seconds}s")
except AuthenticationError:
    print("Invalid or inactive API key.")
finally:
    client.close()
`;

  const getTypescriptSnippet = () => `// npm install @piaa/sdk
import { PiaClient, RateLimitError } from "@piaa/sdk";

const client = new PiaClient({ apiKey: "${selectedKey}" });

async function run() {
  try {
    // 1. Fetch REST Market Prices
    const prices = await client.market.getPrices();
    console.log(\`Received \${prices.total} market instruments.\`);

    const quota = client.getRateLimitInfo();
    console.log(\`Remaining Daily Quota: \${quota.dailyRemaining}/\${quota.dailyLimit}\`);

    // 2. Realtime WebSocket Streaming (In-Band Message Auth)
    client.realtime.on("connect", () => console.log("WebSocket connected."));
    client.realtime.on("authenticated", (tier) => {
      console.log("Authenticated! Subscribing to XAUUSD & BTCUSDT...");
      client.realtime.subscribe(["XAUUSD", "BTCUSDT"]);
    });

    client.realtime.on("tick", (tick) => {
      console.log(\`[TICK] \${tick.symbol} -> \${tick.price}\`);
    });

    client.realtime.connect();
  } catch (err) {
    console.error("API error:", err);
  }
}

run();
`;

  const getCurlSnippet = () => `# 1. Get Live Market Snapshot
curl -X GET "${BASE_API_URL}/api/v1/market/prices" \\
  -H "x-api-key: ${selectedKey}"

# 2. Query 1-Minute OHLCV Candles for Gold (XAUUSD)
curl -X GET "${BASE_API_URL}/api/v1/market/candles?symbol=XAUUSD&tf=1m" \\
  -H "x-api-key: ${selectedKey}"

# 3. Inspect Response Rate-Limit Headers
curl -s -I -X GET "${BASE_API_URL}/api/v1/market/prices" \\
  -H "x-api-key: ${selectedKey}" | grep -iE "x-ratelimit|x-dailyquota"
`;

  const getGoSnippet = () => `package main

import (
	"encoding/json"
	"fmt"
	"net/http"
)

const apiKey = "${selectedKey}"
const endpoint = "${BASE_API_URL}/api/v1/market/prices"

func main() {
	req, _ := http.NewRequest("GET", endpoint, nil)
	req.Header.Set("x-api-key", apiKey)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()

	fmt.Println("RateLimit-Remaining:", resp.Header.Get("X-RateLimit-Remaining"))
	fmt.Println("DailyQuota-Remaining:", resp.Header.Get("X-DailyQuota-Remaining"))

	var result map[string]interface{}
	json.NewDecoder(resp.Body).Decode(&result)
	fmt.Printf("Status: %s, Assets: %v\\n", resp.Status, result["total"])
}
`;

  return (
    <main className="account-page">
      <header className="account-header">
        <div>
          <span className="account-kicker">PIA / DEVELOPER PLATFORM</span>
          <h1>
            API REFERENCE &amp;<br />
            <em>INTEGRATION GUIDE.</em>
          </h1>
          <p>
            Connect algorithmic bots, terminal dashboards, and quantitative scripts to the high-performance ATLSD engine.
          </p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <a className="account-back" href="/portal/account">
            ← MANAGE API KEYS
          </a>
        </div>
      </header>

      {/* Active API Key Switcher Banner */}
      <div
        style={{
          padding: "12px 16px",
          marginBottom: 24,
          background: "rgba(9, 9, 238, 0.04)",
          border: "1px solid rgba(9, 9, 238, 0.2)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: 12,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
            ACTIVE EMBED KEY:
          </span>
          <code
            style={{
              padding: "4px 8px",
              background: "rgba(255, 255, 255, 0.8)",
              border: "1px solid rgba(9, 9, 238, 0.2)",
              color: "var(--blue)",
              font: "12px var(--font-geist-mono), monospace",
              fontWeight: 600,
            }}
          >
            {selectedKey}
          </code>
        </div>

        {user ? (
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <label style={{ font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
              SWITCH KEY:
            </label>
            <select
              value={selectedKey}
              onChange={(e) => setSelectedKey(e.target.value)}
              style={{
                padding: "4px 8px",
                font: "11px var(--font-geist-mono), monospace",
                border: "1px solid var(--blue)",
                background: "#fff",
                color: "var(--blue)",
              }}
            >
              {keys.map((k) => (
                <option key={k.id} value={`${k.key_prefix}••••••••••••`}>
                  {k.label} ({k.key_prefix}...)
                </option>
              ))}
            </select>
          </div>
        ) : (
          <a
            href="/portal/account"
            style={{
              font: "11px var(--font-geist-mono), monospace",
              color: "var(--blue)",
              textDecoration: "underline",
            }}
          >
            Sign in to auto-populate with your live keys →
          </a>
        )}
      </div>

      {/* SECTION 1: CODE GENERATOR */}
      <section className="account-card account-wide" style={{ marginBottom: 28 }}>
        <div className="account-card-head" style={{ marginBottom: 16 }}>
          <div>
            <span className="account-card-label">INTERACTIVE CODE GENERATOR</span>
            <h2>Quickstart Snippets</h2>
            <p className="account-muted">
              Ready-to-run clients pre-configured with your credentials.
            </p>
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            {(["python", "typescript", "curl", "go"] as const).map((lang) => (
              <button
                key={lang}
                type="button"
                className={`account-button ${activeLang === lang ? "account-button-primary" : ""}`}
                onClick={() => setActiveLang(lang)}
                style={{ padding: "0 12px", height: 32, fontSize: 10 }}
              >
                {lang.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        <div style={{ position: "relative" }}>
          <pre
            style={{
              padding: 16,
              background: "rgba(9, 9, 238, 0.03)",
              border: "1px solid rgba(9, 9, 238, 0.15)",
              color: "var(--blue)",
              font: "12px var(--font-geist-mono), monospace",
              overflowX: "auto",
              lineHeight: 1.5,
              margin: 0,
            }}
          >
            <code>
              {activeLang === "python" && getPythonSnippet()}
              {activeLang === "typescript" && getTypescriptSnippet()}
              {activeLang === "curl" && getCurlSnippet()}
              {activeLang === "go" && getGoSnippet()}
            </code>
          </pre>
          <button
            type="button"
            className="account-button"
            onClick={() => {
              const code =
                activeLang === "python"
                  ? getPythonSnippet()
                  : activeLang === "typescript"
                  ? getTypescriptSnippet()
                  : activeLang === "curl"
                  ? getCurlSnippet()
                  : getGoSnippet();
              handleCopy(code, "code-snippet");
            }}
            style={{
              position: "absolute",
              top: 10,
              right: 10,
              padding: "0 10px",
              height: 28,
              fontSize: 9,
            }}
          >
            {copiedSection === "code-snippet" ? "COPIED! ✓" : "COPY CODE"}
          </button>
        </div>
      </section>

      {/* SECTION: OFFICIAL CLIENT SDKs */}
      <section className="account-card account-wide" style={{ marginBottom: 28 }}>
        <div className="account-card-head" style={{ marginBottom: 16 }}>
          <div>
            <span className="account-card-label">CLIENT SDKs</span>
            <h2>Official SDK Libraries</h2>
            <p className="account-muted">
              Production-ready, strongly-typed SDK packages with built-in retry, rate-limit awareness, and realtime WebSocket streaming.
            </p>
          </div>
          <a
            className="account-button account-button-primary"
            href="https://github.com/wignn/pia-sdk"
            target="_blank"
            rel="noopener noreferrer"
            style={{ padding: "0 14px", height: 32, fontSize: 10, textDecoration: "none" }}
          >
            VIEW SOURCE ON GITHUB ↗
          </a>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(300px, 1fr))", gap: 16 }}>
          {/* TypeScript SDK */}
          <div style={{ padding: 16, background: "rgba(255,255,255,0.6)", border: "1px solid rgba(9,9,238,0.15)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 10 }}>
              <span style={{ padding: "2px 8px", background: "var(--blue)", color: "#fff", fontSize: 9, fontWeight: 700, fontFamily: "var(--font-geist-mono), monospace" }}>
                TYPESCRIPT
              </span>
              <a
                href="https://www.npmjs.com/package/@piaa/sdk"
                target="_blank"
                rel="noopener noreferrer"
                style={{ fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}
              >
                @piaa/sdk on npm ↗
              </a>
            </div>
            <pre style={{ margin: 0, padding: 10, background: "rgba(9,9,238,0.04)", border: "1px solid rgba(9,9,238,0.12)", fontSize: 11, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}>
{`npm install @piaa/sdk
# or
bun add @piaa/sdk`}
            </pre>
            <ul style={{ margin: "10px 0 0", padding: "0 0 0 16px", fontSize: 11, color: "#555", lineHeight: 1.7 }}>
              <li>ESM native bundle with full TypeScript declarations</li>
              <li>Exponential backoff with jitter on transient failures</li>
              <li>In-Band WebSocket Auth (no token in URL)</li>
              <li>Market, Social, News, Options resource modules</li>
              <li>Works in Node.js, Bun, Deno, and edge runtimes</li>
            </ul>
          </div>

          {/* Python SDK */}
          <div style={{ padding: 16, background: "rgba(255,255,255,0.6)", border: "1px solid rgba(9,9,238,0.15)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 10 }}>
              <span style={{ padding: "2px 8px", background: "var(--blue)", color: "#fff", fontSize: 9, fontWeight: 700, fontFamily: "var(--font-geist-mono), monospace" }}>
                PYTHON
              </span>
              <a
                href="https://pypi.org/project/piaa-sdk/"
                target="_blank"
                rel="noopener noreferrer"
                style={{ fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}
              >
                piaa-sdk on PyPI ↗
              </a>
            </div>
            <pre style={{ margin: 0, padding: 10, background: "rgba(9,9,238,0.04)", border: "1px solid rgba(9,9,238,0.12)", fontSize: 11, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}>
{`pip install piaa-sdk`}
            </pre>
            <ul style={{ margin: "10px 0 0", padding: "0 0 0 16px", fontSize: 11, color: "#555", lineHeight: 1.7 }}>
              <li>Dual engine: synchronous <code>PiaClient</code> + async <code>AsyncPiaClient</code></li>
              <li>Batteries-included: REST + WebSocket in one package</li>
              <li>Typed dataclasses for all API responses</li>
              <li>Automatic retry with rate-limit header awareness</li>
              <li>Python 3.8+ compatible</li>
            </ul>
          </div>
        </div>

        <div style={{ marginTop: 16, padding: "10px 14px", background: "rgba(9,9,238,0.03)", border: "1px dashed rgba(9,9,238,0.15)", fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "#686d9d", lineHeight: 1.6 }}>
          <strong>ARCHITECTURE:</strong> Both SDKs share the same API surface — <code>market</code>, <code>social</code>, <code>news</code>, <code>realtime</code> resource modules.
          All sensitive tokens are auto-redacted from logs and error stack traces.
          Source code, issue tracker, and contribution guide available at{" "}
          <a href="https://github.com/wignn/pia-sdk" target="_blank" rel="noopener noreferrer" style={{ color: "var(--blue)" }}>
            github.com/wignn/pia-sdk
          </a>.
        </div>
      </section>

      {/* SECTION 2: RATE LIMITS & TELEMETRY HEADERS */}
      <section className="account-card account-wide" style={{ marginBottom: 28 }}>
        <span className="account-card-label">GATEWAY PROTOCOL</span>
        <h2>Rate Limiting &amp; Telemetry Headers</h2>
        <p className="account-muted" style={{ marginBottom: 16 }}>
          Every request validated by the API Gateway returns real-time quota telemetry headers.
        </p>

        <div style={{ overflowX: "auto" }}>
          <table style={{ width: "100%", borderCollapse: "collapse", font: "11px var(--font-geist-mono), monospace" }}>
            <thead>
              <tr style={{ borderBottom: "2px solid var(--blue)", textAlign: "left" }}>
                <th style={{ padding: "8px 12px", color: "#7075a4" }}>HEADER</th>
                <th style={{ padding: "8px 12px", color: "#7075a4" }}>DESCRIPTION</th>
                <th style={{ padding: "8px 12px", color: "#7075a4" }}>EXAMPLE</th>
              </tr>
            </thead>
            <tbody>
              <tr style={{ borderBottom: "1px solid rgba(9, 9, 238, 0.1)" }}>
                <td style={{ padding: "8px 12px", fontWeight: 600, color: "var(--blue)" }}>X-RateLimit-Limit</td>
                <td style={{ padding: "8px 12px" }}>Maximum allowed requests per minute window.</td>
                <td style={{ padding: "8px 12px" }}><code>60</code></td>
              </tr>
              <tr style={{ borderBottom: "1px solid rgba(9, 9, 238, 0.1)" }}>
                <td style={{ padding: "8px 12px", fontWeight: 600, color: "var(--blue)" }}>X-RateLimit-Remaining</td>
                <td style={{ padding: "8px 12px" }}>Remaining requests in current 60-second window.</td>
                <td style={{ padding: "8px 12px" }}><code>54</code></td>
              </tr>
              <tr style={{ borderBottom: "1px solid rgba(9, 9, 238, 0.1)" }}>
                <td style={{ padding: "8px 12px", fontWeight: 600, color: "var(--blue)" }}>X-RateLimit-Reset</td>
                <td style={{ padding: "8px 12px" }}>Seconds until the current minute window resets.</td>
                <td style={{ padding: "8px 12px" }}><code>28</code></td>
              </tr>
              <tr style={{ borderBottom: "1px solid rgba(9, 9, 238, 0.1)" }}>
                <td style={{ padding: "8px 12px", fontWeight: 600, color: "var(--blue)" }}>X-DailyQuota-Limit</td>
                <td style={{ padding: "8px 12px" }}>Total daily request quota assigned to your plan tier.</td>
                <td style={{ padding: "8px 12px" }}><code>5000</code></td>
              </tr>
              <tr style={{ borderBottom: "1px solid rgba(9, 9, 238, 0.1)" }}>
                <td style={{ padding: "8px 12px", fontWeight: 600, color: "var(--blue)" }}>X-DailyQuota-Remaining</td>
                <td style={{ padding: "8px 12px" }}>Remaining request quota for today (resets 00:00 UTC).</td>
                <td style={{ padding: "8px 12px" }}><code>4892</code></td>
              </tr>
              <tr>
                <td style={{ padding: "8px 12px", fontWeight: 600, color: "#a34d4d" }}>HTTP 429 Payload</td>
                <td style={{ padding: "8px 12px" }} colSpan={2}>
                  Returned when burst rate limit or daily envelope is exceeded:
                  <pre style={{ margin: "6px 0 0", padding: 8, background: "rgba(163,77,77,0.06)", border: "1px solid rgba(163,77,77,0.2)", fontSize: 10 }}>
{`{
  "error": "rate_limit_exceeded",
  "message": "Burst rate limit of 60 req/min exceeded. Please retry after 24 seconds.",
  "limit": 60,
  "retry_after_seconds": 24,
  "upgrade_url": "https://pia.wign.dev/portal/account"
}`}
                  </pre>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      {/* SECTION 3: REST API CATALOG */}
      <section className="account-card account-wide" style={{ marginBottom: 28 }}>
        <span className="account-card-label">REST INTERFACE</span>
        <h2>Core Endpoints Reference</h2>
        <p className="account-muted" style={{ marginBottom: 20 }}>
          High-throughput endpoints serving pre-aggregated financial data.
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          {REST_ENDPOINTS.map((ep) => (
            <article
              key={ep.path}
              style={{
                padding: 16,
                background: "rgba(255, 255, 255, 0.6)",
                border: "1px solid rgba(9, 9, 238, 0.15)",
              }}
            >
              <div style={{ display: "flex", alignItems: "baseline", gap: 10, marginBottom: 6 }}>
                <span
                  style={{
                    padding: "2px 6px",
                    background: "var(--blue)",
                    color: "#fff",
                    fontSize: 9,
                    fontWeight: 700,
                    fontFamily: "var(--font-geist-mono), monospace",
                  }}
                >
                  {ep.method}
                </span>
                <code style={{ fontSize: 13, fontWeight: 600, color: "var(--blue)", fontFamily: "var(--font-geist-mono), monospace" }}>
                  {ep.path}
                </code>
              </div>
              <h3 style={{ margin: "4px 0 8px", fontSize: 14 }}>{ep.title}</h3>
              <p className="account-muted" style={{ margin: 0, fontSize: 12, lineHeight: 1.5 }}>
                {ep.desc}
              </p>
              <div style={{ marginTop: 8, fontSize: 11, font: "10px var(--font-geist-mono), monospace", color: "#686d9d" }}>
                <strong>PARAMETERS:</strong> {ep.params}
              </div>

              <details style={{ marginTop: 12 }}>
                <summary style={{ cursor: "pointer", fontSize: 10, color: "var(--blue)", fontFamily: "var(--font-geist-mono), monospace" }}>
                  VIEW SAMPLE RESPONSE JSON
                </summary>
                <pre
                  style={{
                    marginTop: 8,
                    padding: 12,
                    background: "rgba(9, 9, 238, 0.04)",
                    border: "1px dotted rgba(9, 9, 238, 0.2)",
                    fontSize: 10,
                    fontFamily: "var(--font-geist-mono), monospace",
                    color: "var(--blue)",
                    maxHeight: 200,
                    overflowY: "auto",
                  }}
                >
                  <code>{ep.sampleResponse}</code>
                </pre>
              </details>
            </article>
          ))}
        </div>
      </section>

      {/* SECTION 4: WEBSOCKET STREAMING */}
      <section className="account-card account-wide">
        <span className="account-card-label">REALTIME STREAMING</span>
        <h2>WebSocket Gateway Protocol</h2>
        <p className="account-muted" style={{ marginBottom: 16 }}>
          Sub-millisecond ticker feed streaming supporting both In-Band Message Authentication and Handshake Tickets.
        </p>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 16 }}>
          <div style={{ padding: 14, background: "rgba(255,255,255,0.6)", border: "1px solid rgba(9,9,238,0.15)" }}>
            <h3 style={{ fontSize: 13, margin: "0 0 8px" }}>1. In-Band Auth (Cross-Platform)</h3>
            <p className="account-muted" style={{ fontSize: 11, lineHeight: 1.5, margin: 0 }}>
              Connect clean without query tokens, then send an auth frame within 5s:
            </p>
            <pre style={{ margin: "8px 0 0", padding: 8, background: "rgba(9,9,238,0.04)", fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}>
{`${BASE_WS_URL}

Send frame:
{
  "action": "auth",
  "api_key": "${selectedKey}"
}`}
            </pre>
          </div>

          <div style={{ padding: 14, background: "rgba(255,255,255,0.6)", border: "1px solid rgba(9,9,238,0.15)" }}>
            <h3 style={{ fontSize: 13, margin: "0 0 8px" }}>2. Ephemeral Ticket (Browser)</h3>
            <p className="account-muted" style={{ fontSize: 11, lineHeight: 1.5, margin: 0 }}>
              Request a 60s single-use ticket via REST, then connect safely:
            </p>
            <pre style={{ margin: "8px 0 0", padding: 8, background: "rgba(9,9,238,0.04)", fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}>
{`POST /api/v1/ws/ticket (Header: x-api-key)
-> {"ticket": "wst_..."}

wss://api-engine.wign.dev/api/v1/ws?ticket=wst_...`}
            </pre>
          </div>

          <div style={{ padding: 14, background: "rgba(255,255,255,0.6)", border: "1px solid rgba(9,9,238,0.15)" }}>
            <h3 style={{ fontSize: 13, margin: "0 0 8px" }}>3. Dynamic Subscription</h3>
            <p className="account-muted" style={{ fontSize: 11, lineHeight: 1.5, margin: 0 }}>
              Subscribe to specific symbols or multiple channels dynamically:
            </p>
            <pre style={{ margin: "8px 0 0", padding: 8, background: "rgba(9,9,238,0.04)", fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "var(--blue)" }}>
{`{
  "action": "subscribe",
  "symbols": ["XAUUSD", "BTCUSDT", "SPX"]
}`}
            </pre>
          </div>
        </div>
      </section>

      <footer className="account-footer" style={{ marginTop: 32 }}>
        <span>PIA / DEVELOPER PLATFORM</span>
        <span>ZERO DOWNTIME API ENGINE · RFC 6585 RATE LIMITS · SDK: <a href="https://github.com/wignn/pia-sdk" target="_blank" rel="noopener noreferrer" style={{ color: "inherit", textDecoration: "underline" }}>github.com/wignn/pia-sdk</a></span>
      </footer>
    </main>
  );
}
