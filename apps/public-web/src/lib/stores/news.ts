import { writable } from 'svelte/store';
import { apiFetch } from '$lib/api';
import type { NewsItem } from '$lib/types';

export const forexNews = writable<NewsItem[]>([]);
export const stockNews = writable<NewsItem[]>([]);
export const newsLoading = writable(false);

let pollTimer: ReturnType<typeof setInterval> | null = null;

function extractNewsItems(data: unknown): NewsItem[] {
	if (Array.isArray(data)) return data as NewsItem[];
	if (!data || typeof data !== 'object') return [];

	const payload = data as { items?: unknown; data?: unknown; articles?: unknown; news?: unknown };
	if (Array.isArray(payload.items)) return payload.items as NewsItem[];
	if (Array.isArray(payload.articles)) return payload.articles as NewsItem[];
	if (Array.isArray(payload.news)) return payload.news as NewsItem[];
	return extractNewsItems(payload.data);
}

async function fetchForexNews() {
	try {
		const res = await apiFetch('/api/v1/forex/news/latest?limit=15');
		if (!res.ok) {
			console.warn(`[News] forex fetch failed: ${res.status} ${res.statusText}`);
			return;
		}
		const data = await res.json();
		if (data.error) {
			console.warn('[News] forex API error:', data.error);
			return;
		}
		forexNews.set(extractNewsItems(data));
	} catch (e) {
		console.warn('[News] forex fetch error:', e);
	}
}

async function fetchStockNews() {
	try {
		const res = await apiFetch('/api/v1/stock/news?limit=15');
		if (!res.ok) {
			console.warn(`[News] stock fetch failed: ${res.status} ${res.statusText}`);
			return;
		}
		const data = await res.json();
		if (data.error) {
			console.warn('[News] stock API error:', data.error);
			return;
		}
		stockNews.set(extractNewsItems(data));
	} catch (e) {
		console.warn('[News] stock fetch error:', e);
	}
}

export async function fetchAllNews() {
	newsLoading.set(true);
	await Promise.all([fetchForexNews(), fetchStockNews()]);
	newsLoading.set(false);
}

export function startNewsPolling(intervalMs = 60_000) {
	if (pollTimer) clearInterval(pollTimer);
	fetchAllNews();
	pollTimer = setInterval(fetchAllNews, intervalMs);
}

export function stopNewsPolling() {
	if (pollTimer) clearInterval(pollTimer);
	pollTimer = null;
}
