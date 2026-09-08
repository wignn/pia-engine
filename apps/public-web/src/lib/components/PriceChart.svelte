<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import {
		createChart,
		CandlestickSeries,
		AreaSeries,
		type IChartApi,
		type ISeriesApi
	} from 'lightweight-charts';
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
	let candlestickSeries: ISeriesApi<'Candlestick'> | null = null;
	let areaSeries: ISeriesApi<'Area'> | null = null;

	type ChartResolution = '1m' | '5m' | '15m' | '1h' | '4h' | '1D';
	type ChartType = 'candlestick' | 'area';
	const chartResolutions: ChartResolution[] = ['1m', '5m', '15m', '1h', '4h', '1D'];

	let selectedResolution = $state<ChartResolution>('1m');
	let userChartType = $state<ChartType | null>(null);
	let chartType = $derived<ChartType>(userChartType ?? (compact ? 'area' : 'candlestick'));

	interface CandlePoint {
		time: number;
		open: number;
		high: number;
		low: number;
		close: number;
		value: number;
		source?: string;
	}

	let historyData = $state<CandlePoint[]>([]);
	let historySource = $state<'history' | 'last_known' | 'empty'>('empty');
	let loading = $state(true);
	let liveData = $derived(marketStore.getPrice(symbol));
	let currentPrice = $derived(
		Number(liveData?.price) || (historyData.length > 0 ? historyData[historyData.length - 1].close : 0)
	);
	let firstPrice = $derived(
		historyData.length > 0
			? historyData[0].open || historyData[0].close || historyData[0].value
			: currentPrice
	);
	let priceChange = $derived(currentPrice - firstPrice);
	let percentChange = $derived(firstPrice > 0 ? (priceChange / firstPrice) * 100 : 0);
	let direction = $derived(priceChange > 0 ? 'up' : priceChange < 0 ? 'down' : 'none');

	function resolutionSeconds(resolution: ChartResolution) {
		if (resolution === '5m') return 5 * 60;
		if (resolution === '15m') return 15 * 60;
		if (resolution === '1h') return 60 * 60;
		if (resolution === '4h') return 4 * 60 * 60;
		if (resolution === '1D') return 24 * 60 * 60;
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

	function sanitizeChartData(data: unknown): CandlePoint[] {
		const rawItems = Array.isArray(data)
			? data
			: data && typeof data === 'object' && 'items' in data && Array.isArray((data as any).items)
				? (data as any).items
				: [];

		const points = new Map<number, CandlePoint>();
		for (const point of rawItems) {
			if (!point || typeof point !== 'object') continue;
			const row = point as {
				time?: unknown;
				open?: unknown;
				high?: unknown;
				low?: unknown;
				close?: unknown;
				value?: unknown;
				source?: unknown;
			};
			const time = Number(row.time);
			const val = Number(row.close ?? row.value);
			if (!Number.isFinite(time) || !Number.isFinite(val) || time <= 0 || val <= 0) continue;

			const open = Number(row.open ?? val);
			const high = Number(row.high ?? val);
			const low = Number(row.low ?? val);
			const close = Number(row.close ?? val);

			points.set(Math.floor(time), {
				time: Math.floor(time),
				open: Number.isFinite(open) && open > 0 ? open : val,
				high: Number.isFinite(high) && high > 0 ? Math.max(high, open, close) : val,
				low: Number.isFinite(low) && low > 0 ? Math.min(low, open, close) : val,
				close: Number.isFinite(close) && close > 0 ? close : val,
				value: val,
				source: typeof row.source === 'string' ? row.source : undefined
			});
		}

		return [...points.values()].sort((a, b) => a.time - b.time);
	}

	async function loadHistoricalData(
		sym: string,
		resolution: ChartResolution
	): Promise<CandlePoint[]> {
		const upperSym = sym.toUpperCase();
		try {
			const params = new URLSearchParams({ resolution, limit: '240' });
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
		if (!chart) return;
		const theme = getChartTheme(isDarkMode());

		chart.applyOptions({
			layout: { background: { color: theme.background }, textColor: theme.textColor },
			grid: { vertLines: { color: theme.gridColor }, horzLines: { color: theme.gridColor } }
		});

		if (candlestickSeries) {
			candlestickSeries.applyOptions({
				upColor: theme.up,
				downColor: theme.down,
				borderVisible: false,
				wickUpColor: theme.up,
				wickDownColor: theme.down
			});
		}

		if (areaSeries) {
			const colorLine =
				direction === 'up' ? theme.up : direction === 'down' ? theme.down : theme.neutral;
			const colorTop = theme.areaFillTop(colorLine);
			const colorBottom = theme.areaFillBottom(colorLine);
			areaSeries.applyOptions({
				lineColor: colorLine,
				topColor: colorTop,
				bottomColor: colorBottom
			});
		}
	}

	function applySeriesData(data: CandlePoint[]) {
		if (!chart) return;
		if (chartType === 'candlestick') {
			if (areaSeries) areaSeries.setData([]);
			if (candlestickSeries) {
				candlestickSeries.setData(
					data.map((p) => ({
						time: p.time as any,
						open: p.open,
						high: p.high,
						low: p.low,
						close: p.close
					}))
				);
			}
		} else {
			if (candlestickSeries) candlestickSeries.setData([]);
			if (areaSeries) {
				areaSeries.setData(
					data.map((p) => ({
						time: p.time as any,
						value: p.close || p.value
					}))
				);
			}
		}
		if (data.length > 0) chart.timeScale().fitContent();
	}

	function setChartType(type: ChartType) {
		userChartType = type;
		applySeriesData(historyData);
		updateChartColors();
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

		candlestickSeries = chart.addSeries(CandlestickSeries, {
			upColor: theme.up,
			downColor: theme.down,
			borderVisible: false,
			wickUpColor: theme.up,
			wickDownColor: theme.down
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
		const currSym = symbol;
		const currRes = selectedResolution;

		let active = true;
		loading = true;

		async function fetchAndPopulate() {
			const data = await loadHistoricalData(currSym, currRes);
			if (!active) return;

			historyData = data;
			historySource =
				data.length === 0
					? 'empty'
					: data.every((p) => p.source === 'last_known')
						? 'last_known'
						: 'history';

			applySeriesData(data);
			loading = false;
			updateChartColors();
		}

		fetchAndPopulate();
		return () => {
			active = false;
		};
	});

	$effect(() => {
		if (liveData && !loading) {
			const price = Number(liveData.price);
			if (!Number.isFinite(price) || price <= 0) return;

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

			const lastCandle =
				currentHistory.length > 0 ? currentHistory[currentHistory.length - 1] : null;

			let updatedPoint: CandlePoint;
			let nextHistory: CandlePoint[];

			if (!lastCandle) {
				updatedPoint = {
					time: roundedTime,
					open: price,
					high: price,
					low: price,
					close: price,
					value: price,
					source: 'realtime'
				};
				nextHistory = [updatedPoint];
				historySource = 'history';
			} else if (roundedTime === lastCandle.time) {
				updatedPoint = {
					...lastCandle,
					high: Math.max(lastCandle.high, price),
					low: Math.min(lastCandle.low, price),
					close: price,
					value: price,
					source: 'realtime'
				};
				nextHistory = [...currentHistory.slice(0, -1), updatedPoint];
			} else {
				// Roll a new candle
				updatedPoint = {
					time: roundedTime,
					open: lastCandle.close,
					high: Math.max(lastCandle.close, price),
					low: Math.min(lastCandle.close, price),
					close: price,
					value: price,
					source: 'realtime'
				};
				nextHistory = [...currentHistory.slice(-299), updatedPoint];
			}

			historyData = nextHistory;

			if (chartType === 'candlestick' && candlestickSeries) {
				candlestickSeries.update({
					time: updatedPoint.time as any,
					open: updatedPoint.open,
					high: updatedPoint.high,
					low: updatedPoint.low,
					close: updatedPoint.close
				});
			} else if (chartType === 'area' && areaSeries) {
				areaSeries.update({
					time: updatedPoint.time as any,
					value: updatedPoint.close
				});
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
						currentPrice || (historyData.length > 0 ? historyData[historyData.length - 1].close : 0)
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
								(historyData.length > 0 ? historyData[historyData.length - 1].close : 0)
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
				<div class="flex flex-wrap items-center gap-2">
					<div class="flex rounded-lg border border-border bg-surface-2 p-0.5">
						<button
							type="button"
							class="rounded-md px-2 py-1 text-[10px] font-bold tracking-wide uppercase transition-colors {chartType ===
							'candlestick'
								? 'bg-accent text-white shadow-sm'
								: 'text-text-dim hover:bg-surface hover:text-text'}"
							onclick={() => setChartType('candlestick')}
						>
							Candles
						</button>
						<button
							type="button"
							class="rounded-md px-2 py-1 text-[10px] font-bold tracking-wide uppercase transition-colors {chartType ===
							'area'
								? 'bg-accent text-white shadow-sm'
								: 'text-text-dim hover:bg-surface hover:text-text'}"
							onclick={() => setChartType('area')}
						>
							Area
						</button>
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
