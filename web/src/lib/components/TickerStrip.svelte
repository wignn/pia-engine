<script lang="ts">
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';

	let items: PriceData[] = $derived(marketStore.prices.slice(0, 20));

	// Build a doubled list with stable keys for CSS infinite scroll
	let tickerItems = $derived(
		[...items, ...items].map((p, i) => ({
			...p,
			_key: `${p.symbol}-${i}`
		}))
	);
</script>

{#if items.length > 0}
<div class="relative overflow-hidden border-b border-border-subtle bg-surface/50 backdrop-blur-sm">
	<div class="flex animate-[ticker-scroll_40s_linear_infinite] whitespace-nowrap">
		{#each tickerItems as p (p._key)}
			<span class="inline-flex items-center gap-2 px-5 py-2.5 text-xs tracking-wide">
				<span class="font-mono font-semibold text-text">{p.symbol}</span>
				<span class="font-mono {p.direction === 'up' ? 'text-green' : p.direction === 'down' ? 'text-red' : 'text-text-muted'}">
					{p.asset_type === 'crypto' ? '$' : ''}{p.price.toFixed(p.asset_type === 'crypto' ? 2 : p.symbol.includes('JPY') ? 3 : 5)}
				</span>
				<span class="text-[10px] {p.direction === 'up' ? 'text-green' : p.direction === 'down' ? 'text-red' : 'text-text-dim'}">
					{p.direction === 'up' ? '▲' : p.direction === 'down' ? '▼' : '—'}
				</span>
			</span>
		{/each}
	</div>
</div>
{/if}
