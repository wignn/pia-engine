import { json } from '@sveltejs/kit';
import { createRealtimeSession } from '$lib/server/core-proxy';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async () => {
	const session = await createRealtimeSession();
	if (!session) return json({ error: 'Realtime session is unavailable' }, { status: 503 });
	return json(session);
};
