<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { createChart, AreaSeries, type IChartApi, type ISeriesApi } from 'lightweight-charts';
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';
	import { apiFetch } from '$lib/api';
	import { getChartTheme, isDarkMode } from '$lib/chart-theme';
	import { getSymbolMeta } from '$lib/symbol-meta';

	interface Props {
		symbol: string;
		height?: number;
		compact?: boolean;
	}
	let { symbol, height = 380, compact = false }: Props = $props();

	let chartContainer = $state<HTMLDivElement | null>(null);
	let chart: IChartApi | null = null;
	let areaSeries: ISeriesApi<'Area'> | null = null;

	type ChartResolution = '1m' | '5m' | '15m' | '1h';
	type ChartPoint = { time: number; value: number; source?: string };
	const chartResolutions: ChartResolution[] = ['1m', '5m', '15m', '1h'];

	let selectedResolution = $state<ChartResolution>('1m');
	let historyData = $state<ChartPoint[]>([]);
	let historySource = $state<'history' | 'last_known' | 'empty'>('empty');
	let loading = $state(true);
	let errorMsg = $state('');
	let liveData = $derived(marketStore.getPrice(symbol));
	let currentPrice = $derived(liveData?.price ?? 0);
	let firstPrice = $derived(historyData.length > 0 ? historyData[0].value : currentPrice);
	let priceChange = $derived(currentPrice - firstPrice);
	let percentChange = $derived(firstPrice > 0 ? (priceChange / firstPrice) * 100 : 0);
	let direction = $derived(priceChange > 0 ? 'up' : priceChange < 0 ? 'down' : 'none');
	let lastChartTime = $derived(
		historyData.length > 0 ? historyData[historyData.length - 1].time : 0
	);

	function resolutionSeconds(resolution: ChartResolution) {
		if (resolution === '5m') return 5 * 60;
		if (resolution === '15m') return 15 * 60;
		if (resolution === '1h') return 60 * 60;
		return 60;
	}

	let meta = $derived(getSymbolMeta(symbol));
	type FreshnessState = 'live' | 'stale' | 'closed' | 'unknown';

	function isMarketClosed(sym: string, assetType = ''): boolean {
		const now = new Date();
		const day = now.getUTCDay();
		const hour = now.getUTCHours();
		const upper = sym.toUpperCase();
		const type = assetType.toLowerCase();

		if (type === 'crypto' || upper.endsWith('USDT')) return false;
		if (upper === 'XAUUSD')
			return day === 6 || (day === 5 && hour >= 22) || (day === 0 && hour < 23);
		if (type === 'forex' || /^[A-Z]{6}$/.test(upper))
			return day === 6 || (day === 5 && hour >= 22) || (day === 0 && hour < 22);
		return day === 0 || day === 6;
	}

	function priceTimestamp(p: PriceData | undefined): number {
		if (!p) return 0;
		if (p.received_at) {
			const parsed = Date.parse(p.received_at);
			if (!Number.isNaN(parsed)) return parsed;
		}
		return p.updated_at;
	}

	function getFreshness(p: PriceData | undefined): {
		state: FreshnessState;
		label: string;
		className: string;
	} {
		if (!p || p.price <= 0)
			return {
				state: 'unknown',
				label: 'NO DATA',
				className: 'bg-surface-2 text-text-dim border-border'
			};
		const ts = priceTimestamp(p);
		if (!ts)
			return {
				state: 'unknown',
				label: 'NO DATA',
				className: 'bg-surface-2 text-text-dim border-border'
			};

		const ageMs = Date.now() - ts;
		const isCrypto = p.symbol.toUpperCase().endsWith('USDT') || p.asset_type === 'crypto';
		const freshMs = isCrypto ? 15 * 60_000 : 5 * 60_000;
		if (ageMs <= freshMs)
			return { state: 'live', label: 'LIVE', className: 'bg-green/10 text-green border-green/20' };
		if (isMarketClosed(p.symbol, p.asset_type ?? ''))
			return {
				state: 'closed',
				label: 'CLOSED',
				className: 'bg-surface-2 text-text-dim border-border'
			};
		return {
			state: 'stale',
			label: isCrypto ? 'FEED LAG' : 'STALE',
			className: 'bg-yellow-500/10 text-yellow-600 border-yellow-500/20'
		};
	}

	let freshness = $derived(getFreshness(liveData));

	function sanitizeChartData(data: unknown): ChartPoint[] {
		if (!Array.isArray(data)) return [];

		const points = new Map<number, ChartPoint>();
		for (const point of data) {
			if (!point || typeof point !== 'object') continue;
			const row = point as { time?: unknown; value?: unknown; source?: unknown };
			const time = Number(row.time);
			const value = Number(row.value);
			if (!Number.isFinite(time) || !Number.isFinite(value) || time <= 0 || value <= 0) continue;
			points.set(Math.floor(time), {
				time: Math.floor(time),
				value,
				source: typeof row.source === 'string' ? row.source : undefined
			});
		}

		return [...points.values()].sort((a, b) => a.time - b.time);
	}

	async function loadHistoricalData(
		sym: string,
		resolution: ChartResolution
	): Promise<ChartPoint[]> {
		const upperSym = sym.toUpperCase();
		try {
			const params = new URLSearchParams({ resolution });
			const res = await apiFetch(`/api/v1/market/history/${upperSym}?${params}`);
			if (res.ok) {
				return sanitizeChartData(await res.json());
			}
			console.warn(
				`[PriceChart] History fetch failed for ${upperSym}: ${res.status} ${res.statusText}`
			);
		} catch (e) {
			console.warn(`[PriceChart] Backend history fetch failed for ${upperSym}`, e);
		}
		return [];
	}

	function updateChartColors() {
		if (!areaSeries || !chart) return;

		const theme = getChartTheme(isDarkMode());

		chart.applyOptions({
			layout: { background: { color: theme.background }, textColor: theme.textColor },
			grid: { vertLines: { color: theme.gridColor }, horzLines: { color: theme.gridColor } }
		});

		const colorLine = direction === 'up' ? theme.up : direction === 'down' ? theme.down : theme.neutral;
		const colorTop = theme.areaFillTop(colorLine);
		const colorBottom = theme.areaFillBottom(colorLine);

		areaSeries.applyOptions({ lineColor: colorLine, topColor: colorTop, bottomColor: colorBottom });
	}

	function initChart() {
		if (!chartContainer) return;

		const theme = getChartTheme(isDarkMode());

		chart = createChart(chartContainer, {
			width: chartContainer.clientWidth,
			height,
			layout: {
				background: { color: theme.background },
				textColor: theme.textColor,
				fontFamily: "'DM Sans', system-ui, sans-serif",
				attributionLogo: false
			},
			grid: {
				vertLines: { visible: !compact, color: theme.gridColor },
				horzLines: { visible: !compact, color: theme.gridColor }
			},
			rightPriceScale: {
				borderVisible: false,
				scaleMargins: { top: compact ? 0.15 : 0.2, bottom: compact ? 0.1 : 0.15 }
			},
			timeScale: {
				visible: !compact,
				borderVisible: false,
				timeVisible: true,
				secondsVisible: false
			},
			handleScale: {
				mouseWheel: !compact,
				pinch: !compact,
				axisPressedMouseMove: !compact
			},
			handleScroll: {
				mouseWheel: !compact,
				pressedMouseMove: !compact
			}
		});

		areaSeries = chart.addSeries(AreaSeries, {
			lineColor: '#2962FF',
			topColor: 'rgba(41, 98, 255, 0.28)',
			bottomColor: 'rgba(41, 98, 255, 0.0)',
			lineWidth: 2,
			priceLineVisible: true,
			lastValueVisible: true
		});

		const resizeObserver = new ResizeObserver((entries) => {
			if (entries[0] && chart) chart.resize(entries[0].contentRect.width, height);
		});
		resizeObserver.observe(chartContainer);

		const themeObserver = new MutationObserver(() => updateChartColors());
		themeObserver.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['class']
		});

		return () => {
			resizeObserver.disconnect();
			themeObserver.disconnect();
		};
	}

	$effect(() => {
		if (!symbol) return;
		selectedResolution;

		let active = true;
		loading = true;
		errorMsg = '';

		async function fetchAndPopulate() {
			const data = await loadHistoricalData(symbol, selectedResolution);
			if (!active) return;

			historyData = data;
			historySource =
				data.length === 0
					? 'empty'
					: data.every((p) => p.source === 'last_known')
						? 'last_known'
						: 'history';

			if (areaSeries) {
				areaSeries.setData(data as any);
				if (data.length > 0) chart?.timeScale().fitContent();
			}
			loading = false;
			updateChartColors();
		}

		fetchAndPopulate();
		return () => {
			active = false;
		};
	});

	$effect(() => {
		if (liveData && areaSeries && !loading) {
			const value = Number(liveData.price);
			if (!Number.isFinite(value) || value <= 0) return;

			let rawTimeMs = Number(liveData.updated_at);
			if (liveData.received_at) {
				const parsed = Date.parse(liveData.received_at);
				if (!Number.isNaN(parsed)) rawTimeMs = parsed;
			}
			if (!Number.isFinite(rawTimeMs) || rawTimeMs <= 0) return;

			const tickTimeSec = Math.floor(rawTimeMs / 1000);
			const bucketSeconds = resolutionSeconds(selectedResolution);
			let roundedTime = Math.floor(tickTimeSec / bucketSeconds) * bucketSeconds;

			const currentHistory = untrack(() => historyData);
			const lastTime =
				currentHistory.length > 0 ? currentHistory[currentHistory.length - 1].time : 0;
			if (roundedTime < lastTime) roundedTime = lastTime;

			const normalizedTick: ChartPoint = { time: roundedTime, value, source: 'realtime' };

			areaSeries.update(normalizedTick as any);
			if (currentHistory.length === 0) {
				historyData = [normalizedTick];
				historySource = 'history';
			} else if (roundedTime > lastTime) {
				historyData = [...currentHistory.slice(-299), normalizedTick];
			} else {
				historyData = [...currentHistory.slice(0, -1), normalizedTick];
			}
			updateChartColors();
		}
	});

	let cleanupChart: (() => void) | undefined;
	onMount(() => {
		cleanupChart = initChart();
		updateChartColors();
	});

	onDestroy(() => {
		if (cleanupChart) cleanupChart();
		if (chart) {
			chart.remove();
			chart = null;
		}
	});
