<script lang="ts">
	import { onMount } from 'svelte';
	import { Check, Sparkles } from 'lucide-svelte';
	import { listPlans, requestUpgrade } from '$lib/portal/api/plans';
	import { ApiError } from '$lib/portal/api/client';
	import type { Plan } from '$lib/portal/types';
	import { session } from '$lib/portal/stores/session.svelte';
	import { toastStore } from '$lib/portal/stores/toast.svelte';
	import PlanBadge from '$lib/portal/components/PlanBadge.svelte';
	import LoadingBlock from '$lib/portal/components/LoadingBlock.svelte';
	import EmptyState from '$lib/portal/components/EmptyState.svelte';
	import { formatIdr } from '$lib/portal/utils';

	let plans = $state<Plan[]>([]);
	let loading = $state(true);
	let requesting = $state('');
	let error = $state('');

	async function load() {
		loading = true;
		error = '';
		try {
			const data = await listPlans();
			plans = data.plans
				.filter((plan) => plan.is_active)
				.sort((a, b) => a.sort_order - b.sort_order);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load plans.';
		} finally {
			loading = false;
		}
	}

	async function upgrade(planId: string) {
		requesting = planId;
		try {
			const data = await requestUpgrade();
			toastStore.success(data.message || 'Upgrade request submitted.');
		} catch (e) {
			toastStore.error(e instanceof ApiError ? e.message : 'Failed to request upgrade.');
		} finally {
			requesting = '';
		}
	}

	onMount(() => {
		void load();
	});
</script>

<div class="animate-fade-in space-y-6">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
		<div>
			<h1 class="text-3xl font-black tracking-tight text-text">Plans</h1>
			<p class="mt-2 text-sm text-text-muted">
				Choose the quota and data access level that fits your workflow.
			</p>
		</div>
		{#if session.user}
			<div class="flex items-center gap-2 rounded-2xl border border-border bg-surface px-4 py-3">
				<span class="text-sm font-semibold text-text-muted">Current plan</span>
				<PlanBadge plan={session.user.plan} />
			</div>
		{/if}
	</div>

	{#if loading}
		<LoadingBlock label="Loading plans..." />
	{:else if error}
		<EmptyState title="Plans unavailable" description={error} />
	{:else}
		<div class="grid gap-5 lg:grid-cols-2 xl:grid-cols-4">
			{#each plans as plan}
				<section
					class="flex flex-col rounded-2xl border bg-surface p-5 shadow-sm {session.user?.plan ===
					plan.id
						? 'border-accent shadow-accent/10'
						: 'border-border'}"
				>
					<div class="flex items-start justify-between gap-3">
						<div>
							<h2 class="text-xl font-black text-text">{plan.name}</h2>
							<p class="mt-2 text-3xl font-black text-text">{formatIdr(plan.price_idr)}</p>
							<p class="text-sm text-text-dim">per month</p>
						</div>
						{#if session.user?.plan === plan.id}
							<PlanBadge plan={plan.id} />
						{/if}
					</div>

					<div class="mt-5 space-y-3 text-sm text-text-muted">
						<p class="flex items-center gap-2">
							<Check class="h-4 w-4 text-green" />
							{plan.requests_per_day.toLocaleString()} requests/day
						</p>
						<p class="flex items-center gap-2">
							<Check class="h-4 w-4 text-green" />
							{plan.ws_connections} WebSocket connections
						</p>
						<p class="flex items-center gap-2">
							<Check class="h-4 w-4 text-green" />
							{plan.x_usernames_max} X usernames
						</p>
						<p class="flex items-center gap-2">
							<Check class="h-4 w-4 text-green" />
							{plan.tv_symbols_max} TV symbols
						</p>
						<p class="flex items-center gap-2">
							<Check class="h-4 w-4 text-green" />
							{plan.news_history_days} days news history
						</p>
						<p class="flex items-center gap-2">
							<Check class="h-4 w-4 text-green" />
							{plan.rate_limit_per_min} requests/min
						</p>
						{#if plan.can_custom_rss}
							<p class="flex items-center gap-2">
								<Check class="h-4 w-4 text-green" /> Custom RSS feeds
							</p>
						{/if}
					</div>

					<button
						class="mt-6 inline-flex items-center justify-center gap-2 rounded-xl px-4 py-3 text-sm font-bold transition disabled:opacity-60 {session
							.user?.plan === plan.id
							? 'border border-border text-text-muted'
							: 'bg-accent text-white hover:bg-accent-glow'}"
						onclick={() => upgrade(plan.id)}
						disabled={requesting !== '' || session.user?.plan === plan.id}
					>
						<Sparkles class="h-4 w-4" />
						{session.user?.plan === plan.id
							? 'Current plan'
							: requesting === plan.id
								? 'Requesting...'
								: 'Request upgrade'}
					</button>
				</section>
			{/each}
		</div>
	{/if}
</div>
