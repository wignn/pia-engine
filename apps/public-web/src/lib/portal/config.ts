import { env } from '$env/dynamic/public';

export const CONTROL_PLANE_URL = env.PUBLIC_CONTROL_PLANE_URL || 'http://localhost:8081';
export const CORE_REST_URL = env.PUBLIC_CORE_REST_URL || 'http://localhost:8000';
export const CORE_WS_URL = env.PUBLIC_CORE_WS_URL || 'ws://localhost:8020';
