import { goto } from '$app/navigation';
import { CONTROL_PLANE_URL } from '../config';
import { session } from '../stores/session.svelte';
import type { ApiErrorBody } from '../types';

export class ApiError extends Error {
	constructor(
		message: string,
		public status: number
	) {
		super(message);
		this.name = 'ApiError';
	}
}

interface FetchOptions extends RequestInit {
	skipAuthRedirect?: boolean;
	timeout?: number;
}

function urlFor(path: string): string {
	return `${CONTROL_PLANE_URL}${path.startsWith('/') ? path : `/${path}`}`;
}

async function parseError(response: Response): Promise<string> {
	try {
		const body = (await response.json()) as ApiErrorBody;
		return body.error || body.message || `${response.status} ${response.statusText}`;
	} catch {
		return `${response.status} ${response.statusText}`;
	}
}

/**
 * Secure fetch wrapper for control-plane API.
 *
 * Security features:
 * - Sends credentials (HttpOnly cookies) automatically via `credentials: 'include'`
 * - Falls back to Bearer token from session store for explicit auth
 * - Auto-redirects to /portal/login on 401/403
 * - Request timeout (default 15s)
 * - Never stores JWT in localStorage (only memory via session store)
 */
export async function apiFetch<T>(path: string, options: FetchOptions = {}): Promise<T> {
	const { skipAuthRedirect, timeout = 15_000, ...fetchOpts } = options;
	const headers = new Headers(fetchOpts.headers);

	// Set auth header from session token if available
	const token = session.token;
	if (token && !headers.has('Authorization')) {
		headers.set('Authorization', `Bearer ${token}`);
	}

	if (fetchOpts.body && !headers.has('Content-Type')) {
		headers.set('Content-Type', 'application/json');
	}

	const controller = new AbortController();
	const timer = setTimeout(() => controller.abort(), timeout);

	let response: Response;
	try {
		response = await fetch(urlFor(path), {
			...fetchOpts,
			headers,
			credentials: 'include',
			signal: controller.signal
		});
	} catch (err) {
		if (err instanceof DOMException && err.name === 'AbortError') {
			throw new ApiError('Request timed out. Please try again.', 408);
		}
		throw new ApiError('Cannot reach the server. Please check your connection.', 0);
	} finally {
		clearTimeout(timer);
	}

	if (response.status === 401 || response.status === 403) {
		if (!skipAuthRedirect) {
			session.clear();
			void goto('/portal/login');
		}
		throw new ApiError('Session expired. Please log in again.', response.status);
	}

	if (!response.ok) {
		throw new ApiError(await parseError(response), response.status);
	}

	if (response.status === 204) return undefined as T;
	return (await response.json()) as T;
}
