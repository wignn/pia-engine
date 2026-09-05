<script lang="ts">
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { apiFetch } from '$lib/api';
	import type { WhyMoveCause, WhyMoveResponse } from '$lib/types';

	interface Props {
		symbol: string;
	}

	let { symbol }: Props = $props();

	let data = $state<WhyMoveResponse | null>(null);
	let loading = $state(true);
	let error = $state('');
	let loadedSymbol = $state('');

	let topCauses = $derived(data?.causes.news.slice(0, 3) ?? []);
	let llmNarrative = $derived(data?.llm?.status === 'generated' ? data.llm.narrative : null);
	let topCrossAssets = $derived(data?.cross_assets?.slice(0, 3) ?? []);
	let confidenceLabel = $derived(
		typeof data?.confidence === 'string' ? data.confidence : data?.confidence?.label
	);
	let driverLabels = $derived(
		(llmNarrative?.drivers ?? data?.drivers ?? []).map((driver) =>
			typeof driver === 'string' ? driver : driver.name
		)
	);
	let matchedTerms = $derived(
		data?.matched_terms ?? [
			...new Set((data?.causes.news ?? []).flatMap((cause) => cause.matched_terms ?? []))
		]
	);
	let fallbackSummary = $derived(
		data?.explanation ?? data?.summary ?? 'No explanation available yet.'
	);
	let moveTone = $derived(
		data?.move?.direction === 'up'
			? 'text-green'
			: data?.move?.direction === 'down'
				? 'text-red'
				: 'text-text'
	);
	let confidenceClass = $derived(confidenceTone(confidenceLabel));

	function confidenceTone(confidence: string | undefined) {
		if (confidence === 'high')
			return 'border-green/30 bg-green/10 text-green';
		if (confidence === 'medium')
			return 'border-amber/30 bg-amber/10 text-amber';
		return 'border-border/80 bg-surface-2 text-text-muted';
	}

	function sentimentTone(sentiment: string | null) {
		const value = sentiment?.toLowerCase();
		if (value === 'positive' || value === 'bullish')
			return 'border-green/20 bg-green/10 text-green';
		if (value === 'negative' || value === 'bearish') return 'border-red/20 bg-red/10 text-red';
		if (value === 'mixed') return 'border-amber/20 bg-amber/10 text-amber';
		return 'border-border bg-surface-2 text-text-muted';
	}

	function formatMove(value: number | null | undefined) {
		if (value == null) return '—';
		return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
	}

	function formatNumber(value: number | null | undefined) {
		if (value == null) return '—';
		return value.toLocaleString(undefined, { maximumFractionDigits: 5 });
	}

	function formatTime(value: string | null | undefined) {
		if (!value) return '';
		return new Intl.DateTimeFormat('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		}).format(new Date(value));
	}

	function causeTitle(cause: WhyMoveCause) {
		return cause.title || cause.summary || 'Untitled catalyst';
	}

	async function loadWhy(sym: string) {
		const upper = sym.trim().toUpperCase();
		if (!upper || upper === loadedSymbol) return;
		loading = true;
		error = '';
		try {
			const res = await apiFetch(`/api/v1/market/why/${encodeURIComponent(upper)}?window=5m`);
			if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
			data = (await res.json()) as WhyMoveResponse;
			loadedSymbol = upper;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load explanation.';
			data = null;
			loadedSymbol = '';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadWhy(symbol);
	});

	$effect(() => {
		void loadWhy(symbol);
	});
</script>

<section class="flex h-full flex-col">
	<div
		class="flex items-center justify-between gap-4 border-b border-border/60 bg-surface-2/30 p-5"
	>
		<div class="flex items-center gap-3">
			<div
				class="flex h-10 w-10 items-center justify-center rounded-xl border border-accent/20 bg-accent/10 text-accent shadow-inner"
			>
				<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
				</svg>
			</div>
			<div>
				<p class="mb-0.5 text-[10px] font-bold tracking-[0.2em] text-text-dim uppercase">
					Why did it move?
				</p>
				<h3 class="text-lg font-black tracking-tight text-text">
					{symbol.toUpperCase()} Catalyst Scan
				</h3>
			</div>
		</div>
		{#if data}
			<span
				class="rounded-lg border px-3 py-1.5 text-[10px] font-black tracking-wider uppercase {confidenceClass}"
				>{confidenceLabel}</span
			>
		{/if}
	</div>

	<div class="flex flex-1 flex-col p-5">
		{#if loading}
			<div class="space-y-4">
				<div class="flex gap-4">
					<div class="h-24 w-1/3 animate-pulse rounded-xl bg-surface-2"></div>
					<div class="h-24 w-2/3 animate-pulse rounded-xl bg-surface-2"></div>
				</div>
				<div class="h-16 w-full animate-pulse rounded-xl bg-surface-2"></div>
			</div>
		{:else if error}
			<div
				class="flex items-start gap-3 rounded-xl border border-red/30 bg-red/5 p-4 text-sm font-medium text-red"
			>
				<svg class="mt-0.5 h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
					/>
				</svg>
				Unable to load explanation: {error}
			</div>
		{:else if data}
			<div class="animate-fade-in-up grid gap-5 md:grid-cols-[0.8fr_1.2fr]">
				<div
					class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface-2/40 p-4 shadow-sm transition-all hover:border-border hover:bg-surface-2/60"
				>
					<div>
						<p
							class="flex items-center gap-1.5 text-[11px] font-bold tracking-wider text-text-dim uppercase"
						>
							<span class="h-1.5 w-1.5 rounded-full bg-text-dim"></span>
							5m Move Analysis
						</p>
						<p class="mt-3 font-mono text-3xl font-black tracking-tighter {moveTone}">
							{formatMove(data.move?.move_pct)}
						</p>
					</div>
					<div
						class="mt-4 space-y-2.5 rounded-lg border border-border/40 bg-surface/50 p-3 text-xs text-text-muted"
					>
						<div class="flex items-center justify-between">
							<span class="font-medium">Latest</span>
							<b class="rounded bg-surface-2 px-1.5 font-mono text-text"
								>{formatNumber(data.move?.latest_price)}</b
							>
						</div>
						<div class="flex items-center justify-between">
							<span class="font-medium">Baseline</span>
							<b class="rounded bg-surface-2 px-1.5 font-mono text-text"
								>{formatNumber(data.move?.baseline_price)}</b
							>
						</div>
						<div class="flex items-center justify-between">
							<span class="font-medium">Tick Vol</span>
							<b class="rounded bg-surface-2 px-1.5 font-mono text-text"
								>{data.move?.tick_count ?? 0}</b
							>
						</div>
					</div>
				</div>

				<div class="flex flex-col gap-4">
					{#if llmNarrative}
						<div
							class="group relative overflow-hidden rounded-xl border border-accent/20 bg-accent/5 p-4 shadow-sm"
						>
							<div
								class="absolute inset-0 bg-gradient-to-br from-accent/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100"
							></div>
							<p
								class="relative z-10 flex items-center gap-1.5 text-[10px] font-black tracking-[0.15em] text-accent uppercase"
							>
								<svg
									class="h-3.5 w-3.5"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2.5"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
								</svg>
								AI Analyst Note
							</p>
							<p
								class="relative z-10 mt-2.5 text-base leading-snug font-black tracking-tight text-text"
							>
								{llmNarrative.headline}
							</p>
							<p class="relative z-10 mt-2 text-sm leading-relaxed font-medium text-text-muted">
								{llmNarrative.explanation}
							</p>
						</div>
					{:else}
						<div class="rounded-xl border border-border/80 bg-surface-2/40 p-4">
							<p class="text-sm leading-relaxed font-medium text-text-muted">{fallbackSummary}</p>
						</div>
					{/if}

					{#if driverLabels.length > 0 || matchedTerms.length > 0}
						<div class="flex flex-col gap-2">
							{#if driverLabels.length > 0}
								<div class="flex flex-wrap gap-2">
									<span
										class="mr-1 py-1 text-[10px] font-bold tracking-wider text-text-dim uppercase"
										>Drivers:</span
									>
									{#each driverLabels.slice(0, 4) as driver (driver)}
										<span
											class="rounded-lg border border-green/30 bg-green/10 px-2.5 py-1 text-[10px] font-black tracking-wide text-green uppercase shadow-[0_2px_8px_rgba(34,197,94,0.1)] transition-transform hover:scale-105"
											>{driver}</span
										>
									{/each}
								</div>
							{/if}
							{#if matchedTerms.length > 0}
								<div class="flex flex-wrap gap-2">
									<span
										class="mr-1 py-1 text-[10px] font-bold tracking-wider text-text-dim uppercase"
										>Signals:</span
									>
									{#each matchedTerms.slice(0, 5) as term (term)}
										<span
											class="rounded-lg border border-accent/20 bg-accent/10 px-2.5 py-1 text-[10px] font-black tracking-wide text-accent uppercase shadow-sm transition-transform hover:scale-105"
											>{term}</span
										>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</section>
