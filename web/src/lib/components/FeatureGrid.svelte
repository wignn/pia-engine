<script lang="ts">
	import type { Component } from 'svelte';

	// Inline SVG icons as Svelte snippets — no external dependency needed
	const features = [
		{
			title: 'Live Price Alerts',
			description: 'Set custom price alerts for any symbol. Get instant DM notifications when your target is hit.',
			command: '/market_alert XAUUSD 3400',
			iconPath: 'M13 2L3 14h9l-1 8 10-12h-9l1-8z' // Zap
		},
		{
			title: 'Forex News Feed',
			description: 'Auto-post high-impact forex news from FXStreet, Forex Factory, and Investing.com to your channel.',
			command: '/forex_news_setup #channel',
			iconPath: 'M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2M18 14h-8M15 18h-5M10 6h8v4h-8V6z' // Newspaper
		},
		{
			title: 'Stock News',
			description: 'Real-time Indonesian equity news from CNBC, Kontan, Bisnis Indonesia, and IDX Channel.',
			command: '/stocknews subscribe',
			iconPath: 'M3 3v18h18M18 17V9M13 17V5M8 17v-3' // BarChart3
		},
		{
			title: 'Economic Calendar',
			description: 'Get reminded 15 minutes before high-impact events. NFP, CPI, interest rate decisions — never miss one.',
			command: '/calendar_setup #channel',
			iconPath: 'M8 2v4M16 2v4M3 10h18M21 8v13a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V8a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2zM8 14h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01M16 18h.01' // CalendarDays
		},
		{
			title: 'X / Twitter Feed',
			description: 'Auto-forward tweets from configured accounts directly to your Discord channel.',
			command: '/twitter_setup #channel',
			iconPath: 'M22 4s-.7 2.1-2 3.4c1.6 10-9.4 17.3-18 11.6 2.2.1 4.4-.6 6-2C3 15.5.5 9.6 3 5c2.2 2.6 5.6 4.1 9 4-.9-4.2 4-6.6 7-3.8 1.1 0 3-1.2 3-1.2z' // Twitter
		},
		{
			title: 'Volatility Detector',
			description: 'Monitors XAUUSD ATR in real-time. Alerts when current ATR exceeds 2x the historical average.',
			command: '/volatility_setup #channel',
			iconPath: 'M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z' // Flame
		}
	];
</script>

<section id="features" class="px-4 py-16 md:px-8 lg:px-16">
	<div class="mx-auto max-w-7xl">
		<div class="mb-10 text-center">
			<h2 class="text-3xl font-bold tracking-tight">What Fio Does</h2>
			<p class="mx-auto mt-2 max-w-lg text-sm text-text-muted">
				Every feature runs in real-time. No delays. No polls. Direct stream from the source.
			</p>
		</div>

		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each features as f, i}
				<div
					class="group rounded-xl border border-border bg-surface p-6 transition-all duration-300 hover:border-accent/30 hover:bg-surface-2"
					style="animation-delay: {i * 80}ms"
				>
					<div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-accent/10 text-accent transition-colors group-hover:bg-accent/20">
						<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
							<path d={f.iconPath}></path>
						</svg>
					</div>
					<h3 class="text-base font-semibold text-text">{f.title}</h3>
					<p class="mt-2 text-sm leading-relaxed text-text-muted">{f.description}</p>
					<div class="mt-4 rounded-md bg-bg px-3 py-2">
						<code class="font-mono text-xs text-accent">{f.command}</code>
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>
