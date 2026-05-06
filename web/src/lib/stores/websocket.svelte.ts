import { CORE_WS_URL } from '$lib/config';
import type { PriceData } from '$lib/types';

// Use Svelte 5 runes for fine-grained reactivity
// Using a plain object for priceMap is often more reliable in early Svelte 5 versions
let priceMap = $state<Record<string, PriceData>>({});
let isConnected = $state(false);

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 2000;

function connect() {
	if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;

	const url = `${CORE_WS_URL}/api/v1/ws/market?bot_id=web_client&api_key=olin`;
	console.log('[WS] Connecting to:', url);
	ws = new WebSocket(url);

	ws.onopen = () => {
		console.log('[WS] Connected to market feed');
		reconnectDelay = 2000;
		isConnected = true;
	};

	ws.onmessage = (evt) => {
		try {
			const msg = JSON.parse(evt.data);
			if (msg.event === 'market.trade' && msg.data?.tick) {
				const tick = msg.data.tick;
				const symbol = tick.symbol;
				const prev = priceMap[symbol];
				const prevPrice = prev?.price ?? tick.price;
				const direction: 'up' | 'down' | 'none' =
					tick.price > prevPrice ? 'up' : tick.price < prevPrice ? 'down' : (prev?.direction ?? 'none');

				priceMap[symbol] = {
					symbol,
					price: tick.price,
					bid: tick.bid ?? null,
					ask: tick.ask ?? null,
					volume: tick.volume ?? null,
					source: tick.source ?? '',
					asset_type: tick.asset_type ?? '',
					received_at: tick.received_at ?? null,
					direction,
					prev_price: prevPrice,
					updated_at: Date.now()
				};
			}
		} catch {
			// ignore non-JSON or irrelevant messages
		}
	};

	ws.onclose = (evt) => {
		console.log(`[WS] Disconnected (code=${evt.code}), reconnecting in ${reconnectDelay / 1000}s...`);
		isConnected = false;
		scheduleReconnect();
	};

	ws.onerror = (err) => {
		console.error('[WS] Error:', err);
		ws?.close();
	};
}

function scheduleReconnect() {
	if (reconnectTimer) clearTimeout(reconnectTimer);
	reconnectTimer = setTimeout(() => {
		reconnectDelay = Math.min(reconnectDelay * 1.5, 30000);
		connect();
	}, reconnectDelay);
}

export function startWebSocket() {
	connect();
}

export function stopWebSocket() {
	if (reconnectTimer) clearTimeout(reconnectTimer);
	reconnectTimer = null;
	ws?.close();
	ws = null;
}

export const marketStore = {
	get prices() {
		return Object.values(priceMap).sort((a, b) => a.symbol.localeCompare(b.symbol));
	},
	get connected() {
		return isConnected;
	},
	getPrice(symbol: string) {
		return priceMap[symbol.toUpperCase()];
	}
};

