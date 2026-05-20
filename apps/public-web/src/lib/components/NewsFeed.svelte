<script lang="ts">
	import type { NewsItem } from '$lib/types';

	interface Props {
		title: string;
		items: NewsItem[];
		loading?: boolean;
	}

	let { title, items, loading = false }: Props = $props();

	function getTitle(item: NewsItem): string {
		return item.translated_title || item.original_title || item.title || item.summary || '';
	}

	let displayItems = $derived(items.filter(i => getTitle(i).length > 0));

	function formatTime(iso: string | null): string {
		if (!iso) return '';
		try {
			const d = new Date(iso);
			return d.toLocaleString('id-ID', { hour: '2-digit', minute: '2-digit', timeZone: 'Asia/Jakarta' }) + ' WIB';
		} catch {
			return '';
		}
	}

	function sentimentColor(s: string | null): string {
		if (!s) return 'text-text-dim';
		if (s === 'positive' || s === 'bullish') return 'text-green';
		if (s === 'negative' || s === 'bearish') return 'text-red';
		return 'text-text-dim';
	}

	function sentimentDot(s: string | null): string {
		if (!s) return 'bg-text-dim';
		if (s === 'positive' || s === 'bullish') return 'bg-green';
		if (s === 'negative' || s === 'bearish') return 'bg-red';
		return 'bg-text-dim';
	}
</script>

<div class="flex flex-col p-4">
	<div class="mb-4 flex items-center justify-between">
		<h3 class="text-xs font-semibold uppercase tracking-wider text-text-muted">{title}</h3>
	</div>

	{#if loading && items.length === 0}
		<div class="space-y-3">
			{#each Array(4) as _}
				<div class="h-12 animate-pulse rounded bg-surface-2"></div>
			{/each}
		</div>
	{:else if items.length === 0}
		<div class="p-8 text-center text-sm text-text-muted">
			No news available
		</div>
	{:else}
		<div class="flex flex-col gap-1">
			{#each displayItems as item, i (item.id ?? i)}
				<article class="group rounded border-b border-border py-3 px-2 transition-colors hover:bg-surface-2 cursor-pointer last:border-0">
					<div class="flex flex-col gap-1.5">
						<div class="flex items-center justify-between gap-2 text-xs text-text-dim">
							<div class="flex items-center gap-2">
								<span class="inline-block h-2 w-2 shrink-0 rounded-full {sentimentDot(item.sentiment)}"></span>
								<span class="font-medium text-text-muted">{item.source_name || 'Source'}</span>
							</div>
							<span>{formatTime(item.published_at ?? item.processed_at)}</span>
						</div>
						<h4 class="text-sm leading-relaxed text-text group-hover:text-accent transition-colors line-clamp-2">
							{#if item.original_url || item.url}
								<a href={item.original_url ?? item.url} target="_blank" rel="noopener noreferrer" class="hover:underline">
									{getTitle(item)}
								</a>
							{:else}
								{item.translated_title ?? item.original_title ?? item.title}
							{/if}
						</h4>
						<div class="flex flex-wrap gap-2 mt-1">
							{#if item.currency_pairs}
								<span class="rounded bg-surface px-1.5 py-0.5 text-[10px] font-mono text-accent border border-border">{item.currency_pairs}</span>
							{/if}
							{#if item.tickers}
								<span class="rounded bg-surface px-1.5 py-0.5 text-[10px] font-mono text-blue border border-border">{item.tickers}</span>
							{/if}
						</div>
					</div>
				</article>
			{/each}
		</div>
	{/if}
</div>
