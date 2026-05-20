import { CORE_WS_URL, API_KEY, CORE_REST_URL } from '$lib/config';
import type { PriceData, NewsItem } from '$lib/types';

let priceMap = $state<Record<string, PriceData>>({});
let isConnected = $state(false);

// Newest-first realtime news buffers are capped to bound memory usage.
let realtimeForexNews = $state<NewsItem[]>([]);
let realtimeEquityNews = $state<NewsItem[]>([]);
const MAX_REALTIME_NEWS = 30;

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 2000;

function connect() {
	if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;

	const url = `${CORE_WS_URL}/api/v1/ws?bot_id=web_client&api_key=${API_KEY}&channels=market_data,news,equity_news`;
	ws = new WebSocket(url);

	ws.onopen = () => {
		console.log('[WS] Connected to core (market + news)');
		reconnectDelay = 2000;
		isConnected = true;
	};

	ws.onmessage = (evt) => {
		try {
			const msg = JSON.parse(evt.data);

			if (msg.event === 'market.trade' && msg.data?.tick) {
				handleMarketTrade(msg.data.tick);
			} else if (msg.event === 'news.new' || msg.event === 'news.high_impact') {
				handleForexNews(msg.data);
			} else if (
				msg.event === 'equity.news.new' ||
				msg.event === 'stock.news.new' ||
				msg.event === 'stock.news.high_impact'
			) {
				handleEquityNews(msg.data);
			}
		} catch {
			// Ignore non-JSON frames and channel messages outside this client contract.
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

function handleMarketTrade(tick: any) {
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

function handleEquityNews(data: any) {
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

	if (realtimeEquityNews.some((n) => n.id === item.id)) return;

	realtimeEquityNews = [item, ...realtimeEquityNews].slice(0, MAX_REALTIME_NEWS);
}

function scheduleReconnect() {
	if (reconnectTimer) clearTimeout(reconnectTimer);
	reconnectTimer = setTimeout(() => {
		reconnectDelay = Math.min(reconnectDelay * 1.5, 30000);
		connect();
	}, reconnectDelay);
}

async function fetchInitialPrices() {
	try {
		const restBaseUrl = CORE_REST_URL.replace(/^ws/, 'http');
		const res = await fetch(`${restBaseUrl}/api/v1/market/prices`);
		if (res.ok) {
			const data = await res.json();
			if (data.items) {
				for (const item of data.items) {
					if (!priceMap[item.symbol]) {
						priceMap[item.symbol] = {
							symbol: item.symbol,
							price: item.price,
							bid: item.bid ?? null,
							ask: item.ask ?? null,
							volume: item.volume ?? null,
							source: item.source ?? '',
							asset_type: item.asset_type ?? '',
							received_at: item.received_at ?? null,
							direction: 'none',
							prev_price: item.price,
							updated_at: Date.now()
						};
					}
				}
			}
		}
	} catch (e) {
		console.warn('[WS] Failed to fetch initial prices:', e);
	}
}

export function startWebSocket() {
	fetchInitialPrices();
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

export const realtimeNewsStore = {
	get forexNews() {
		return realtimeForexNews;
	},
	get equityNews() {
		return realtimeEquityNews;
	},
	/** Merge REST-fetched and realtime items, keeping newest realtime records first. */
	mergeForex(restItems: NewsItem[]): NewsItem[] {
		return dedupeAndMerge(realtimeForexNews, restItems);
	},
	mergeEquity(restItems: NewsItem[]): NewsItem[] {
		return dedupeAndMerge(realtimeEquityNews, restItems);
	}
};

function dedupeAndMerge(realtime: NewsItem[], rest: NewsItem[]): NewsItem[] {
	const seen = new Set<string>();
	const merged: NewsItem[] = [];

	for (const item of realtime) {
		if (!seen.has(item.id)) {
			seen.add(item.id);
			merged.push(item);
		}
	}
	for (const item of rest) {
		if (!seen.has(item.id)) {
			seen.add(item.id);
			merged.push(item);
		}
	}

	return merged.slice(0, 30);
}
