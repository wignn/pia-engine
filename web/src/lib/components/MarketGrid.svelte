<script lang="ts">
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';

	let allPrices: PriceData[] = $derived(marketStore.prices);

	function formatPrice(p: PriceData): string {
		if (p.asset_type === 'crypto') return `$${p.price.toFixed(2)}`;
		if (p.symbol.includes('JPY')) return p.price.toFixed(3);
		return p.price.toFixed(5);
	}

	function timeSince(ts: number): string {
		const sec = Math.floor((Date.now() - ts) / 1000);
		if (sec < 5) return 'now';
		if (sec < 60) return `${sec}s`;
		return `${Math.floor(sec / 60)}m`;
	}

	// Flash tracking with automatic cleanup
	let flashMap = $state<Map<string, 'up' | 'down'>>(new Map());
	const activeTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	$effect(() => {
		// Only iterate over prices that were updated recently (last 1000ms)
		// to allow for slower updates or network jitter
		const now = Date.now();
		for (const p of allPrices) {
			if (p.direction !== 'none' && now - p.updated_at < 1000) {
				// If already flashing the same direction, don't restart to avoid flicker
				if (activeTimeouts.has(p.symbol) && flashMap.get(p.symbol) === p.direction) {
					continue;
				}

				// Clear existing timeout if any
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

<div class="flex flex-col bg-bg">
	<!-- Header -->
	<div class="flex h-10 shrink-0 items-center justify-between border-b border-border bg-surface px-4">
		<div class="flex items-center gap-4">
			<h2 class="text-xs font-semibold uppercase tracking-wider text-text">Market Watch</h2>
		</div>
		<div class="flex items-center gap-2 text-[11px] text-text-dim">
			<span class="inline-block h-1.5 w-1.5 rounded-full {allPrices.length > 0 ? 'bg-green' : 'bg-red'}"></span>
			{allPrices.length} symbols
		</div>
	</div>

	{#if allPrices.length === 0}
		<div class="flex items-center justify-center p-16 text-center">
			<div>
				<p class="text-sm text-text-muted">Waiting for market data...</p>
				<p class="mt-1 text-[11px] text-text-dim">WebSocket connecting to Core</p>
			</div>
		</div>
	{:else}
		<!-- Table Container -->
		<div class="max-h-[600px] overflow-auto custom-scrollbar">
			<table class="w-full text-[13px]">
				<thead class="sticky top-0 z-10 bg-surface">
					<tr class="border-b border-border text-left text-[11px] text-text-dim">
						<th class="py-1.5 pl-4 pr-2 font-normal">Symbol</th>
						<th class="py-1.5 px-2 text-right font-normal">Last</th>
						<th class="py-1.5 px-2 text-right font-normal">Chg</th>
						<th class="hidden py-1.5 px-2 text-right font-normal lg:table-cell">Source</th>
						<th class="hidden py-1.5 pl-2 pr-4 text-right font-normal lg:table-cell">Vol</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-border">
					{#each allPrices as p (p.symbol)}
						{@const flash = flashMap.get(p.symbol)}
						<tr class="cursor-pointer transition-colors hover:bg-surface-2 {flash === 'up' ? 'flash-green' : flash === 'down' ? 'flash-red' : ''}">
							<td class="py-1 pl-4 pr-2">
								<div class="flex items-center gap-2">
									<span class="font-semibold text-text">{p.symbol}</span>
									<span class="rounded bg-surface-2 border border-border px-1 py-0.5 text-[9px] uppercase text-text-dim">{p.asset_type === 'crypto' ? 'CRYPTO' : p.asset_type === 'equity' ? 'STOCK' : 'FOREX'}</span>
								</div>
							</td>
							<td class="py-1 px-2 text-right font-mono {p.direction === 'up' ? 'text-green' : p.direction === 'down' ? 'text-red' : 'text-text'}">
								{formatPrice(p)}
							</td>
							<td class="py-1 px-2 text-right font-mono text-xs {p.direction === 'up' ? 'text-green' : p.direction === 'down' ? 'text-red' : 'text-text-dim'}">
								{p.direction === 'up' ? '+' : p.direction === 'down' ? '-' : ''}{(Math.random() * 0.5).toFixed(2)}% <!-- Mocking change % as the backend only returns direction currently -->
							</td>
							<td class="hidden py-1 px-2 text-right text-[11px] text-text-dim lg:table-cell">
								{p.source.toUpperCase()}
							</td>
							<td class="hidden py-1 pl-2 pr-4 text-right font-mono text-[11px] text-text-dim lg:table-cell">
								{p.volume ? (p.volume > 1000000 ? (p.volume/1000000).toFixed(1)+'M' : p.volume) : '—'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
