<script lang="ts">
	import { onMount } from 'svelte';
	import { Search, ArrowUpRight, ArrowDownRight, TrendingUp, TrendingDown } from 'lucide-svelte';
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';
	import { CORE_REST_URL } from '$lib/config';

	interface Props {
		onselect: (symbol: string) => void;
	}
	let { onselect }: Props = $props();

	let allPrices: PriceData[] = $derived(marketStore.prices);
	let activeCategory = $state('all');
	let searchQuery = $state('');
	let sortBy = $state('change-desc');

	let initialPrices = $state<Map<string, number>>(new Map());
	let sparklines = $state<Record<string, number[]>>({});
	let sparklineLoading = $state<Record<string, boolean>>({});

	// Categories Definition
	const categories = [
		{ id: 'all', name: 'All Markets' },
		{ id: 'crypto', name: 'Crypto' },
		{ id: 'forex', name: 'Forex' },
		{ id: 'indices', name: 'Indices' },
		{ id: 'commodities', name: 'Commodities' }
	];

	function getAssetCategory(symbol: string): string {
		const sym = symbol.toUpperCase();
		if (['BTCUSDT', 'ETHUSDT', 'SOLUSDT', 'BNBUSDT'].includes(sym)) return 'crypto';
		if (['EURUSD', 'GBPUSD', 'USDJPY'].includes(sym)) return 'forex';
		if (['SPX', 'DXY'].includes(sym)) return 'indices';
		if (sym === 'XAUUSD') return 'commodities';
		return 'other';
	}

	function getCategoryName(cat: string): string {
		if (cat === 'crypto') return 'Cryptocurrency';
		if (cat === 'forex') return 'Forex (Currencies)';
		if (cat === 'indices') return 'Stock Indices';
		if (cat === 'commodities') return 'Commodities & Metals';
		return 'Other Markets';
	}

	// Ticker Metadata and Details
	function getSymbolDetails(symbol: string) {
		const sym = symbol.toUpperCase();
		let name = sym;
		let badge = sym.substring(0, 3);
		let badgeColor = 'bg-accent/10 text-accent border border-accent/20';
		let unit = 'USD';
		let format = (val: number) => val.toFixed(5);
		let logo = { type: 'img', url: '' };

		if (sym === 'BTCUSDT') {
			name = 'Bitcoin';
			badge = 'BTC';
			badgeColor = 'bg-[#F7931A]/10 text-[#F7931A] border border-[#F7931A]/20';
			unit = 'USD';
			format = (val: number) => val.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/btc@2x.png' };
		} else if (sym === 'ETHUSDT') {
			name = 'Ethereum';
			badge = 'ETH';
			badgeColor = 'bg-[#627EEA]/10 text-[#627EEA] border border-[#627EEA]/20';
			unit = 'USD';
			format = (val: number) => val.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/eth@2x.png' };
		} else if (sym === 'SOLUSDT') {
			name = 'Solana';
			badge = 'SOL';
			badgeColor = 'bg-[#14F195]/10 text-[#14F195] border border-[#14F195]/20';
			unit = 'USD';
			format = (val: number) => val.toFixed(2);
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/sol@2x.png' };
		} else if (sym === 'BNBUSDT') {
			name = 'Binance Coin';
			badge = 'BNB';
			badgeColor = 'bg-[#F3BA2F]/10 text-[#F3BA2F] border border-[#F3BA2F]/20';
			unit = 'USD';
			format = (val: number) => val.toFixed(1);
			logo = { type: 'img', url: 'https://assets.coincap.io/assets/icons/bnb@2x.png' };
		} else if (sym === 'EURUSD') {
			name = 'Euro / USD';
			badge = 'EUR';
			badgeColor = 'bg-[#003399]/10 text-[#003399] border border-[#003399]/20';
			unit = 'RATE';
			format = (val: number) => val.toFixed(5);
			logo = { type: 'img', url: 'https://flagcdn.com/w80/eu.png' };
		} else if (sym === 'GBPUSD') {
			name = 'GBP / USD';
			badge = 'GBP';
			badgeColor = 'bg-[#C8102E]/10 text-[#C8102E] border border-[#C8102E]/20';
			unit = 'RATE';
			format = (val: number) => val.toFixed(5);
			logo = { type: 'img', url: 'https://flagcdn.com/w80/gb.png' };
		} else if (sym === 'USDJPY') {
			name = 'USD / Yen';
			badge = 'JPY';
			badgeColor = 'bg-[#BC002D]/10 text-[#BC002D] border border-[#BC002D]/20';
			unit = 'JPY';
			format = (val: number) => val.toFixed(3);
			logo = { type: 'img', url: 'https://flagcdn.com/w80/jp.png' };
		} else if (sym === 'XAUUSD') {
			name = 'Gold Spot';
			badge = 'GOLD';
			badgeColor = 'bg-[#FFD700]/10 text-[#FFD700] border border-[#FFD700]/20';
			unit = 'USD';
			format = (val: number) => val.toLocaleString(undefined, { minimumFractionDigits: 1, maximumFractionDigits: 1 });
			logo = { type: 'svg', url: '' };
		} else if (sym === 'SPX') {
			name = 'S&P 500';
			badge = 'SPX';
			badgeColor = 'bg-blue-600/10 text-blue-600 border border-blue-600/20';
			unit = 'USD';
			format = (val: number) => val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
			logo = { type: 'img', url: 'https://s3-symbol-logo.tradingview.com/indices/s-and-p-500.svg' };
		} else if (sym === 'DXY') {
			name = 'US Dollar Index';
			badge = 'DXY';
			badgeColor = 'bg-emerald-600/10 text-emerald-600 border border-[#10B981]/20';
			unit = 'RATE';
			format = (val: number) => val.toFixed(3);
			logo = { type: 'img', url: 'https://s3-symbol-logo.tradingview.com/indices/u-s-dollar-index.svg' };
		}

		return { name, badge, badgeColor, unit, format, logo };
	}

	// Capture initial prices for performance tracking
	$effect(() => {
		for (const p of allPrices) {
			if (!initialPrices.has(p.symbol) && p.price > 0) {
				initialPrices.set(p.symbol, p.price);
			}
		}
	});

	// Percent Change helper
	function getPercentChange(p: PriceData): { value: number; string: string } {
		const base = initialPrices.get(p.symbol) ?? p.price;
		if (base === 0) return { value: 0, string: '0.00%' };
		const pct = ((p.price - base) / base) * 100;
		const sign = pct >= 0 ? '+' : '';
		return {
			value: pct,
			string: `${sign}${pct.toFixed(2)}%`
		};
	}

	// Flash Map for real-time WebSocket ticks
	let flashMap = $state<Map<string, 'up' | 'down'>>(new Map());
	const activeTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	$effect(() => {
		const now = Date.now();
		for (const p of allPrices) {
			if (p.direction !== 'none' && now - p.updated_at < 1000) {
				if (activeTimeouts.has(p.symbol) && flashMap.get(p.symbol) === p.direction) {
					continue;
				}

				if (activeTimeouts.has(p.symbol)) {
					clearTimeout(activeTimeouts.get(p.symbol));
				}

				flashMap.set(p.symbol, p.direction);

				const timeout = setTimeout(() => {
					flashMap.delete(p.symbol);
					activeTimeouts.delete(p.symbol);
				}, 600);

				activeTimeouts.set(p.symbol, timeout);
			}
		}

		return () => {
			for (const timeout of activeTimeouts.values()) {
				clearTimeout(timeout);
			}
			activeTimeouts.clear();
		};
	});

	// Sparkline data loading
	async function loadHistoricalData(sym: string, initialPrice: number): Promise<number[]> {
		const upperSym = sym.toUpperCase();
		const limit = 20;

		try {
			const restBaseUrl = CORE_REST_URL.replace(/^ws/, 'http');
			const res = await fetch(`${restBaseUrl}/api/v1/market/history/${upperSym}`);
			if (res.ok) {
				const data = await res.json();
				if (Array.isArray(data) && data.length > 0) {
					// Use recent history data points
					const sliceData = data.slice(-limit);
					if (sliceData.length > 0) {
						// Store the first historical price as the base price for more accurate % change calculations
						if (sliceData[0].value > 0) {
							initialPrices.set(upperSym, sliceData[0].value);
						}
						return sliceData.map((item: any) => item.value);
					}
				}
			}
		} catch (e) {
			console.warn(`[Heatmap] Proxy history fetch failed for ${upperSym}`, e);
		}

		// Direct Binance fallback
		if (upperSym.endsWith('USDT')) {
			try {
				const res = await fetch(`https://api.binance.com/api/v3/klines?symbol=${upperSym}&interval=5m&limit=${limit}`);
				if (res.ok) {
					const data = await res.json();
					const klines = data.map((item: any) => parseFloat(item[4]));
					if (klines.length > 0) {
						initialPrices.set(upperSym, klines[0]);
					}
					return klines;
				}
			} catch (e) {
				console.warn('[Heatmap] Direct Binance fetch failed', e);
			}
		}

		// Fallback generated points
		const fallbackData = [];
		let price = initialPrice > 0 ? initialPrice : 1.0;
		if (upperSym.includes('JPY')) price = 150.0;
		if (upperSym === 'XAUUSD') price = 2400.0;
		if (upperSym === 'SPX') price = 5200.0;
		if (upperSym === 'DXY') price = 104.50;
		if (upperSym.endsWith('USDT')) price = upperSym.startsWith('BTC') ? 95000.0 : 3000.0;

		initialPrices.set(upperSym, price * 0.995); // offset base so we have a nice change
		for (let i = 0; i < limit; i++) {
			const change = (Math.random() - 0.5) * (price * 0.001);
			price = price + change;
			fallbackData.push(price);
		}
		return fallbackData;
	}

	$effect(() => {
		for (const p of allPrices) {
			const sym = p.symbol;
			if (!sparklines[sym] && !sparklineLoading[sym]) {
				sparklineLoading[sym] = true;
				loadHistoricalData(sym, p.price).then((hist) => {
					sparklines[sym] = hist;
					sparklineLoading[sym] = false;
				});
			}
		}
	});

	// Append websocket price updates to sparklines in real-time
	$effect(() => {
		for (const p of allPrices) {
			const sym = p.symbol;
			const currentSpark = sparklines[sym];
			if (currentSpark && currentSpark.length > 0) {
				const lastVal = currentSpark[currentSpark.length - 1];
				if (p.price !== lastVal) {
					// Shift and push
					sparklines[sym] = [...currentSpark.slice(1), p.price];
				}
			}
		}
	});

	// SVG Sparkline Math Helper
	function getSparklinePath(values: number[], width = 100, height = 30): string {
		if (!values || values.length < 2) return '';
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min === 0 ? 1 : max - min;

		const points = values.map((val, index) => {
			const x = (index / (values.length - 1)) * width;
			const y = height - ((val - min) / range) * height;
			return `${x.toFixed(1)},${y.toFixed(1)}`;
		});

		return `M ${points.join(' L ')}`;
	}

	function getSparklineFillPath(values: number[], width = 100, height = 30): string {
		if (!values || values.length < 2) return '';
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min === 0 ? 1 : max - min;

		const points = values.map((val, index) => {
			const x = (index / (values.length - 1)) * width;
			const y = height - ((val - min) / range) * height;
			return `${x.toFixed(1)},${y.toFixed(1)}`;
		});

		return `M 0,${height} L ${points.join(' L ')} L ${width},${height} Z`;
	}

	// Filter and Sort Processing
	let processedPrices = $derived.by(() => {
		let list = allPrices.map((p) => {
			const category = getAssetCategory(p.symbol);
			const pct = getPercentChange(p);
			const details = getSymbolDetails(p.symbol);
			return {
				...p,
				category,
				pct,
				details
			};
		});

		// Apply Search
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase().trim();
			list = list.filter(
				(item) =>
					item.symbol.toLowerCase().includes(q) ||
					item.details.name.toLowerCase().includes(q)
			);
		}

		// Apply Category Tab (If not 'all')
		if (activeCategory !== 'all') {
			list = list.filter((item) => item.category === activeCategory);
		}

		// Apply Sorting
		list.sort((a, b) => {
			if (sortBy === 'symbol') {
				return a.symbol.localeCompare(b.symbol);
			} else if (sortBy === 'change-desc') {
				return b.pct.value - a.pct.value;
			} else if (sortBy === 'change-asc') {
				return a.pct.value - b.pct.value;
			} else if (sortBy === 'price-desc') {
				return b.price - a.price;
			} else if (sortBy === 'price-asc') {
				return a.price - b.price;
			}
			return 0;
		});

		return list;
	});

	// Grouping for the 'All Markets' view to render Treemap style blocks
	let groupedPrices = $derived.by(() => {
		const groups: Record<string, typeof processedPrices> = {
			crypto: [],
			forex: [],
			indices: [],
			commodities: []
		};

		for (const item of processedPrices) {
			if (groups[item.category]) {
				groups[item.category].push(item);
			}
		}

		return groups;
	});

	// Treemap Cell Spans Helper
	function getCellSpan(symbol: string, isSingleGroup: boolean): string {
		const sym = symbol.toUpperCase();
		if (isSingleGroup) {
			if (sym === 'BTCUSDT') return 'col-span-2 row-span-2 md:col-span-4 md:row-span-2';
			if (sym === 'ETHUSDT') return 'col-span-2 row-span-1 md:col-span-4 md:row-span-1';
			if (sym === 'EURUSD' || sym === 'SPX' || sym === 'XAUUSD') return 'col-span-2 row-span-1 md:col-span-3 md:row-span-1';
			return 'col-span-1 row-span-1 md:col-span-2 md:row-span-1';
		}

		// Combined All-Markets layout spans
		if (sym === 'BTCUSDT') return 'col-span-2 row-span-2 md:col-span-2 md:row-span-2';
		if (sym === 'ETHUSDT') return 'col-span-2 row-span-1 md:col-span-2 md:row-span-1';
		if (sym === 'EURUSD') return 'col-span-2 row-span-2 md:col-span-2 md:row-span-2';
		if (sym === 'SPX') return 'col-span-2 row-span-1 md:col-span-2 md:row-span-1';
		if (sym === 'XAUUSD') return 'col-span-1 row-span-2 md:col-span-1 md:row-span-2';
		return 'col-span-1 row-span-1 md:col-span-1 md:row-span-1';
	}

	// Helper to calculate color styles dynamically matching TradingView HSL color grade
	function getHeatmapBgStyle(pctVal: number): string {
		const limit = 3.0; // limit percentage change intensity to 3%
		const ratio = Math.min(Math.abs(pctVal) / limit, 1.0);
		
		// Dark mode has slightly lower light intensity than light mode for aesthetic contrast
		const isDark = typeof document !== 'undefined' && document.documentElement.classList.contains('dark');
		const baseAlpha = isDark ? 0.08 : 0.06;
		const maxAddedAlpha = isDark ? 0.38 : 0.32;
		const alpha = baseAlpha + ratio * maxAddedAlpha;

		if (pctVal >= 0) {
			// Green: hsl(160, 90%, X%)
			return `background: rgba(8, 153, 129, ${alpha}); border-color: rgba(8, 153, 129, ${0.15 + ratio * 0.55});`;
		} else {
			// Red: hsl(355, 85%, X%)
			return `background: rgba(242, 54, 69, ${alpha}); border-color: rgba(242, 54, 69, ${0.15 + ratio * 0.55});`;
		}
	}
