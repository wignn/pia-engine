<script lang="ts">
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';

	interface Props {
		selected: string;
		onselect: (symbol: string) => void;
	}
	let { selected, onselect }: Props = $props();

	let allPrices: PriceData[] = $derived(marketStore.prices);

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
			name = 'Gold / USD';
			badge = 'GOLD';
			badgeColor = 'bg-[#FFD700]/10 text-[#FFD700] border border-[#FFD700]/20';
			unit = 'USD';
			format = (val: number) => val.toLocaleString(undefined, { minimumFractionDigits: 1, maximumFractionDigits: 1 });
			logo = { type: 'svg', url: '' };
		} else if (sym === 'SPX') {
			name = 'S&P 500 Index';
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

<div class="flex flex-col bg-surface border border-border rounded-lg shadow-sm overflow-hidden h-full">
	<!-- Sidebar Header -->
	<div class="flex h-14 shrink-0 items-center justify-between border-b border-border bg-surface px-4">
		<h2 class="text-sm font-bold tracking-tight text-text">Live Instruments</h2>
		<div class="flex items-center gap-1.5 text-xs text-text-dim">
			<span class="inline-block h-2 w-2 rounded-full {allPrices.length > 0 ? 'bg-green animate-pulse' : 'bg-red'}"></span>
			<span class="font-medium font-mono">{allPrices.length} live</span>
		</div>
	</div>

	<!-- Sidebar Instruments Grid (Kotak-Kotak, Tanpa Scrollbar Dalam) -->
	{#if allPrices.length === 0}
		<div class="flex flex-col items-center justify-center p-12 text-center flex-1 min-h-[300px]">
			<div class="h-6 w-6 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
			<p class="text-xs text-text-muted mt-3 font-semibold">Connecting to live feed...</p>
		</div>
	{:else}
		<div class="grid grid-cols-2 gap-2.5 p-3.5 bg-surface">
			{#each allPrices as p (p.symbol)}
				{@const details = getSymbolDetails(p.symbol)}
				{@const pct = getPercentChange(p)}
				{@const flash = flashMap.get(p.symbol)}
				{@const isSelected = selected === p.symbol}

				<button
					onclick={() => onselect(p.symbol)}
					class="w-full text-left p-2.5 rounded-lg border flex flex-col justify-between transition-all duration-200 cursor-pointer focus:outline-none h-[82px] relative overflow-hidden group
					{isSelected 
						? 'bg-accent/5 border-accent shadow-sm shadow-accent/5 dark:bg-accent/10' 
						: 'bg-surface border-border hover:border-text-dim/40 hover:bg-surface-2/40'}
					{flash === 'up' ? 'flash-green' : flash === 'down' ? 'flash-red' : ''}"
				>

					<div class="flex items-center justify-between w-full">
						{#if details.logo.type === 'img'}
							<div class="flex h-5.5 w-5.5 shrink-0 items-center justify-center rounded-full overflow-hidden border border-border bg-surface-2">
								<img src={details.logo.url} alt={details.badge} class="h-full w-full object-cover rounded-full" />
							</div>
						{:else}
							<div class="flex h-5.5 w-5.5 shrink-0 items-center justify-center rounded-full bg-accent/10 border border-accent/20">
								{#if p.symbol.toUpperCase() === 'XAUUSD'}
									<svg class="h-3.5 w-3.5" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
										<path d="M16 28 L48 28 L40 18 L24 18 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
										<path d="M8 46 L36 46 L30 36 L14 36 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
										<path d="M28 46 L56 46 L50 36 L34 36 Z" fill="#FFD700" stroke="#DAA520" stroke-width="2"/>
									</svg>
								{:else if p.symbol.toUpperCase() === 'SPX'}
									<svg class="h-3.5 w-3.5 text-blue-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
										<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"></polyline>
										<polyline points="16 7 22 7 22 13"></polyline>
									</svg>
								{:else}
									<svg class="h-3.5 w-3.5 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
										<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"></polyline>
										<polyline points="16 7 22 7 22 13"></polyline>
									</svg>
								{/if}
							</div>
						{/if}
						<span class="text-[9.5px] font-bold text-text-dim uppercase font-mono tracking-tighter">
							{p.symbol}
						</span>
					</div>

					<div class="flex flex-col mt-1.5 w-full">
						<!-- Price -->
						<div class="flex items-baseline justify-between w-full">
							<span class="font-mono text-sm font-bold text-text truncate max-w-[100px]">
								{details.format(p.price)}
							</span>
							<span class="text-[8px] text-text-dim uppercase font-mono font-semibold pl-1 shrink-0">
								{details.unit}
							</span>
						</div>
						<div class="flex items-center gap-1 text-[10px] font-bold font-mono mt-0.5 {pct.value >= 0 ? 'text-green' : 'text-red'}">
							<span>{pct.value >= 0 ? '▲' : '▼'}</span>
							<span>{pct.string}</span>
						</div>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>

