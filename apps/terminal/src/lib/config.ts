// Terminal app runtime config.
// Server-side calls go through the internal traffic-router; the browser WS
// uses the public realtime endpoint.

// Client-exposed (inlined at build time by Next via NEXT_PUBLIC_*).
export const PUBLIC_CORE_WS_URL =
  process.env.NEXT_PUBLIC_CORE_WS_URL || "wss://realtime-engine.wign.dev";
export const PUBLIC_CORE_REST_URL =
  process.env.NEXT_PUBLIC_CORE_REST_URL || "https://api-engine.wign.dev";

// Server-only (never shipped to the browser).
export const CORE_REST_URL = process.env.CORE_REST_URL || "http://traffic-router";
export const CORE_API_KEY = process.env.CORE_API_KEY || "silvia";

// Realtime ticket issuance endpoint. Server-side we hit the traffic-router,
// which routes /api/v1/ws* to the active realtime-gateway.
export const CORE_WS_HTTP_TICKET_URL =
  process.env.CORE_WS_HTTP_TICKET_URL ||
  `${CORE_REST_URL}/api/v1/ws/ticket`;
