import { writable } from 'svelte/store';
import { CORE_REST_URL } from '$lib/config';
import type { CalendarEvent } from '$lib/types';

export const calendarEvents = writable<CalendarEvent[]>([]);
export const calendarLoading = writable(false);

let pollTimer: ReturnType<typeof setInterval> | null = null;

async function fetchCalendar() {
	calendarLoading.set(true);
	try {
		const res = await fetch(`${CORE_REST_URL}/api/v1/forex/calendar?impact=high&limit=15`);
		if (!res.ok) return;
		const data = await res.json();
		if (data.items) calendarEvents.set(data.items);
	} catch {
		// silent
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
