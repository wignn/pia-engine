import { PiaClient } from '@piaa/sdk';
import { CORE_REST_URL } from '$lib/config';
import { coreApiKey } from './core-proxy';

export function createPiaClient() {
	const apiKey = coreApiKey();
	if (!apiKey) throw new Error('Server API key is not configured');

	return new PiaClient({
		apiKey,
		baseUrl: CORE_REST_URL,
		timeoutMs: 15_000,
		maxRetries: 2
	});
}
