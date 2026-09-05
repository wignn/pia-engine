import { goto } from '$app/navigation';
import type { MeResponse, Plan, User } from '../types';

let user = $state<User | null>(null);
let token = $state<string | null>(null);
let activeKeys = $state(0);
let planLimits = $state<Plan | null>(null);
let linkedProviders = $state<string[]>([]);
let ready = $state(false);
let loading = $state(false);

/**
 * Reactive session store.
 *
 * Token is stored only in memory (never localStorage/sessionStorage).
 * The HttpOnly `wi_jwt` cookie is the primary auth mechanism.
 * The in-memory token is a fallback for explicit Bearer auth.
 */
export const session = {
	get user() {
		return user;
	},
	get token() {
		return token;
	},
	get activeKeys() {
		return activeKeys;
	},
	get planLimits() {
		return planLimits;
	},
	get linkedProviders() {
		return linkedProviders;
	},
	get ready() {
		return ready;
	},
	get loading() {
		return loading;
	},
	get isAuthenticated() {
		return Boolean(user);
	},

	setFromLogin(data: { user: User; token: string }) {
		user = data.user;
		token = data.token;
		ready = true;
	},

	setFromMe(data: MeResponse) {
		user = data.user;
		activeKeys = data.active_keys;
		planLimits = data.plan_limits;
		linkedProviders = data.linked_providers;
		ready = true;
	},

	setLoading(value: boolean) {
		loading = value;
	},

	setReady(value: boolean) {
		ready = value;
	},

	clear() {
		user = null;
		token = null;
		activeKeys = 0;
		planLimits = null;
		linkedProviders = [];
		ready = true;
		loading = false;
	},

	logout() {
		this.clear();
		// Clear the cookie by setting it expired (client-side best effort)
		if (typeof document !== 'undefined') {
			document.cookie = 'wi_jwt=; Path=/; Max-Age=0; SameSite=Lax';
		}
		void goto('/portal/login');
	}
};
