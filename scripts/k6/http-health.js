import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8000';
const API_KEY = __ENV.API_KEY || '';
const PATHS = (__ENV.PATHS || '/health').split(',').map((path) => path.trim()).filter(Boolean);

export const okRate = new Rate('http_ok_rate');
export const latency = new Trend('http_latency');

export const options = {
  scenarios: {
    http_ramp: {
      executor: 'ramping-vus',
      stages: [
        { duration: __ENV.RAMP_UP || '30s', target: Number(__ENV.VUS || 50) },
        { duration: __ENV.HOLD || '1m', target: Number(__ENV.VUS || 50) },
        { duration: __ENV.RAMP_DOWN || '20s', target: 0 },
      ],
      gracefulRampDown: '10s',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<500'],
  },
};

function joinUrl(base, path) {
  return `${base.replace(/\/$/, '')}/${path.replace(/^\//, '')}`;
}

export default function () {
  const path = PATHS[Math.floor(Math.random() * PATHS.length)];
  const res = http.get(joinUrl(BASE_URL, path), {
    headers: API_KEY ? { 'X-API-Key': API_KEY } : {},
  });
  const ok = check(res, {
    'status is 2xx/3xx': (r) => r.status >= 200 && r.status < 400,
  });
  okRate.add(ok);
  latency.add(res.timings.duration);
  sleep(Number(__ENV.SLEEP || 1));
}
