export interface PlatformStats {
  total_users: number;
  active_users: number;
  total_api_keys: number;
  users_by_plan: Record<string, number>;
}

export interface AdminUser {
  id: string;
  email: string;
  name: string;
  plan: string;
  is_active: boolean;
  email_verified: boolean;
  created_at: string;
  active_keys: number;
}

export interface AdminApiKey {
  id: string;
  key_prefix: string;
  label: string;
  permissions: string[];
  is_active: boolean;
  max_ws_connections: number | null;
  last_used_at: string | null;
  created_at: string;
}

export interface UserUsage {
  user_id: string;
  plan: string;
  today: number;
  this_week: number;
  this_month: number;
  daily_limit: number;
  remaining_today: number;
}

export interface PlanItem {
  id: string;
  name: string;
  price_idr: number;
  requests_per_day: number;
  rate_limit_per_min: number;
  ws_connections: number;
  news_history_days: number;
}

export interface MarketPriceItem {
  symbol: string;
  price: number;
  asset_type: string;
  session?: {
    state: string;
    exchange: string;
    is_open: boolean;
  };
  received_at: string;
}

export interface MarketPricesResponse {
  items: MarketPriceItem[];
  total: number;
}

function getStoredAdminKey(): string {
  if (typeof window !== "undefined") {
    return localStorage.getItem("pia_admin_key") || "silvia";
  }
  return "silvia";
}

async function fetchAdmin<T>(path: string, options: RequestInit = {}): Promise<T> {
  const adminKey = getStoredAdminKey();
  const res = await fetch(path, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      "x-admin-key": adminKey,
      ...options.headers,
    },
  });

  if (!res.ok) {
    let msg = `Request failed: ${res.status}`;
    try {
      const err = await res.json();
      if (err.error || err.message) msg = err.error || err.message;
    } catch {
      // ignore
    }
    throw new Error(msg);
  }

  return res.json() as Promise<T>;
}

export const adminClient = {
  getStats: () => fetchAdmin<PlatformStats>("/api/admin/admin/stats"),
  
  getUsers: async () => {
    const res = await fetchAdmin<{ users: AdminUser[]; total: number }>("/api/admin/admin/users");
    return res.users || [];
  },

  getUserKeys: async (userId: string) => {
    const res = await fetchAdmin<{ keys: AdminApiKey[]; total: number }>(`/api/admin/admin/users/${userId}/keys`);
    return res.keys || [];
  },

  setUserPlan: (userId: string, plan: string) =>
    fetchAdmin<{ message: string; plan: string }>(`/api/admin/admin/users/${userId}/plan`, {
      method: "POST",
      body: JSON.stringify({ plan }),
    }),

  toggleUser: (userId: string) =>
    fetchAdmin<{ message: string; is_active: boolean }>(`/api/admin/admin/users/${userId}/toggle`, {
      method: "POST",
    }),

  getUserUsage: (userId: string) =>
    fetchAdmin<UserUsage>(`/api/admin/admin/users/${userId}/usage`),

  resetUserQuota: (userId: string) =>
    fetchAdmin<{ message: string; user_id: string; today: number }>(`/api/admin/admin/users/${userId}/quota/reset`, {
      method: "POST",
    }),

  flushCache: () =>
    fetchAdmin<{ message: string; timestamp: string }>("/api/admin/admin/system/flush-cache", {
      method: "POST",
    }),

  getPlans: async () => {
    const res = await fetchAdmin<{ plans: PlanItem[] }>("/api/admin/plans");
    return res.plans || [];
  },

  getMarketPrices: async () => {
    const res = await fetch("/api/market/prices");
    if (!res.ok) throw new Error("Failed to load market prices");
    return (await res.json()) as MarketPricesResponse;
  },
};
