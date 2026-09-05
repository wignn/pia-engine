export type PlanId = 'free' | 'starter' | 'pro' | 'enterprise';

export interface User {
	id: string;
	email: string;
	name: string;
	plan: PlanId | string;
	email_verified: boolean;
	avatar_url?: string | null;
	created_at: string;
	has_password?: boolean;
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

export interface ApiKeyInfo {
	id: string;
	key_prefix: string;
	label: string;
	permissions: string[];
	is_active: boolean;
	last_used_at: string | null;
	expires_at: string | null;
	created_at: string;
}

export interface UsageSummary {
	today: number;
	this_week: number;
	this_month: number;
	daily_limit: number;
	remaining_today: number;
}

export interface DailyUsage {
	day: string;
	count: number;
}

export interface TenantConfig {
	[key: string]: unknown;
}

export interface MeResponse {
	user: User;
	active_keys: number;
	plan_limits: Plan | null;
	linked_providers: string[];
}

export interface LoginResponse {
	user: User;
	token: string;
	api_key?: string;
	message?: string;
}

export interface RegisterResponse extends LoginResponse {
	api_key: string;
}

export interface ApiErrorBody {
	error?: string;
	message?: string;
}

export interface HealthStatus {
	status: 'healthy' | 'unhealthy';
	service: string;
	latency_ms?: number;
	error?: string;
}
