const BASE_URL = process.env.NEXT_PUBLIC_CONTROL_PLANE_URL || "";

type User = {
  id: string;
  email: string;
  name: string;
  plan: string;
  avatar_url?: string | null;
};

type KeyInfo = {
  id: string;
  key_prefix: string;
  label: string;
  permissions?: string[];
  is_active: boolean;
  max_ws_connections?: number | null;
  last_used_at?: string | null;
  created_at: string;
};

type Plan = {
  id: string;
  name: string;
  price_idr: number;
  requests_per_day: number;
  ws_connections: number;
  news_history_days: number;
  rate_limit_per_min: number;
};

type UsageSummary = {
  today: number;
  this_week: number;
  this_month: number;
  daily_limit: number;
  remaining_today: number;
};

type DailyUsage = {
  day: string;
  count: number;
};

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    ...init,
    credentials: "include",
    headers: { "Content-Type": "application/json", ...init.headers },
  });
  if (!response.ok) {
    let errorMsg = `API request failed (${response.status})`;
    try {
      const err = await response.json();
      if (err.error || err.message) errorMsg = err.error || err.message;
    } catch {
      // ignore
    }
    throw new Error(errorMsg);
  }
  return response.json() as Promise<T>;
}

export const accountApi = {
  me: () => request<{ user: User; active_keys: number; plan_limits: Plan }>("/api/v1/auth/me"),
  login: (email: string, password: string) =>
    request<{ user: User; token: string }>("/api/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    }),
  register: (email: string, name: string, password: string) =>
    request<{ user: User; token: string; api_key: string; message: string }>("/api/v1/auth/register", {
      method: "POST",
      body: JSON.stringify({ email, name, password }),
    }),
  logout: () => request<{ message: string }>("/api/v1/auth/logout", { method: "POST" }),
  keys: async () => (await request<{ keys: KeyInfo[] }>("/api/v1/keys")).keys,
  createKey: (label: string, permissions: string[] = ["market:read", "realtime:ws"]) =>
    request<{ api_key: string; key_info: KeyInfo }>("/api/v1/keys", {
      method: "POST",
      body: JSON.stringify({ label, permissions }),
    }),
  updateKey: (id: string, label: string) =>
    request<{ key: KeyInfo; message: string }>(`/api/v1/keys/${id}`, {
      method: "PATCH",
      body: JSON.stringify({ label }),
    }),
  revokeKey: (id: string) => request<{ message: string }>(`/api/v1/keys/${id}`, { method: "DELETE" }),
  usage: () => request<UsageSummary>("/api/v1/usage"),
  usageHistory: async (days = 14) =>
    (await request<{ history: DailyUsage[]; days: number }>(`/api/v1/usage/history?days=${days}`)).history,
  plans: async () => (await request<{ plans: Plan[] }>("/api/v1/plans")).plans,
  upgradePlan: (planId: string) =>
    request<{ status: string; plan: string; message: string; limits?: Plan; error?: string }>("/api/v1/plans/upgrade", {
      method: "POST",
      body: JSON.stringify({ plan_id: planId }),
    }),
  oauthUrl: (provider: "google" | "github") =>
    request<{ url?: string; error?: string }>(`/api/v1/auth/oauth/${provider}/url`),
  oauthCallback: (provider: string, code: string, state: string) =>
    request<{ user?: User; error?: string }>(`/api/v1/auth/oauth/${provider}/callback`, {
      method: "POST",
      body: JSON.stringify({ code, state }),
    }),
};

export type { KeyInfo, Plan, User, UsageSummary, DailyUsage };
