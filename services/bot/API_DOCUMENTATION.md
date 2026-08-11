# 📖 ATLSD API Comprehensive Developer Documentation

Selamat datang di dokumentasi resmi API **ATLSD (Advanced Trading & Financial Intelligence Engine)**. Dokumentasi ini disusun secara detail mencakup semua endpoint, parameter, struktur request body, serta contoh respons JSON untuk membantu pengembang mengintegrasikan platform ATLSD.

---

## 📑 Daftar Isi
1. [Overview & Base URLs](#1-overview--base-urls)
2. [Autentikasi & Authorization](#2-autentikasi--authorization)
3. [Format Respons & Error Codes](#3-format-respons--error-codes)
4. [Market Data API](#4-market-data-api)
   - [Get All Prices](#get-all-prices)
   - [Get Price by Symbol](#get-price-by-symbol)
   - [Get Price History (OHLCV)](#get-price-history-ohlcv)
   - [Get Market Session](#get-market-session)
   - [Get Market Spikes](#get-market-spikes)
   - [Get Data Quality Metrics](#get-data-quality-metrics)
5. [Options Data Pack API](#5-options-data-pack-api)
   - [Get Options Summary](#get-options-summary)
   - [Get Options Chain](#get-options-chain)
   - [Get Options GEX (Gamma Exposure)](#get-options-gex-gamma-exposure)
6. [News & Macroeconomic Intelligence API](#6-news--macroeconomic-intelligence-api)
   - [Get Economic Calendar](#get-economic-calendar)
   - [Get Forex News (Latest)](#get-forex-news-latest)
   - [Get Stock News](#get-stock-news)
   - [Get Macro Dashboard](#get-macro-dashboard)
   - [Get SEC Filings](#get-sec-filings)
   - [Get Central Bank Documents](#get-central-bank-documents)
   - [Get Geosignals](#get-geosignals)
7. [AI & Financial Intelligence API](#7-ai--financial-intelligence-api)
   - [Market "Why Did It Move" Analysis](#market-why-did-it-move-analysis)
   - [Text Sentiment Analysis](#text-sentiment-analysis)
8. [Control Plane & Tenant Management API](#8-control-plane--tenant-management-api)
   - [User Authentication (Login)](#user-authentication-login)
   - [List API Keys](#list-api-keys)
   - [Create API Key](#create-api-key)
   - [Get Subscription Plans](#get-subscription-plans)
   - [Get Daily API Usage](#get-daily-api-usage)
9. [Realtime WebSocket API](#9-realtime-websocket-api)

---

## 1. Overview & Base URLs

| Environment | REST API Base URL | Realtime WebSocket URL |
|---|---|---|
| **Production Gateway** | `https://slv-gateway.wign.dev/api/v1` | `wss://slv-realtime.wign.dev/ws/v1` |
| **Engine Direct Gateway** | `https://api-engine.wign.dev/api/v1` | `wss://realtime-engine.wign.dev/ws/v1` |
| **Local Development** | `http://localhost:8000/api/v1` | `ws://localhost:8020/ws/v1` |

---

## 2. Autentikasi & Authorization

Semua request yang menuju ke endpoint terlindungi (`/api/v1/*`) memerlukan API Key valid dengan format `wi_live_<string_hex_48_karakter>`.

### Cara Mengirim API Key:
1. **HTTP Header (Direkomendasikan):**
   ```http
   X-API-Key: wi_live_7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e
   ```
2. **Authorization Bearer Header:**
   ```http
   Authorization: Bearer wi_live_7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e
   ```
3. **URL Query Parameter:**
   ```http
   GET https://slv-gateway.wign.dev/api/v1/market/prices?api_key=wi_live_7f8a9b0c...
   ```

*Catatan: Preflight request (`OPTIONS`) dapat dilakukan tanpa menyertakan API Key.*

---

## 3. Format Respons & Error Codes

### Standard HTTP Status Codes:
- `200 OK` — Request berhasil.
- `400 Bad Request` — Parameter query atau body request tidak valid.
- `401 Unauthorized` — API Key hilang atau tidak valid.
- `403 Forbidden` — API Key tidak memiliki izin (*permission*) untuk mengakses resource ini.
- `429 Too Many Requests` — Kuota panggilan harian API telah habis.
- `500 Internal Server Error` — Kesalahan sistem internal.

### Standard JSON Error Payload:
```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid or expired API Key provided."
}
```

---

## 4. Market Data API

### Get All Prices
Mengambil snapshot harga pasar terkini untuk semua instrumen keuangan.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/prices`
- **Auth Required:** Ya

#### Response Example (`200 OK`):
```json
{
  "timestamp": "2026-08-11T12:00:00Z",
  "data": [
    {
      "symbol": "SPX",
      "asset_class": "Index",
      "price": 5420.50,
      "bid": 5420.25,
      "ask": 5420.75,
      "change_24h": 12.40,
      "change_pct_24h": 0.23,
      "updated_at": "2026-08-11T11:59:58Z"
    },
    {
      "symbol": "XAUUSD",
      "asset_class": "Commodity",
      "price": 2415.80,
      "bid": 2415.70,
      "ask": 2415.90,
      "change_24h": -5.20,
      "change_pct_24h": -0.21,
      "updated_at": "2026-08-11T12:00:00Z"
    }
  ]
}
```

---

### Get Price by Symbol
Mengambil data harga snapshot instrumen spesifik.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/prices/{symbol}`
- **Path Parameters:**
  - `symbol` (string, required) — Kode simbol instrumen (contoh: `SPX`, `XAUUSD`, `BTCUSDT`, `DXY`).

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPX",
  "asset_class": "Index",
  "venue": "tiingo",
  "price": 5420.50,
  "bid": 5420.25,
  "ask": 5420.75,
  "volume": 1245000.0,
  "ts_exchange": "2026-08-11T11:59:58Z",
  "ts_received": "2026-08-11T12:00:00Z"
}
```

---

### Get Price History (OHLCV)
Mengambil data historis candlestick (OHLCV).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/history/{symbol}`
- **Query Parameters:**
  - `resolution` (string, optional) — Waktu resolusi candle (`1s`, `1m`, `5m`, `15m`, `1h`, `1d`). Default: `1m`.
  - `limit` (integer, optional) — Jumlah candle yang diambil (1 - 1000). Default: `100`.

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPX",
  "resolution": "1m",
  "candles": [
    {
      "bucket_start": "2026-08-11T11:58:00Z",
      "open": 5418.00,
      "high": 5421.20,
      "low": 5417.80,
      "close": 5420.50,
      "volume": 1420.0,
      "tick_count": 85
    }
  ]
}
```

---

### Get Market Session
Mendapatkan status sesi perdagangan (buka/tutup/pre-market/after-hours).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/session/{symbol}`

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPX",
  "is_open": true,
  "session_type": "Regular",
  "next_close": "2026-08-11T20:00:00Z"
}
```

---

### Get Market Spikes
Menampilkan indikasi lonjakan volatilitas atau anomali harga.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/spikes`

#### Response Example (`200 OK`):
```json
{
  "spikes": [
    {
      "symbol": "BTCUSDT",
      "magnitude_pct": 2.45,
      "direction": "UP",
      "detected_at": "2026-08-11T11:55:10Z"
    }
  ]
}
```

---

### Get Data Quality Metrics
Metrik kualitas ingestion data pasar.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/data-quality`

#### Response Example (`200 OK`):
```json
{
  "overall_score": 0.998,
  "sources": [
    {
      "name": "tiingo",
      "status": "Ok",
      "latency_ms": 42
    }
  ]
}
```

---

## 5. Options Data Pack API

### Get Options Summary
Menampilkan ringkasan volume opsi (Put/Call ratio, Put/Call volume).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/options/summary`
- **Query Parameters:**
  - `symbol` (string, required) — Kode saham/indeks underlying (contoh: `SPY`, `AAPL`).

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPY",
  "underlying_price": 540.25,
  "total_call_volume": 450210,
  "total_put_volume": 380120,
  "put_call_ratio": 0.844,
  "updated_at": "2026-08-11T12:00:00Z"
}
```

---

### Get Options Chain
Data rantai opsi lengkap (Contracts Call & Put).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/options/chain`
- **Query Parameters:**
  - `symbol` (string, required) — Simbol underlying.

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPY",
  "underlying_price": 540.25,
  "expiration_dates": ["2026-08-15", "2026-08-22"],
  "chain": [
    {
      "contract_symbol": "SPY260815C00540000",
      "option_type": "call",
      "strike": 540.0,
      "expiration": "2026-08-15",
      "bid": 3.45,
      "ask": 3.55,
      "implied_volatility": 0.142,
      "delta": 0.52,
      "gamma": 0.045,
      "open_interest": 12540
    }
  ]
}
```

---

### Get Options GEX (Gamma Exposure)
Menampilkan analisis Gamma Exposure (GEX) untuk mendeteksi area support/resistance dealer opsi.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/options/gex`
- **Query Parameters:**
  - `symbol` (string, required) — Simbol underlying (`SPY`).

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPY",
  "underlying_price": 540.25,
  "total_gex": 142500000.0,
  "zero_gamma_strike": 538.0,
  "gex_by_strike": [
    {
      "strike": 540.0,
      "call_gex": 25000000.0,
      "put_gex": -12000000.0,
      "net_gex": 13000000.0
    }
  ]
}
```

---

## 6. News & Macroeconomic Intelligence API

### Get Economic Calendar
Kalender agenda ekonomi dunia (misal CPI, NFP, keputusan suku bunga).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/forex/calendar`
- **Query Parameters:**
  - `impact` (string, optional) — Filter dampak (`high`, `medium`, `low`). Default: `high`.
  - `limit` (integer, optional) — Jumlah entri (1-100). Default: `15`.

#### Response Example (`200 OK`):
```json
{
  "events": [
    {
      "id": "evt-8812",
      "country": "USD",
      "title": "Core CPI (MoM)",
      "impact": "high",
      "time": "2026-08-12T12:30:00Z",
      "forecast": "0.3%",
      "previous": "0.2%",
      "actual": null
    }
  ]
}
```

---

### Get Forex News (Latest)
Berita pasar mata uang & Forex terbaru dari berbagai feed terverifikasi.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/forex/news/latest`
- **Query Parameters:**
  - `limit` (integer, optional) — Jumlah berita. Default: `15`.

#### Response Example (`200 OK`):
```json
{
  "news": [
    {
      "id": "news-9012",
      "source": "reuters",
      "headline": "Dollar steady ahead of US inflation data",
      "sentiment": "neutral",
      "url": "https://...",
      "published_at": "2026-08-11T11:45:00Z"
    }
  ]
}
```

---

### Get Stock News
Berita ekosistem pasar saham.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/stock/news`
- **Query Parameters:**
  - `limit` (integer, optional) — Jumlah berita. Default: `15`.

---

### Get Macro Dashboard
Ringkasan indikator makroekonomi global.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/macro/dashboard`

---

### Get SEC Filings
Filings dokumen resmi dari SEC Amerika Serikat (10-K, 10-Q, 8-K).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/sec/filings`

---

### Get Central Bank Documents
Dokumen & rilis resmi Bank Sentral (Federal Reserve, ECB, BOE, BOJ).

- **HTTP Method:** `GET`
- **Path:** `/api/v1/central-banks/latest`

---

### Get Geosignals
Sinyal insiden geopolitik berpengaruh pasar.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/geosignals`

---

## 7. AI & Financial Intelligence API

### Market "Why Did It Move" Analysis
Penjelasan berbasis AI mengenai alasan terjadinya pergerakan signifikan pada harga pasar.

- **HTTP Method:** `GET`
- **Path:** `/api/v1/market/why/{symbol}`
- **Query Parameters:**
  - `window` (string, optional) — Rentang waktu pergerakan (`5m`, `15m`, `1h`, `24h`). Default: `5m`.

#### Response Example (`200 OK`):
```json
{
  "symbol": "SPX",
  "window": "5m",
  "price_change_pct": -0.45,
  "summary": "SPX terkoreksi -0.45% dalam 5 menit terakhir dipicu oleh rilis laporan tak terduga terkait penundaan kebijakan suku bunga.",
  "top_drivers": [
    {
      "headline": "Fed Official hints at prolonged higher rates",
      "relevance_score": 0.94
    }
  ]
}
```

---

### Text Sentiment Analysis
Melakukan ekstraksi sentimen dan entity recognition dari teks finansial secara instan.

- **HTTP Method:** `POST`
- **Path:** `/api/v1/analyze`
- **Headers:** `Content-Type: application/json`

#### Request Body Example:
```json
{
  "text": "Apple reported record Q3 earnings beating analyst estimates on iPhone sales, though guidance remains cautious."
}
```

#### Response Example (`200 OK`):
```json
{
  "sentiment": "bullish",
  "confidence": 0.88,
  "entities": [
    {"name": "Apple", "type": "company", "ticker": "AAPL"}
  ],
  "summary": "Record Q3 earnings and iPhone sales drive bullish sentiment."
}
```

---

## 8. Control Plane & Tenant Management API

### User Authentication (Login)
- **HTTP Method:** `POST`
- **Path:** `/api/v1/auth/login`
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "yourpassword"
  }
  ```
- **Response:**
  ```json
  {
    "token": "eyJhbGciOiJIUzI1Ni...",
    "user": {
      "id": "usr-1234",
      "email": "user@example.com"
    }
  }
  ```

---

### List API Keys
- **HTTP Method:** `GET`
- **Path:** `/api/v1/keys`

---

### Create API Key
- **HTTP Method:** `POST`
- **Path:** `/api/v1/keys`
- **Request Body:**
  ```json
  {
    "name": "My Trading Bot Key"
  }
  ```

---

### Get Subscription Plans
- **HTTP Method:** `GET`
- **Path:** `/api/v1/plans`

---

### Get Daily API Usage
- **HTTP Method:** `GET`
- **Path:** `/api/v1/usage`

---

## 9. Realtime WebSocket API

API WebSocket menyediakan stream data pasar secara instan tanpa perlu polling HTTP.

- **Connection URL:** `wss://slv-realtime.wign.dev/ws/v1?api_key=wi_live_xxxxxxxx`

### Format Payload Subscribe (Client -> Server):
```json
{
  "action": "subscribe",
  "channels": ["market_prices", "news_feed"]
}
```

### Format Event Tick Data (Server -> Client):
```json
{
  "channel": "market_prices",
  "event": "tick",
  "data": {
    "symbol": "XAUUSD",
    "price": 2416.10,
    "bid": 2416.00,
    "ask": 2416.20,
    "timestamp": "2026-08-11T12:01:05Z"
  }
}
```

---

Dokumentasi ini mencakup secara menyeluruh seluruh endpoint ATLSD API v1.
