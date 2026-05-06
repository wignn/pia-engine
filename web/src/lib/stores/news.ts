import { writable } from 'svelte/store';
import { CORE_REST_URL } from '$lib/config';
import type { NewsItem } from '$lib/types';

export const forexNews = writable<NewsItem[]>([]);
export const equityNews = writable<NewsItem[]>([]);
export const newsLoading = writable(false);

let pollTimer: ReturnType<typeof setInterval> | null = null;

async function fetchForexNews() {
	try {
		const res = await fetch(`${CORE_REST_URL}/api/v1/forex/news/latest?limit=15`);
		if (!res.ok) return;
		const data = await res.json();
		if (data.items) forexNews.set(data.items);
	} catch {
		// silent
	}
}

async function fetchEquityNews() {
	try {
		const res = await fetch(`${CORE_REST_URL}/api/v1/equity/news?limit=15`);
		if (!res.ok) return;
		const data = await res.json();
		if (data.items) equityNews.set(data.items);
	} catch {
		// silent
	}
}

export async function fetchAllNews() {
	newsLoading.set(true);
	await Promise.all([fetchForexNews(), fetchEquityNews()]);
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
