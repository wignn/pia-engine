import { apiFetch } from './client';
import type { DailyUsage, UsageSummary } from '../types';

export function fetchUsageSummary() {
	return apiFetch<UsageSummary>('/api/v1/usage');
}

export function fetchUsageHistory(days = 30) {
	return apiFetch<{ history: DailyUsage[]; days: number }>(`/api/v1/usage/history?days=${days}`);
}
