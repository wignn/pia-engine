import { writable } from 'svelte/store';
import type { CalendarEvent } from '$lib/types';

export const calendarEvents = writable<CalendarEvent[]>([]);
export const calendarLoading = writable(false);
export const calendarError = writable<string | null>(null);

let pollTimer: ReturnType<typeof setInterval> | null = null;

async function fetchCalendar() {
	calendarLoading.set(true);
	calendarError.set(null);
	try {
		const res = await fetch('/api/pia/economic/calendar');
		if (!res.ok) {
			const body = await res.json().catch(() => null);
			throw new Error(body?.error || `Calendar request failed (${res.status})`);
		}
		const data = await res.json();
		if (data.error) throw new Error(data.error);
		const items = Array.isArray(data?.items) ? data.items : Array.isArray(data?.events) ? data.events : [];
		calendarEvents.set(
			items
				.filter((event: CalendarEvent) => /high|red/i.test(event.impact || ''))
				.slice(0, 15)
		);
	} catch (error) {
		calendarError.set(error instanceof Error ? error.message : 'Calendar could not be loaded');
	} finally {
		calendarLoading.set(false);
	}
}

export function startCalendarPolling(intervalMs = 300_000) {
	if (pollTimer) clearInterval(pollTimer);
	fetchCalendar();
	pollTimer = setInterval(fetchCalendar, intervalMs);
}

export function stopCalendarPolling() {
	if (pollTimer) clearInterval(pollTimer);
	pollTimer = null;
}
