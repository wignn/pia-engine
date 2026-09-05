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

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    ...init,
    credentials: "include",
    headers: { "Content-Type": "application/json", ...init.headers },
  });
  if (!response.ok) throw new Error(`API request failed (${response.status})`);
  return response.json() as Promise<T>;
}

export const accountApi = {
  me: () => request<{ user: User; active_keys: number; plan_limits: Plan }>("/api/v1/auth/me"),
  keys: async () => (await request<{ keys: KeyInfo[] }>("/api/v1/keys")).keys,
  createKey: (label: string) =>
    request<{ api_key: string; key_info: KeyInfo }>("/api/v1/keys", {
      method: "POST",
      body: JSON.stringify({ label }),
    }),
  revokeKey: (id: string) => request<{ message: string }>(`/api/v1/keys/${id}`, { method: "DELETE" }),
  plans: async () => (await request<{ plans: Plan[] }>("/api/v1/plans")).plans,
  oauthUrl: (provider: "google" | "github") =>
    request<{ url?: string; error?: string }>(`/api/v1/auth/oauth/${provider}/url`),
  oauthCallback: (provider: string, code: string, state: string) =>
    request<{ user?: User; error?: string }>(`/api/v1/auth/oauth/${provider}/callback`, {
      method: "POST",
      body: JSON.stringify({ code, state }),
    }),
};

export type { KeyInfo, Plan, User };
