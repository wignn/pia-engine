<script lang="ts">
	import { reveal } from '$lib/actions/reveal';

	const features = [
		{
			title: 'Live Price Alerts',
			description:
				'Set custom price alerts for any symbol. Get instant DM notifications when your target is hit.',
			command: '/market_alert XAUUSD 3400',
			iconPath: 'M13 2L3 14h9l-1 8 10-12h-9l1-8z'
		},
		{
			title: 'Forex News Feed',
			description:
				'Auto-post high-impact forex news from FXStreet, Forex Factory, and Investing.com to your channel.',
			command: '/forex_news_setup #channel',
			iconPath:
				'M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2M18 14h-8M15 18h-5M10 6h8v4h-8V6z'
		},
		{
			title: 'Stock News',
			description:
				'Real-time equity headlines and market-moving stock updates from configured sources.',
			command: '/stocknews subscribe',
			iconPath: 'M3 3v18h18M18 17V9M13 17V5M8 17v-3'
		},
		{
			title: 'Economic Calendar',
			description:
				'Get reminded 15 minutes before high-impact events. NFP, CPI, interest rate decisions — never miss one.',
			command: '/calendar_setup #channel',
			iconPath:
				'M8 2v4M16 2v4M3 10h18M21 8v13a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V8a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2zM8 14h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01M16 18h.01'
		},
		{
			title: 'X / Twitter Feed',
			description: 'Auto-forward tweets from configured accounts directly to your Discord channel.',
			command: '/twitter_setup #channel',
			iconPath:
				'M22 4s-.7 2.1-2 3.4c1.6 10-9.4 17.3-18 11.6 2.2.1 4.4-.6 6-2C3 15.5.5 9.6 3 5c2.2 2.6 5.6 4.1 9 4-.9-4.2 4-6.6 7-3.8 1.1 0 3-1.2 3-1.2z'
		},
		{
			title: 'Volatility Detector',
			description:
				'Monitors XAUUSD ATR in real-time. Alerts when current ATR exceeds 2x the historical average.',
			command: '/volatility_setup #channel',
			iconPath:
				'M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z'
		}
	];
</script>

<section id="features" class="relative overflow-hidden bg-surface-2/10 px-4 py-20 md:px-8 lg:px-16">
	<div
		class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-accent/5 via-transparent to-transparent"
	></div>

	<div class="relative z-10 mx-auto max-w-7xl">
		<div class="mb-14 text-center">
			<div
				class="mb-4 inline-flex items-center justify-center rounded-full bg-accent/10 px-3 py-1 text-xs font-semibold text-accent"
			>
				Powerful Capabilities
			</div>
			<h2 class="text-4xl font-black tracking-tight text-text md:text-5xl">What SLV Does</h2>
			<p class="mx-auto mt-4 max-w-xl text-base leading-relaxed text-text-muted">
				Every feature runs in real-time. No delays. No polls. Direct stream from the source straight
				to your Discord.
			</p>
		</div>

		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
			{#each features as f, i (f.title)}
				<div
					use:reveal={{ delay: i * 100, y: 30 }}
					class="group relative overflow-hidden rounded-2xl border border-border/80 bg-surface/80 p-7 backdrop-blur transition-all duration-300 hover:-translate-y-1 hover:border-accent/30 hover:bg-surface hover:shadow-md"
				>
					<!-- Subtle inner glow on hover -->
					<div
						class="pointer-events-none absolute inset-0 bg-gradient-to-br from-accent/5 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100"
					></div>

					<div
						class="mb-5 flex h-12 w-12 items-center justify-center rounded-xl bg-accent/10 text-accent shadow-sm transition-transform duration-300 group-hover:scale-110 group-hover:bg-accent group-hover:text-white"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							width="22"
							height="22"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d={f.iconPath}></path>
						</svg>
					</div>
					<h3
						class="text-lg font-bold tracking-tight text-text transition-colors group-hover:text-accent"
					>
						{f.title}
					</h3>
					<p class="mt-2.5 text-sm leading-relaxed text-text-muted">{f.description}</p>

					<div
						class="mt-6 flex items-center justify-between rounded-lg border border-border/50 bg-surface-2/80 px-3.5 py-2.5 transition-colors group-hover:border-border"
					>
						<code
							class="font-mono text-[11px] font-bold text-text-dim transition-colors group-hover:text-text"
							>{f.command}</code
						>
						<div
							class="-translate-x-2 opacity-0 transition-all duration-300 group-hover:translate-x-0 group-hover:opacity-100"
						>
							<svg
								class="h-4 w-4 text-accent"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M14 5l7 7m0 0l-7 7m7-7H3" />
							</svg>
						</div>
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>
