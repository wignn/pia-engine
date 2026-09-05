import { CORE_WS_URL } from '$lib/config';
import { apiFetch } from '$lib/api';
import type { PriceData, NewsItem } from '$lib/types';

const priceMap = $state<Record<string, PriceData>>({});
let isConnected = $state(false);

let realtimeForexNews = $state<NewsItem[]>([]);
let realtimeStockNews = $state<NewsItem[]>([]);
const MAX_REALTIME_NEWS = 30;

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 1000;
let requestId = 1;
let isOpening = false;
const desiredStreams = new Set(['market_data', 'forex_news', 'stock_news']);
const hiddenMarketSymbols = new Set<string>();

function isHiddenMarketSymbol(symbol: string): boolean {
	return hiddenMarketSymbols.has(symbol.toUpperCase());
}

async function createRealtimeTicket(): Promise<string> {
	const res = await fetch('/api/realtime/session', {
		method: 'POST',
		cache: 'no-store',
		credentials: 'same-origin'
	});
	if (!res.ok) throw new Error(`Realtime session failed: ${res.status}`);
	const data = await res.json();
	if (!data?.ticket) throw new Error('Realtime session response did not include a ticket');
	return data.ticket;
}

async function connect() {
	if (
		isOpening ||
		(ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING))
	)
		return;
	isOpening = true;

	let url: string;
	try {
		const ticket = await createRealtimeTicket();
		url = `${CORE_WS_URL}/ws/v1?bot_id=web_client&ticket=${encodeURIComponent(ticket)}`;
	} catch (e) {
		console.warn('[WS] Failed to create realtime session:', e);
		isConnected = false;
		isOpening = false;
		scheduleReconnect();
		return;
	}

	ws = new WebSocket(url);

	ws.onopen = () => {
		console.log('[WS] Connected to core (market + news)');
		reconnectDelay = 1000;
		isOpening = false;
		isConnected = true;
		sendCommand('SUBSCRIBE', [...desiredStreams]);
	};

	ws.onmessage = (evt) => {
		try {
			const msg = JSON.parse(evt.data);

			if (msg.event === 'market.trade' && msg.data?.tick) {
				handleMarketTrade(msg.data.tick);
			} else if (msg.event === 'forex_news.new' || msg.event === 'forex_news.high_impact') {
				handleForexNews(msg.data);
			} else if (
				msg.event === 'stock_news.new' ||
				msg.event === 'stock.news.new' ||
				msg.event === 'stock.news.high_impact'
			) {
				handleStockNews(msg.data);
			}
		} catch {
			// Ignore non-JSON frames
		}
	};

	ws.onclose = (evt) => {
		console.log(
			`[WS] Disconnected (code=${evt.code}), reconnecting in ${reconnectDelay / 1000}s...`
		);
		isOpening = false;
		isConnected = false;
		scheduleReconnect();
	};

	ws.onerror = (err) => {
		console.error('[WS] Error:', err);
		ws?.close();
	};
}

function handleMarketTrade(tick: any) {
	const symbol = String(tick.symbol ?? '').toUpperCase();
	if (!symbol || isHiddenMarketSymbol(symbol)) return;

	const prev = priceMap[symbol];
	const receivedAt = tick.received_at ?? null;
	const receivedAtMs = receivedAt ? Date.parse(receivedAt) : 0;
	const prevReceivedAtMs = prev?.received_at ? Date.parse(prev.received_at) : 0;
	if (receivedAtMs && prevReceivedAtMs && receivedAtMs < prevReceivedAtMs) return;

	const now = Date.now();
	if (prev && tick.price === prev.price && now - prev.updated_at < 1_000) return;

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
		received_at: receivedAt,
		session: tick.session,
		direction,
		prev_price: prevPrice,
		updated_at: now
	};
}

function handleForexNews(data: any) {
	const article = data?.article;
	if (!article) return;

	const item: NewsItem = {
		id: article.id ?? article.content_hash ?? crypto.randomUUID(),
		title: article.title ?? article.original_title ?? '',
		original_title: article.title ?? article.original_title,
		translated_title: article.title_id ?? article.translated_title,
		summary: article.summary ?? article.summary_id ?? null,
		source_name: article.source_name ?? '',
		original_url: article.url ?? article.original_url,
		url: article.url ?? article.original_url,
		sentiment: article.sentiment ?? null,
		impact_level: article.impact_level ?? null,
		published_at: article.published_at ?? null,
		processed_at: article.processed_at ?? null,
		currency_pairs: article.currency_pairs?.join?.(', ') ?? article.currency_pairs ?? null
	};

	if (realtimeForexNews.some((n) => n.id === item.id)) return;
	realtimeForexNews = [item, ...realtimeForexNews].slice(0, MAX_REALTIME_NEWS);
}

