import type { Handle } from '@sveltejs/kit';

const csp = [
	"default-src 'self'",
	"script-src 'self' 'unsafe-inline'",
	"style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
	"font-src 'self' https://fonts.gstatic.com",
	"img-src 'self' data: https:",
	"connect-src 'self' http://localhost:8081 http://localhost:8000 https://slv-gateway.wign.dev https://api-engine.wign.dev ws://localhost:8090 ws://localhost:8020 https: wss://slv-realtime.wign.dev wss://api-atlsd.wign.cloud wss://realtime-engine.wign.dev",
	"frame-ancestors 'none'",
	"base-uri 'self'",
	"form-action 'self'"
].join('; ');

export const handle: Handle = async ({ event, resolve }) => {
	const response = await resolve(event);
	response.headers.set('Content-Security-Policy', csp);
	response.headers.set('X-Content-Type-Options', 'nosniff');
	response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
	response.headers.set('Permissions-Policy', 'camera=(), microphone=(), geolocation=()');
	return response;
};
