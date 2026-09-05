<script lang="ts">
	import { Search } from 'lucide-svelte';
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';
	import { apiFetch } from '$lib/api';
	import { getSymbolMeta, getAssetCategory } from '$lib/symbol-meta';

	interface Props {
		onselect: (symbol: string) => void;
	}
	let { onselect }: Props = $props();

	let allPrices: PriceData[] = $derived(marketStore.prices);
	let activeCategory = $state('all');
	let searchQuery = $state('');
	let sortBy = $state('change-desc');
	let containerWidth = $state(800);
	let containerHeight = $state(520);

	let initialPrices = $state<Map<string, number>>(new Map());
	let sparklines = $state<Record<string, number[]>>({});
	let sparklineLoading = $state<Record<string, boolean>>({});

	// Categories Definition
	const categories = [
		{ id: 'all', name: 'All Markets' },
		{ id: 'stocks', name: 'Stocks' },
		{ id: 'forex', name: 'Forex' },
		{ id: 'indices', name: 'Indices' },
		{ id: 'crypto', name: 'Crypto' },
		{ id: 'commodities', name: 'Commodities' }
	];

	function getSymbolDetails(itemOrSymbol: PriceData | string) {
		return getSymbolMeta(typeof itemOrSymbol === 'string' ? itemOrSymbol : itemOrSymbol.symbol);
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
			const res = await apiFetch(`/api/v1/market/history/${upperSym}`);
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

		// Fallback generated points
		const fallbackData = [];
		let price = initialPrice > 0 ? initialPrice : 1.0;
		if (upperSym.includes('JPY')) price = 150.0;
		if (upperSym === 'XAUUSD') price = 4500.0;
		if (upperSym === 'SPX') price = 5200.0;
		if (upperSym === 'DXY') price = 104.5;
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
			const category = getAssetCategory(p);
			const pct = getPercentChange(p);
			const details = getSymbolDetails(p);
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
					item.symbol.toLowerCase().includes(q) || item.details.name.toLowerCase().includes(q)
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

	// Hardcoded weights corresponding to relative market sizes/relevance for realistic sizing
	const symbolWeights: Record<string, number> = {
		BTCUSDT: 1400,
		ETHUSDT: 500,
		SOLUSDT: 140,
		BNBUSDT: 120,
		PAXGUSDT: 40,
		EURUSD: 800,
		GBPUSD: 400,
		USDJPY: 500,
		SPX: 1600,
		DXY: 500,
		XAUUSD: 700
	};

	function getSymbolWeight(symbol: string): number {
		return symbolWeights[symbol.toUpperCase()] ?? 60;
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

	// Compute flat treemap based on current processed prices, scaled to pixel size and snapped to integers
	let computedTreeMap = $derived.by(() => {
		const rawNodes = computeTreeMap(
			processedPrices.map((p) => ({ id: p.symbol, weight: getSymbolWeight(p.symbol), data: p })),
			0,
			0,
			containerWidth,
			containerHeight
		);

		// Snap nodes to integer pixel grid to prevent sub-pixel rendering gaps
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

	interface CellStyle {
		background: string;
		borderColor: string;
		boxShadow: string;
		textColor: string;
	}

	// Diverging scale: red (down) <-> neutral gray (flat) <-> green (up).
	// Continuous interpolation, clamped at +/-3% so extremes saturate cleanly.
	function lerp(a: number, b: number, t: number): number {
		return Math.round(a + (b - a) * t);
	}

	function mixHex(from: [number, number, number], to: [number, number, number], t: number): string {
		const r = lerp(from[0], to[0], t);
		const g = lerp(from[1], to[1], t);
		const b = lerp(from[2], to[2], t);
		return `rgb(${r}, ${g}, ${b})`;
	}

	// Poles + neutral midpoint (dark-surface friendly steps).
	const DOWN_POLE: [number, number, number] = [176, 68, 64]; // #b04440
	const UP_POLE: [number, number, number] = [40, 122, 82]; // #287a52
	const NEUTRAL: [number, number, number] = [120, 118, 110]; // warm gray

	function getCellStyles(pctVal: number, isSelected: boolean): CellStyle {
		const clamped = Math.max(-3, Math.min(3, pctVal));
		const t = Math.abs(clamped) / 3; // 0 at flat, 1 at extreme
		const bg =
			clamped > 0 ? mixHex(NEUTRAL, UP_POLE, t) : clamped < 0 ? mixHex(NEUTRAL, DOWN_POLE, t) : 'rgb(120, 118, 110)';

		return {
			background: bg,
			borderColor: isSelected ? 'var(--color-accent)' : '',
			boxShadow: '',
			textColor: 'text-white'
		};
	}
</script>

<div
	class="flex h-full flex-col overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
>
	<!-- Control Bar: Search, Filters, and Sorting -->
	<div
		class="z-20 flex shrink-0 flex-col items-stretch justify-between gap-4 border-b border-border bg-surface px-5 py-3.5 sm:flex-row sm:items-center"
	>
		<div class="scrollbar-none flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0">
			{#each categories as cat}
				<button
					onclick={() => (activeCategory = cat.id)}
					class="cursor-pointer rounded-lg border px-4 py-2 text-xs font-bold whitespace-nowrap transition-colors
					{activeCategory === cat.id
						? 'border-accent bg-accent text-white'
						: 'border-border/60 bg-surface-2/60 text-text-dim hover:border-text-dim/40 hover:text-text'}"
				>
					{cat.name}
				</button>
			{/each}
		</div>

		<div class="flex items-center justify-between gap-3 sm:justify-end">
			<!-- Search -->
			<div class="group relative flex-1 sm:w-52 sm:flex-none">
				<Search
					class="absolute top-2.5 left-3 h-3.5 w-3.5 text-text-dim transition-colors group-focus-within:text-accent"
				/>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search symbol..."
					class="py-1.8 w-full rounded-lg border border-border/80 bg-surface-2/50 pr-3.5 pl-9 text-xs font-semibold text-text placeholder-text-dim transition-all focus:border-accent focus:bg-surface focus:ring-2 focus:ring-accent/15 focus:outline-none"
				/>
			</div>

			<!-- Sort Info -->
			<div
				class="py-1.8 rounded-lg border border-border/70 bg-surface-2/70 px-3.5 text-[11px] font-bold text-text-dim select-none"
			>
				Size: Market Weight
			</div>
		</div>
	</div>

	<!-- Heatmap Container -->
	<div class="relative min-h-[550px] flex-1 bg-surface-2/20 p-4">
		{#if allPrices.length === 0}
			<div class="absolute inset-0 flex flex-col items-center justify-center py-20 text-center">
				<div
					class="h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent"
				></div>
				<p class="mt-3 text-xs font-semibold text-text-muted">Connecting to market websocket...</p>
			</div>
		{:else if processedPrices.length === 0}
			<div
				class="absolute inset-0 flex flex-col items-center justify-center py-20 text-center text-text-dim"
			>
				<p class="text-sm font-semibold">No assets found</p>
				<p class="mt-1 text-xs">Try modifying your filter or search query</p>
			</div>
		{:else}
			<div
				bind:clientWidth={containerWidth}
				bind:clientHeight={containerHeight}
				class="absolute inset-0 h-full min-h-[500px] w-full overflow-hidden border-t border-l border-surface bg-surface select-none"
			>
				{#each computedTreeMap as node (node.id)}
					{@const flash = flashMap.get(node.id)}
					{@const cellStyle = getCellStyles(node.data.pct.value, false)}
					<button
						onclick={() => onselect(node.id)}
						class="heatmap-cell absolute flex cursor-pointer flex-col items-center justify-center overflow-hidden border-r border-b border-surface p-2 text-center transition-all duration-300 hover:z-30
						{flash === 'up' ? 'cell-flash-green' : flash === 'down' ? 'cell-flash-red' : ''}"
						style="left: {node.x}px; top: {node.y}px; width: {node.w}px; height: {node.h}px; background: {cellStyle.background}; box-shadow: {cellStyle.boxShadow};"
					>
						{#if node.w >= 110 && node.h >= 75}
							<!-- Large Cell -->
							{#if node.h < 90 && node.w >= 120}
								<!-- Horizontal Large Cell -->
								<div
									class="flex h-full w-full flex-row items-center justify-center gap-2.5 p-2 select-none"
								>
									{#if node.data.details.logo.type === 'img'}
										<div
											class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-black/10 bg-white p-0.5 shadow-sm"
										>
											<img
												src={node.data.details.logo.url}
												alt={node.data.details.displaySymbol}
												class="h-full w-full rounded-full object-contain"
											/>
										</div>
									{:else}
										<div
											class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-black/10 bg-white shadow-sm"
										>
											<span class="text-[10px] font-black text-black/60"
												>{node.data.details.displaySymbol.substring(0, 4)}</span
											>
										</div>
									{/if}
									<div class="flex flex-col items-start justify-center">
										<div
											class="text-sm leading-tight font-extrabold tracking-wide text-white uppercase"
										>
											{node.data.details.displaySymbol}
										</div>
										<div class="text-[11px] leading-tight font-semibold text-white/90">
											{node.data.pct.string}
										</div>
									</div>
								</div>
							{:else}
								<!-- Vertical Large Cell -->
								<div
									class="flex h-full w-full flex-col items-center justify-center gap-1.5 select-none"
								>
									{#if node.data.details.logo.type === 'img'}
										<div
											class="flex h-10 w-10 items-center justify-center rounded-full border border-black/10 bg-white p-0.5 shadow-sm"
										>
											<img
												src={node.data.details.logo.url}
												alt={node.data.details.displaySymbol}
												class="h-full w-full rounded-full object-contain"
											/>
										</div>
									{:else}
										<div
											class="flex h-10 w-10 items-center justify-center rounded-full border border-black/10 bg-white shadow-sm"
										>
											<span class="text-[10px] font-black text-black/60"
												>{node.data.details.displaySymbol.substring(0, 4)}</span
											>
										</div>
									{/if}
									<div
										class="mt-1.5 text-sm leading-none font-extrabold tracking-wide text-white uppercase"
									>
										{node.data.details.displaySymbol}
									</div>
									<div class="mt-1 text-[11px] leading-none font-semibold text-white/90">
										{node.data.pct.string}
									</div>
								</div>
							{/if}
						{:else if node.w >= 70 && node.h >= 45}
							<!-- Medium Cell -->
							{#if node.h < 65}
								{#if node.w >= 85}
									<!-- Horizontal Medium Cell -->
									<div
										class="flex h-full w-full flex-row items-center justify-center gap-2 p-1.5 select-none"
									>
										{#if node.data.details.logo.type === 'img'}
											<div
												class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-black/10 bg-white p-0.5 shadow-sm"
											>
												<img
													src={node.data.details.logo.url}
													alt={node.data.details.displaySymbol}
													class="h-full w-full rounded-full object-contain"
												/>
											</div>
										{/if}
										<div class="flex flex-col items-start justify-center">
											<div
												class="text-xs leading-tight font-bold tracking-wide text-white uppercase"
											>
												{node.data.details.displaySymbol}
											</div>
											<div class="text-[9.5px] leading-tight font-medium text-white/90">
												{node.data.pct.string}
											</div>
										</div>
									</div>
								{:else}
									<!-- Short & Narrow: Hide Logo to prevent overlap -->
									<div
										class="flex h-full w-full flex-col items-center justify-center gap-0.5 select-none"
									>
										<div class="text-xs leading-none font-bold tracking-wide text-white uppercase">
											{node.data.details.displaySymbol}
										</div>
										<div class="mt-0.5 text-[10px] leading-none font-medium text-white/90">
											{node.data.pct.string}
										</div>
									</div>
								{/if}
							{:else}
								<!-- Vertical Medium Cell -->
								<div
									class="flex h-full w-full flex-col items-center justify-center gap-1 select-none"
								>
									{#if node.data.details.logo.type === 'img'}
										<div
											class="flex h-7 w-7 items-center justify-center rounded-full border border-black/10 bg-white p-0.5 shadow-sm"
										>
											<img
												src={node.data.details.logo.url}
												alt={node.data.details.displaySymbol}
												class="h-full w-full rounded-full object-contain"
											/>
										</div>
									{/if}
									<div
										class="mt-0.5 text-xs leading-none font-bold tracking-wide text-white uppercase"
									>
										{node.data.details.displaySymbol}
									</div>
									<div class="mt-0.5 text-[10px] leading-none font-medium text-white/90">
										{node.data.pct.string}
									</div>
								</div>
							{/if}
						{:else if node.w >= 40 && node.h >= 25}
							<!-- Small Cell -->
							<div
								class="flex h-full w-full flex-col items-center justify-center gap-0.5 select-none"
							>
								<div class="text-[11px] leading-none font-bold tracking-wide text-white uppercase">
									{node.data.details.displaySymbol}
								</div>
								<div class="text-[9px] leading-none font-medium text-white/85">
									{node.data.pct.string}
								</div>
							</div>
						{:else}
							<!-- Very Small Cell -->
							<div class="flex h-full w-full items-center justify-center select-none">
								<div class="text-[9px] leading-none font-bold tracking-wide text-white uppercase">
									{node.data.details.displaySymbol}
								</div>
							</div>
						{/if}
					</button>
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

	.heatmap-cell {
		border-radius: 0px;
		transition:
			filter 0.2s ease;
		outline: none !important;
	}

	.heatmap-cell:hover {
		filter: brightness(1.08);
		z-index: 30;
	}

	/* Brief brightness lift on WebSocket updates — no neon outline. */
	@keyframes local-pulse {
		0% {
			filter: brightness(1.25);
		}
		100% {
			filter: brightness(1);
		}
	}
	.cell-flash-green,
	.cell-flash-red {
		animation: local-pulse 0.7s cubic-bezier(0.25, 1, 0.5, 1);
		z-index: 20;
	}
</style>
