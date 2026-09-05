<script lang="ts">
	import type { NewsItem } from '$lib/types';
	import { reveal } from '$lib/actions/reveal';

	interface Props {
		title: string;
		items: NewsItem[];
		loading?: boolean;
	}

	let { title, items, loading = false }: Props = $props();

	function getTitle(item: NewsItem): string {
		return item.translated_title || item.original_title || item.title || item.summary || '';
	}

	let displayItems = $derived(items.filter((i) => getTitle(i).length > 0));

	function formatTime(iso: string | null): string {
		if (!iso) return '';
		try {
			const d = new Date(iso);
			return d.toLocaleString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
		} catch {
			return '';
		}
	}

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	function sentimentColor(s: string | null): string {
		if (!s) return 'text-text-dim';
		const v = s.toLowerCase();
		if (v === 'positive' || v === 'bullish') return 'text-green';
		if (v === 'negative' || v === 'bearish') return 'text-red';
		if (v === 'mixed' || v === 'neutral') return 'text-amber';
		return 'text-text-dim';
	}

	function sentimentDot(s: string | null): string {
		if (!s) return 'bg-text-dim/50 border-border';
		const v = s.toLowerCase();
		if (v === 'positive' || v === 'bullish')
			return 'bg-green border-green/30';
		if (v === 'negative' || v === 'bearish')
			return 'bg-red border-red/30';
		if (v === 'mixed' || v === 'neutral') return 'bg-amber border-amber/30';
		return 'bg-text-dim/50 border-border';
	}

	// Calculate reactive aggregate sentiment statistics
	let stats = $derived.by(() => {
		let positive = 0;
		let negative = 0;
		let neutral = 0;
		let total = 0;
		for (const item of items) {
			const s = item.sentiment?.toLowerCase();
			if (s === 'positive' || s === 'bullish') {
				positive++;
				total++;
			} else if (s === 'negative' || s === 'bearish') {
				negative++;
				total++;
			} else if (s === 'neutral' || s === 'mixed') {
				neutral++;
				total++;
			}
		}
		return { positive, negative, neutral, total };
	});
</script>

<div class="flex flex-col p-5">
	<div class="mb-4 flex items-center justify-between">
		<h3 class="text-xs font-bold tracking-wider text-text-muted uppercase">{title}</h3>
		{#if stats.total > 0}
			<div
				class="flex items-center gap-3 rounded-md border border-border/50 bg-surface-2/50 px-2 py-1 font-mono text-[11px]"
			>
				<span class="flex items-center gap-1 font-bold text-green"
					><span class="text-[8px]">▲</span>
					{((stats.positive / stats.total) * 100).toFixed(0)}%</span
				>
				<span class="flex items-center gap-1 font-bold text-text-dim"
					><span class="text-[8px]">■</span>
					{((stats.neutral / stats.total) * 100).toFixed(0)}%</span
				>
				<span class="flex items-center gap-1 font-bold text-red"
					><span class="text-[8px]">▼</span>
					{((stats.negative / stats.total) * 100).toFixed(0)}%</span
				>
			</div>
		{/if}
	</div>

	{#if stats.total > 0}
		<div
			class="mb-6 flex h-1.5 w-full overflow-hidden rounded-full border border-border/50 bg-surface-2 shadow-inner"
		>
			<div
				class="h-full bg-green transition-all duration-1000 ease-out"
				style="width: {(stats.positive / stats.total) * 100}%"
				title="Positive"
			></div>
			<div
				class="h-full bg-text-dim/40 transition-all duration-1000 ease-out"
				style="width: {(stats.neutral / stats.total) * 100}%"
				title="Neutral"
			></div>
			<div
				class="h-full bg-red transition-all duration-1000 ease-out"
				style="width: {(stats.negative / stats.total) * 100}%"
				title="Negative"
			></div>
		</div>
	{/if}

	{#if loading && items.length === 0}
		<div class="space-y-4">
			{#each Array(4) as _, idx (idx)}
				<div class="h-20 animate-pulse rounded-lg bg-surface-2"></div>
			{/each}
		</div>
	{:else if items.length === 0}
		<div
			class="rounded-lg border border-dashed border-border bg-surface-2/30 py-12 text-center text-sm font-medium text-text-muted"
		>
			No news available
		</div>
	{:else}
		<div class="flex flex-col gap-3">
			{#each displayItems as item, i (item.id ?? i)}
				<article
					use:reveal={{ delay: Math.min(i * 80, 500), y: 15 }}
					class="group cursor-pointer rounded-xl border border-border/60 bg-surface p-4 transition-all duration-300 hover:border-accent/40 hover:bg-surface-2/50 hover:shadow-md"
				>
					<div class="flex flex-col gap-2.5">
						<div class="flex items-center justify-between gap-2 text-xs">
							<div class="flex items-center gap-2">
								<span
									class="inline-block h-2 w-2 shrink-0 rounded-full border {sentimentDot(
										item.sentiment
									)} transition-transform duration-300 group-hover:scale-125"
								></span>
								<span class="font-bold tracking-wide text-text-muted"
									>{item.source_name || 'Source'}</span
								>
							</div>
							<span class="font-mono text-[10px] font-medium text-text-dim"
								>{formatTime(item.published_at ?? item.processed_at)}</span
							>
						</div>
						<h4
							class="line-clamp-2 text-sm leading-relaxed font-medium text-text transition-colors group-hover:text-accent"
						>
							{#if item.original_url || item.url}
								<a
									href={item.original_url ?? item.url}
									target="_blank"
									rel="noopener noreferrer"
									class="decoration-accent/50 underline-offset-2 hover:underline"
								>
									{getTitle(item)}
								</a>
							{:else}
								{item.translated_title ?? item.original_title ?? item.title}
							{/if}
						</h4>
						<div
							class="mt-1 flex flex-wrap gap-2 opacity-80 transition-opacity group-hover:opacity-100"
						>
							{#if item.currency_pairs}
								<span
									class="rounded border border-accent/20 bg-surface px-2 py-0.5 font-mono text-[10px] font-bold text-accent shadow-sm"
									>{item.currency_pairs}</span
								>
							{/if}
							{#if item.tickers}
								<span
									class="rounded border border-blue-500/20 bg-surface px-2 py-0.5 font-mono text-[10px] font-bold text-blue shadow-sm"
									>{item.tickers}</span
								>
							{/if}
						</div>
					</div>
				</article>
			{/each}
		</div>
	{/if}
</div>
