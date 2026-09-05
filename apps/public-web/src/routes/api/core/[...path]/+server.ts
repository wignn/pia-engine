import { proxyCore } from '$lib/server/core-proxy';
import type { RequestHandler } from './$types';

const forward: RequestHandler = (event) => proxyCore(event, event.params.path);

export const GET = forward;
export const POST = forward;
export const PUT = forward;
export const PATCH = forward;
export const DELETE = forward;
