import { writable } from 'svelte/store';
import { apiFetch } from '$lib/api';
import type {
	FearGreedComponentsResponse,
	FearGreedHistoryResponse,
	FearGreedRecord,
	MacroState,
	RateSpreadObservation,
	RateObservation,
	YieldCurveData
} from '$lib/types';

const initialState: MacroState = {
	loading: false,
	yieldCurve: null,
	scrapedBondCurve: null,
	fearGreed: null,
	fearGreedComponents: null,
	fearGreedHistory: null,
	errors: { yieldCurve: null, scrapedBondCurve: null, fearGreed: null, history: null },
	lastFetchedAt: null
};

export const macroState = writable<MacroState>(initialState);

let pollTimer: ReturnType<typeof setInterval> | null = null;
let requestId = 0;

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null;
}

function finiteNumber(value: unknown): value is number {
	return typeof value === 'number' && Number.isFinite(value);
}

function normalizeRatePoint(value: unknown): RateObservation | null {
	if (!isRecord(value)) return null;
	if (
		typeof value.source !== 'string' ||
		typeof value.country !== 'string' ||
		typeof value.tenor !== 'string' ||
		typeof value.date !== 'string' ||
		!finiteNumber(value.value) ||
		typeof value.unit !== 'string' ||
		typeof value.raw_series_id !== 'string' ||
		typeof value.updated_at !== 'string'
	)
		return null;
	return value as unknown as RateObservation;
}

function normalizeSpread(value: unknown): RateSpreadObservation | null {
	if (!isRecord(value)) return null;
	if (
		typeof value.country !== 'string' ||
		typeof value.spread !== 'string' ||
		typeof value.date !== 'string' ||
		!finiteNumber(value.value) ||
		typeof value.updated_at !== 'string'
	)
		return null;
	return value as unknown as RateSpreadObservation;
}

function normalizeYieldCurve(value: unknown): YieldCurveData | null {
	if (!isRecord(value) || typeof value.country !== 'string' || typeof value.source !== 'string')
		return null;
	const points = Array.isArray(value.points)
		? value.points
				.map(normalizeRatePoint)
				.filter((point): point is RateObservation => point !== null)
		: [];
	const spreads = Array.isArray(value.spreads)
		? value.spreads
				.map(normalizeSpread)
				.filter((spread): spread is RateSpreadObservation => spread !== null)
		: [];
	return {
		country: value.country,
		source: value.source,
		date: typeof value.date === 'string' ? value.date : null,
		points,
		spreads,
		stale: value.stale === true,
		updated_at: typeof value.updated_at === 'string' ? value.updated_at : null
	};
}

function normalizeFearGreedRecord(value: unknown): FearGreedRecord | null {
	if (!isRecord(value)) return null;
	if (
		typeof value.id !== 'string' ||
		typeof value.scope !== 'string' ||
		typeof value.date !== 'string' ||
		!finiteNumber(value.score) ||
		typeof value.label !== 'string' ||
		!isRecord(value.components) ||
		!isRecord(value.source_status) ||
		typeof value.created_at !== 'string'
	)
		return null;
	return {
		id: value.id,
		scope: value.scope,
		date: value.date,
		score: Math.max(0, Math.min(100, value.score)),
		label: value.label,
		components: Object.fromEntries(
			Object.entries(value.components).filter(([, score]) => finiteNumber(score))
		) as Record<string, number>,
		source_status: Object.fromEntries(
			Object.entries(value.source_status).filter(([, status]) => typeof status === 'string')
		) as Record<string, string>,
		created_at: value.created_at
	};
}

function normalizeComponents(value: unknown): FearGreedComponentsResponse | null {
	if (
		!isRecord(value) ||
		typeof value.scope !== 'string' ||
		!finiteNumber(value.score) ||
		typeof value.label !== 'string'
	)
		return null;
	const components = Array.isArray(value.components)
		? value.components
				.map((item): FearGreedComponentsResponse['components'][number] | null => {
					if (!isRecord(item) || typeof item.name !== 'string' || !finiteNumber(item.base_weight))
						return null;
					return {
						name: item.name,
						score:
							item.score === null
								? null
								: finiteNumber(item.score)
									? Math.max(0, Math.min(100, item.score))
									: null,
						base_weight: item.base_weight,
						status: typeof item.status === 'string' ? item.status : 'missing',
						description: typeof item.description === 'string' ? item.description : ''
					};
				})
				.filter((item): item is FearGreedComponentsResponse['components'][number] => item !== null)
		: [];
	return {
		scope: value.scope,
		score: Math.max(0, Math.min(100, value.score)),
		label: value.label,
		updated_at: typeof value.updated_at === 'string' ? value.updated_at : new Date().toISOString(),
		components
	};
}

