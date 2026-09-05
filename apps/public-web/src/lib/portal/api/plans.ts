import { apiFetch } from './client';
import type { Plan } from '../types';

export function listPlans() {
	return apiFetch<{ plans: Plan[] }>('/api/v1/plans', { skipAuthRedirect: true });
}

export function requestUpgrade() {
	return apiFetch<{ status: string; message: string }>('/api/v1/plans/upgrade', {
		method: 'POST'
	});
}
