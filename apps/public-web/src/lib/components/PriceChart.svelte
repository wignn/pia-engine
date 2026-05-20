<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { createChart, AreaSeries, type IChartApi, type ISeriesApi } from 'lightweight-charts';
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';
	import { CORE_REST_URL } from '$lib/config';

	interface Props {
		symbol: string;
		height?: number;
		compact?: boolean;
	}
	let { symbol, height = 380, compact = false }: Props = $props();

	let chartContainer = $state<HTMLDivElement | null>(null);
	let chart: IChartApi | null = null;
	let areaSeries: ISeriesApi<'Area'> | null = null;

	let historyData = $state<{ time: number; value: number }[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');
	let liveData = $derived(marketStore.getPrice(symbol));
	let currentPrice = $derived(liveData?.price ?? 0);
	let firstPrice = $derived(historyData.length > 0 ? historyData[0].value : currentPrice);
	let priceChange = $derived(currentPrice - firstPrice);
	let percentChange = $derived(firstPrice > 0 ? (priceChange / firstPrice) * 100 : 0);
	let direction = $derived(priceChange > 0 ? 'up' : priceChange < 0 ? 'down' : 'none');
	let lastChartTime = $derived(historyData.length > 0 ? historyData[historyData.length - 1].time : 0);
	function getSymbolMeta(sym: string) {
		const upper = sym.toUpperCase();
		let name = upper;
		let badge = upper.substring(0, 3);
		let badgeColor = 'bg-accent/10 text-accent';
		let format = (val: number) => val.toFixed(5);
		let logo = { type: 'img', url: '' };

		if (upper === 'BTCUSDT') {
			name = 'Bitcoin';
			badge = 'BTC';
			badgeColor = 'bg-[#F7931A]/10 text-[#F7931A] border border-[#F7931A]/20';
			format = (val: number) => `$${val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/btc@2x.png' };
		} else if (upper === 'ETHUSDT') {
			name = 'Ethereum';
			badge = 'ETH';
			badgeColor = 'bg-[#627EEA]/10 text-[#627EEA] border border-[#627EEA]/20';
			format = (val: number) => `$${val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/eth@2x.png' };
		} else if (upper === 'SOLUSDT') {
			name = 'Solana';
			badge = 'SOL';
			badgeColor = 'bg-[#14F195]/10 text-[#14F195] border border-[#14F195]/20';
			format = (val: number) => `$${val.toFixed(2)}`;
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/sol@2x.png' };
		} else if (upper === 'BNBUSDT') {
			name = 'Binance Coin';
			badge = 'BNB';
			badgeColor = 'bg-[#F3BA2F]/10 text-[#F3BA2F] border border-[#F3BA2F]/20';
			format = (val: number) => `$${val.toFixed(2)}`;
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/bnb@2x.png' };
		} else if (upper === 'EURUSD') {
			name = 'Euro / US Dollar';
			badge = 'EUR';
			badgeColor = 'bg-[#003399]/10 text-[#003399] border border-[#003399]/20';
			format = (val: number) => val.toFixed(5);
			logo = { type: 'img', url: 'https://flagcdn.com/w80/eu.png' };
		} else if (upper === 'GBPUSD') {
			name = 'Pound Sterling / US Dollar';
			badge = 'GBP';
			badgeColor = 'bg-[#C8102E]/10 text-[#C8102E] border border-[#C8102E]/20';
			format = (val: number) => val.toFixed(5);
			logo = { type: 'img', url: 'https://flagcdn.com/w80/gb.png' };
		} else if (upper === 'USDJPY') {
			name = 'US Dollar / Japanese Yen';
			badge = 'JPY';
			badgeColor = 'bg-[#BC002D]/10 text-[#BC002D] border border-[#BC002D]/20';
			format = (val: number) => val.toFixed(3);
			logo = { type: 'img', url: 'https://flagcdn.com/w80/jp.png' };
		} else if (upper === 'XAUUSD') {
			name = 'Gold Spot / US Dollar';
			badge = 'GOLD';
			badgeColor = 'bg-[#FFD700]/10 text-[#FFD700] border border-[#FFD700]/20';
			format = (val: number) => `$${val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
			logo = { type: 'svg', url: '' };
		} else if (upper === 'SPX') {
			name = 'S&P 500 Index';
			badge = 'SPX';
			badgeColor = 'bg-blue-600/10 text-blue-600 border border-blue-600/20';
			format = (val: number) => val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
			logo = { type: 'img', url: 'https://s3-symbol-logo.tradingview.com/indices/s-and-p-500.svg' };
		} else if (upper === 'DXY') {
			name = 'US Dollar Index';
			badge = 'DXY';
			badgeColor = 'bg-emerald-600/10 text-emerald-600 border border-emerald-600/20';
			format = (val: number) => val.toFixed(3);
			logo = { type: 'img', url: 'https://s3-symbol-logo.tradingview.com/indices/u-s-dollar-index.svg' };
		}

		return { name, badge, badgeColor, format, logo };
	}

	let meta = $derived(getSymbolMeta(symbol));

	async function loadHistoricalData(sym: string, initialPrice: number): Promise<{ time: number; value: number }[]> {
		const upperSym = sym.toUpperCase();
		const now = Math.floor(Date.now() / 1000);
		const limit = 120; 

		try {
			const restBaseUrl = CORE_REST_URL.replace(/^ws/, 'http');
			const res = await fetch(`${restBaseUrl}/api/v1/market/history/${upperSym}`);
			if (res.ok) {
				const data = await res.json();
				if (Array.isArray(data) && data.length > 0) {
					return data;
				}
			}
		} catch (e) {
			console.warn(`[PriceChart] Backend history proxy fetch failed for ${upperSym}, trying direct Binance/fallback`, e);
		}

		if (upperSym.endsWith('USDT')) {
			try {
				const res = await fetch(`https://api.binance.com/api/v3/klines?symbol=${upperSym}&interval=1m&limit=${limit}`);
				if (res.ok) {
					const data = await res.json();
					return data.map((item: any) => ({
						time: Math.floor(item[0] / 1000),
						value: parseFloat(item[4])
					}));
				}
			} catch (e) {
				console.warn('[PriceChart] Direct Binance fallback fetch failed', e);
			}
		}

		const fallbackData = [];
		let price = initialPrice > 0 ? initialPrice : 1.0850; 
		if (upperSym.includes('JPY')) price = 150.0;
		if (upperSym === 'XAUUSD') price = 2400.0;
		if (upperSym === 'SPX') price = 5200.0;
		if (upperSym === 'DXY') price = 104.50;
		if (upperSym.endsWith('USDT')) price = upperSym.startsWith('BTC') ? 95000.0 : 3000.0;

		const intervalSec = 60;
		for (let i = limit - 1; i >= 0; i--) {
			const time = now - (i * intervalSec);
			const change = (Math.random() - 0.5) * (price * 0.0004);
			price = price + change;
			fallbackData.push({ time, value: price });
		}
		return fallbackData;
	}

	function updateChartColors() {
		if (!areaSeries || !chart) return;

		const isDark = document.documentElement.classList.contains('dark');
		const gridColor = isDark ? '#2A2E39' : '#e0e3eb';
		const textColor = isDark ? '#B2B5BE' : '#434651';
		const bgColor = isDark ? '#1E222D' : '#ffffff';

		chart.applyOptions({
			layout: {
				background: { color: bgColor },
				textColor: textColor,
			},
			grid: {
				vertLines: { color: gridColor },
				horzLines: { color: gridColor },
			}
		});

		const colorLine = direction === 'up' ? '#089981' : direction === 'down' ? '#F23645' : '#2962FF';
		const colorTop = direction === 'up' ? 'rgba(8, 153, 129, 0.28)' : direction === 'down' ? 'rgba(242, 54, 69, 0.28)' : 'rgba(41, 98, 255, 0.28)';
		const colorBottom = direction === 'up' ? 'rgba(8, 153, 129, 0.0)' : direction === 'down' ? 'rgba(242, 54, 69, 0.0)' : 'rgba(41, 98, 255, 0.0)';

		areaSeries.applyOptions({
			lineColor: colorLine,
			topColor: colorTop,
			bottomColor: colorBottom,
		});
	}

	function initChart() {
		if (!chartContainer) return;

		const isDark = document.documentElement.classList.contains('dark');
		const gridColor = isDark ? '#2A2E39' : '#e0e3eb';
		const textColor = isDark ? '#B2B5BE' : '#434651';
		const bgColor = isDark ? '#1E222D' : '#ffffff';

		chart = createChart(chartContainer, {
			width: chartContainer.clientWidth,
			height: height,
			layout: {
				background: { color: bgColor },
				textColor: textColor,
				fontFamily: "'Inter', sans-serif",
				attributionLogo: false,
			},
			grid: {
				vertLines: { visible: !compact, color: gridColor, style: 2 },
				horzLines: { visible: !compact, color: gridColor, style: 2 },
			},
			rightPriceScale: {
				borderVisible: false,
				scaleMargins: {
					top: compact ? 0.15 : 0.2,
					bottom: compact ? 0.1 : 0.15,
				},
			},
			timeScale: {
				visible: !compact,
				borderVisible: false,
				timeVisible: true,
				secondsVisible: false,
			},
			handleScale: {
				mouseWheel: !compact,
				pinch: !compact,
				axisPressedMouseMove: !compact,
			},
			handleScroll: {
				mouseWheel: !compact,
				pressedMouseMove: !compact,
			},
		});

		areaSeries = chart.addSeries(AreaSeries, {
			lineColor: '#2962FF',
			topColor: 'rgba(41, 98, 255, 0.28)',
			bottomColor: 'rgba(41, 98, 255, 0.0)',
			lineWidth: 2,
			priceLineVisible: true,
			lastValueVisible: true,
		});
		const resizeObserver = new ResizeObserver((entries) => {
			if (entries[0] && chart) {
				const { width } = entries[0].contentRect;
				chart.resize(width, height);
			}
		});
		resizeObserver.observe(chartContainer);

		const themeObserver = new MutationObserver(() => {
			updateChartColors();
		});
		themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });

		return () => {
			resizeObserver.disconnect();
			themeObserver.disconnect();
		};
	}

	$effect(() => {
		if (!symbol) return;
		
		let active = true;
		loading = true;
		errorMsg = '';

		async function fetchAndPopulate() {
			const basePrice = untrack(() => liveData?.price ?? 0);
			const data = await loadHistoricalData(symbol, basePrice);
			
			if (!active) return;
			
			historyData = data;
			
			if (areaSeries && data.length > 0) {
				areaSeries.setData(data as any);
				chart?.timeScale().fitContent();
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
			let rawTimeMs = liveData.updated_at;
			if (liveData.received_at) {
				const parsed = Date.parse(liveData.received_at);
				if (!isNaN(parsed)) {
					rawTimeMs = parsed;
				}
			}

			const tickTimeSec = Math.floor(rawTimeMs / 1000);
			let roundedTime = Math.floor(tickTimeSec / 60) * 60;
			const lastTime = lastChartTime;
			if (roundedTime < lastTime) {
				roundedTime = lastTime;
			}

			areaSeries.update({
				time: roundedTime as any,
				value: liveData.price
			});
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

<div class="flex flex-col bg-surface border border-border rounded-lg shadow-sm overflow-hidden animate-fade-in">
	{#if compact}
		<div class="flex items-center justify-between border-b border-border bg-surface px-4 py-2.5">
			<div class="flex items-center gap-2">
				{#if meta.logo.type === 'img'}
					<div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full overflow-hidden border border-border bg-surface-2">
						<img src={meta.logo.url} alt={meta.name} class="h-full w-full object-cover rounded-full" />
					</div>
				{:else}
					<div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-accent/10 border border-accent/20">
						{#if symbol.toUpperCase() === 'XAUUSD'}
							<svg class="h-4 w-4" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
								<path d="M16 28 L48 28 L40 18 L24 18 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
								<path d="M8 46 L36 46 L30 36 L14 36 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
								<path d="M28 46 L56 46 L50 36 L34 36 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
							</svg>
						{:else}
							<svg class="h-4 w-4 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"></polyline>
								<polyline points="16 7 22 7 22 13"></polyline>
							</svg>
						{/if}
					</div>
				{/if}
				<span class="rounded bg-surface-2 border border-border px-1 py-0.5 text-[9px] font-semibold uppercase tracking-wider text-text-dim">{symbol}</span>
				<span class="text-xs font-bold text-text truncate max-w-[85px]">{meta.name}</span>
			</div>
			<div class="text-right flex items-center gap-1.5">
				<span class="font-mono text-xs font-bold text-text">
					{meta.format(currentPrice || (historyData.length > 0 ? historyData[historyData.length-1].value : 0))}
				</span>
				<span class="text-[9px] font-bold font-mono {direction === 'up' ? 'text-green' : direction === 'down' ? 'text-red' : 'text-text-dim'}">
					{priceChange >= 0 ? '+' : ''}{percentChange.toFixed(2)}%
				</span>
			</div>
		</div>
	{:else}
		<div class="flex items-center justify-between border-b border-border bg-surface px-6 py-4">
			<div class="flex items-center gap-3">
				{#if meta.logo.type === 'img'}
					<div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full overflow-hidden border border-border bg-surface-2">
						<img src={meta.logo.url} alt={meta.name} class="h-full w-full object-cover rounded-full" />
					</div>
				{:else}
					<div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-accent/10 border border-accent/20">
						{#if symbol.toUpperCase() === 'XAUUSD'}
							<svg class="h-6 w-6" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
								<path d="M16 28 L48 28 L40 18 L24 18 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
								<path d="M8 46 L36 46 L30 36 L14 36 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
								<path d="M28 46 L56 46 L50 36 L34 36 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
							</svg>
						{:else}
							<svg class="h-6 w-6 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"></polyline>
								<polyline points="16 7 22 7 22 13"></polyline>
							</svg>
						{/if}
					</div>
				{/if}
				<div>
					<div class="flex items-center gap-2">
						<h3 class="text-base font-bold text-text">{meta.name}</h3>
						<span class="rounded bg-surface-2 border border-border px-1.5 py-0.5 text-[9.5px] font-semibold uppercase tracking-wider text-text-dim">{symbol}</span>
					</div>
				</div>
			</div>

			<div class="text-right">
				<div class="text-2xl font-bold font-mono text-text">
					{meta.format(currentPrice || (historyData.length > 0 ? historyData[historyData.length-1].value : 0))}
				</div>
				<div class="flex items-center justify-end gap-1.5 text-xs font-semibold font-mono mt-0.5 {direction === 'up' ? 'text-green' : direction === 'down' ? 'text-red' : 'text-text-dim'}">
					<span>{direction === 'up' ? '▲' : direction === 'down' ? '▼' : '■'}</span>
					<span>{priceChange >= 0 ? '+' : ''}{meta.format(priceChange)} ({priceChange >= 0 ? '+' : ''}{percentChange.toFixed(2)}%)</span>
				</div>
			</div>
		</div>
	{/if}

	<div class="relative bg-surface p-2.5">
		{#if loading}
			<div class="absolute inset-0 flex items-center justify-center bg-surface/75 z-10">
				<div class="flex flex-col items-center">
					<div class="h-6 w-6 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
					{#if !compact}
						<span class="text-xs text-text-muted mt-2 font-semibold">Loading real market data...</span>
					{/if}
				</div>
			</div>
		{/if}
		<div bind:this={chartContainer} class="w-full"></div>
	</div>
</div>

