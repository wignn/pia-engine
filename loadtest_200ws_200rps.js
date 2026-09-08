import ws from 'k6/ws';
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  scenarios: {
    // Skenario 1: 200 WebSocket koneksi bertahan secara bersamaan selama 5 menit
    ws_200_concurrent: {
      executor: 'constant-vus',
      vus: 200,
      duration: '5m',
      exec: 'wsScenario',
      gracefulStop: '10s',
    },
    // Skenario 2: 200 HTTP Request per detik secara konstan selama 5 menit
    http_200_rps: {
      executor: 'constant-arrival-rate',
      rate: 200,
      timeUnit: '1s',
      duration: '5m',
      preAllocatedVUs: 50,
      maxVUs: 250,
      exec: 'httpScenario',
      gracefulStop: '10s',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'], // Gagal HTTP < 1%
    http_req_duration: ['p(95)<150'], // 95% request di bawah 150ms
    ws_connecting: ['p(95)<200'], // 95% WS handshake di bawah 200ms
  },
};

// Skenario WebSocket (200 VU)
export function wsScenario() {
  // 1. Minta tiket sesi realtime dari gateway
  const ticketRes = http.post(
    'http://127.0.0.1:8020/api/v1/ws/ticket',
    null,
    { headers: { 'x-api-key': 'silvia' } }
  );

  const ticketOk = check(ticketRes, {
    'ticket issued (200)': (r) => r.status === 200,
  });

  if (!ticketOk) return;

  const ticket = JSON.parse(ticketRes.body).ticket;
  const wsUrl = `ws://127.0.0.1:8020/ws/v1?bot_id=k6_load_${__VU}&ticket=${ticket}`;

  // 2. Buka koneksi WebSocket dan pertahankan selama 5 menit
  const res = ws.connect(wsUrl, {}, function (socket) {
    socket.on('open', () => {
      // Subscribe ke data feed pasar
      socket.send(
        JSON.stringify({
          method: 'SUBSCRIBE',
          params: ['market_data', 'forex_news'],
          id: 1,
        })
      );
    });

    socket.on('message', (msg) => {
      // Menerima tick pasar realtime
    });

    socket.on('error', (err) => {
      console.error(`[VU ${__VU}] WS Error:`, err);
    });

    // Pertahankan koneksi selama 295 detik (hampir 5 menit)
    socket.setTimeout(() => {
      socket.close();
    }, 295000);
  });

  check(res, {
    'ws handshake 101': (r) => r && r.status === 101,
  });
}

// Skenario HTTP 200 req/s
export function httpScenario() {
  // Bergantian request harga live dan candle history
  const endpoints = [
    'http://127.0.0.1:8000/api/v1/market/prices',
    'http://127.0.0.1:8000/api/v1/market/history/BTCUSDT?resolution=1m',
    'http://127.0.0.1:8000/api/v1/market/history/XAUUSD?resolution=15m',
  ];
  const url = endpoints[__ITER % endpoints.length];

  const res = http.get(url, {
    headers: {
      'x-api-key': 'silvia',
      'Accept': 'application/json',
    },
  });

  check(res, {
    'http status 200': (r) => r.status === 200,
  });
}
