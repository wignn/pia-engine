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
	import NewsFeed from '$lib/components/NewsFeed.svelte';
	import CalendarTable from '$lib/components/CalendarTable.svelte';
	import FeatureGrid from '$lib/components/FeatureGrid.svelte';
	import CommandRef from '$lib/components/CommandRef.svelte';
	import logoUrl from '$lib/assets/logo.png';

	let isDarkTheme = $state(false);

	onMount(() => {
		// Check local storage or system preference for theme
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
	<!-- Top Navigation -->
	<header class="sticky top-0 z-50 flex h-[52px] shrink-0 items-center justify-between border-b border-border bg-surface px-4 shadow-sm">
		<div class="flex items-center gap-6">
			<!-- Logo -->
			<a href="/" class="flex items-center gap-2">
				<div class="flex h-8 w-8 items-center justify-center rounded bg-accent/10">
					<img src={logoUrl} alt="Fio" class="h-6 w-6 object-contain" />
				</div>
				<span class="text-lg font-bold tracking-tight">Fio</span>
			</a>

			<!-- Search / Symbol Input Mock -->
			<div class="hidden items-center gap-2 rounded bg-bg px-3 py-1.5 text-sm text-text-muted md:flex border border-border focus-within:border-accent/50 transition-colors">
				<Search class="h-4 w-4" />
				<input 
					type="text" 
					placeholder="Symbol Search" 
					class="w-48 bg-transparent text-text outline-none placeholder:text-text-dim"
				/>
			</div>
		</div>

		<!-- Top Nav Actions -->
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
				href={DISCORD_INVITE} 
				target="_blank" 
				class="rounded bg-accent px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-accent-glow"
			>
				Discord
			</a>
		</div>
	</header>

	<!-- Sticky ticker -->
	<div class="sticky top-[52px] z-40">
		<TickerStrip />
	</div>

	<!-- Hero -->
	<header class="relative px-4 py-24 md:px-8 md:py-32 lg:px-16 overflow-hidden">
		<!-- Subtle Grid Background -->
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
		</div>
	</header>

	<div class="h-px bg-border w-full"></div>

	<!-- Live Market -->
	<section id="market" class="px-4 py-16 md:px-8 lg:px-16">
		<div class="mx-auto max-w-7xl">
			<div class="mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
				<div>
					<h2 class="text-2xl font-bold tracking-tight text-text">Market Quotes</h2>
					<p class="text-sm text-text-muted">Real-time tick data</p>
				</div>
			</div>
			
			<div class="overflow-hidden rounded border border-border bg-surface shadow-sm">
				<MarketGrid />
			</div>
		</div>
	</section>

	<div class="h-px bg-border w-full"></div>

	<!-- News Section -->
	<section id="news" class="px-4 py-16 md:px-8 lg:px-16">
		<div class="mx-auto max-w-7xl">
			<div class="mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
				<div>
					<h2 class="text-2xl font-bold tracking-tight text-text">Latest News</h2>
					<p class="text-sm text-text-muted">Aggregated market intelligence</p>
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

	<!-- Calendar -->
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

	<!-- Features -->
	<FeatureGrid />

	<div class="h-px bg-border w-full"></div>

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
</div>
