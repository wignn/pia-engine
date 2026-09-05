import { env } from '$env/dynamic/public';

export const CORE_REST_URL = env.PUBLIC_CORE_REST_URL || 'http://localhost:8000';
export const CORE_WS_URL = env.PUBLIC_CORE_WS_URL || 'ws://localhost:8020';
export const API_KEY = env.PUBLIC_API_KEY || '';
export const DISCORD_INVITE = env.PUBLIC_DISCORD_INVITE || '';
