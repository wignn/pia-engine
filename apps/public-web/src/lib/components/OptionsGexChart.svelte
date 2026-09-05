<script lang="ts">
	import type { OptionsGexItem, OptionsContract } from '$lib/types';
	import { BarChart3, TrendingUp, ShieldAlert, Target } from 'lucide-svelte';

	interface Props {
		gexItems?: OptionsGexItem[];
		gex?: OptionsGexItem[];
		items?: OptionsGexItem[];
		contracts?: OptionsContract[];
		underlyingPrice?: number;
		height?: number;
	}

	let { gexItems, gex, items, contracts = [], underlyingPrice, height = 340 }: Props = $props();

	let rawItems = $derived.by(() => {
		const direct = gexItems ?? gex ?? items ?? [];
		if (direct.length > 0) return direct;

		if (contracts.length > 0) {
			const map = new Map<number, { strike: number; call_gex: number; put_gex: number; total_gex: number }>();
			for (const c of contracts) {
				const existing = map.get(c.strike) ?? { strike: c.strike, call_gex: 0, put_gex: 0, total_gex: 0 };
				const contractGex = c.gex ?? ((c.gamma ?? 0) * (c.open_interest ?? 0) * 100 * (underlyingPrice ?? 1));
				if (c.option_type === 'call') {
					existing.call_gex += Math.abs(contractGex);
				} else {
					existing.put_gex += -Math.abs(contractGex);
				}
				existing.total_gex = existing.call_gex + existing.put_gex;
				map.set(c.strike, existing);
			}
			return Array.from(map.values());
		}

		return [];
	});

	let sortedItems = $derived([...rawItems].sort((a, b) => a.strike - b.strike));

	let maxGex = $derived(
		Math.max(
			1,
			...sortedItems.map((item) =>
				Math.max(Math.abs(item.call_gex), Math.abs(item.put_gex), Math.abs(item.total_gex))
			)
		)
	);

	let totalCallGex = $derived(sortedItems.reduce((acc, curr) => acc + Math.abs(curr.call_gex), 0));
	let totalPutGex = $derived(sortedItems.reduce((acc, curr) => acc + -Math.abs(curr.put_gex), 0));
	let netGex = $derived(totalCallGex + totalPutGex);

	function formatGex(val: number): string {
		const prefix = val > 0 ? '+' : '';
		const formatted = new Intl.NumberFormat('en-US', {
			notation: 'compact',
			compactDisplay: 'short',
			maximumFractionDigits: 1
		}).format(val);
		return `${prefix}$${formatted.replace('$', '')}`;
	}

	function formatStrike(val: number): string {
		return val % 1 === 0 ? val.toFixed(0) : val.toFixed(1);
	}
</script>

