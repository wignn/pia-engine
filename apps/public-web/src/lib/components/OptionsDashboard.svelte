<script lang="ts">
	import { RefreshCw, AlertCircle } from 'lucide-svelte';
	import { apiFetch } from '$lib/api';
	import type { OptionsSnapshot, OptionsContract, OptionsGexItem } from '$lib/types';
	import OptionsSummaryCards from '$lib/components/OptionsSummaryCards.svelte';
	import OptionsGexChart from '$lib/components/OptionsGexChart.svelte';
	import OptionChainTable from '$lib/components/OptionChainTable.svelte';

	const SYMBOLS = [
		{ label: 'SPY', value: 'SPY' },
		{ label: 'QQQ', value: 'QQQ' },
		{ label: 'AAPL', value: 'AAPL' },
		{ label: 'NVDA', value: 'NVDA' },
		{ label: 'GOLD', value: 'GLD' },
		{ label: 'BTC', value: 'BTC' },
		{ label: 'ETH', value: 'ETH' }
	] as const;

	let selectedSymbol = $state<string>('SPY');
	let loading = $state<boolean>(false);
	let error = $state<string | null>(null);

	let snapshot = $state<OptionsSnapshot | null>(null);
	let chain = $state<OptionsContract[]>([]);
	let gex = $state<OptionsGexItem[]>([]);

	let requestId = 0;

	async function loadOptionsData(sym: string) {
		const currentId = ++requestId;
		loading = true;
		error = null;

		try {
			const [summaryRes, chainRes, gexRes] = await Promise.all([
				apiFetch(`/api/v1/options/summary?symbol=${encodeURIComponent(sym)}`),
				apiFetch(`/api/v1/options/chain?symbol=${encodeURIComponent(sym)}`),
				apiFetch(`/api/v1/options/gex?symbol=${encodeURIComponent(sym)}`)
			]);

			if (currentId !== requestId) return;

			if (!summaryRes.ok || !chainRes.ok || !gexRes.ok) {
				throw new Error('Failed to load options data');
			}

			const summaryJson = await summaryRes.json();
			const chainJson = await chainRes.json();
			const gexJson = await gexRes.json();

			if (currentId !== requestId) return;

			snapshot = Array.isArray(summaryJson?.data)
				? (summaryJson.data[0] ?? null)
				: (summaryJson?.data ?? null);
			const chainData = chainJson?.data?.chain ?? chainJson?.data?.contracts ?? chainJson?.data ?? chainJson;
				chain = Array.isArray(chainData) ? chainData : [];
			const gexData = gexJson?.data?.items ?? gexJson?.data?.gex ?? gexJson?.data?.strikes ?? gexJson?.data ?? gexJson;
				gex = Array.isArray(gexData) ? gexData : [];
		} catch (err: unknown) {
			if (currentId === requestId) {
				error = err instanceof Error ? err.message : 'An error occurred fetching options data';
			}
		} finally {
			if (currentId === requestId) {
				loading = false;
			}
		}
	}

	$effect(() => {
		loadOptionsData(selectedSymbol);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Top Bar: Title, Symbol Selectors & Refresh -->
	<div class="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
		<div>
			<div class="flex items-center gap-3">
				<h2 class="text-2xl font-black tracking-tight text-text">Options Analytics</h2>
				<span
					class="rounded-md border border-accent/20 bg-accent/5 px-2.5 py-1 font-mono text-[11px] font-bold text-accent shadow-xs"
				>
					GEX & Chains
				</span>
			</div>
			<p class="mt-1 text-sm text-text-muted">
				Real-time options snapshots, gamma exposure, and full contract chains
			</p>
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<!-- Symbol selector tabs -->
			<div class="flex flex-wrap items-center gap-1 rounded-lg border border-border bg-surface p-1 shadow-xs">
				{#each SYMBOLS as sym}
					<button
						onclick={() => (selectedSymbol = sym.value)}
						class="cursor-pointer rounded-md px-3 py-1 text-xs font-bold transition-all {selectedSymbol ===
						sym.value
							? 'border border-border/50 bg-surface-2 text-text shadow-xs'
							: 'text-text-dim hover:bg-surface-2/50 hover:text-text'}"
					>
						{sym.label}
					</button>
				{/each}
			</div>

			<!-- Refresh Button -->
			<button
				onclick={() => loadOptionsData(selectedSymbol)}
				disabled={loading}
				title="Refresh data"
				class="flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-surface text-text-muted transition-colors hover:bg-surface-2 hover:text-text disabled:opacity-50"
			>
				<RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
			</button>
		</div>
	</div>

	<!-- Main Content Area -->
	{#if error && !snapshot && chain.length === 0 && gex.length === 0}
		<div class="flex flex-col items-center justify-center rounded-xl border border-border bg-surface p-8 text-center shadow-xs">
			<AlertCircle class="h-8 w-8 text-red mb-2" />
			<h3 class="text-base font-semibold text-text">Failed to load options data</h3>
			<p class="mt-1 text-xs text-text-muted max-w-md">{error}</p>
			<button
				onclick={() => loadOptionsData(selectedSymbol)}
				class="mt-4 rounded-md bg-accent px-4 py-2 text-xs font-semibold text-white shadow-xs transition-all hover:bg-accent-glow"
			>
				Retry
			</button>
		</div>
	{:else}
		<div class="flex flex-col gap-6">
			<!-- Summary Cards -->
			<OptionsSummaryCards {snapshot} />

			<!-- GEX Chart -->
			<OptionsGexChart gexItems={gex} contracts={chain} underlyingPrice={snapshot?.underlying_price} />

			<!-- Options Chain Table -->
			<OptionChainTable contracts={chain} underlyingPrice={snapshot?.underlying_price} />
		</div>
	{/if}
</div>
