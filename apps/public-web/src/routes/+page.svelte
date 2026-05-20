<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Search,
		Bell,
		Settings,
		BookOpen,
		Moon,
		Sun,
	} from 'lucide-svelte';
	import { DISCORD_INVITE } from '$lib/config';
	import { startWebSocket, stopWebSocket, marketStore, realtimeNewsStore } from '$lib/stores/websocket.svelte';
	import { forexNews, equityNews, newsLoading, startNewsPolling, stopNewsPolling } from '$lib/stores/news';
	import { startCalendarPolling, stopCalendarPolling } from '$lib/stores/calendar';
	import TickerStrip from '$lib/components/TickerStrip.svelte';
	import MarketGrid from '$lib/components/MarketGrid.svelte';
	import PriceChart from '$lib/components/PriceChart.svelte';
	import MarketHeatmap from '$lib/components/MarketHeatmap.svelte';
	import NewsFeed from '$lib/components/NewsFeed.svelte';
	import CalendarTable from '$lib/components/CalendarTable.svelte';
	import FeatureGrid from '$lib/components/FeatureGrid.svelte';
	import CommandRef from '$lib/components/CommandRef.svelte';
	import logoUrl from '$lib/assets/logo.png';

	let selectedSymbol = $state('SPX');
	let viewMode = $state('chart');

	let isDarkTheme = $state(false);

	onMount(() => {
		if (typeof window !== 'undefined') {
			const storedTheme = localStorage.getItem('theme');
			if (storedTheme === 'dark' || (!storedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
				isDarkTheme = true;
				document.documentElement.classList.add('dark');
			}
		}

		startWebSocket();
		startNewsPolling(60_000);
		startCalendarPolling(300_000);
	});

	onDestroy(() => {
		stopWebSocket();
		stopNewsPolling();
		stopCalendarPolling();
	});

	function toggleTheme() {
		isDarkTheme = !isDarkTheme;
		if (isDarkTheme) {
			document.documentElement.classList.add('dark');
			localStorage.setItem('theme', 'dark');
		} else {
			document.documentElement.classList.remove('dark');
			localStorage.setItem('theme', 'light');
		}
	}
</script>

<div class="min-h-screen bg-bg text-text">
	<header class="sticky top-0 z-50 flex h-[52px] shrink-0 items-center justify-between border-b border-border bg-surface px-4 shadow-sm">
		<div class="flex items-center gap-6">
			<a href="/" class="flex items-center gap-2">
				<div class="flex h-8 w-8 items-center justify-center rounded bg-accent/10">
					<img src={logoUrl} alt="Fio" class="h-6 w-6 object-contain" />
				</div>
				<span class="text-lg font-bold tracking-tight">Fio</span>
			</a>


		</div>

		<div class="flex items-center gap-4">
			<div class="hidden items-center gap-3 border-r border-border pr-4 md:flex text-xs">
				<div class="flex items-center gap-1.5">
					<span class="inline-block h-2 w-2 rounded-full {marketStore.connected ? 'bg-green' : 'bg-red'} flash-green"></span>
					<span class="text-text-muted">{marketStore.connected ? 'Connected' : 'Disconnected'}</span>
				</div>
			</div>
			
			<button class="text-text-dim transition-colors hover:text-text" onclick={toggleTheme} title="Toggle theme">
				{#if isDarkTheme}
					<Sun class="h-5 w-5" />
				{:else}
					<Moon class="h-5 w-5" />
				{/if}
			</button>
			<a href="/docs" class="text-sm font-medium text-text-muted transition-colors hover:text-text">Docs</a>
			
			<a 
				href="https://github.com/wignn/atlsd" 
				target="_blank"
				rel="noopener noreferrer"
				class="flex items-center gap-1.5 rounded-full border border-border bg-surface-2/60 px-3 py-1.5 text-xs font-semibold text-text-muted transition-all hover:bg-border/30 hover:text-text hover:border-text-dim/30 shadow-sm"
			>
				<svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
					<path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.603-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.464-1.11-1.464-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.579.688.481C19.137 20.162 22 16.418 22 12c0-5.523-4.477-10-10-10z" />
				</svg>
				<span>GitHub</span>
			</a>
			
			
			<a 
				href={DISCORD_INVITE} 
				target="_blank" 
				class="rounded bg-accent px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-accent-glow"
			>
				Discord
			</a>
		</div>
	</header>

	<div class="sticky top-[52px] z-40">
		<TickerStrip />
	</div>

	<header class="relative px-4 py-24 md:px-8 md:py-32 lg:px-16 overflow-hidden">
		<div class="absolute inset-0 bg-[linear-gradient(to_right,var(--color-border)_1px,transparent_1px),linear-gradient(to_bottom,var(--color-border)_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] opacity-20"></div>
		
		<div class="relative mx-auto max-w-4xl text-center">
			<h1 class="text-4xl font-extrabold tracking-tight sm:text-5xl md:text-6xl text-text">
				Real-time Market
				<br />
				<span class="text-accent">Intelligence</span>
			</h1>

			<p class="mx-auto mt-6 max-w-xl text-lg leading-relaxed text-text-muted">
				Live forex & crypto prices, breaking news, economic calendar, and volatility detection — 
				streamed directly to your Discord server.
			</p>

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
		</div>
	</header>

	<div class="h-px bg-border w-full"></div>

	<section id="market" class="px-4 py-16 md:px-8 lg:px-16">
		<div class="mx-auto max-w-7xl">
			<div class="mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
				<div>
					<h2 class="text-2xl font-bold tracking-tight text-text">Market Board</h2>
				</div>
				<div class="flex items-center gap-1.5 bg-surface-2 p-1 rounded-lg border border-border self-start sm:self-auto">
					<button
						onclick={() => viewMode = 'chart'}
						class="px-3.5 py-1.5 text-xs font-semibold rounded-md transition-all cursor-pointer
						{viewMode === 'chart'
							? 'bg-surface text-text shadow-sm border border-border/80'
							: 'text-text-dim hover:text-text'}"
					>
						Chart & List
					</button>
					<button
						onclick={() => viewMode = 'heatmap'}
						class="px-3.5 py-1.5 text-xs font-semibold rounded-md transition-all cursor-pointer
						{viewMode === 'heatmap'
							? 'bg-surface text-text shadow-sm border border-border/80'
							: 'text-text-dim hover:text-text'}"
					>
						Market Heatmap
					</button>
				</div>
			</div>
			
			{#if viewMode === 'chart'}
				<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-fade-in">
					<div class="lg:col-span-2">
						<PriceChart symbol={selectedSymbol} />
					</div>
					<div class="lg:col-span-1">
						<MarketGrid selected={selectedSymbol} onselect={(sym) => selectedSymbol = sym} />
					</div>
				</div>

				<!-- 4 Compact Mini-Charts at bottom row -->
				<div class="mt-6 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 animate-fade-in">
					<PriceChart symbol="SPX" height={160} compact={true} />
					<PriceChart symbol="XAUUSD" height={160} compact={true} />
					<PriceChart symbol="BTCUSDT" height={160} compact={true} />
					<PriceChart symbol="DXY" height={160} compact={true} />
				</div>
			{:else}
				<div class="animate-fade-in">
					<MarketHeatmap
						onselect={(sym) => {
							selectedSymbol = sym;
							viewMode = 'chart';
						}}
					/>
				</div>
			{/if}
		</div>
	</section>

	<div class="h-px bg-border w-full"></div>

	<section id="news" class="px-4 py-16 md:px-8 lg:px-16">
		<div class="mx-auto max-w-7xl">
			<div class="mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
				<div>
					<h2 class="text-2xl font-bold tracking-tight text-text">Latest News</h2>
				</div>
			</div>

			<div class="grid gap-6 lg:grid-cols-2">
				<div class="rounded border border-border bg-surface shadow-sm">
					<NewsFeed title="Forex & Global" items={realtimeNewsStore.mergeForex($forexNews)} loading={$newsLoading} />
				</div>
				<div class="rounded border border-border bg-surface shadow-sm">
					<NewsFeed title="Equity / Saham" items={realtimeNewsStore.mergeEquity($equityNews)} loading={$newsLoading} />
				</div>
			</div>
		</div>
	</section>

	<div class="h-px bg-border w-full"></div>

	<section id="calendar" class="px-4 py-16 md:px-8 lg:px-16">
		<div class="mx-auto max-w-7xl">
			<div class="mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
				<div>
					<h2 class="text-2xl font-bold tracking-tight text-text">Economic Calendar</h2>
					<p class="text-sm text-text-muted">High-impact events</p>
				</div>
			</div>

			<div class="overflow-hidden rounded border border-border bg-surface shadow-sm">
				<CalendarTable />
			</div>
		</div>
	</section>

	<div class="h-px bg-border w-full"></div>

	<FeatureGrid />

	<div class="h-px bg-border w-full"></div>

	<CommandRef />

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
</div>
