<script lang="ts">
	import { Search, Maximize2, Minimize2, BarChart2, Info } from 'lucide-svelte';
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';
	import { apiFetch } from '$lib/api';
	import { getSymbolMeta, getAssetCategory } from '$lib/symbol-meta';
	import { US_EQUITIES, IDX_EQUITIES, OTHER_MARKETS, getCompanyInfo } from '$lib/market-companies';

	interface Props {
		onselect: (symbol: string) => void;
	}
	let { onselect }: Props = $props();

	let allPrices: PriceData[] = $derived(marketStore.prices);
	let activeCategory = $state('sp500'); // Default to S&P 500 / US Stocks to match Finviz/TradingView
	let searchQuery = $state('');
	let sizingMode = $state<'cap' | 'equal'>('cap');
	let isFullscreen = $state(false);

	let containerWidth = $state(900);
	let containerHeight = $state(620);

	let initialPrices = $state<Map<string, number>>(new Map());
	let sparklines = $state<Record<string, number[]>>({});
	let sparklineLoading = $state<Record<string, boolean>>({});

	// Hover tooltip state
	let hoveredNode = $state<any>(null);
	let tooltipX = $state(0);
	let tooltipY = $state(0);

	// Categories Definition
	const categories = [
		{ id: 'sp500', name: 'S&P 500 / US' },
		{ id: 'all', name: 'All Markets' },
		{ id: 'crypto', name: 'Crypto' },
		{ id: 'commodities', name: 'Commodities' },
		{ id: 'forex', name: 'Forex' },
		{ id: 'indices', name: 'Indices' },
		{ id: 'idx', name: 'Indonesia (IDX)' }
	];

	// Realistic baseline reference percentages for US stocks when no history is loaded yet
	const DEFAULT_US_BENCHMARK_PCT: Record<string, number> = {
		AAPL: -0.93,
		MSFT: -1.45,
		GOOGL: -1.06,
		AMZN: -0.62,
		NVDA: -1.82,
		META: -1.15,
		TSLA: -2.30,
		BRKB: -0.76,
		LLY: -1.75,
		AVGO: -1.42,
		JPM: -0.73,
		WMT: -0.67,
		V: -0.87,
		UNH: -1.25,
		XOM: -0.95,
		MA: -1.04,
		COST: -0.88,
		HD: -1.35,
		JNJ: -2.06,
		ABBV: -0.85,
		BAC: -1.12,
		CRM: -4.18,
		NFLX: -2.65,
		KO: -0.45,
		CVX: -0.82,
		AMD: -2.75,
		PEP: -0.52,
		ADBE: -3.10,
		ORCL: -1.20,
		MCD: -0.68,
		CSCO: -1.15,
		NOW: -4.90,
		QCOM: -1.85,
		IBM: -0.92,
		DIS: -1.40,
		TXN: -1.65,
		AMGN: -7.08,
		INTC: -3.20,
		PLTR: -0.90,
		NKE: -1.75,
		WFC: -0.95,
		GS: -1.05,
		MS: -1.15,
		C: -1.30,
		BLK: -0.85,
		AXP: -0.78,
		MRK: -1.10,
		PFE: -1.45,
		ASML: -2.40,
		TSM: -1.85,
		AMAT: -2.90,
		LRCX: -3.15,
		MU: -2.50,
		PANW: -1.95,
		OXY: -1.10,
		SLB: -1.40,
		COP: -0.85
	};

	function getSymbolDetails(itemOrSymbol: PriceData | string) {
		return getSymbolMeta(typeof itemOrSymbol === 'string' ? itemOrSymbol : itemOrSymbol.symbol);
	}

	// Capture initial prices for performance tracking
	$effect(() => {
		for (const p of allPrices) {
			if (!initialPrices.has(p.symbol) && p.price > 0) {
				// If we have a benchmark pct, set baseline accordingly so pct matches immediately
				const symUpper = p.symbol.toUpperCase();
				if (DEFAULT_US_BENCHMARK_PCT[symUpper] !== undefined) {
					const targetPct = DEFAULT_US_BENCHMARK_PCT[symUpper];
					const base = p.price / (1 + targetPct / 100);
					initialPrices.set(p.symbol, base);
				} else {
					initialPrices.set(p.symbol, p.price);
				}
			}
		}
	});

	// Percent Change helper
	function getPercentChange(p: PriceData): { value: number; string: string } {
		const base = initialPrices.get(p.symbol);
		if (!base || base === 0) {
			const symUpper = p.symbol.toUpperCase();
			if (DEFAULT_US_BENCHMARK_PCT[symUpper] !== undefined) {
				const pct = DEFAULT_US_BENCHMARK_PCT[symUpper];
				const sign = pct >= 0 ? '+' : '';
				return { value: pct, string: `${sign}${pct.toFixed(2)}%` };
			}
			return { value: 0, string: '0.00%' };
		}
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
			const res = await apiFetch(`/api/v1/market/history/${upperSym}`);
			if (res.ok) {
				const data = await res.json();
				const rows = Array.isArray(data)
					? data
					: data && typeof data === 'object' && 'items' in data && Array.isArray((data as any).items)
						? (data as any).items
						: [];
				if (Array.isArray(rows) && rows.length > 0) {
					const sliceData = rows.slice(-limit);
					if (sliceData.length > 0) {
						// Oldest candle in the slice is our session base
						const firstVal = Number(sliceData[0].close ?? sliceData[0].value ?? 0);
						if (firstVal > 0) {
							initialPrices.set(upperSym, firstVal);
						}
						return sliceData.map((item: any) => Number(item.close ?? item.value ?? 0));
					}
				}
			}
		} catch (e) {
			// Fallback silently
		}

		// Fallback generated points for visual sparkline
		const fallbackData = [];
		let price = initialPrice > 0 ? initialPrice : 100.0;
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

	// Filter and Sort Processing
	let processedPrices = $derived.by(() => {
		// Pool of all available prices in store
		const priceBySym = new Map<string, PriceData>();
		for (const p of allPrices) {
			priceBySym.set(p.symbol.toUpperCase(), p);
		}

		let list: {
			symbol: string;
			price: number;
			category: string;
			pct: { value: number; string: string };
			details: ReturnType<typeof getSymbolMeta>;
			data: PriceData;
		}[] = [];

		if (activeCategory === 'sp500') {
			// Curated list of S&P 500 Equities (matches user's screenshot)
			for (const [sym, info] of Object.entries(US_EQUITIES)) {
				const existing = priceBySym.get(sym);
				const price = existing ? existing.price : 100.0;
				const details = getSymbolMeta(sym);
				const pData: PriceData = existing || {
					symbol: sym,
					price,
					bid: null,
					ask: null,
					volume: null,
					source: 'synthetic',
					asset_type: 'stock',
					received_at: null,
					direction: 'none',
					prev_price: price,
					updated_at: Date.now()
				};
				const pct = getPercentChange(pData);
				list.push({
					symbol: sym,
					price,
					category: 'sp500',
					pct,
					details,
					data: pData
				});
			}
		} else if (activeCategory === 'idx') {
			// Indonesian Equities
			for (const [sym, info] of Object.entries(IDX_EQUITIES)) {
				const existing = priceBySym.get(sym);
				const price = existing ? existing.price : 1000.0;
				const details = getSymbolMeta(sym);
				const pData: PriceData = existing || {
					symbol: sym,
					price,
					bid: null,
					ask: null,
					volume: null,
					source: 'synthetic',
					asset_type: 'stock',
					received_at: null,
					direction: 'none',
					prev_price: price,
					updated_at: Date.now()
				};
				const pct = getPercentChange(pData);
				list.push({
					symbol: sym,
					price,
					category: 'idx',
					pct,
					details,
					data: pData
				});
			}
		} else {
			// All Markets or other tabs
			for (const p of allPrices) {
				const category = getAssetCategory(p);
				if (activeCategory !== 'all' && category !== activeCategory) {
					continue;
				}
				const pct = getPercentChange(p);
				const details = getSymbolDetails(p);
				list.push({
					symbol: p.symbol,
					price: p.price,
					category,
					pct,
					details,
					data: p
				});
			}
		}

		// Apply Search filter
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase().trim();
			list = list.filter(
				(item) =>
					item.symbol.toLowerCase().includes(q) ||
					item.details.name.toLowerCase().includes(q) ||
					(item.details.sector && item.details.sector.toLowerCase().includes(q))
			);
		}

		return list;
	});

	function getNodeWeight(item: (typeof processedPrices)[0]): number {
		if (sizingMode === 'equal') return 100;
		if (item.details.marketCap && item.details.marketCap > 0) {
			return item.details.marketCap;
		}
		const comp = getCompanyInfo(item.symbol);
		if (comp && comp.marketCapBillion > 0) {
			return comp.marketCapBillion;
		}
		return 50;
	}

	interface TreeMapNode {
		id: string;
		weight: number;
		x: number;
		y: number;
		w: number;
		h: number;
		data: any;
	}

	// Squarified Treemap Algorithm implementation
	function worst(row: { weight: number }[], w: number, h: number, totalWeight: number): number {
		if (row.length === 0) return Infinity;
		const rowWeight = row.reduce((sum, n) => sum + n.weight, 0);
		const s = Math.min(w, h);
		if (s <= 0 || rowWeight <= 0) return Infinity;

		const scale = (w * h) / totalWeight;
		const minW = Math.min(...row.map((n) => n.weight)) * scale;
		const maxW = Math.max(...row.map((n) => n.weight)) * scale;
		const sumW = rowWeight * scale;

		return Math.max((s * s * maxW) / (sumW * sumW), (sumW * sumW) / (s * s * minW));
	}

	function layoutRow(
		row: TreeMapNode[],
		w: number,
		h: number,
		x: number,
		y: number,
		totalWeight: number,
		result: TreeMapNode[]
	) {
		const rowWeight = row.reduce((sum, n) => sum + n.weight, 0);
		if (totalWeight <= 0 || rowWeight <= 0) return;

		const scale = (w * h) / totalWeight;
		const thickness = (rowWeight * scale) / Math.min(w, h);

		let offset = 0;
		for (const node of row) {
			const nodeArea = node.weight * scale;
			const nodeLength = nodeArea / thickness;

			if (w >= h) {
				node.x = x;
				node.y = y + offset;
				node.w = thickness;
				node.h = nodeLength;
			} else {
				node.x = x + offset;
				node.y = y;
				node.w = nodeLength;
				node.h = thickness;
			}
			offset += nodeLength;
			result.push(node);
		}
	}

	function squarify(
		remaining: TreeMapNode[],
		row: TreeMapNode[],
		w: number,
		h: number,
		x: number,
		y: number,
		totalWeight: number,
		result: TreeMapNode[]
	) {
		if (remaining.length === 0) {
			if (row.length > 0) {
				layoutRow(row, w, h, x, y, totalWeight, result);
			}
			return;
		}

		const nextNode = remaining[0];
		const newRow = [...row, nextNode];

		const currentWorst = worst(row, w, h, totalWeight);
		const newWorst = worst(newRow, w, h, totalWeight);

		if (row.length === 0 || newWorst <= currentWorst) {
			squarify(remaining.slice(1), newRow, w, h, x, y, totalWeight, result);
		} else {
			const rowWeight = row.reduce((sum, n) => sum + n.weight, 0);
			if (totalWeight <= 0) return;
			const ratio = rowWeight / totalWeight;

			let newX = x;
			let newY = y;
			let newW = w;
			let newH = h;

			if (w >= h) {
				newX += w * ratio;
				newW -= w * ratio;
			} else {
				newY += h * ratio;
				newH -= h * ratio;
			}

			layoutRow(row, w, h, x, y, totalWeight, result);
			squarify(remaining, [], newW, newH, newX, newY, totalWeight - rowWeight, result);
		}
	}

	function computeTreeMap(
		nodes: { id: string; weight: number; data?: any }[],
		x: number,
		y: number,
		w: number,
		h: number
	): TreeMapNode[] {
		if (nodes.length === 0) return [];
		if (w <= 0 || h <= 0) return [];

		const sorted = nodes
			.map((n) => ({
				id: n.id,
				weight: Math.max(n.weight, 1),
				x: 0,
				y: 0,
				w: 0,
				h: 0,
				data: n.data
			}))
			.sort((a, b) => b.weight - a.weight);

		const totalWeight = sorted.reduce((sum, n) => sum + n.weight, 0);
		const result: TreeMapNode[] = [];
		squarify(sorted, [], w, h, x, y, totalWeight, result);
		return result;
	}

	// Compute flat treemap with 1px black border gaps
	let computedTreeMap = $derived.by(() => {
		const rawNodes = computeTreeMap(
			processedPrices.map((p) => ({
				id: p.symbol,
				weight: getNodeWeight(p),
				data: p
			})),
			0,
			0,
			containerWidth,
			containerHeight
		);

		return rawNodes.map((node) => {
			const x = Math.round(node.x);
			const y = Math.round(node.y);
			const w = Math.round(node.x + node.w) - x;
			const h = Math.round(node.y + node.h) - y;
			return {
				...node,
				x,
				y,
				w,
				h
			};
		});
	});

	// Authentic Finviz / TradingView financial color scale
	function getHeatmapBgColor(pct: number): string {
		if (isNaN(pct) || pct === 0) {
			return '#1e222d'; // Neutral dark slate
		}

		if (pct < 0) {
			const val = Math.min(Math.abs(pct), 7.0);
			if (val >= 6.0) return '#ef4444'; // Bright vivid red (< -6%)
			if (val >= 4.5) return '#dc2626';
			if (val >= 3.5) return '#c5221f';
			if (val >= 2.5) return '#b91c1c';
			if (val >= 1.8) return '#991b1b';
			if (val >= 1.2) return '#881337';
			if (val >= 0.7) return '#701a20';
			if (val >= 0.3) return '#581c1c';
			return '#451212'; // Very mild loss
		} else {
			const val = Math.min(pct, 7.0);
			if (val >= 6.0) return '#22c55e'; // Bright neon green (> +6%)
			if (val >= 4.5) return '#16a34a';
			if (val >= 3.5) return '#15803d';
			if (val >= 2.5) return '#166534';
			if (val >= 1.8) return '#14532d';
			if (val >= 1.2) return '#064e3b';
			if (val >= 0.7) return '#064433';
			if (val >= 0.3) return '#043528';
			return '#022c22'; // Very mild gain
		}
	}

	function handleCellMouseEnter(e: MouseEvent, node: any) {
		hoveredNode = node;
		updateTooltipPosition(e);
	}

	function handleCellMouseMove(e: MouseEvent) {
		if (hoveredNode) {
			updateTooltipPosition(e);
		}
	}

	function handleCellMouseLeave() {
		hoveredNode = null;
	}

	function updateTooltipPosition(e: MouseEvent) {
		tooltipX = e.clientX + 14;
		tooltipY = e.clientY + 14;
	}