function handleStockNews(data: any) {
	const article = data?.article;
	if (!article) return;

	const item: NewsItem = {
		id: article.id ?? article.content_hash ?? crypto.randomUUID(),
		title: article.title ?? '',
		original_title: article.title,
		summary: article.summary ?? null,
		source_name: article.source_name ?? '',
		original_url: article.url ?? article.source_url,
		url: article.url ?? article.source_url,
		sentiment: article.sentiment ?? null,
		impact_level: article.impact_level ?? null,
		published_at: article.published_at ?? null,
		processed_at: article.processed_at ?? null,
		tickers: article.tickers?.join?.(', ') ?? article.tickers ?? null,
		category: article.category ?? undefined
	};

	if (realtimeStockNews.some((n) => n.id === item.id)) return;
	realtimeStockNews = [item, ...realtimeStockNews].slice(0, MAX_REALTIME_NEWS);
}

function sendCommand(method: string, params: string[] = []) {
	if (!ws || ws.readyState !== WebSocket.OPEN) return;
	ws.send(JSON.stringify({ method, params, id: requestId++ }));
}

function scheduleReconnect() {
	if (reconnectTimer) clearTimeout(reconnectTimer);
	const delay = reconnectDelay + Math.floor(Math.random() * 250);
	reconnectTimer = setTimeout(() => {
		reconnectTimer = null;
		reconnectDelay = Math.min(reconnectDelay * 1.5, 30000);
		void connect();
	}, delay);
}

async function fetchInitialPrices() {
	try {
		const res = await apiFetch('/api/v1/market/prices');
		if (!res.ok) return;

		const data = await res.json();
		if (!data.items) return;

		for (const item of data.items) {
			const symbol = String(item.symbol ?? '').toUpperCase();
			if (!symbol || isHiddenMarketSymbol(symbol)) continue;
			if (!priceMap[symbol]) {
				priceMap[symbol] = {
					symbol,
					price: item.price,
					bid: item.bid ?? null,
					ask: item.ask ?? null,
					volume: item.volume ?? null,
					source: item.source ?? '',
					asset_type: item.asset_type ?? '',
					received_at: item.received_at ?? null,
					session: item.session,
					direction: 'none',
					prev_price: item.price,
					updated_at: Date.now()
				};
			}
		}
	} catch (e) {
		console.warn('[WS] Failed to fetch initial prices:', e);
	}
}

export function startWebSocket() {
	fetchInitialPrices();
	void connect();
}

export function subscribe(streams: string[]) {
	for (const stream of streams) desiredStreams.add(stream);
	sendCommand('SUBSCRIBE', streams);
}

export function unsubscribe(streams: string[]) {
	for (const stream of streams) desiredStreams.delete(stream);
	sendCommand('UNSUBSCRIBE', streams);
}

export function listSubscriptions() {
	sendCommand('LIST_SUBSCRIPTIONS');
}

export function stopWebSocket() {
	if (reconnectTimer) clearTimeout(reconnectTimer);
	reconnectTimer = null;
	isOpening = false;
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

export const realtimeNewsStore = {
	get forexNews() {
		return realtimeForexNews;
	},
	get stockNews() {
		return realtimeStockNews;
	},
	mergeForex(restItems: NewsItem[]): NewsItem[] {
		return dedupeAndMerge(realtimeForexNews, restItems);
	},
	mergeStock(restItems: NewsItem[]): NewsItem[] {
		return dedupeAndMerge(realtimeStockNews, restItems);
	}
};

function dedupeAndMerge(realtime: NewsItem[], rest: NewsItem[]): NewsItem[] {
	const seen = new Set<string>();
	const merged: NewsItem[] = [];

	for (const item of [...realtime, ...rest]) {
		if (!seen.has(item.id)) {
			seen.add(item.id);
			merged.push(item);
		}
	}

	return merged.slice(0, 30);
}
