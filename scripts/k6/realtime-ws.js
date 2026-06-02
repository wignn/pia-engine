import ws from 'k6/ws';
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

const WS_URL = __ENV.WS_URL || 'ws://localhost:8020';
const API_KEY = __ENV.API_KEY || '';
const BOT_ID = __ENV.BOT_ID || 'k6_stress';
const SYMBOL = __ENV.SYMBOL || 'EURUSD';
const USE_TICKET = (__ENV.USE_TICKET || 'false').toLowerCase() === 'true';
const HOLD_SECONDS = Number(__ENV.HOLD_SECONDS || 30);
const MESSAGE_INTERVAL_SECONDS = Number(__ENV.MESSAGE_INTERVAL_SECONDS || 10);

export const wsConnects = new Counter('ws_connects');
export const wsMessages = new Counter('ws_messages');
export const wsErrors = new Counter('ws_errors');
export const wsConnected = new Rate('ws_connected');
export const atlsdWsSessionDuration = new Trend('atlsd_ws_session_duration');

export const options = {
  scenarios: {
    ws_ramp: {
      executor: 'ramping-vus',
      stages: [
        { duration: __ENV.RAMP_UP || '30s', target: Number(__ENV.VUS || 20) },
        { duration: __ENV.HOLD || '1m', target: Number(__ENV.VUS || 20) },
        { duration: __ENV.RAMP_DOWN || '20s', target: 0 },
      ],
      gracefulRampDown: '10s',
    },
  },
  thresholds: {
    ws_connected: ['rate>0.95'],
    ws_errors: ['count<10'],
  },
};

function joinUrl(base, path) {
  return `${base.replace(/\/$/, '')}/${path.replace(/^\//, '')}`;
}

function httpBaseFromWs(base) {
  return base.replace(/^wss:/, 'https:').replace(/^ws:/, 'http:');
}

function ticket() {
  if (!USE_TICKET) return null;
  const res = http.post(joinUrl(httpBaseFromWs(WS_URL), '/api/v1/ws/ticket'), null, {
    headers: { 'X-API-Key': API_KEY },
  });
  check(res, { 'ticket status is 200': (r) => r.status === 200 });
  if (res.status !== 200) return null;
  return res.json('ticket');
}

function query(params) {
  return Object.entries(params)
    .filter(([, value]) => value !== undefined && value !== null && value !== '')
    .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`)
    .join('&');
}

function connectionUrl() {
  const t = ticket();
  const auth = t ? { ticket: t } : { api_key: API_KEY };
  return `${joinUrl(WS_URL, '/ws/v1')}?${query({
    bot_id: `${BOT_ID}_${__VU}_${__ITER}`,
    symbols: SYMBOL,
    ...auth,
  })}`;
}

export default function () {
  const startedAt = Date.now();
  const res = ws.connect(connectionUrl(), {}, (socket) => {
    wsConnects.add(1);
    wsConnected.add(1);

    socket.on('open', () => {
      socket.send(JSON.stringify({ type: 'subscribe', symbols: [SYMBOL] }));
    });

    socket.on('message', () => {
      wsMessages.add(1);
    });

    socket.on('error', () => {
      wsErrors.add(1);
      wsConnected.add(0);
    });

    const interval = socket.setInterval(() => {
      socket.send(JSON.stringify({ type: 'ping', ts: Date.now() }));
    }, MESSAGE_INTERVAL_SECONDS * 1000);

    socket.setTimeout(() => {
      socket.clearInterval(interval);
      socket.close();
    }, HOLD_SECONDS * 1000);
  });

  check(res, { 'ws upgrade is 101': (r) => r && r.status === 101 });
  if (!res || res.status !== 101) {
    wsErrors.add(1);
    wsConnected.add(0);
  }
  atlsdWsSessionDuration.add(Date.now() - startedAt);
  sleep(1);
}
