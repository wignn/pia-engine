<script lang="ts">
	import type { Command } from '$lib/types';

	const commands: Command[] = [
		// Market
		{ name: '/price', description: 'Check current price for a symbol with autocomplete', category: 'Market' },
		{ name: '/prices', description: 'Show all live market prices', category: 'Market' },
		{ name: '/market_alert', description: 'Set a price alert — get DM when target is hit', category: 'Market' },
		{ name: '/market_alerts', description: 'View your active price alerts', category: 'Market' },
		{ name: '/market_alert_remove', description: 'Remove a price alert by ID', category: 'Market' },
		// News
		{ name: '/forex_news_setup', description: 'Setup forex news channel', permission: 'Admin', category: 'News' },
		{ name: '/forex_news_enable', description: 'Enable forex news', permission: 'Admin', category: 'News' },
		{ name: '/forex_news_disable', description: 'Disable forex news', permission: 'Admin', category: 'News' },
		{ name: '/forex_news_status', description: 'Check forex news status', category: 'News' },
		{ name: '/forex_calendar', description: 'View high-impact forex calendar events', category: 'News' },
		{ name: '/stocknews subscribe', description: 'Subscribe to equity news alerts', permission: 'Manage Channels', category: 'News' },
		{ name: '/stocknews unsubscribe', description: 'Unsubscribe from equity news', permission: 'Manage Channels', category: 'News' },
		{ name: '/stocknews status', description: 'Check stock news subscription', category: 'News' },
		{ name: '/stocknews latest', description: 'Show latest equity news', category: 'News' },
		// Calendar
		{ name: '/calendar_setup', description: 'Setup economic calendar reminders', permission: 'Admin', category: 'Calendar' },
		{ name: '/calendar_enable', description: 'Enable calendar reminders', permission: 'Admin', category: 'Calendar' },
		{ name: '/calendar_disable', description: 'Disable calendar reminders', permission: 'Admin', category: 'Calendar' },
		{ name: '/calendar_status', description: 'Check calendar status', category: 'Calendar' },
		{ name: '/calendar_mention', description: 'Toggle @everyone for events', permission: 'Admin', category: 'Calendar' },
		// X/Twitter
		{ name: '/twitter_setup', description: 'Setup X/Twitter feed channel', permission: 'Admin', category: 'X Feed' },
		{ name: '/twitter_enable', description: 'Enable X feed', permission: 'Admin', category: 'X Feed' },
		{ name: '/twitter_disable', description: 'Disable X feed', permission: 'Admin', category: 'X Feed' },
		{ name: '/twitter_status', description: 'Check X feed status', category: 'X Feed' },
		// Volatility
		{ name: '/volatility_setup', description: 'Setup gold volatility alerts', permission: 'Admin', category: 'Volatility' },
		{ name: '/volatility_disable', description: 'Disable volatility alerts', permission: 'Admin', category: 'Volatility' },
		{ name: '/volatility_status', description: 'Check volatility alert status', category: 'Volatility' },
		// Moderation
		{ name: '/warn', description: 'Warn a member', permission: 'Manage Messages', category: 'Moderation' },
		{ name: '/warnings', description: 'View warnings for a member', permission: 'Manage Messages', category: 'Moderation' },
		{ name: '/mute', description: 'Timeout a member', permission: 'Moderate Members', category: 'Moderation' },
		{ name: '/unmute', description: 'Remove timeout from a member', permission: 'Moderate Members', category: 'Moderation' },
		{ name: '/kick', description: 'Kick a member', permission: 'Kick Members', category: 'Moderation' },
		{ name: '/ban', description: 'Ban a member', permission: 'Ban Members', category: 'Moderation' },
	];

	const categories = [...new Set(commands.map((c) => c.category))];
	let activeCategory = $state(categories[0]);
	let filtered = $derived(commands.filter((c) => c.category === activeCategory));
</script>

<section id="commands" class="px-4 py-16 md:px-8 lg:px-16">
	<div class="mx-auto max-w-7xl">
		<div class="mb-8">
			<h2 class="text-2xl font-bold tracking-tight">Commands</h2>
			<p class="mt-1 text-sm text-text-muted">Full slash command reference</p>
		</div>

		<!-- Category tabs -->
		<div class="mb-6 flex flex-wrap gap-2">
			{#each categories as cat}
				<button
					onclick={() => activeCategory = cat}
					class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors
					{activeCategory === cat
						? 'bg-accent text-white'
						: 'bg-surface text-text-muted hover:bg-surface-2 hover:text-text'}"
				>
					{cat}
				</button>
			{/each}
		</div>

		<!-- Command list -->
		<div class="overflow-hidden rounded-lg border border-border bg-surface">
			<div class="divide-y divide-border-subtle">
				{#each filtered as cmd (cmd.name)}
					<div class="flex items-center justify-between px-4 py-3 transition-colors hover:bg-surface-2">
						<div class="flex items-center gap-3">
							<code class="font-mono text-sm font-semibold text-accent">{cmd.name}</code>
							<span class="text-sm text-text-muted">{cmd.description}</span>
						</div>
						{#if cmd.permission}
							<span class="hidden shrink-0 rounded border border-border px-2 py-0.5 text-[10px] font-medium text-text-dim sm:inline-block">
								{cmd.permission}
							</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	</div>
</section>
