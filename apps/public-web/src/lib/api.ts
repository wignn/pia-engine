import { API_KEY } from '$lib/config';

export async function apiFetch(path: string, init?: RequestInit): Promise<Response> {
	const url = path.startsWith('http') ? path : `/api/core${path}`;
	const headers = new Headers(init?.headers);
	headers.set('X-API-Key', API_KEY || '');
	if (init?.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');

	return fetch(url, {
		...init,
		headers
	});
}
