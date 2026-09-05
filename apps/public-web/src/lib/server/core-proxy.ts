import { env as privateEnv } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';
import { json, type RequestEvent } from '@sveltejs/kit';
import { CORE_REST_URL, CORE_WS_URL } from '$lib/config';

function apiKey() {
	return privateEnv.API_KEY || privateEnv.CORE_API_KEY || publicEnv.PUBLIC_API_KEY || '';
}

function joinUrl(base: string, path: string) {
	return `${base.replace(/\/$/, '')}/${path.replace(/^\//, '')}`;
}

function realtimeHttpUrl() {
	return CORE_WS_URL.replace(/^wss:/, 'https:').replace(/^ws:/, 'http:');
}

function responseHeaders(response: Response) {
	const headers = new Headers(response.headers);
	for (const name of [
		'content-encoding',
		'content-length',
		'transfer-encoding',
		'connection',
		'keep-alive',
		'proxy-authenticate',
		'proxy-authorization',
		'te',
		'trailer',
		'upgrade'
	]) {
		headers.delete(name);
	}
	return headers;
}

export async function proxyCore(event: RequestEvent, path: string) {
	const key = apiKey();
	if (!key) return json({ error: 'Server API key is not configured' }, { status: 500 });

	const url = new URL(joinUrl(CORE_REST_URL, path));
	url.search = event.url.search;

	const headers = new Headers(event.request.headers);
	headers.set('X-API-Key', key);
	headers.set('Accept-Encoding', 'identity');
	headers.delete('host');
	headers.delete('cookie');

	const response = await fetch(url, {
		method: event.request.method,
		headers,
		body:
			event.request.method === 'GET' || event.request.method === 'HEAD'
				? undefined
				: event.request.body,
		duplex: 'half'
	} as RequestInit & { duplex: 'half' });

	return new Response(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers: responseHeaders(response)
	});
}

export async function createRealtimeSession() {
	const key = apiKey();
	if (!key) return null;

	try {
		const response = await fetch(joinUrl(realtimeHttpUrl(), '/api/v1/ws/ticket'), {
			method: 'POST',
			headers: { 'X-API-Key': key, 'Accept-Encoding': 'identity' }
		});
		if (!response.ok) return null;
		return response.json();
	} catch {
		return null;
	}
}