</script>

<div
	class="flex flex-col overflow-hidden rounded-xl border border-[#27272a] bg-[#0c0d12] shadow-2xl transition-all duration-300
	{isFullscreen ? 'fixed inset-0 z-50 rounded-none' : 'h-[640px] w-full'}"
>
	<!-- Top Control Bar -->
	<div
		class="z-20 flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-[#27272a] bg-[#12131a] px-4 py-2.5"
	>
		<!-- Category Tabs -->
		<div class="scrollbar-none flex items-center gap-1.5 overflow-x-auto pb-0.5">
			{#each categories as cat}
				<button
					onclick={() => (activeCategory = cat.id)}
					class="cursor-pointer rounded-md px-3 py-1.5 text-xs font-bold whitespace-nowrap transition-all
					{activeCategory === cat.id
						? 'bg-[#2962ff] text-white shadow-sm'
						: 'bg-[#1a1c26] text-[#94a3b8] hover:bg-[#262837] hover:text-white'}"
				>
					{cat.name}
				</button>
			{/each}
		</div>

		<!-- Right Controls: Search, Size Mode, Fullscreen -->
		<div class="flex items-center gap-2.5">
			<!-- Search -->
			<div class="group relative w-44 sm:w-52">
				<Search
					class="absolute top-2 left-2.5 h-3.5 w-3.5 text-[#64748b] transition-colors group-focus-within:text-[#2962ff]"
				/>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search ticker, sector..."
					class="w-full rounded-md border border-[#27272a] bg-[#181924] py-1.5 pr-3 pl-8 text-xs font-semibold text-white placeholder-[#64748b] transition-all focus:border-[#2962ff] focus:outline-none"
				/>
			</div>

			<!-- Sizing Toggle -->
			<button
				onclick={() => (sizingMode = sizingMode === 'cap' ? 'equal' : 'cap')}
				class="hidden cursor-pointer items-center gap-1.5 rounded-md border border-[#27272a] bg-[#181924] px-3 py-1.5 text-xs font-semibold text-[#94a3b8] transition-colors hover:text-white sm:flex"
				title="Toggle size mode"
			>
				<BarChart2 class="h-3.5 w-3.5 text-[#2962ff]" />
				<span>{sizingMode === 'cap' ? 'Market Cap' : 'Equal Size'}</span>
			</button>

			<!-- Fullscreen Toggle -->
			<button
				onclick={() => (isFullscreen = !isFullscreen)}
				class="cursor-pointer rounded-md border border-[#27272a] bg-[#181924] p-1.5 text-[#94a3b8] transition-colors hover:text-white"
				title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
			>
				{#if isFullscreen}
					<Minimize2 class="h-3.5 w-3.5" />
				{:else}
					<Maximize2 class="h-3.5 w-3.5" />
				{/if}
			</button>
		</div>
	</div>

	<!-- Treemap Canvas Area -->
	<div
		class="relative flex-1 overflow-hidden bg-black select-none"
		bind:clientWidth={containerWidth}
		bind:clientHeight={containerHeight}
	>
		{#if processedPrices.length === 0}
			<div class="absolute inset-0 flex flex-col items-center justify-center p-6 text-center text-[#64748b]">
				<p class="text-sm font-semibold">No assets found</p>
				<p class="mt-1 text-xs">Try selecting a different market or clearing your search.</p>
			</div>
		{:else}
			{#each computedTreeMap as node (node.id)}
				{@const pctVal = node.data.pct.value}
				{@const bgColor = getHeatmapBgColor(pctVal)}
				{@const flash = flashMap.get(node.id)}
				{@const displaySym = node.data.details.displaySymbol || node.id}
				{@const logoSvg = node.data.details.svgLogo}
				{@const localLogo = node.data.details.logo?.url}

				<button
					type="button"
					onclick={() => onselect(node.id)}
					onmouseenter={(e) => handleCellMouseEnter(e, node)}
					onmousemove={handleCellMouseMove}
					onmouseleave={handleCellMouseLeave}
					class="heatmap-tile absolute flex cursor-pointer flex-col items-center justify-center overflow-hidden border border-black text-center transition-all duration-200 hover:z-30 hover:brightness-125
					{flash === 'up' ? 'cell-flash-green' : flash === 'down' ? 'cell-flash-red' : ''}"
					style="left: {node.x}px; top: {node.y}px; width: {node.w}px; height: {node.h}px; background-color: {bgColor};"
				>
					{#if node.w >= 110 && node.h >= 75}
						<!-- MEGA TILE (e.g. AAPL, MSFT, GOOGL, AMZN, NVDA, META) -->
						<div class="flex h-full w-full flex-col items-center justify-center p-2">
							<!-- Logo badge -->
							{#if node.h >= 95}
								<div class="mb-1.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-black/40 p-1.5 shadow-md">
									{#if logoSvg}
										{@html logoSvg}
									{:else if localLogo}
										<img src={localLogo} alt={displaySym} class="h-full w-full rounded-full object-contain" />
									{:else}
										<span class="text-xs font-black text-white/80">{displaySym.slice(0, 3)}</span>
									{/if}
								</div>
							{/if}

							<!-- Ticker -->
							<div class="text-base leading-tight font-black tracking-tight text-white drop-shadow-sm sm:text-lg">
								{displaySym}
							</div>

							<!-- Percent Change -->
							<div class="mt-0.5 text-xs font-bold text-white/95 sm:text-sm">
								{node.data.pct.string}
							</div>
						</div>
					{:else if node.w >= 65 && node.h >= 45}
						<!-- MEDIUM TILE (e.g. LLY, JPM, WMT, V, MA, JNJ, ABBV, PLTR, AMGN, NOW, CRM, NFLX) -->
						<div class="flex h-full w-full flex-col items-center justify-center p-1">
							{#if node.h >= 68 && node.w >= 80}
								<div class="mb-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-black/35 p-1 shadow-sm">
									{#if logoSvg}
										{@html logoSvg}
									{:else if localLogo}
										<img src={localLogo} alt={displaySym} class="h-full w-full rounded-full object-contain" />
									{:else}
										<span class="text-[9px] font-black text-white/80">{displaySym.slice(0, 2)}</span>
									{/if}
								</div>
							{/if}
							<div class="text-xs leading-tight font-black tracking-tight text-white drop-shadow-sm sm:text-sm">
								{displaySym}
							</div>
							<div class="text-[11px] leading-tight font-bold text-white/90">
								{node.data.pct.string}
							</div>
						</div>
					{:else if node.w >= 36 && node.h >= 22}
						<!-- SMALL TILE -->
						<div class="flex h-full w-full flex-col items-center justify-center p-0.5">
							<div class="text-[10.5px] leading-tight font-black tracking-tight text-white">
								{displaySym}
							</div>
							{#if node.h >= 32}
								<div class="text-[9px] leading-tight font-bold text-white/85">
									{node.data.pct.string}
								</div>
							{/if}
						</div>
					{:else}
						<!-- MICRO TILE -->
						<div class="flex h-full w-full items-center justify-center">
							{#if node.w >= 20 && node.h >= 14}
								<span class="text-[8px] leading-none font-bold text-white/80">{displaySym.slice(0, 3)}</span>
							{/if}
						</div>
					{/if}
				</button>
			{/each}
		{/if}
	</div>

	<!-- Bottom Legend Scale -->
	<div class="z-20 flex shrink-0 items-center justify-between border-t border-[#27272a] bg-[#12131a] px-4 py-2 text-[11px] text-[#94a3b8]">
		<div class="flex items-center gap-1.5 font-medium">
			<Info class="h-3.5 w-3.5 text-[#2962ff]" />
			<span>Showing {processedPrices.length} assets • Click tile to open detailed chart</span>
		</div>

		<!-- Gradient Legend -->
		<div class="flex items-center gap-1.5 font-mono text-[10px]">
			<span>-4%</span>
			<div class="flex h-2.5 items-center gap-0.5 overflow-hidden rounded-sm border border-black/60">
				<div class="h-full w-3.5 bg-[#ef4444]"></div>
				<div class="h-full w-3.5 bg-[#b91c1c]"></div>
				<div class="h-full w-3.5 bg-[#701a20]"></div>
				<div class="h-full w-3.5 bg-[#1e222d]"></div>
				<div class="h-full w-3.5 bg-[#064e3b]"></div>
				<div class="h-full w-3.5 bg-[#16a34a]"></div>
				<div class="h-full w-3.5 bg-[#22c55e]"></div>
			</div>
			<span>+4%</span>
		</div>
	</div>
</div>

<!-- Floating Hover Tooltip -->
{#if hoveredNode}
	{@const d = hoveredNode.data}
	<div
		class="pointer-events-none fixed z-50 w-56 rounded-lg border border-[#3f3f46] bg-[#18181b]/95 p-3 text-white shadow-2xl backdrop-blur-md transition-opacity duration-150"
		style="left: {tooltipX}px; top: {tooltipY}px;"
	>
		<div class="flex items-center justify-between border-b border-[#27272a] pb-2">
			<div>
				<div class="text-sm font-black tracking-tight">{d.details.displaySymbol}</div>
				<div class="line-clamp-1 text-[11px] text-[#a1a1aa]">{d.details.name}</div>
			</div>
			<span
				class="rounded px-1.5 py-0.5 text-[10px] font-bold"
				style="background-color: {getHeatmapBgColor(d.pct.value)}; color: #ffffff;"
			>
				{d.pct.string}
			</span>
		</div>

		<div class="mt-2 space-y-1 text-xs">
			<div class="flex justify-between">
				<span class="text-[#71717a]">Price:</span>
				<span class="font-mono font-bold">${Number(d.price).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
			</div>
			{#if d.details.sector}
				<div class="flex justify-between">
					<span class="text-[#71717a]">Sector:</span>
					<span class="text-[#d4d4d8]">{d.details.sector}</span>
				</div>
			{/if}
			{#if d.details.marketCap}
				<div class="flex justify-between">
					<span class="text-[#71717a]">Market Cap:</span>
					<span class="font-mono text-[#d4d4d8]">${d.details.marketCap}B</span>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	/* Hide scrollbars for overflow tabs */
	.scrollbar-none::-webkit-scrollbar {
		display: none;
	}
	.scrollbar-none {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}

	.heatmap-tile {
		border-radius: 0px;
		outline: none !important;
		user-select: none;
	}

	@keyframes local-pulse {
		0% {
			filter: brightness(1.3);
		}
		100% {
			filter: brightness(1);
		}
	}
	.cell-flash-green,
	.cell-flash-red {
		animation: local-pulse 0.6s cubic-bezier(0.25, 1, 0.5, 1);
		z-index: 25;
	}
</style>
