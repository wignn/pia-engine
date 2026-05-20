<script lang="ts">
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';

	let items: PriceData[] = $derived(marketStore.prices.slice(0, 20));

	// Duplicate items give the ticker a seamless CSS scroll loop.
	let tickerItems = $derived(
		[...items, ...items].map((p, i) => ({
			...p,
			_key: `${p.symbol}-${i}`
		}))
	);
</script>

{#if items.length > 0}
<div class="relative flex h-8 items-center overflow-hidden border-b border-border bg-bg">
	<div class="flex animate-[ticker-scroll_40s_linear_infinite] whitespace-nowrap">
		{#each tickerItems as p (p._key)}
			<span class="inline-flex items-center gap-1.5 px-4 text-xs tracking-wide border-r border-border/50 hover:bg-surface-2 cursor-default transition-colors">
				<span class="font-semibold text-text">{p.symbol}</span>
				<span class="{p.direction === 'up' ? 'text-green' : p.direction === 'down' ? 'text-red' : 'text-text-muted'}">
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
