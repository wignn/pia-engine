<script lang="ts">
	import type { OptionsSnapshot } from '$lib/types';
	import { DollarSign, Percent, BarChart3, Zap, Layers, AlertCircle } from 'lucide-svelte';

	interface Props {
		snapshot?: OptionsSnapshot | null;
		data?: OptionsSnapshot | null;
	}

	let { snapshot, data }: Props = $props();

	let currentSnapshot = $derived(snapshot ?? data ?? null);

	function formatCurrency(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '$0.00';
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			minimumFractionDigits: 2,
			maximumFractionDigits: 2
		}).format(val);
	}

	function formatCompact(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '0';
		return new Intl.NumberFormat('en-US', {
			notation: 'compact',
			compactDisplay: 'short',
			maximumFractionDigits: 1
		}).format(val);
	}

	function formatGex(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '$0.00';
		const prefix = val > 0 ? '+' : '';
		const formatted = new Intl.NumberFormat('en-US', {
			notation: 'compact',
			compactDisplay: 'short',
			maximumFractionDigits: 2
		}).format(val);
		return `${prefix}$${formatted.replace('$', '')}`;
	}

	function formatPercent(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return 'N/A';
		const pct = val <= 1 ? val * 100 : val;
		return `${pct.toFixed(1)}%`;
	}
</script>

<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
	<!-- Underlying Price -->
	<div class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface p-3.5 shadow-xs transition hover:border-border">
		<div class="flex items-center justify-between">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">Underlying</span>
			<span class="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] font-bold text-text-dim uppercase">
				{currentSnapshot?.symbol ?? '—'}
			</span>
		</div>
		<div class="mt-2 text-xl font-bold tracking-tight text-text font-mono">
			{formatCurrency(currentSnapshot?.underlying_price)}
		</div>
		<div class="mt-1 flex items-center gap-1.5 text-[10px] text-text-dim">
			<span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
			<span>Live Cash Price</span>
		</div>
	</div>

	<!-- Put / Call Ratio -->
	<div class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface p-3.5 shadow-xs transition hover:border-border">
		<div class="flex items-center justify-between">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">PCR (Put/Call)</span>
			<span class="text-[10px] font-semibold text-text-dim">Volume/OI</span>
		</div>
		<div class="mt-2 text-xl font-bold tracking-tight text-text font-mono">
			{currentSnapshot?.put_call_ratio !== undefined && currentSnapshot?.put_call_ratio !== null
				? currentSnapshot.put_call_ratio.toFixed(2)
				: '0.00'}
		</div>
		<div class="mt-1 text-[10px] font-semibold">
			{#if (currentSnapshot?.put_call_ratio ?? 0) > 1}
				<span class="text-red-400">Bearish Sentiment</span>
			{:else if (currentSnapshot?.put_call_ratio ?? 0) > 0}
				<span class="text-emerald-400">Bullish Sentiment</span>
			{:else}
				<span class="text-text-muted">Neutral Equilibrium</span>
			{/if}
		</div>
	</div>

	<!-- Max Pain Strike -->
	<div class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface p-3.5 shadow-xs transition hover:border-border">
		<div class="flex items-center justify-between">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">Max Pain Strike</span>
			<span class="text-[10px] font-semibold text-text-dim">Anchor</span>
		</div>
		<div class="mt-2 text-xl font-bold tracking-tight text-text font-mono">
			{formatCurrency(currentSnapshot?.max_pain_strike)}
		</div>
		<div class="mt-1 text-[10px] text-text-dim">Expiry Magnet Level</div>
	</div>

	<!-- ATM IV -->
	<div class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface p-3.5 shadow-xs transition hover:border-border">
		<div class="flex items-center justify-between">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">ATM Volatility</span>
			<span class="text-[10px] font-semibold text-text-dim">IV</span>
		</div>
		<div class="mt-2 text-xl font-bold tracking-tight text-text font-mono">
			{formatPercent(currentSnapshot?.iv_atm)}
		</div>
		<div class="mt-1 text-[10px] text-text-dim">Implied Pricing Vol</div>
	</div>

	<!-- Total OI & Volume -->
	<div class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface p-3.5 shadow-xs transition hover:border-border">
		<div class="flex items-center justify-between">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">Total OI / Vol</span>
			<span class="text-[10px] font-semibold text-text-dim">Contracts</span>
		</div>
		<div class="mt-2 text-xl font-bold tracking-tight text-text font-mono">
			{formatCompact(currentSnapshot?.total_open_interest)} / {formatCompact(
				currentSnapshot?.total_volume
			)}
		</div>
		<div class="mt-1 text-[10px] text-text-dim">Aggregate Liquidity</div>
	</div>

	<!-- Total Dollar GEX -->
	<div class="flex flex-col justify-between rounded-xl border border-border/80 bg-surface p-3.5 shadow-xs transition hover:border-border">
		<div class="flex items-center justify-between">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">Net Dollar GEX</span>
			<span class="text-[10px] font-semibold text-text-dim">Gamma</span>
		</div>
		<div
			class="mt-2 text-xl font-bold tracking-tight font-mono"
			class:text-emerald-400={(currentSnapshot?.total_gex ?? 0) > 0}
			class:text-red-400={(currentSnapshot?.total_gex ?? 0) < 0}
			class:text-text={(currentSnapshot?.total_gex ?? 0) === 0}
		>
			{formatGex(currentSnapshot?.total_gex)}
		</div>
		<div class="mt-1 text-[10px] text-text-dim">Market Maker Exposure</div>
	</div>
</div>
