<script lang="ts">
	import { SvelteMap, SvelteSet } from 'svelte/reactivity';
	import { Search, Maximize2, Minimize2, BarChart2, Info } from 'lucide-svelte';
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';
	import { apiFetch } from '$lib/api';
	import { getSymbolMeta, getAssetCategory } from '$lib/symbol-meta';
	import { US_EQUITIES, IDX_EQUITIES, getCompanyInfo } from '$lib/market-companies';

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

	let initialPrices = $state<Record<string, number>>({});
	const referenceRequests = new SvelteSet<string>();

	// Hover tooltip state
	let hoveredNode = $state<TreeMapNode | null>(null);
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

	interface HeatmapItem {
		symbol: string;
		price: number;
		category: string;
		pct: { value: number | null; string: string };
		details: ReturnType<typeof getSymbolMeta>;
		data: PriceData;
	}

	interface TreeMapNode {
		id: string;
		weight: number;
		x: number;
		y: number;
		w: number;
		h: number;
		data: HeatmapItem;
	}

	function getSymbolDetails(itemOrSymbol: PriceData | string) {
		return getSymbolMeta(typeof itemOrSymbol === 'string' ? itemOrSymbol : itemOrSymbol.symbol);
	}

	// Percent change is measured against the previous completed daily candle.
	function getPercentChange(p: PriceData): { value: number | null; string: string } {
		const base = initialPrices[p.symbol.toUpperCase()];
		if (!base || base <= 0 || p.price <= 0) return { value: null, string: '—' };
		const pct = ((p.price - base) / base) * 100;
		const sign = pct >= 0 ? '+' : '';
		return {
			value: pct,
			string: `${sign}${pct.toFixed(2)}%`
		};
	}

	// Flash Map for real-time WebSocket ticks
	let flashMap = new SvelteMap<string, 'up' | 'down'>();
	const activeTimeouts = new SvelteMap<string, ReturnType<typeof setTimeout>>();

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

	async function loadReferencePrice(sym: string): Promise<void> {
		const upperSym = sym.toUpperCase();

		try {
			const res = await apiFetch(`/api/v1/market/history/${upperSym}?resolution=1D&limit=2`);
			if (res.ok) {
				const data = await res.json();
				const payload = data as { items?: Array<{ close?: number; value?: number }> };
				const rows = Array.isArray(payload?.items) ? payload.items : [];
				const reference = rows.length > 1 ? rows.at(-2) : rows[0];
				const close = Number(reference?.close ?? reference?.value);
				if (Number.isFinite(close) && close > 0) initialPrices[upperSym] = close;
			}
		} catch {
			// Missing historical data stays neutral; never invent a return.
		}
	}

	$effect(() => {
		for (const p of processedPrices) {
			const sym = p.symbol.toUpperCase();
			if (!initialPrices[sym] && !referenceRequests.has(sym)) {
				referenceRequests.add(sym);
				void loadReferencePrice(sym);
			}
		}
	});

	// Filter and Sort Processing
	let processedPrices = $derived.by(() => {
		// Pool of all available prices in store
		const priceBySym: Record<string, PriceData> = Object.create(null);
		for (const p of allPrices) {
			priceBySym[p.symbol.toUpperCase()] = p;
		}

		let list: HeatmapItem[] = [];

		if (activeCategory === 'sp500') {
			// Curated list of S&P 500 Equities (matches user's screenshot)
			for (const sym of Object.keys(US_EQUITIES)) {
				const existing = priceBySym[sym];
				if (!existing || existing.price <= 0) continue;
				const details = getSymbolMeta(sym);
				const pct = getPercentChange(existing);
				list.push({
					symbol: sym,
					price: existing.price,
					category: 'sp500',
					pct,
					details,
					data: existing
				});
			}
		} else if (activeCategory === 'idx') {
			// Indonesian Equities
			for (const sym of Object.keys(IDX_EQUITIES)) {
				const existing = priceBySym[sym];
				if (!existing || existing.price <= 0) continue;
				const details = getSymbolMeta(sym);
				const pct = getPercentChange(existing);
				list.push({
					symbol: sym,
					price: existing.price,
					category: 'idx',
					pct,
					details,
					data: existing
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
		nodes: { id: string; weight: number; data: HeatmapItem }[],
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
	function getHeatmapBgColor(pct: number | null): string {
		if (pct === null || !Number.isFinite(pct) || pct === 0) return '#526057';
		const strength = Math.min(Math.abs(pct), 5) / 5;
		const alpha = (0.2 + strength * 0.8).toFixed(2);
		return pct > 0
			? `color-mix(in srgb, #317451 ${Number(alpha) * 100}%, #173b2b)`
			: `color-mix(in srgb, #b45b50 ${Number(alpha) * 100}%, #4a2928)`;
	}

	function handleCellMouseEnter(e: MouseEvent, node: TreeMapNode) {
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
	class="flex flex-col overflow-hidden rounded-xl border border-border bg-surface shadow-xl transition-all duration-300
	{isFullscreen ? 'fixed inset-0 z-50 rounded-none' : 'h-[640px] w-full'}"
>
	<!-- Top Control Bar -->
	<div
		class="z-20 flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-border bg-surface px-4 py-2.5"
	>
		<!-- Category Tabs -->
		<div class="scrollbar-none flex items-center gap-1.5 overflow-x-auto pb-0.5">
			{#each categories as cat (cat.id)}
				<button
					onclick={() => (activeCategory = cat.id)}
					class="cursor-pointer rounded-md px-3 py-1.5 text-xs font-bold whitespace-nowrap transition-all
					{activeCategory === cat.id
						? 'bg-accent text-white shadow-sm'
						: 'bg-surface-2 text-text-muted hover:bg-accent/10 hover:text-text'}"
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
					class="absolute top-2 left-2.5 h-3.5 w-3.5 text-text-dim transition-colors group-focus-within:text-accent"
				/>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search ticker, sector..."
					class="w-full rounded-md border border-border bg-bg py-1.5 pr-3 pl-8 text-xs font-semibold text-text transition-all placeholder:text-text-dim focus:border-accent focus:outline-none"
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
		class="relative flex-1 overflow-hidden bg-surface-2 select-none"
		bind:clientWidth={containerWidth}
		bind:clientHeight={containerHeight}
	>
		{#if processedPrices.length === 0}
			<div
				class="absolute inset-0 flex flex-col items-center justify-center p-6 text-center text-[#64748b]"
			>
				<p class="text-sm font-semibold">No assets found</p>
				<p class="mt-1 text-xs">Try selecting a different market or clearing your search.</p>
			</div>
		{:else}
			{#each computedTreeMap as node (node.id)}
				{@const pctVal = node.data.pct.value}
				{@const bgColor = getHeatmapBgColor(pctVal)}
				{@const flash = flashMap.get(node.id)}
				{@const displaySym = node.data.details.displaySymbol || node.id}
				{@const logoSrc = node.data.details.svgLogo
					? `data:image/svg+xml,${encodeURIComponent(node.data.details.svgLogo)}`
					: node.data.details.logo?.url}

				<button
					type="button"
					onclick={() => onselect(node.id)}
					onmouseenter={(e) => handleCellMouseEnter(e, node)}
					onmousemove={handleCellMouseMove}
					onmouseleave={handleCellMouseLeave}
					class="heatmap-tile absolute flex cursor-pointer flex-col items-center justify-center overflow-hidden border border-black/30 text-center transition-all duration-200 hover:z-30 hover:brightness-125
					{flash === 'up' ? 'cell-flash-green' : flash === 'down' ? 'cell-flash-red' : ''}"
					style="left: {node.x}px; top: {node.y}px; width: {node.w}px; height: {node.h}px; background-color: {bgColor};"
				>
					{#if node.w >= 110 && node.h >= 75}
						<!-- MEGA TILE (e.g. AAPL, MSFT, GOOGL, AMZN, NVDA, META) -->
						<div class="flex h-full w-full flex-col items-center justify-center p-2">
							<!-- Logo badge -->
							{#if node.h >= 95}
								<div
									class="mb-1.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-black/40 p-1.5 shadow-md"
								>
									{#if logoSrc}
										<img src={logoSrc} alt="" class="h-full w-full rounded-full object-contain" />
									{:else}
										<span class="text-xs font-black text-white/80">{displaySym.slice(0, 3)}</span>
									{/if}
								</div>
							{/if}

							<!-- Ticker -->
							<div
								class="text-base leading-tight font-black tracking-tight text-white drop-shadow-sm sm:text-lg"
							>
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
								<div
									class="mb-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-black/35 p-1 shadow-sm"
								>
									{#if logoSrc}
										<img src={logoSrc} alt="" class="h-full w-full rounded-full object-contain" />
									{:else}
										<span class="text-[9px] font-black text-white/80">{displaySym.slice(0, 2)}</span
										>
									{/if}
								</div>
							{/if}
							<div
								class="text-xs leading-tight font-black tracking-tight text-white drop-shadow-sm sm:text-sm"
							>
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
								<span class="text-[8px] leading-none font-bold text-white/80"
									>{displaySym.slice(0, 3)}</span
								>
							{/if}
						</div>
					{/if}
				</button>
			{/each}
		{/if}
	</div>

	<!-- Bottom Legend Scale -->
	<div
		class="z-20 flex shrink-0 items-center justify-between border-t border-border bg-surface px-4 py-2 text-[11px] text-text-muted"
	>
		<div class="flex items-center gap-1.5 font-medium">
			<Info class="h-3.5 w-3.5 text-[#2962ff]" />
			<span>Showing {processedPrices.length} assets • Click tile to open detailed chart</span>
		</div>

		<!-- Gradient Legend -->
		<div class="flex items-center gap-1.5 font-mono text-[10px]">
			<span>-5%</span>
			<div
				class="flex h-2.5 items-center gap-0.5 overflow-hidden rounded-sm border border-black/60"
			>
				<div class="h-full w-3.5 bg-[#4a2928]"></div>
				<div class="h-full w-3.5 bg-[#b45b50]"></div>
				<div class="h-full w-3.5 bg-[#526057]"></div>
				<div class="h-full w-3.5 bg-[#173b2b]"></div>
				<div class="h-full w-3.5 bg-[#317451]"></div>
			</div>
			<span>+5%</span>
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
				<span class="font-mono font-bold"
					>${Number(d.price).toLocaleString(undefined, {
						minimumFractionDigits: 2,
						maximumFractionDigits: 2
					})}</span
				>
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
		border-radius: 3px;
		outline: none;
		user-select: none;
	}
	.heatmap-tile:focus-visible {
		z-index: 40;
		outline: 2px solid #f4f4ef;
		outline-offset: -3px;
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
