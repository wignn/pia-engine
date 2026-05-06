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

<div>
	<div class="mb-4 flex items-center justify-between">
		<h3 class="text-lg font-semibold tracking-tight">{title}</h3>
		<span class="text-xs text-text-dim">{displayItems.length} items</span>
	</div>

	{#if loading}
		<div class="space-y-3">
			{#each Array(4) as _}
				<div class="h-20 animate-pulse rounded-lg bg-surface"></div>
			{/each}
		</div>
	{:else if items.length === 0}
		<div class="rounded-lg border border-border-subtle bg-surface p-8 text-center text-sm text-text-muted">
			No news available
		</div>
	{:else}
		<div class="space-y-2">
			{#each displayItems as item, i (item.id ?? i)}
				<article class="group rounded-lg border border-border-subtle bg-surface p-4 transition-all hover:border-border hover:bg-surface-2">
					<div class="flex items-start gap-3">
						<span class="mt-1.5 inline-block h-2 w-2 shrink-0 rounded-full {sentimentDot(item.sentiment)}"></span>
						<div class="min-w-0 flex-1">
							<h4 class="text-sm font-medium leading-snug text-text group-hover:text-accent transition-colors">
								{#if item.original_url || item.url}
									<a href={item.original_url ?? item.url} target="_blank" rel="noopener noreferrer" class="hover:underline">
										{getTitle(item)}
									</a>
								{:else}
									{item.translated_title ?? item.original_title ?? item.title}
								{/if}
							</h4>
							{#if item.summary}
								<p class="mt-1 text-xs leading-relaxed text-text-muted line-clamp-2">{item.summary}</p>
							{/if}
							<div class="mt-2 flex flex-wrap items-center gap-3 text-[11px] text-text-dim">
								{#if item.source_name}
									<span>{item.source_name}</span>
								{/if}
								{#if item.published_at || item.processed_at}
									<span>{formatTime(item.published_at ?? item.processed_at)}</span>
								{/if}
								{#if item.sentiment}
									<span class="{sentimentColor(item.sentiment)} font-medium uppercase">{item.sentiment}</span>
								{/if}
								{#if item.currency_pairs}
									<span class="font-mono text-accent">{item.currency_pairs}</span>
								{/if}
								{#if item.tickers}
									<span class="font-mono text-blue">{item.tickers}</span>
								{/if}
							</div>
						</div>
					</div>
				</article>
			{/each}
		</div>
	{/if}
</div>
