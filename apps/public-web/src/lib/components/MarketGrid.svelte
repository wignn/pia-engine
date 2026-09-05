<script lang="ts">
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';

	import { getSymbolMeta } from '$lib/symbol-meta';

	interface Props {
		selected: string;
		onselect: (symbol: string) => void;
	}
	let { selected, onselect }: Props = $props();

	let allPrices: PriceData[] = $derived(marketStore.prices);
	const primarySymbols = [
		'IHSG',
		'SPX',
		'XAUUSD',
		'WTI',
		'BTCUSDT',
		'DXY',
		'ETHUSDT',
		'EURUSD',
		'GBPUSD',
		'USDJPY',
		'BBCA',
		'BBRI',
		'BMRI',
		'AAPL',
		'NVDA'
	];
	let livePrices: PriceData[] = $derived.by(() => {
		const bySymbol = new Map(allPrices.map((price) => [price.symbol.toUpperCase(), price]));
		return primarySymbols.map((symbol) => bySymbol.get(symbol) ?? placeholderPrice(symbol));
	});

	function placeholderPrice(symbol: string): PriceData {
		const assetType = symbol.endsWith('USDT')
			? 'crypto'
			: symbol === 'SPX' || symbol === 'DXY'
				? 'index'
				: symbol === 'XAUUSD' || symbol === 'WTI'
					? 'commodity'
					: symbol === 'IHSG'
						? 'index'
						: ['BBCA', 'BBRI', 'BMRI'].includes(symbol)
							? 'stock'
							: 'forex';
		return {
			symbol,
			price: 0,
			bid: null,
			ask: null,
			volume: null,
			source: 'placeholder',
			asset_type: assetType,
			received_at: null,
			direction: 'none',
			prev_price: 0,
			updated_at: 0
		};
	}

	function getSymbolDetails(itemOrSymbol: PriceData | string) {
		return getSymbolMeta(typeof itemOrSymbol === 'string' ? itemOrSymbol : itemOrSymbol.symbol);
	}

	let flashMap = $state<Map<string, 'up' | 'down'>>(new Map());
	const activeTimeouts = new Map<string, ReturnType<typeof setTimeout>>();
	let initialPrices = $state<Map<string, number>>(new Map());

	$effect(() => {
		for (const p of allPrices) {
			if (!initialPrices.has(p.symbol) && p.price > 0) {
				// Base price for percentage calculation
				initialPrices.set(p.symbol, p.price);
			}
		}
	});

	type FreshnessState = 'live' | 'stale' | 'closed' | 'unknown';

	function hasValidPrice(p: PriceData): boolean {
		return p.source !== 'placeholder' && p.price > 0;
	}

	function priceTimestamp(p: PriceData): number {
		if (p.received_at) {
			const parsed = Date.parse(p.received_at);
			if (!Number.isNaN(parsed)) return parsed;
		}
		return p.updated_at;
	}

	function getFreshness(p: PriceData): { state: FreshnessState; label: string; className: string } {
		if (!hasValidPrice(p))
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

		const session = p.session;
		const ageMs = Date.now() - ts;
		const isCrypto = p.symbol.toUpperCase().endsWith('USDT') || p.asset_type === 'crypto';
		const freshMs = isCrypto ? 15 * 60_000 : 5 * 60_000;
		if (session?.is_open && ageMs <= freshMs)
			return { state: 'live', label: 'LIVE', className: 'bg-green/10 text-green border-green/20' };
		if (session && !session.is_open)
			return {
				state: 'closed',
				label: session.state === 'break' ? 'BREAK' : 'CLOSED',
				className: 'bg-surface-2 text-text-dim border-border'
			};
		if (ageMs <= freshMs)
			return { state: 'live', label: 'LIVE', className: 'bg-green/10 text-green border-green/20' };
		return {
			state: 'stale',
			label: isCrypto ? 'FEED LAG' : 'STALE',
			className: 'bg-yellow-500/10 text-yellow-600 border-yellow-500/20'
		};
	}

	function getPercentChange(p: PriceData): { value: number; string: string } {
		if (!hasValidPrice(p)) return { value: 0, string: '--' };
		const base = initialPrices.get(p.symbol) ?? p.price;
		if (base === 0) return { value: 0, string: '0.00%' };
		const pct = ((p.price - base) / base) * 100;
		const sign = pct >= 0 ? '+' : '';
		return {
			value: pct,
			string: `${sign}${pct.toFixed(2)}%`
		};
	}

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
</script>

