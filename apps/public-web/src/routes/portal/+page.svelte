<script lang="ts">
	import { onMount } from 'svelte';
	import { Activity, BarChart3, Gauge, KeyRound, TrendingUp } from 'lucide-svelte';
	import { fetchUsageSummary } from '$lib/portal/api/usage';
	import { session } from '$lib/portal/stores/session.svelte';
	import type { UsageSummary } from '$lib/portal/types';
	import PlanBadge from '$lib/portal/components/PlanBadge.svelte';
	import StatCard from '$lib/portal/components/StatCard.svelte';
	import LoadingBlock from '$lib/portal/components/LoadingBlock.svelte';

	let usage = $state<UsageSummary | null>(null);
	let loading = $state(true);
	let error = $state('');

	const usagePct = $derived(
		usage ? Math.min(Math.round((usage.today / usage.daily_limit) * 100), 100) : 0
	);
	const usageTone = $derived(usagePct >= 90 ? 'red' : usagePct >= 70 ? 'amber' : 'green');

	onMount(async () => {
		try {
			usage = await fetchUsageSummary();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load usage data.';
		} finally {
			loading = false;
		}
	});
</script>

<div class="animate-fade-in space-y-6">
	<!-- Header -->
	<div>
		<div class="flex items-center gap-3">
			<h1 class="text-3xl font-black tracking-tight text-text">Dashboard</h1>
			{#if session.user}
				<PlanBadge plan={session.user.plan} />
			{/if}
		</div>
		<p class="mt-2 text-sm text-text-muted">
			Welcome back{session.user ? `, ${session.user.name}` : ''}. Here's your API usage overview.
		</p>
	</div>

	{#if loading}
		<LoadingBlock label="Loading dashboard..." />
	{:else if error}
		<div class="rounded-2xl border border-red/20 bg-red/10 p-4 text-sm text-red">{error}</div>
	{:else if usage}
		<!-- Stats -->
		<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
			<StatCard
				label="Today's requests"
				value={usage.today}
				help="{usage.remaining_today} remaining"
				icon={Activity}
				tone={usageTone}
			/>
			<StatCard label="This week" value={usage.this_week} icon={BarChart3} tone="blue" />
			<StatCard label="This month" value={usage.this_month} icon={TrendingUp} tone="blue" />
			<StatCard
				label="Active API keys"
				value={session.activeKeys}
				help="Manage in API Keys page"
				icon={KeyRound}
				tone="amber"
			/>
		</div>

		<!-- Usage bar -->
		<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
			<div class="flex items-center justify-between">
				<div>
					<h2 class="text-lg font-black text-text">Daily quota</h2>
					<p class="mt-1 text-sm text-text-muted">
						{usage.today.toLocaleString()} / {usage.daily_limit.toLocaleString()} requests used today
					</p>
				</div>
				<div class="flex items-center gap-2">
					<Gauge class="h-4 w-4 text-text-dim" />
					<span class="font-mono text-sm font-bold text-text">{usagePct}%</span>
				</div>
			</div>
			<div class="mt-4 h-3 overflow-hidden rounded-full bg-surface-2">
				<div
					class="h-full rounded-full transition-all duration-500 {usageTone === 'red'
						? 'bg-red'
						: usageTone === 'amber'
							? 'bg-amber'
							: 'bg-green'}"
					style="width: {usagePct}%"
				></div>
			</div>
			{#if usagePct >= 90}
				<p class="mt-3 text-xs font-semibold text-red">
					You're approaching your daily limit. Consider upgrading your plan.
				</p>
			{/if}
		</section>

		<!-- Plan info -->
		{#if session.planLimits}
			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<h2 class="text-lg font-black text-text">Your plan</h2>
				<div class="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
					<div class="rounded-xl bg-surface-2 p-3">
						<p class="text-xs font-bold text-text-dim">Requests/day</p>
						<p class="mt-1 font-mono text-lg font-black text-text">
							{session.planLimits.requests_per_day.toLocaleString()}
						</p>
					</div>
					<div class="rounded-xl bg-surface-2 p-3">
						<p class="text-xs font-bold text-text-dim">WebSocket conn.</p>
						<p class="mt-1 font-mono text-lg font-black text-text">
							{session.planLimits.ws_connections}
						</p>
					</div>
					<div class="rounded-xl bg-surface-2 p-3">
						<p class="text-xs font-bold text-text-dim">Rate limit/min</p>
						<p class="mt-1 font-mono text-lg font-black text-text">
							{session.planLimits.rate_limit_per_min}
						</p>
					</div>
					<div class="rounded-xl bg-surface-2 p-3">
						<p class="text-xs font-bold text-text-dim">News history</p>
						<p class="mt-1 font-mono text-lg font-black text-text">
							{session.planLimits.news_history_days} days
						</p>
					</div>
				</div>
				<div class="mt-4">
					<a href="/portal/plans" class="text-sm font-bold text-accent hover:text-accent-glow">
						View all plans &rarr;
					</a>
				</div>
			</section>
		{/if}

		<!-- Quick links -->
		<div class="grid gap-4 md:grid-cols-3">
			<a
				href="/portal/keys"
				class="rounded-2xl border border-border bg-surface p-5 shadow-sm transition hover:-translate-y-0.5 hover:border-accent/25"
			>
				<KeyRound class="h-5 w-5 text-accent" />
				<p class="mt-3 font-black text-text">Manage API keys</p>
				<p class="mt-1 text-sm text-text-muted">Create, revoke, and label keys.</p>
			</a>
			<a
				href="/portal/settings"
				class="rounded-2xl border border-border bg-surface p-5 shadow-sm transition hover:-translate-y-0.5 hover:border-accent/25"
			>
				<TrendingUp class="h-5 w-5 text-green" />
				<p class="mt-3 font-black text-text">Configure feeds</p>
				<p class="mt-1 text-sm text-text-muted">Set X usernames and TV symbols.</p>
			</a>
			<a
				href="/portal/usage"
				class="rounded-2xl border border-border bg-surface p-5 shadow-sm transition hover:-translate-y-0.5 hover:border-accent/25"
			>
				<BarChart3 class="h-5 w-5 text-amber" />
				<p class="mt-3 font-black text-text">Usage history</p>
				<p class="mt-1 text-sm text-text-muted">Daily request breakdown.</p>
			</a>
		</div>
	{/if}
</div>
