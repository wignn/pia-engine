import { writable } from 'svelte/store';
import { apiFetch } from '$lib/api';
import type { NewsItem } from '$lib/types';

export const forexNews = writable<NewsItem[]>([]);
export const stockNews = writable<NewsItem[]>([]);
export const newsLoading = writable(false);
export const newsError = writable<string | null>(null);

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
		const res = await fetch('/api/pia/news');
		if (!res.ok) {
			const body = await res.json().catch(() => null);
			newsError.set(body?.error || `News request failed (${res.status})`);
			return;
		}
		const data = await res.json();
		if (data.error) {
			newsError.set(data.error);
			return;
		}
		forexNews.set(extractNewsItems(data).slice(0, 15));
	} catch (e) {
		console.warn('[News] forex fetch error:', e);
		newsError.set(e instanceof Error ? e.message : 'News could not be loaded');
	}
}

async function fetchStockNews() {
	try {
		const res = await apiFetch('/api/v1/news?category=stock&limit=15');
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
	newsError.set(null);
	try {
		await Promise.all([fetchForexNews(), fetchStockNews()]);
	} finally {
		newsLoading.set(false);
	}
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