<div
	class="flex h-full flex-col overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
>
	<div
		class="flex h-11 shrink-0 items-center justify-between border-b border-border bg-surface px-3"
	>
		<div>
			<h2 class="text-xs font-black tracking-tight text-text">Live Instruments</h2>
			<p class="font-mono text-[9px] font-bold text-text-dim uppercase">Watchlist</p>
		</div>
		<div class="flex items-center gap-1.5 text-[10px] text-text-dim">
			<span
				class="inline-block h-1.5 w-1.5 rounded-full {allPrices.length > 0
					? 'animate-pulse bg-green'
					: 'bg-red'}"
			></span>
			<span class="font-mono font-bold"
				>{livePrices.filter(hasValidPrice).length}/{primarySymbols.length}</span
			>
		</div>
	</div>

	<div class="divide-y divide-border bg-surface">
		{#each livePrices as p (p.symbol)}
			{@const details = getSymbolDetails(p)}
			{@const pct = getPercentChange(p)}
			{@const flash = flashMap.get(p.symbol)}
			{@const isSelected = selected === p.symbol}
			{@const validPrice = hasValidPrice(p)}
			{@const freshness = getFreshness(p)}

			<button
				onclick={() => onselect(p.symbol)}
				class="group relative flex h-14 w-full cursor-pointer items-center gap-2 overflow-hidden px-3 text-left transition-all duration-150 focus:outline-none
					{isSelected
					? 'bg-accent/7 shadow-[inset_2px_0_0_var(--color-accent)] dark:bg-accent/10'
					: 'bg-surface hover:bg-surface-2/45'}
					{flash === 'up' ? 'flash-green' : flash === 'down' ? 'flash-red' : ''}"
			>
				<div class="flex w-8 shrink-0 items-center justify-center">
					{#if details.logo.type === 'img'}
						<div
							class="flex h-5.5 w-5.5 shrink-0 items-center justify-center overflow-hidden rounded-full border border-border bg-surface-2"
						>
							<img
								src={details.logo.url}
								alt={details.badge}
								class="h-full w-full rounded-full object-cover"
							/>
						</div>
					{:else}
						<div class="flex h-5.5 w-5.5 shrink-0 items-center justify-center rounded-full {details.badgeClass}">
							<svg
								class="h-3.5 w-3.5 text-accent"
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
				</div>

				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-1.5">
						<span class="font-mono text-xs font-black tracking-tight text-text"
							>{details.displaySymbol}</span
						>
						<span
							class="rounded border px-1 py-0.5 font-mono text-[7px] font-bold {freshness.className}"
							>{freshness.label}</span
						>
					</div>
					<p class="truncate text-[10px] font-semibold text-text-dim">{details.name}</p>
				</div>

				<div class="shrink-0 text-right">
					<div class="font-mono text-sm font-black text-text">
						{validPrice ? details.format(p.price) : '--'}
					</div>
					<div
						class="flex items-center justify-end gap-1 font-mono text-[10px] font-bold {pct.value >=
						0
							? 'text-green'
							: 'text-red'}"
					>
						<span>{validPrice ? (pct.value >= 0 ? '▲' : '▼') : '■'}</span>
						<span>{pct.string}</span>
					</div>
				</div>
			</button>
		{/each}
	</div>
</div>
