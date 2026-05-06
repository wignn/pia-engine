<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { DISCORD_INVITE } from '$lib/config';
	import { startWebSocket, stopWebSocket, marketStore } from '$lib/stores/websocket.svelte';
	import { forexNews, equityNews, newsLoading, startNewsPolling, stopNewsPolling } from '$lib/stores/news';
	import { startCalendarPolling, stopCalendarPolling } from '$lib/stores/calendar';

	import TickerStrip from '$lib/components/TickerStrip.svelte';
	import MarketGrid from '$lib/components/MarketGrid.svelte';
	import NewsFeed from '$lib/components/NewsFeed.svelte';
	import CalendarTable from '$lib/components/CalendarTable.svelte';
	import FeatureGrid from '$lib/components/FeatureGrid.svelte';
	import CommandRef from '$lib/components/CommandRef.svelte';

	onMount(() => {
		startWebSocket();
		startNewsPolling(60_000);
		startCalendarPolling(300_000);
	});

	onDestroy(() => {
		stopWebSocket();
		stopNewsPolling();
		stopCalendarPolling();
	});
</script>

<!-- Sticky ticker -->
<div class="sticky top-0 z-50">
	<TickerStrip />
</div>

<!-- Hero -->
<header class="relative overflow-hidden px-4 py-24 md:px-8 md:py-32 lg:px-16">
	<!-- Background glow -->
	<div class="pointer-events-none absolute inset-0">
		<div class="absolute left-1/2 top-0 h-[500px] w-[800px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-accent/5 blur-[120px]"></div>
	</div>

	<div class="relative mx-auto max-w-4xl text-center">
		<!-- Logo -->
		<div class="mb-6 inline-flex items-center gap-3">
			<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-accent/10 border border-accent/20">
				<span class="text-2xl font-black text-accent">F</span>
			</div>
			<span class="text-3xl font-bold tracking-tight">Fio</span>
		</div>

		<h1 class="text-4xl font-extrabold tracking-tight sm:text-5xl md:text-6xl">
			Real-time Market
			<br />
			<span class="text-gradient">Intelligence</span>
		</h1>

		<p class="mx-auto mt-6 max-w-xl text-lg leading-relaxed text-text-muted">
			Live forex & crypto prices, breaking news, economic calendar, and volatility detection — 
			streamed directly to your Discord server.
		</p>

		<!-- CTA -->
		<div class="mt-10 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
			<a
				href={DISCORD_INVITE}
				target="_blank"
				rel="noopener noreferrer"
				class="inline-flex items-center gap-2 rounded-lg bg-accent px-6 py-3 text-sm font-semibold text-white transition-all hover:bg-accent-glow hover:shadow-lg hover:shadow-accent/20"
			>
				<svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
					<path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/>
				</svg>
				Add to Discord
			</a>
			<a href="#market" class="text-sm font-medium text-text-muted transition-colors hover:text-text">
				View Live Data ↓
			</a>
		</div>

		<!-- Stats -->
		<div class="mt-12 flex justify-center gap-8 text-center">
			<div>
				<div class="text-2xl font-bold font-mono text-text">{marketStore.prices.length}</div>
				<div class="text-xs text-text-dim">Symbols</div>
			</div>
			<div class="h-10 w-px bg-border"></div>
			<div>
				<div class="text-2xl font-bold font-mono text-text">6+</div>
				<div class="text-xs text-text-dim">News Sources</div>
			</div>
			<div class="h-10 w-px bg-border"></div>
			<div>
				<div class="flex items-center justify-center gap-1.5">
					<span class="inline-block h-2 w-2 rounded-full {marketStore.connected ? 'bg-green' : 'bg-red'}"></span>
					<span class="text-2xl font-bold font-mono text-text">{marketStore.connected ? 'Live' : '—'}</span>
				</div>
				<div class="text-xs text-text-dim">Stream</div>
			</div>
		</div>
	</div>
</header>

<!-- Divider -->
<div class="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-border to-transparent"></div>

<!-- Live Market -->
<MarketGrid />

<div class="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-border to-transparent"></div>

<!-- News Section -->
<section id="news" class="px-4 py-12 md:px-8 lg:px-16">
	<div class="mx-auto max-w-7xl">
		<div class="mb-8">
			<h2 class="text-2xl font-bold tracking-tight">News Feed</h2>
			<p class="mt-1 text-sm text-text-muted">Latest market news · Auto-refreshes every 60s</p>
		</div>

		<div class="grid gap-8 lg:grid-cols-2">
			<NewsFeed title="Forex & Global" items={$forexNews} loading={$newsLoading} />
			<NewsFeed title="Equity / Saham" items={$equityNews} loading={$newsLoading} />
		</div>
	</div>
</section>

<div class="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-border to-transparent"></div>

<!-- Calendar -->
<CalendarTable />

<div class="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-border to-transparent"></div>

<!-- Features -->
<FeatureGrid />

<div class="mx-auto h-px max-w-7xl bg-gradient-to-r from-transparent via-border to-transparent"></div>

<!-- Commands -->
<CommandRef />

<!-- Footer -->
<footer class="border-t border-border px-4 py-8 md:px-8 lg:px-16">
	<div class="mx-auto flex max-w-7xl flex-col items-center justify-between gap-4 sm:flex-row">
		<div class="flex items-center gap-2 text-sm text-text-muted">
			<span class="font-semibold text-text">Fio</span>
			<span class="text-text-dim">×</span>
			<span>Core</span>
		</div>
		<div class="flex items-center gap-1.5 text-xs text-text-dim">
			<span class="inline-block h-1.5 w-1.5 rounded-full {marketStore.connected ? 'bg-green' : 'bg-red'}"></span>
			{marketStore.connected ? 'Connected to Core' : 'Disconnected'}
		</div>
	</div>
</footer>
