# k6 Stress Tests

Small load-test scripts for ATLSD HTTP and realtime WebSocket paths.

## WebSocket realtime

Local direct API key test:

```bash
API_KEY=olin WS_URL=ws://localhost:8020 VUS=20 HOLD=1m HOLD_SECONDS=30 k6 run scripts/k6/realtime-ws.js
```

VPS/public test:

```bash
API_KEY=*** WS_URL=wss://realtime-engine.wign.dev VUS=20 HOLD=1m HOLD_SECONDS=30 k6 run scripts/k6/realtime-ws.js
```

Ticket-based test:

```bash
API_KEY=*** WS_URL=wss://realtime-engine.wign.dev USE_TICKET=true VUS=20 HOLD=1m HOLD_SECONDS=30 k6 run scripts/k6/realtime-ws.js
```

Useful knobs:

- `VUS` - concurrent virtual users / WS clients.
- `HOLD` - duration to hold target VUs.
- `HOLD_SECONDS` - how long each socket stays open.
- `SYMBOL` - symbol sent in the subscribe message, default `EURUSD`.
- `USE_TICKET=true` - request `/api/v1/ws/ticket` first, then connect with `ticket`.

## HTTP health

```bash
BASE_URL=http://localhost:8000 VUS=50 HOLD=1m k6 run scripts/k6/http-health.js
```

Multiple paths:

```bash
BASE_URL=https://api-atlsd.wign.cloud PATHS=/health,/api/v1/market/quotes VUS=50 k6 run scripts/k6/http-health.js
```

## Watch while running

Grafana panels to watch:

- Service status blocks
- `Active WS`
- `WS Throughput`
- `Fanout / Backpressure / Rejects`
- CPU / Memory
- Docker logs error rate

Start small first. For VPS, try `VUS=20`, then `50`, `100`, and only then go higher.
