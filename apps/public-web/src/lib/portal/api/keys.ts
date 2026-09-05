import { apiFetch } from './client';
import type { ApiKeyInfo } from '../types';

export function listKeys() {
	return apiFetch<{ keys: ApiKeyInfo[]; total: number }>('/api/v1/keys');
}

export function createKey(label?: string, permissions?: string[]) {
	return apiFetch<{ api_key: string; key_info: ApiKeyInfo; message: string }>('/api/v1/keys', {
		method: 'POST',
		body: JSON.stringify({ label, permissions })
	});
}

export function revokeKey(keyId: string) {
	return apiFetch<{ message: string }>(`/api/v1/keys/${keyId}`, {
		method: 'DELETE'
	});
}

export function updateKeyLabel(keyId: string, label: string) {
	return apiFetch<{ message: string }>(`/api/v1/keys/${keyId}`, {
		method: 'PATCH',
		body: JSON.stringify({ label })
	});
}
