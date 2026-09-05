import { apiFetch } from './client';
import type { Plan, TenantConfig } from '../types';

export function listConfig() {
	return apiFetch<{ configs: TenantConfig; plan_limits: Plan | null }>('/api/v1/config');
}

export function setConfig(key: string, value: unknown) {
	return apiFetch<{ config: { key: string; value: unknown; updated_at: string }; message: string }>(
		`/api/v1/config/${key}`,
		{
			method: 'PUT',
			body: JSON.stringify({ value })
		}
	);
}

export function deleteConfig(key: string) {
	return apiFetch<{ message: string }>(`/api/v1/config/${key}`, {
		method: 'DELETE'
	});
}
