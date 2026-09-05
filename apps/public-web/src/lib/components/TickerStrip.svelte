<script lang="ts">
	import { marketStore } from '$lib/stores/websocket.svelte';
	import type { PriceData } from '$lib/types';

	let items: PriceData[] = $derived(marketStore.prices.slice(0, 20));

	let tickerItems = $derived(
		[...items, ...items].map((p, i) => ({
			...p,
			_key: `${p.symbol}-${i}`
		}))
	);
</script>

{#if items.length > 0}
	<div
		class="relative flex h-8 items-center overflow-hidden border-b border-border bg-surface-2/80 backdrop-blur-sm"
	>
		<div class="flex animate-[ticker-scroll_40s_linear_infinite] whitespace-nowrap">
			{#each tickerItems as p (p._key)}
				<span
					class="inline-flex cursor-default items-center gap-2 border-r border-border/40 px-5 text-xs tracking-wide transition-colors select-none hover:bg-surface"
				>
					<span class="font-bold text-text">{p.symbol}</span>
					<span
						class={p.direction === 'up'
							? 'font-semibold text-green'
							: p.direction === 'down'
								? 'font-semibold text-red'
								: 'text-text-muted'}
					>
						{p.asset_type === 'crypto' ? '$' : ''}{p.price.toFixed(
							p.asset_type === 'crypto' ? 2 : p.symbol.includes('JPY') ? 3 : 5
						)}
					</span>
					<span
						class="flex h-4 w-4 items-center justify-center rounded-full {p.direction === 'up'
							? 'bg-green/10'
							: p.direction === 'down'
								? 'bg-red/10'
								: 'bg-transparent'}"
					>
						<svg
							class="h-2.5 w-2.5 {p.direction === 'up'
								? 'text-green'
								: p.direction === 'down'
									? 'text-red'
									: 'text-text-dim'}"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="3"
						>
							{#if p.direction === 'up'}
								<path stroke-linecap="round" stroke-linejoin="round" d="M5 15l7-7 7 7" />
							{:else if p.direction === 'down'}
								<path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
							{:else}
								<path stroke-linecap="round" stroke-linejoin="round" d="M5 12h14" />
							{/if}
						</svg>
					</span>
				</span>
			{/each}
		</div>
	</div>
{/if}
