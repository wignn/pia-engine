<script lang="ts">
	import { resolve } from '$app/paths';
	import { onMount, onDestroy } from 'svelte';
	import { Moon, Sun } from 'lucide-svelte';
	import {
		startWebSocket,
		stopWebSocket,
		marketStore,
		realtimeNewsStore
	} from '$lib/stores/websocket.svelte';
	import { forexNews, newsLoading, startNewsPolling, stopNewsPolling } from '$lib/stores/news';
	import { startCalendarPolling, stopCalendarPolling } from '$lib/stores/calendar';
	import { startMacroPolling, stopMacroPolling } from '$lib/stores/macro';
	import TickerStrip from '$lib/components/TickerStrip.svelte';
	import MacroDashboard from '$lib/components/MacroDashboard.svelte';
	import MarketGrid from '$lib/components/MarketGrid.svelte';
	import PriceChart from '$lib/components/PriceChart.svelte';
	import WhyDidItMoveCard from '$lib/components/WhyDidItMoveCard.svelte';
	import MarketHeatmap from '$lib/components/MarketHeatmap.svelte';
	import NewsFeed from '$lib/components/NewsFeed.svelte';
	import CalendarTable from '$lib/components/CalendarTable.svelte';
	import SentimentDashboard from '$lib/components/SentimentDashboard.svelte';
	import OptionsDashboard from '$lib/components/OptionsDashboard.svelte';
	import logoUrl from '$lib/assets/logo.png';
	import { reveal } from '$lib/actions/reveal';

	let selectedSymbol = $state('XAUUSD');
	let viewMode = $state('chart');

	let isDarkTheme = $state(false);
	let lenis: {
		raf: (time: number) => void;
		scrollTo: (target: string) => void;
		destroy: () => void;
	} | null = null;
	let lenisFrame = 0;
	let pageMounted = false;

	onMount(() => {
		pageMounted = true;
		if (typeof window !== 'undefined') {
			const day = new Date().getUTCDay();
			if (day === 0 || day === 6) {
				selectedSymbol = 'BTCUSDT';
			}
			const storedTheme = localStorage.getItem('theme');
			if (
				storedTheme === 'dark' ||
				(!storedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)
			) {
				isDarkTheme = true;
				document.documentElement.classList.add('dark');
			}
		}

		startWebSocket();
		startNewsPolling(60_000);
		startCalendarPolling(300_000);
		startMacroPolling(600_000);

		if (!window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
			void import('lenis').then(({ default: Lenis }) => {
				if (!pageMounted || !document.body.isConnected) return;
				lenis = new Lenis({
					duration: 1.05,
					smoothWheel: true,
					lerp: 0.08,
					anchors: true,
					gestureOrientation: 'vertical'
				});
				const raf = (time: number) => {
					lenis?.raf(time);
					lenisFrame = requestAnimationFrame(raf);
				};
				lenisFrame = requestAnimationFrame(raf);
			});
		}
	});

	onDestroy(() => {
		if (!pageMounted) return;
		pageMounted = false;
		cancelAnimationFrame(lenisFrame);
		lenis?.destroy();
		lenis = null;
		stopWebSocket();
		stopNewsPolling();
		stopCalendarPolling();
		stopMacroPolling();
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
	<header
		class="sticky top-0 z-50 flex h-[68px] shrink-0 items-center justify-between border-b border-border bg-bg/95 px-4 backdrop-blur-xl md:px-8"
	>
		<div class="flex items-center">
			<a href={resolve('/')} class="group flex items-center gap-3" aria-label="SLV Home">
				<img
					src={logoUrl}
					alt="SLV"
					class="h-11 w-11 shrink-0 object-contain md:h-12 md:w-12"
				/>

				<div class="flex flex-col leading-none">
					<span class="text-[17px] font-black tracking-[-0.03em] text-text">SLV</span>
					<span
						class="mt-1 hidden text-[9px] font-semibold tracking-[0.18em] text-text-dim uppercase sm:block"
					>
						Market Intelligence
					</span>
				</div>
			</a>
		</div>

		<div class="flex items-center gap-2 sm:gap-4">
			<div class="hidden items-center gap-3 border-r border-border pr-4 text-xs md:flex">
				<div class="flex items-center gap-1.5">
					<span
						class="inline-block h-2 w-2 rounded-full {marketStore.connected
							? 'bg-green'
							: 'bg-red'}"
					></span>
					<span class="text-text-muted">
						{marketStore.connected ? 'Connected' : 'Disconnected'}
					</span>
				</div>
			</div>

			<button
				class="rounded-full p-2 text-text-dim transition-colors hover:bg-surface-2 hover:text-text"
				onclick={toggleTheme}
				title={isDarkTheme ? 'Switch to light mode' : 'Switch to dark mode'}
				aria-label={isDarkTheme ? 'Switch to light mode' : 'Switch to dark mode'}
			>
				{#if isDarkTheme}
					<Sun class="h-5 w-5" />
				{:else}
					<Moon class="h-5 w-5" />
				{/if}
			</button>

			<a
				href="https://pia.wign.dev/portal#api"
				class="hidden text-sm font-medium text-text-muted transition-colors hover:text-text sm:block"
			>
				Docs
			</a>

			<a
				href="https://github.com/wignn/atlsd"
				data-sveltekit-reload
				target="_blank"
				rel="noopener noreferrer"
				class="hidden items-center gap-1.5 rounded-full border border-border bg-surface-2/60 px-3 py-1.5 text-xs font-semibold text-text-muted transition-colors hover:border-text-dim/30 hover:bg-border/30 hover:text-text md:flex"
			>
				<svg
					class="h-3.5 w-3.5"
					viewBox="0 0 24 24"
					fill="currentColor"
					xmlns="http://www.w3.org/2000/svg"
				>
					<path
						fill-rule="evenodd"
						clip-rule="evenodd"
						d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.603-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.464-1.11-1.464-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.579.688.481C19.137 20.162 22 16.418 22 12c0-5.523-4.477-10-10-10z"
					/>
				</svg>
				<span>GitHub</span>
			</a>

			<a
				href="https://pia.wign.dev/portal"
				data-sveltekit-reload
				target="_blank"
				rel="noopener noreferrer"
				class="rounded-sm bg-accent px-3 py-2 text-xs font-semibold text-white transition-colors hover:bg-accent-glow sm:px-4 sm:text-sm"
			>
				Portal
			</a>
		</div>
	</header>

	<div class="relative sticky top-[68px] z-40 border-b border-border">
		<TickerStrip />
		<div
			class="pointer-events-none absolute inset-y-0 left-0 z-10 w-8 bg-gradient-to-r from-bg to-transparent"
		></div>
		<div
			class="pointer-events-none absolute inset-y-0 right-0 z-10 w-8 bg-gradient-to-l from-bg to-transparent"
		></div>
	</div>

	<section
		class="relative min-h-[calc(100vh-104px)] overflow-hidden bg-bg text-text transition-colors duration-300"
	>
		<div class="hero-glow pointer-events-none absolute inset-0"></div>

		<div
			class="relative z-10 mx-auto grid min-h-[calc(100vh-104px)] w-full max-w-[1500px] grid-cols-1 items-center px-6 py-16 lg:grid-cols-[1fr_0.92fr] lg:px-16 xl:px-24"
		>
			<!-- Left -->
			<div class="relative z-20 flex flex-col items-start">
				<div
					class="mb-7 flex items-center gap-3 font-mono text-[10px] font-medium tracking-[0.24em] text-text-dim"
				>
					<span>OPEN SOURCE</span>
					<span class="h-[3px] w-[3px] rounded-full bg-current"></span>
					<span>MARKET INTELLIGENCE</span>
				</div>

				<h1
					class="m-0 max-w-[760px] font-[Georgia,'Times_New_Roman',serif] text-[clamp(52px,6.4vw,108px)] leading-[0.84] font-normal tracking-[-0.07em] text-text"
				>
					<span class="block lg:whitespace-nowrap">THE MARKET</span>
					<span class="block lg:whitespace-nowrap">THAT MOVES</span>
					<span class="block lg:whitespace-nowrap">WITH YOU</span>
				</h1>

				<p class="mt-7 max-w-[520px] text-sm leading-7 text-text-muted sm:text-base">
					Live forex, crypto, global news, economic events, sentiment, and volatility — built into
					one real-time market intelligence system.
				</p>

				<div class="mt-10">
					<a
						href="https://pia.wign.dev/"
						class="group inline-flex items-center gap-3 rounded-sm bg-accent px-6 py-4 font-mono text-[11px] font-bold tracking-[0.13em] text-white transition-colors duration-200 hover:bg-accent-glow"
					>
						GET STARTED PORTAL
						<span class="ml-1 transition-transform duration-200 group-hover:translate-x-1">→</span>
					</a>
				</div>
			</div>

			<!-- Right artwork -->
			<div class="relative mt-12 flex h-[520px] items-center justify-center lg:mt-0 lg:h-[690px]">
				<div
					class="relative z-10 flex h-[500px] w-[500px] items-center justify-center max-lg:h-[440px] max-lg:w-[440px] max-sm:h-[350px] max-sm:w-[350px]"
				>
					<img
						src={logoUrl}
						alt="SLV artwork"
						class="relative z-10 max-h-[500px] w-[86%] object-contain"
					/>
				</div>

				<div
					class="absolute right-2 bottom-16 hidden font-mono text-[8px] tracking-[0.25em] text-text-dim lg:block"
				>
					Nodus Research &amp; Development
				</div>
			</div>
		</div>
	</section>
	<div class="h-px w-full bg-border"></div>

	<!-- Market Board Section -->
	<section
		id="market"
		class="relative overflow-hidden border-y border-border bg-surface-2/25 px-3 py-12 md:px-5 lg:px-8"
		use:reveal={{ y: 40 }}
	>
		<div class="relative z-10 mx-auto flex w-full max-w-[1600px] flex-col">
			<div class="mb-4 flex shrink-0 flex-col justify-between gap-3 sm:flex-row sm:items-center">
				<div class="flex items-center gap-4">
					<h2 class="text-2xl font-black tracking-tight text-text">Market Board</h2>
					<span
						class="rounded-md border border-accent/20 bg-accent/5 px-2.5 py-1 font-mono text-[11px] font-bold text-accent"
					>
						{selectedSymbol}
					</span>
				</div>
				<div
					class="flex items-center gap-1 self-start rounded-lg border border-border/80 bg-surface p-1 shadow-sm sm:self-auto"
				>
					<button
						onclick={() => (viewMode = 'chart')}
						class="cursor-pointer rounded-md px-4 py-1.5 text-xs font-bold transition-all
						{viewMode === 'chart'
							? 'border border-border/50 bg-surface-2 text-text shadow-sm'
							: 'text-text-dim hover:bg-surface-2/50 hover:text-text'}"
					>
						Chart
					</button>
					<button
						onclick={() => (viewMode = 'heatmap')}
						class="cursor-pointer rounded-md px-4 py-1.5 text-xs font-bold transition-all
						{viewMode === 'heatmap'
							? 'border border-border/50 bg-surface-2 text-text shadow-sm'
							: 'text-text-dim hover:bg-surface-2/50 hover:text-text'}"
					>
						Heatmap
					</button>
				</div>
			</div>

			{#if viewMode === 'chart'}
				<div
					class="animate-fade-in grid min-h-0 flex-1 grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_320px]"
				>
					<div class="min-h-0 min-w-0 space-y-4">
						<div
							class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
						>
							<PriceChart symbol={selectedSymbol} height={380} />
						</div>
						<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
							<WhyDidItMoveCard symbol={selectedSymbol} />
							<div class="grid grid-cols-2 gap-3">
								<div
									class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
								>
									<PriceChart symbol="SPX" height={140} compact={true} />
								</div>
								<div
									class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
								>
									<PriceChart symbol="XAUUSD" height={140} compact={true} />
								</div>
								<div
									class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
								>
									<PriceChart symbol="BTCUSDT" height={140} compact={true} />
								</div>
								<div
									class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
								>
									<PriceChart symbol="DXY" height={140} compact={true} />
								</div>
							</div>
						</div>
					</div>
					<aside
						class="min-w-0 overflow-hidden rounded-lg border border-border bg-surface shadow-sm xl:sticky xl:top-[104px] xl:self-start"
					>
						<MarketGrid selected={selectedSymbol} onselect={(sym) => (selectedSymbol = sym)} />
					</aside>
				</div>
			{:else}
				<div
					class="animate-fade-in min-h-0 flex-1 overflow-hidden rounded-lg border border-border bg-surface p-2 shadow-sm"
				>
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

	<div class="h-px w-full bg-border"></div>

	<section
		id="options"
		class="border-b border-border bg-bg px-3 py-12 md:px-5 lg:px-8"
		use:reveal={{ y: 40 }}
	>
		<div class="mx-auto max-w-[1600px]">
			<OptionsDashboard />
		</div>
	</section>

	<div class="h-px w-full bg-border"></div>

	<section
		id="macro"
		class="relative border-b border-border bg-surface-2/20 px-3 py-12 md:px-5 lg:px-8"
		use:reveal={{ y: 40 }}
	>
		<div class="relative z-10 mx-auto max-w-[1600px]">
			<MacroDashboard />
		</div>
	</section>

	<div class="h-px w-full bg-border"></div>

	<section
		id="analyzer"
		class="relative border-b border-border bg-bg px-3 py-12 md:px-5 lg:px-8"
		use:reveal={{ y: 40 }}
	>
		<div class="relative z-10 mx-auto max-w-[1600px]">
			<div class="grid gap-6 xl:grid-cols-[minmax(0,1.15fr)_minmax(360px,0.85fr)]">
				<div>
					<div class="mb-4 flex items-center gap-2">
						<h2 class="text-2xl font-black tracking-tight text-text">Market Sentiment</h2>
						<div class="mx-2 h-5 w-px bg-border"></div>
						<span class="text-sm font-medium text-text-muted">Real-time NLP Analysis</span>
					</div>
					<div
						class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
					>
						<SentimentDashboard
							forexItems={realtimeNewsStore.mergeForex($forexNews)}
							stockItems={[]}
							{selectedSymbol}
						/>
					</div>
				</div>

				<div>
					<div class="mb-4 flex items-center gap-2">
						<h2 class="text-2xl font-black tracking-tight text-text">Global News</h2>
						<div
							class="ml-2 flex items-center gap-1.5 rounded border border-border bg-surface px-2 py-0.5"
						>
							<span class="h-1.5 w-1.5 animate-pulse rounded-full bg-green"></span>
							<span class="text-[10px] font-bold text-text-muted uppercase">Live</span>
						</div>
					</div>
					<div
						class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
					>
						<NewsFeed
							title="Forex & Global"
							items={realtimeNewsStore.mergeForex($forexNews)}
							loading={$newsLoading}
						/>
					</div>
				</div>
			</div>
		</div>
	</section>

	<div class="h-px w-full bg-border"></div>

	<section
		id="calendar"
		class="border-b border-border bg-surface-2/20 px-3 py-12 md:px-5 lg:px-8"
		use:reveal={{ y: 40 }}
	>
		<div class="mx-auto max-w-[1600px]">
			<div class="mb-5 flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
				<div>
					<h2 class="mb-1 text-2xl font-black tracking-tight text-text">Economic Calendar</h2>
					<p class="text-sm font-medium text-text-muted">High-impact macro events and releases</p>
				</div>
			</div>

			<div
				class="overflow-hidden rounded-lg border border-border bg-surface shadow-sm"
			>
				<CalendarTable />
			</div>
		</div>
	</section>

	<div class="h-px w-full bg-border"></div>

	<footer class="border-t border-border bg-surface px-4 py-8 md:px-8 lg:px-16">
		<div class="mx-auto flex max-w-7xl flex-col items-center justify-between gap-6 sm:flex-row">
			<div class="flex items-center gap-3">
				<img src={logoUrl} alt="SLV" class="h-9 w-9 object-contain" />
				<div class="flex items-center gap-2 text-sm font-medium text-text-muted">
					<span class="font-bold text-text">SLV</span>
					<span class="text-text-dim">×</span>
					<span>ATLSD ENGINE</span>
				</div>
			</div>

			<div class="flex items-center gap-6">
				<a
					href="https://pia.wign.dev/portal#api"
					class="text-sm font-medium text-text-muted transition-colors hover:text-text"
					>Documentation</a
				>
				<a
					href="https://pia.wign.dev/portal"
					class="text-sm font-medium text-text-muted transition-colors hover:text-text">Portal</a
				>

				<div
					class="flex items-center gap-2 rounded-full border border-border bg-surface-2 px-3 py-1.5 text-xs font-semibold text-text-dim"
				>
					<span
						class="inline-block h-2 w-2 rounded-full {marketStore.connected
							? 'bg-green'
							: 'bg-red'}"
					></span>
					{marketStore.connected ? 'System Operational' : 'Disconnected'}
				</div>
			</div>
		</div>
	</footer>
</div>