<div class="flex flex-col rounded-xl border border-border/80 bg-surface shadow-xs overflow-hidden">
	<!-- Header -->
	<div class="flex flex-wrap items-center justify-between gap-3 border-b border-border/70 bg-surface-2/20 px-5 py-3.5">
		<div class="flex items-center gap-2.5">
			<div class="flex h-7 w-7 items-center justify-center rounded-lg border border-accent/20 bg-accent/10">
				<BarChart3 class="h-4 w-4 text-accent" />
			</div>
			<div>
				<h3 class="text-xs font-bold tracking-wide uppercase text-text">Gamma Exposure (GEX) Distribution</h3>
				<p class="text-[11px] text-text-muted">Net dealer hedging requirements by option strike level</p>
			</div>
		</div>

		<!-- Summary Badges & Legend -->
		<div class="flex flex-wrap items-center gap-2.5 text-xs">
			<div class="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1">
				<span class="h-2 w-2 rounded-full bg-emerald-400"></span>
				<span class="text-text-muted">Calls</span>
				<span class="font-mono font-bold text-text ml-1">{formatGex(totalCallGex)}</span>
			</div>
			<div class="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1">
				<span class="h-2 w-2 rounded-full bg-rose-400"></span>
				<span class="text-text-muted">Puts</span>
				<span class="font-mono font-bold text-text ml-1">{formatGex(totalPutGex)}</span>
			</div>
			<div class="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1">
				<span class="text-text-muted">Regime:</span>
				<span class="font-mono font-bold {netGex >= 0 ? 'text-emerald-400' : 'text-rose-400'}">
					{netGex >= 0 ? 'Long Gamma (Stabilizing)' : 'Short Gamma (Volatile)'}
				</span>
			</div>
		</div>
	</div>

	<!-- Chart Area -->
	{#if sortedItems.length === 0}
		<div
			class="flex items-center justify-center text-xs font-semibold text-text-muted"
			style="height: {height}px"
		>
			No GEX strike data available for this underlying
		</div>
	{:else}
		<div class="relative flex flex-col gap-1 overflow-x-auto px-5 py-6" style="height: {height}px">
			<div class="flex h-full min-w-[640px] flex-1 items-end gap-2.5 pt-4 pb-4">
				{#each sortedItems as item (item.strike)}
					{@const callHeightPct = Math.min(100, (Math.abs(item.call_gex) / maxGex) * 100)}
					{@const putHeightPct = Math.min(100, (Math.abs(item.put_gex) / maxGex) * 100)}
					{@const isAtm =
						underlyingPrice !== undefined &&
						Math.abs(item.strike - underlyingPrice) ===
							Math.min(...sortedItems.map((i) => Math.abs(i.strike - underlyingPrice)))}

					<div class="group relative flex h-full flex-1 flex-col items-center justify-end">
						<!-- Hover Tooltip -->
						<div
							class="pointer-events-none absolute bottom-full z-20 mb-3 hidden w-44 rounded-lg border border-border/80 bg-surface-2/95 p-2.5 text-xs shadow-lg backdrop-blur-sm group-hover:block"
						>
							<div class="flex items-center justify-between border-b border-border/60 pb-1.5">
								<span class="font-mono font-bold text-text">Strike ${formatStrike(item.strike)}</span>
								{#if isAtm}
									<span class="rounded bg-accent/20 px-1 py-0.2 text-[9px] font-bold text-accent">ATM</span>
								{/if}
							</div>
							<div class="mt-2 flex items-center justify-between">
								<span class="flex items-center gap-1 text-[11px] text-text-muted">
									<span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span> Call GEX
								</span>
								<span class="font-mono text-emerald-400 font-semibold">{formatGex(item.call_gex)}</span>
							</div>
							<div class="mt-1 flex items-center justify-between">
								<span class="flex items-center gap-1 text-[11px] text-text-muted">
									<span class="h-1.5 w-1.5 rounded-full bg-rose-400"></span> Put GEX
								</span>
								<span class="font-mono text-rose-400 font-semibold">{formatGex(item.put_gex)}</span>
							</div>
							<div class="mt-1.5 flex items-center justify-between border-t border-border/60 pt-1 font-medium text-text">
								<span class="text-[11px]">Net GEX</span>
								<span class="font-mono font-bold {item.total_gex >= 0 ? 'text-emerald-400' : 'text-rose-400'}">
									{formatGex(item.total_gex)}
								</span>
							</div>
						</div>

						<!-- Underlying ATM marker indicator -->
						{#if isAtm}
							<div
								class="absolute -top-3.5 left-1/2 -translate-x-1/2 rounded-full border border-accent/40 bg-accent/15 px-2 py-0.5 text-[9.5px] font-bold font-mono whitespace-nowrap text-accent shadow-xs"
							>
								ATM ${underlyingPrice?.toFixed(1)}
							</div>
						{/if}

						<!-- Bars Container -->
						<div
							class="mx-auto flex w-full max-w-[20px] items-end justify-center gap-1"
							style="height: 75%"
						>
							<!-- Call Bar -->
							<div
								class="w-1/2 rounded-t-sm bg-emerald-500/85 transition-all duration-200 group-hover:bg-emerald-400 group-hover:shadow-[0_0_8px_rgba(16,185,129,0.4)]"
								style="height: {Math.max(callHeightPct, 2)}%"
							></div>
							<!-- Put Bar -->
							<div
								class="w-1/2 rounded-t-sm bg-rose-500/85 transition-all duration-200 group-hover:bg-rose-400 group-hover:shadow-[0_0_8px_rgba(244,63,94,0.4)]"
								style="height: {Math.max(putHeightPct, 2)}%"
							></div>
						</div>

						<!-- X-Axis Strike Label -->
						<div
							class="mt-2.5 text-[10px] font-mono transition-colors text-text-dim group-hover:text-text"
							class:font-bold={isAtm}
							class:text-accent={isAtm}
						>
							${formatStrike(item.strike)}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
