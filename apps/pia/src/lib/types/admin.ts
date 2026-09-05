export type PlanId = 'free' | 'starter' | 'pro' | 'enterprise';

export interface AdminUser {
  id: string;
  email: string;
  name: string;
  plan: PlanId | string;
  is_active: boolean;
  email_verified: boolean;
  created_at: string;
  active_keys: number;
}

export interface AdminStats {
  total_users: number;
  active_users: number;
  total_api_keys: number;
  users_by_plan: Record<string, number>;
}

export interface AdminApiKey {
  id: string;
  key_prefix: string;
  label: string;
  permissions: string[];
  max_ws_connections: number | null;
  is_active: boolean;
  last_used_at: string | null;
  expires_at: string | null;
  created_at: string;
}

export interface Plan {
  id: PlanId | string;
  name: string;
  price_idr: number;
  requests_per_day: number;
  ws_connections: number;
  x_usernames_max: number;
  tv_symbols_max: number;
  news_history_days: number;
  rate_limit_per_min: number;
  can_scrape: boolean;
  can_custom_rss: boolean;
  is_active: boolean;
  sort_order: number;
}

export interface HealthStatus {
  status: 'healthy' | 'unhealthy';
  service: string;
  latency_ms?: number;
  error?: string;
}

export interface FeedSourceStatus {
  name: string;
  url: string;
  rss_url: string;
  category: string;
  status: 'pending' | 'ok' | 'error' | 'blocked' | string;
  last_success_at: string | null;
  last_error_at: string | null;
  blocked_until: string | null;
  next_allowed_poll_at: string | null;
  consecutive_403: number;
  success_count: number;
  error_count: number;
  forbidden_count: number;
  parse_error_count: number;
  last_status: number | null;
  last_latency_ms: number | null;
}

export interface FeedSourceStatusResponse {
  items: FeedSourceStatus[];
  total: number;
}

export interface MarketPriceSnapshot {
  symbol: string;
  price: number;
  bid: number | null;
  ask: number | null;
  volume: number | null;
  source: string;
  asset_type: string;
  received_at: string | null;
  session?: {
    exchange: string;
    timezone: string;
    state: 'open' | 'closed' | 'break' | 'unknown' | string;
    is_open: boolean;
    reason: string;
  };
}

export interface MarketPricesResponse {
  items: MarketPriceSnapshot[];
  total: number;
}

export interface MarketDataQualityItem {
  symbol: string;
  asset_type: string;
  latest_price: number;
  source: string;
  received_at: string | null;
  age_sec: number | null;
  ticks_5m: number;
  ticks_1h: number;
  unique_prices_1h: number;
  status: 'ok' | 'flat' | 'stale' | 'quiet' | 'unknown' | string;
}

export interface MarketDataQualityResponse {
  items: MarketDataQualityItem[];
  total: number;
  generated_at: string;
}

export interface MarketVolatilitySpike {
  symbol: string;
  asset_type: string;
  window: string;
  latest_price: number;
  baseline_price: number;
  move_pct: number;
  direction: 'up' | 'down' | string;
  severity: 'medium' | 'high' | string;
  threshold_pct: number;
  tick_count: number;
  latest_at: string;
}

export interface MarketVolatilitySpikesResponse {
  items: MarketVolatilitySpike[];
  total: number;
  window: string;
  generated_at: string;
}

export interface MacroSignal {
  series_id: string;
  title: string;
  category: string;
  signal_date: string;
  latest_value: number | null;
  change_1d: number | null;
  change_7d: number | null;
  change_30d: number | null;
  direction: 'up' | 'down' | 'flat' | string;
  severity: 'low' | 'medium' | 'high' | string;
  narrative: string;
}

export interface MacroDashboardResponse {
  items: MacroSignal[];
  total: number;
  source: string;
}

export interface WhyMoveResponse {
  symbol: string;
  window: string;
  lookback_minutes?: number;
  move: (Partial<MarketVolatilitySpike> & { is_active_spike?: boolean }) | null;
  headline?: string;
  explanation?: string;
  summary?: string;
  confidence: { label: string; score: number } | string;
  drivers: Array<{ name: string; score: number; evidence: string[] }> | string[];
  causes: { news: Array<{ title: string; summary: string | null; source_name: string | null; url: string | null; score: number }>; calendar: unknown[]; macro?: MacroSignal[] };
  llm: { provider: string; model: string | null; status: string; narrative: { headline: string; explanation: string; drivers: string[]; confidence: string } | null };
  generated_at: string;
}

export interface ForexFeedSource {
  id: string;
  name: string;
  slug: string;
  url: string;
  rss_url: string | null;
  category: string;
  poll_interval_sec: number;
  priority: number;
  is_active: boolean;
  last_success_at: string | null;
  last_error_at: string | null;
  blocked_until: string | null;
  success_count: number;
  error_count: number;
  forbidden_count: number;
  parse_error_count: number;
  last_status: number | null;
  last_latency_ms: number | null;
}

export interface FeedSourcePayload {
  name: string;
  url: string;
  rss_url: string;
  category?: string;
  poll_interval_sec?: number;
  priority?: number;
  is_active?: boolean;
}

export interface FeedSourceTestResult {
  ok: boolean;
  entries?: number;
  latency_ms?: number;
  error?: string;
}
