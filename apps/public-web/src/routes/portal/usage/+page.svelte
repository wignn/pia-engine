<script lang="ts">
	import { onMount } from 'svelte';
	import { Activity, BarChart3, CalendarDays, Gauge } from 'lucide-svelte';
	import { fetchUsageHistory, fetchUsageSummary } from '$lib/portal/api/usage';
	import type { DailyUsage, UsageSummary } from '$lib/portal/types';
	import UsageChart from '$lib/portal/components/UsageChart.svelte';
	import StatCard from '$lib/portal/components/StatCard.svelte';
	import LoadingBlock from '$lib/portal/components/LoadingBlock.svelte';
	import EmptyState from '$lib/portal/components/EmptyState.svelte';
	import { formatDate } from '$lib/portal/utils';

	let summary = $state<UsageSummary | null>(null);
	let history = $state<DailyUsage[]>([]);
	let days = $state(30);
	let loading = $state(true);
	let error = $state('');

	const usagePct = $derived(
		summary ? Math.min(Math.round((summary.today / summary.daily_limit) * 100), 100) : 0
	);
	const usageTone = $derived(usagePct >= 90 ? 'red' : usagePct >= 70 ? 'amber' : 'green');

	async function load(nextDays = days) {
		loading = true;
		error = '';
		days = nextDays;
		try {
			const [summaryData, historyData] = await Promise.all([
				fetchUsageSummary(),
				fetchUsageHistory(nextDays)
			]);
			summary = summaryData;
			history = historyData.history;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load usage data.';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void load();
	});
</script>

<div class="animate-fade-in space-y-6">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
		<div>
			<h1 class="text-3xl font-black tracking-tight text-text">Usage</h1>
			<p class="mt-2 text-sm text-text-muted">
				Track API request volume and daily quota consumption.
			</p>
		</div>
		<div class="flex gap-2">
			{#each [7, 30, 90] as option}
				<button
					class="rounded-xl border px-3 py-2 text-sm font-bold transition {days === option
						? 'border-accent bg-accent/10 text-accent'
						: 'border-border bg-surface text-text-muted hover:bg-surface-2'}"
					onclick={() => load(option)}
					disabled={loading}
				>
					{option}d
				</button>
			{/each}
		</div>
	</div>

	{#if loading}
		<LoadingBlock label="Loading usage..." />
	{:else if error}
		<EmptyState title="Usage unavailable" description={error} />
	{:else if summary}
		<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
			<StatCard
				label="Today"
				value={summary.today}
				help={`${summary.remaining_today} remaining`}
				icon={Activity}
				tone={usageTone}
			/>
			<StatCard label="This week" value={summary.this_week} icon={BarChart3} tone="blue" />
			<StatCard label="This month" value={summary.this_month} icon={CalendarDays} tone="blue" />
			<StatCard
				label="Daily quota"
				value={summary.daily_limit}
				help={`${usagePct}% used`}
				icon={Gauge}
				tone={usageTone}
			/>
		</div>

		<UsageChart data={history} limit={summary.daily_limit} />

		<section class="overflow-hidden rounded-2xl border border-border bg-surface shadow-sm">
			<div class="border-b border-border bg-surface-2/50 px-4 py-3">
				<h2 class="text-sm font-bold text-text">Daily breakdown</h2>
			</div>
			<div class="divide-y divide-border">
				{#each history as item}
					<div class="flex items-center justify-between px-4 py-3">
						<span class="text-sm font-semibold text-text">{formatDate(item.day)}</span>
						<span class="font-mono text-sm font-bold text-text-muted"
							>{item.count.toLocaleString()}</span
						>
					</div>
				{/each}
			</div>
		</section>
	{/if}
</div>
