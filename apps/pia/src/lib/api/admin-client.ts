import type {
  AdminApiKey,
  AdminStats,
  AdminUser,
  FeedSourcePayload,
  FeedSourceStatusResponse,
  FeedSourceTestResult,
  ForexFeedSource,
  HealthStatus,
  MacroDashboardResponse,
  MarketDataQualityResponse,
  MarketPricesResponse,
  MarketVolatilitySpikesResponse,
  Plan,
  PlanId,
  WhyMoveResponse,
} from '../types/admin';

const CONTROL_PLANE_URL = process.env.NEXT_PUBLIC_CONTROL_PLANE_URL || '';
const CORE_REST_URL = process.env.NEXT_PUBLIC_CORE_REST_URL || '';

class AdminApiError extends Error {
  constructor(message: string, public status: number) {
    super(message);
    this.name = 'AdminApiError';
  }
}

async function fetchJson<T>(url: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers);
  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }
  const adminKey = process.env.ADMIN_API_KEY || '';
  if (adminKey) {
    headers.set('X-API-Key', adminKey);
  }

  const response = await fetch(url, { ...options, headers });
  if (!response.ok) {
    let errorMsg = `${response.status} ${response.statusText}`;
    try {
      const errBody = await response.json();
      errorMsg = errBody.error || errBody.message || errorMsg;
    } catch {
      // Ignore JSON parse failure
    }
    throw new AdminApiError(errorMsg, response.status);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export const adminApi = {
  getStats: () => fetchJson<AdminStats>(`${CONTROL_PLANE_URL}/api/v1/admin/stats`),
  getUsers: () => fetchJson<{ users: AdminUser[]; total: number }>(`${CONTROL_PLANE_URL}/api/v1/admin/users`),
  getUserKeys: (userId: string) => fetchJson<{ keys: AdminApiKey[]; total: number }>(`${CONTROL_PLANE_URL}/api/v1/admin/users/${encodeURIComponent(userId)}/keys`),
  updateUserPlan: (userId: string, plan: PlanId | string) => fetchJson<{ message: string }>(`${CONTROL_PLANE_URL}/api/v1/admin/users/${userId}/plan`, { method: 'POST', body: JSON.stringify({ plan }) }),
  toggleUserStatus: (userId: string) => fetchJson<{ message: string; is_active: boolean }>(`${CONTROL_PLANE_URL}/api/v1/admin/users/${userId}/toggle`, { method: 'POST' }),
  getPlans: () => fetchJson<{ plans: Plan[] }>(`${CONTROL_PLANE_URL}/api/v1/plans`),
  updatePlanWsLimit: (planId: string, ws_connections: number) => fetchJson<{ message: string }>(`${CONTROL_PLANE_URL}/api/v1/admin/plans/${encodeURIComponent(planId)}/ws-connections`, { method: 'POST', body: JSON.stringify({ ws_connections }) }),
  updateApiKeyLimit: (keyId: string, label: string, max_ws_connections: number | null) => fetchJson<{ message: string }>(`${CONTROL_PLANE_URL}/api/v1/keys/${encodeURIComponent(keyId)}`, { method: 'PATCH', body: JSON.stringify({ label, max_ws_connections }) }),

  // Core service endpoints
  getHealth: () => fetchJson<HealthStatus>(`${CORE_REST_URL}/health`),
  getFeedStatus: () => fetchJson<FeedSourceStatusResponse>(`${CORE_REST_URL}/api/v1/forex/sources/status`),
  getMarketPrices: () => fetchJson<MarketPricesResponse>(`${CORE_REST_URL}/api/v1/market/prices`),
  getMarketQuality: () => fetchJson<MarketDataQualityResponse>(`${CORE_REST_URL}/api/v1/market/data-quality`),
  getMarketSpikes: (window = '5m') => fetchJson<MarketVolatilitySpikesResponse>(`${CORE_REST_URL}/api/v1/market/spikes?window=${encodeURIComponent(window)}`),
  getWhyMove: (symbol: string, window = '5m') => fetchJson<WhyMoveResponse>(`${CORE_REST_URL}/api/v1/market/why/${encodeURIComponent(symbol)}?window=${encodeURIComponent(window)}`),
  getMacroDashboard: (limit = 50) => fetchJson<MacroDashboardResponse>(`${CORE_REST_URL}/api/v1/macro/dashboard?limit=${limit}`),
  getForexSources: () => fetchJson<{ items: ForexFeedSource[]; total: number }>(`${CORE_REST_URL}/api/v1/admin/forex/sources`),
  createForexSource: (payload: FeedSourcePayload) => fetchJson<{ id?: string; message?: string }>(`${CORE_REST_URL}/api/v1/admin/forex/sources`, { method: 'POST', body: JSON.stringify(payload) }),
  updateForexSource: (id: string, payload: FeedSourcePayload) => fetchJson<{ message?: string }>(`${CORE_REST_URL}/api/v1/admin/forex/sources/${id}`, { method: 'POST', body: JSON.stringify(payload) }),
  toggleForexSource: (id: string) => fetchJson<{ message?: string; is_active?: boolean }>(`${CORE_REST_URL}/api/v1/admin/forex/sources/${id}/toggle`, { method: 'POST' }),
  testForexSource: (payload: FeedSourcePayload) => fetchJson<FeedSourceTestResult>(`${CORE_REST_URL}/api/v1/admin/forex/sources/test`, { method: 'POST', body: JSON.stringify(payload) }),
};
