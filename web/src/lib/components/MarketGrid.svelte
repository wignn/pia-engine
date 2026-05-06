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

<section id="market" class="px-4 py-12 md:px-8 lg:px-16">
	<div class="mx-auto max-w-7xl">
		<div class="mb-8 flex items-center justify-between">
			<div>
				<h2 class="text-2xl font-bold tracking-tight">Live Market</h2>
				<p class="mt-1 text-sm text-text-muted">Real-time tick data from all sources</p>
			</div>
			<div class="flex items-center gap-2 text-xs text-text-dim">
				<span class="inline-block h-2 w-2 rounded-full {allPrices.length > 0 ? 'bg-green' : 'bg-red'}"></span>
				{allPrices.length} symbols
			</div>
		</div>

		{#if allPrices.length === 0}
			<div class="rounded-lg border border-border bg-surface p-12 text-center">
				<p class="text-text-muted">Waiting for market data...</p>
				<p class="mt-1 text-xs text-text-dim">WebSocket connecting to Core</p>
			</div>
		{:else}
			<!-- Table header -->
			<div class="overflow-x-auto rounded-lg border border-border bg-surface">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b border-border text-left text-xs font-medium uppercase tracking-wider text-text-dim">
							<th class="px-4 py-3">Symbol</th>
							<th class="px-4 py-3 text-right">Price</th>
							<th class="px-4 py-3 text-right">Source</th>
							<th class="hidden px-4 py-3 text-right lg:table-cell">Updated</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-border-subtle">
						{#each allPrices as p (p.symbol)}
							{@const flash = flashMap.get(p.symbol)}
							<tr class="transition-colors hover:bg-surface-2 {flash === 'up' ? 'flash-green' : flash === 'down' ? 'flash-red' : ''}">
								<td class="px-4 py-3">
									<div class="flex items-center gap-2">
										<span class="inline-block h-1.5 w-1.5 rounded-full {p.direction === 'up' ? 'bg-green' : p.direction === 'down' ? 'bg-red' : 'bg-text-dim'}"></span>
										<span class="font-mono font-semibold text-text">{p.symbol}</span>
										<span class="rounded bg-border px-1.5 py-0.5 text-[10px] font-medium uppercase text-text-dim">{p.asset_type}</span>
									</div>
								</td>
								<td class="px-4 py-3 text-right font-mono font-semibold {p.direction === 'up' ? 'text-green' : p.direction === 'down' ? 'text-red' : 'text-text'}">
									{formatPrice(p)}
								</td>
								<td class="px-4 py-3 text-right text-xs text-text-dim">
									{p.source}
								</td>
								<td class="hidden px-4 py-3 text-right font-mono text-xs text-text-dim lg:table-cell">
									{timeSince(p.updated_at)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</section>