function normalizeHistory(value: unknown): FearGreedHistoryResponse | null {
	if (!isRecord(value) || !Array.isArray(value.data)) return null;
	const data = value.data
		.map(normalizeFearGreedRecord)
		.filter((record): record is FearGreedRecord => record !== null)
		.sort((a, b) => a.date.localeCompare(b.date));
	return {
		data,
		scope: typeof value.scope === 'string' ? value.scope : 'global',
		total: typeof value.total === 'number' ? value.total : data.length
	};
}

async function fetchJson(path: string): Promise<unknown> {
	const response = await apiFetch(path);
	const body = await response.json().catch(() => null);
	if (!response.ok || (isRecord(body) && typeof body.error === 'string')) {
		throw new Error(
			isRecord(body) && typeof body.error === 'string'
				? body.error
				: `Request failed (${response.status})`
		);
	}
	return body;
}

export async function fetchMacroData() {
	const currentRequest = ++requestId;
	let previousState: MacroState = initialState;
	macroState.update((state) => {
		previousState = state;
		return {
			...state,
			loading: true,
			errors: { yieldCurve: null, scrapedBondCurve: null, fearGreed: null, history: null }
		};
	});

	const [yieldResult, fearGreedResult, componentsResult, historyResult] = await Promise.allSettled([
		fetchJson('/api/v1/rates/yield-curve?country=US'),
		fetchJson('/api/v1/fear-greed?scope=global'),
		fetchJson('/api/v1/fear-greed/components?scope=global'),
		fetchJson('/api/v1/fear-greed/history?scope=global&limit=30')
	]);

	if (currentRequest !== requestId) return;

	const next: MacroState = {
		...previousState,
		loading: false,
		errors: { yieldCurve: null, scrapedBondCurve: null, fearGreed: null, history: null },
		lastFetchedAt: new Date().toISOString()
	};

	if (yieldResult.status === 'fulfilled') {
		next.yieldCurve = normalizeYieldCurve(yieldResult.value);
		if (!next.yieldCurve) next.errors.yieldCurve = 'Yield curve response is unavailable.';
	} else {
		next.errors.yieldCurve =
			yieldResult.reason instanceof Error
				? yieldResult.reason.message
				: 'Yield curve is unavailable.';
	}

	if (fearGreedResult.status === 'fulfilled') {
		next.fearGreed = normalizeFearGreedRecord(fearGreedResult.value);
		if (!next.fearGreed) next.errors.fearGreed = 'Fear & Greed response is unavailable.';
	} else {
		next.errors.fearGreed =
			fearGreedResult.reason instanceof Error
				? fearGreedResult.reason.message
				: 'Fear & Greed is unavailable.';
	}

	if (componentsResult.status === 'fulfilled') {
		next.fearGreedComponents = normalizeComponents(componentsResult.value);
		if (!next.fearGreedComponents && !next.errors.fearGreed)
			next.errors.fearGreed = 'Fear & Greed components are unavailable.';
	} else if (!next.errors.fearGreed) {
		next.errors.fearGreed =
			componentsResult.reason instanceof Error
				? componentsResult.reason.message
				: 'Fear & Greed components are unavailable.';
	}

	if (historyResult.status === 'fulfilled') {
		next.fearGreedHistory = normalizeHistory(historyResult.value);
		if (!next.fearGreedHistory)
			next.errors.history = 'Fear & Greed history response is unavailable.';
	} else {
		next.errors.history =
			historyResult.reason instanceof Error
				? historyResult.reason.message
				: 'Fear & Greed history is unavailable.';
	}

	macroState.set(next);
}

export function startMacroPolling(intervalMs = 300_000) {
	stopMacroPolling();
	void fetchMacroData();
	pollTimer = setInterval(() => void fetchMacroData(), intervalMs);
}

export function stopMacroPolling() {
	if (pollTimer) clearInterval(pollTimer);
	pollTimer = null;
}