</script>

<!-- Template tidak berubah sama sekali -->
<div class="flex flex-col">
	{#if compact}
		<div class="flex items-center justify-between border-b border-border bg-surface px-4 py-2.5">
			<div class="flex items-center gap-2">
				{#if meta.logo.type === 'img'}
					<div
						class="flex h-6 w-6 shrink-0 items-center justify-center overflow-hidden rounded-full border border-border bg-surface-2"
					>
						<img
							src={meta.logo.url}
							alt={meta.name}
							class="h-full w-full rounded-full object-cover"
						/>
					</div>
				{:else}
					<div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full {meta.badgeClass}">
						<svg
							class="h-4 w-4 text-accent"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"></polyline>
							<polyline points="16 7 22 7 22 13"></polyline>
						</svg>
					</div>
				{/if}
				<span
					class="rounded border border-border bg-surface-2 px-1 py-0.5 text-[9px] font-semibold tracking-wider text-text-dim uppercase"
					>{meta.displaySymbol}</span
				>
				<span
					class="rounded border px-1 py-0.5 font-mono text-[8px] font-bold {freshness.className}"
					>{freshness.label}</span
				>
				<span class="max-w-[85px] truncate text-xs font-bold text-text">{meta.name}</span>
			</div>
			<div class="flex items-center gap-1.5 text-right">
				<span class="font-mono text-xs font-bold text-text">
					{meta.format(
						currentPrice || (historyData.length > 0 ? historyData[historyData.length - 1].value : 0)
					)}
				</span>
				<span
					class="font-mono text-[9px] font-bold {direction === 'up'
						? 'text-green'
						: direction === 'down'
							? 'text-red'
							: 'text-text-dim'}"
				>
					{priceChange >= 0 ? '+' : ''}{percentChange.toFixed(2)}%
				</span>
			</div>
		</div>
	{:else}
		<div
			class="flex flex-col gap-3 border-b border-border bg-surface px-6 py-4 md:flex-row md:items-center md:justify-between"
		>
			<div class="flex items-center gap-3">
				{#if meta.logo.type === 'img'}
					<div
						class="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-full border border-border bg-surface-2"
					>
						<img
							src={meta.logo.url}
							alt={meta.name}
							class="h-full w-full rounded-full object-cover"
						/>
					</div>
				{:else}
					<div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {meta.badgeClass}">
						<svg
							class="h-6 w-6 text-accent"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"></polyline>
							<polyline points="16 7 22 7 22 13"></polyline>
						</svg>
					</div>
				{/if}
				<div>
					<div class="flex items-center gap-2">
						<h3 class="text-base font-bold text-text">{meta.name}</h3>
						<span
							class="rounded border border-border bg-surface-2 px-1.5 py-0.5 text-[9.5px] font-semibold tracking-wider text-text-dim uppercase"
							>{meta.displaySymbol}</span
						>
					</div>
				</div>
			</div>
			<div class="flex flex-col items-start gap-2 md:items-end">
				<div class="text-left md:text-right">
					<div class="font-mono text-2xl font-bold text-text">
						{meta.format(
							currentPrice ||
								(historyData.length > 0 ? historyData[historyData.length - 1].value : 0)
						)}
					</div>
					<div
						class="mt-0.5 flex items-center justify-start gap-1.5 font-mono text-xs font-semibold md:justify-end {direction ===
						'up'
							? 'text-green'
							: direction === 'down'
								? 'text-red'
								: 'text-text-dim'}"
					>
						<span>{direction === 'up' ? '▲' : direction === 'down' ? '▼' : '■'}</span>
						<span
							>{priceChange >= 0 ? '+' : ''}{meta.format(priceChange)} ({priceChange >= 0
								? '+'
								: ''}{percentChange.toFixed(2)}%)</span
						>
					</div>
				</div>
				<div class="flex rounded-lg border border-border bg-surface-2 p-0.5">
					{#each chartResolutions as resolution}
						<button
							type="button"
							class="rounded-md px-2 py-1 text-[10px] font-bold tracking-wide uppercase transition-colors {selectedResolution ===
							resolution
								? 'bg-accent text-white shadow-sm'
								: 'text-text-dim hover:bg-surface hover:text-text'}"
							onclick={() => (selectedResolution = resolution)}
						>
							{resolution}
						</button>
					{/each}
				</div>
			</div>
		</div>
	{/if}

	<div class="relative bg-surface p-2.5">
		{#if !loading && historySource === 'last_known'}
			<div
				class="absolute top-4 left-4 z-10 rounded border border-border bg-surface/90 px-2 py-1 text-[10px] font-bold text-text-dim shadow-sm"
			>
				Last known price
			</div>
		{:else if !loading && historySource === 'empty'}
			<div
				class="absolute inset-0 z-10 flex items-center justify-center bg-surface/75 text-xs font-semibold text-text-dim"
			>
				No chart history available
			</div>
		{/if}
		{#if loading}
			<div class="absolute inset-0 z-10 flex items-center justify-center bg-surface/75">
				<div class="flex flex-col items-center">
					<div
						class="h-6 w-6 animate-spin rounded-full border-2 border-accent border-t-transparent"
					></div>
					{#if !compact}
						<span class="mt-2 text-xs font-semibold text-text-muted"
							>Loading real market data...</span
						>
					{/if}
				</div>
			</div>
		{/if}
		<div bind:this={chartContainer} class="w-full"></div>
	</div>
</div>
