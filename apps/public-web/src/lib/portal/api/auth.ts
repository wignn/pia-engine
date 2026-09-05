import { apiFetch } from './client';
import type { LoginResponse, MeResponse, RegisterResponse } from '../types';

export function register(email: string, name: string, password: string) {
	return apiFetch<RegisterResponse>('/api/v1/auth/register', {
		method: 'POST',
		body: JSON.stringify({ email, name, password }),
		skipAuthRedirect: true
	});
}

export function login(email: string, password: string) {
	return apiFetch<LoginResponse>('/api/v1/auth/login', {
		method: 'POST',
		body: JSON.stringify({ email, password }),
		skipAuthRedirect: true
	});
}

export function verifyEmail(token: string) {
	return apiFetch<{ message?: string; error?: string }>('/api/v1/auth/verify', {
		method: 'POST',
		body: JSON.stringify({ token }),
		skipAuthRedirect: true
	});
}

export function fetchMe() {
	return apiFetch<MeResponse>('/api/v1/auth/me');
}

export function getOAuthUrl(provider: string) {
	return apiFetch<{ url: string; state: string; error?: string }>(
		`/api/v1/auth/oauth/${provider}/url`,
		{ skipAuthRedirect: true }
	);
}

export function oauthCallback(provider: string, code: string, state: string) {
	return apiFetch<LoginResponse>(`/api/v1/auth/oauth/${provider}/callback`, {
		method: 'POST',
		body: JSON.stringify({ code, state }),
		skipAuthRedirect: true
	});
}