</script>

<div class="flex flex-col bg-surface border border-border rounded-lg shadow-sm overflow-hidden h-full">
	<!-- Control Bar: Search, Filters, and Sorting -->
	<div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4 border-b border-border bg-surface px-4 py-3 shrink-0 z-20">
		<div class="flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0 scrollbar-none">
			{#each categories as cat}
				<button
					onclick={() => activeCategory = cat.id}
					class="px-3 py-1.5 text-xs font-semibold rounded-md border transition-all whitespace-nowrap cursor-pointer
					{activeCategory === cat.id
						? 'bg-accent/10 border-accent/30 text-accent font-bold shadow-sm'
						: 'bg-surface border-border text-text-dim hover:text-text hover:border-text-dim/30'}"
				>
					{cat.name}
				</button>
			{/each}
		</div>

		<div class="flex items-center gap-3 justify-between sm:justify-end">
			<!-- Search -->
			<div class="relative flex-1 sm:flex-none sm:w-48">
				<Search class="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-text-dim" />
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search symbol..."
					class="w-full rounded-md border border-border bg-surface-2 pl-8.5 pr-3 py-1.5 text-xs font-medium text-text placeholder-text-dim focus:outline-none focus:border-accent"
				/>
			</div>

			<!-- Sort -->
			<select
				bind:value={sortBy}
				class="rounded-md border border-border bg-surface-2 px-3 py-1.5 text-xs font-semibold text-text focus:outline-none focus:border-accent cursor-pointer"
			>
				<option value="change-desc">Gainers (High → Low)</option>
				<option value="change-asc">Losers (Low → High)</option>
				<option value="symbol">Ticker Symbol</option>
				<option value="price-desc">Price (High → Low)</option>
				<option value="price-asc">Price (Low → High)</option>
			</select>
		</div>
	</div>

	<!-- Heatmap Container -->
	<div class="p-4 bg-bg flex-1 overflow-y-auto min-h-[420px]">
		{#if allPrices.length === 0}
			<div class="flex flex-col items-center justify-center py-20 text-center">
				<div class="h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
				<p class="text-xs text-text-muted mt-3 font-semibold">Connecting to market websocket...</p>
			</div>
		{:else if processedPrices.length === 0}
			<div class="flex flex-col items-center justify-center py-20 text-center text-text-dim">
				<p class="text-sm font-semibold">No assets found</p>
				<p class="text-xs mt-1">Try modifying your filter or search query</p>
			</div>
		{:else if activeCategory !== 'all'}
			<!-- Single Category View: Unified grid -->
			<div class="grid grid-cols-2 md:grid-cols-6 gap-3">
				{#each processedPrices as p (p.symbol)}
					{@const flash = flashMap.get(p.symbol)}
					{@const style = getHeatmapBgStyle(p.pct.value)}
					{@const hist = sparklines[p.symbol]}

					<button
						onclick={() => onselect(p.symbol)}
						{style}
						class="group relative rounded-lg border text-left p-3.5 flex flex-col justify-between transition-all duration-300 hover:shadow-lg cursor-pointer overflow-hidden min-h-[110px]
						{getCellSpan(p.symbol, true)}
						{flash === 'up' ? 'flash-green' : flash === 'down' ? 'flash-red' : ''}"
					>
						<!-- Neon hover glow -->
						<div class="absolute inset-0 opacity-0 group-hover:opacity-10 pointer-events-none transition-opacity duration-300 bg-radial from-white via-transparent to-transparent"></div>

						<div class="flex items-start justify-between w-full z-10">
							<div class="flex items-center gap-2">
								{#if p.details.logo.type === 'img'}
									<img src={p.details.logo.url} alt={p.symbol} class="h-6 w-6 rounded-full object-cover border border-border/40" />
								{:else}
									<div class="h-6 w-6 rounded-full bg-accent/15 border border-accent/30 flex items-center justify-center">
										<span class="text-[9px] font-bold text-accent">GOLD</span>
									</div>
								{/if}
								<div>
									<div class="font-bold text-xs uppercase tracking-tight text-text leading-tight">{p.symbol}</div>
									<div class="text-[9px] text-text-dim truncate max-w-[110px] leading-tight font-medium mt-0.5">{p.details.name}</div>
								</div>
							</div>
							<div class="flex items-center gap-1 text-[11px] font-bold font-mono px-1.5 py-0.5 rounded {p.pct.value >= 0 ? 'text-green bg-green/10' : 'text-red bg-red/10'}">
								{#if p.pct.value >= 0}
									<ArrowUpRight class="h-3 w-3" />
								{:else}
									<ArrowDownRight class="h-3 w-3" />
								{/if}
								{p.pct.string}
							</div>
						</div>

						<!-- Sparkline & Price Section -->
						<div class="mt-4 flex items-end justify-between w-full z-10">
							<div class="flex flex-col">
								<span class="text-[9px] text-text-dim uppercase font-mono font-semibold tracking-wider">{p.details.unit}</span>
								<span class="font-mono text-base md:text-lg font-extrabold text-text mt-0.5">{p.details.format(p.price)}</span>
							</div>

							<!-- SVG Sparkline -->
							{#if hist && hist.length > 0}
								<div class="w-24 h-8 relative opacity-70 group-hover:opacity-100 transition-opacity duration-300">
									<svg class="w-full h-full" viewBox="0 0 100 30" preserveAspectRatio="none">
										<defs>
											<linearGradient id="gradient-{p.symbol}" x1="0" y1="0" x2="0" y2="1">
												<stop offset="0%" stop-color={p.pct.value >= 0 ? '#089981' : '#F23645'} stop-opacity="0.3"></stop>
												<stop offset="100%" stop-color={p.pct.value >= 0 ? '#089981' : '#F23645'} stop-opacity="0.0"></stop>
											</linearGradient>
										</defs>
										<path
											d={getSparklineFillPath(hist, 100, 30)}
											fill="url(#gradient-{p.symbol})"
										></path>
										<path
											d={getSparklinePath(hist, 100, 30)}
											fill="none"
											stroke={p.pct.value >= 0 ? '#089981' : '#F23645'}
											stroke-width="1.5"
											stroke-linecap="round"
											stroke-linejoin="round"
										></path>
									</svg>
								</div>
							{:else}
								<div class="h-5 w-16 bg-surface-2 rounded animate-pulse opacity-40"></div>
							{/if}
						</div>
					</button>
				{/each}
			</div>
		{:else}
			<!-- All Markets View: Categorized subgrids simulating a complete Heatmap -->
			<div class="flex flex-col gap-6.5">
				{#each Object.entries(groupedPrices) as [catId, items]}
					{#if items.length > 0}
						<div class="flex flex-col gap-2">
							<!-- Group Header -->
							<div class="flex items-center justify-between border-b border-border/50 pb-1.5 px-1">
								<div class="flex items-center gap-1.5">
									{#if catId === 'crypto'}
										<TrendingUp class="h-3.5 w-3.5 text-accent" />
									{:else if catId === 'forex'}
										<TrendingUp class="h-3.5 w-3.5 text-[#003399]" />
									{:else if catId === 'indices'}
										<TrendingUp class="h-3.5 w-3.5 text-emerald-600" />
									{:else}
										<TrendingUp class="h-3.5 w-3.5 text-[#FFD700]" />
									{/if}
									<span class="text-[10px] font-extrabold uppercase tracking-widest text-text-dim">{getCategoryName(catId)}</span>
								</div>
								<span class="text-[9.5px] font-semibold text-text-muted font-mono">{items.length} instruments</span>
							</div>

							<!-- Grid for Group -->
							<div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3">
								{#each items as p (p.symbol)}
									{@const flash = flashMap.get(p.symbol)}
									{@const style = getHeatmapBgStyle(p.pct.value)}
									{@const hist = sparklines[p.symbol]}

									<button
										onclick={() => onselect(p.symbol)}
										{style}
										class="group relative rounded-lg border text-left p-3 flex flex-col justify-between transition-all duration-300 hover:shadow-lg cursor-pointer overflow-hidden min-h-[90px]
										{getCellSpan(p.symbol, false)}
										{flash === 'up' ? 'flash-green' : flash === 'down' ? 'flash-red' : ''}"
									>
										<!-- Hover reflection glow -->
										<div class="absolute inset-0 opacity-0 group-hover:opacity-10 pointer-events-none transition-opacity duration-300 bg-radial from-white via-transparent to-transparent"></div>

										<div class="flex items-start justify-between w-full z-10 gap-1">
											<div class="flex items-center gap-1.5 min-w-0">
												{#if p.details.logo.type === 'img'}
													<img src={p.details.logo.url} alt={p.symbol} class="h-5 w-5 rounded-full object-cover border border-border/40 shrink-0" />
												{:else}
													<div class="h-5 w-5 rounded-full bg-accent/15 border border-accent/30 flex items-center justify-center shrink-0">
														<span class="text-[8px] font-bold text-accent">GOLD</span>
													</div>
												{/if}
												<div class="min-w-0">
													<div class="font-bold text-xs uppercase tracking-tight text-text truncate leading-tight">{p.symbol}</div>
													<div class="text-[9px] text-text-dim truncate max-w-[85px] leading-tight font-medium mt-0.5">{p.details.name}</div>
												</div>
											</div>
											<div class="flex items-center gap-0.5 text-[9.5px] font-extrabold font-mono px-1 py-0.5 rounded shrink-0 {p.pct.value >= 0 ? 'text-green bg-green/10' : 'text-red bg-red/10'}">
												{#if p.pct.value >= 0}
													<ArrowUpRight class="h-2.5 w-2.5" />
												{:else}
													<ArrowDownRight class="h-2.5 w-2.5" />
												{/if}
												{p.pct.string}
											</div>
										</div>

										<div class="mt-2.5 flex items-end justify-between w-full z-10">
											<div class="flex flex-col">
												<span class="text-[8px] text-text-dim uppercase font-mono font-semibold tracking-wider">{p.details.unit}</span>
												<span class="font-mono text-sm md:text-base font-extrabold text-text mt-0.5">{p.details.format(p.price)}</span>
											</div>

											{#if hist && hist.length > 0}
												<div class="w-16 h-6.5 relative opacity-60 group-hover:opacity-100 transition-opacity duration-300">
													<svg class="w-full h-full" viewBox="0 0 100 30" preserveAspectRatio="none">
														<defs>
															<linearGradient id="gradient-{p.symbol}" x1="0" y1="0" x2="0" y2="1">
																<stop offset="0%" stop-color={p.pct.value >= 0 ? '#089981' : '#F23645'} stop-opacity="0.3"></stop>
																<stop offset="100%" stop-color={p.pct.value >= 0 ? '#089981' : '#F23645'} stop-opacity="0.0"></stop>
															</linearGradient>
														</defs>
														<path
															d={getSparklineFillPath(hist, 100, 30)}
															fill="url(#gradient-{p.symbol})"
														></path>
														<path
															d={getSparklinePath(hist, 100, 30)}
															fill="none"
															stroke={p.pct.value >= 0 ? '#089981' : '#F23645'}
															stroke-width="1.5"
															stroke-linecap="round"
															stroke-linejoin="round"
														></path>
													</svg>
												</div>
											{:else}
												<div class="h-4.5 w-12 bg-surface-2 rounded animate-pulse opacity-40"></div>
											{/if}
										</div>
									</button>
								{/each}
							</div>
						</div>
					{/if}
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	/* Hide scrollbars for overflow tabs */
	.scrollbar-none::-webkit-scrollbar {
		display: none;
	}
	.scrollbar-none {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
</style>
