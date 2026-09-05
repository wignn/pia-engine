<script lang="ts">
	import type { OptionsContract } from '$lib/types';
	import { Calendar, Search, Filter, Layers, ArrowUpDown } from 'lucide-svelte';

	interface Props {
		contracts?: OptionsContract[];
		chain?: OptionsContract[];
		underlyingPrice?: number;
	}

	let { contracts, chain, underlyingPrice }: Props = $props();

	let rawContracts = $derived(contracts ?? chain ?? []);

	let selectedExpiration = $state<string>('all');
	let selectedType = $state<'all' | 'call' | 'put'>('all');
	let searchQuery = $state<string>('');

	function getExpiration(c: any): string {
		return c.expiration_date ?? c.expiration ?? c.expire_date ?? c.exp_date ?? '-';
	}
	function getType(c: any): string {
		return (c.option_type ?? c.type ?? c.side ?? c.optionType ?? '').toLowerCase();
	}
	function getStrike(c: any): number {
		return typeof c.strike === 'number' ? c.strike : parseFloat(c.strike ?? c.strike_price ?? 0);
	}
	function getMark(c: any): number | null {
		return c.mark_price ?? c.mark ?? c.price ?? c.last_price ?? null;
	}
	function getIv(c: any): number | null {
		return c.implied_volatility ?? c.iv ?? c.impliedVolatility ?? null;
	}
	function getOi(c: any): number | null {
		return c.open_interest ?? c.oi ?? c.openInterest ?? null;
	}
	function getVol(c: any): number | null {
		return c.volume ?? c.vol ?? null;
	}
	function getGex(c: any): number {
		return c.gex ?? ((c.gamma ?? 0) * (getOi(c) ?? 0) * 100 * (underlyingPrice ?? 1));
	}

	// Unique expiration dates sorted ascending
	let expirationDates = $derived(
		Array.from(new Set(rawContracts.map(getExpiration).filter((d) => d !== '-'))).sort(
			(a, b) => new Date(a).getTime() - new Date(b).getTime()
		)
	);

	// Filtered contracts
	let filteredContracts = $derived(
		rawContracts.filter((c) => {
			const exp = getExpiration(c);
			const optType = getType(c);
			const strike = getStrike(c);
			const symbolStr = (c.contract_symbol ?? c.symbol ?? '').toLowerCase();

			if (selectedExpiration !== 'all' && exp !== selectedExpiration) {
				return false;
			}
			if (selectedType !== 'all' && optType !== selectedType) {
				return false;
			}
			if (searchQuery.trim() !== '') {
				const q = searchQuery.toLowerCase();
				const strikeStr = strike.toString();
				if (!strikeStr.includes(q) && !symbolStr.includes(q)) {
					return false;
				}
			}
			return true;
		})
	);

	// Sorted contracts by strike then option_type
	let sortedContracts = $derived(
		[...filteredContracts].sort((a, b) => {
			const sA = getStrike(a);
			const sB = getStrike(b);
			if (sA !== sB) return sA - sB;
			return getType(a).localeCompare(getType(b));
		})
	);

	function formatCurrency(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '—';
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			minimumFractionDigits: 2,
			maximumFractionDigits: 2
		}).format(val);
	}

	function formatCompact(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '—';
		return new Intl.NumberFormat('en-US', {
			notation: 'compact',
			compactDisplay: 'short',
			maximumFractionDigits: 1
		}).format(val);
	}

	function formatGreek(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '—';
		return val.toFixed(3);
	}

	function formatIv(val: number | null | undefined): string {
		if (val === null || val === undefined || isNaN(val)) return '—';
		const pct = val <= 1 ? val * 100 : val;
		return `${pct.toFixed(1)}%`;
	}

	function isItm(contract: any, undPrice?: number): boolean {
		if (undPrice === undefined) return false;
		const s = getStrike(contract);
		const t = getType(contract);
		if (t === 'call') {
			return undPrice > s;
		}
		if (t === 'put') {
			return undPrice < s;
		}
		return false;
	}
</script>

<div class="flex flex-col rounded-xl border border-border/80 bg-surface shadow-xs overflow-hidden">
	<!-- Header & Filters -->
	<div class="flex flex-wrap items-center justify-between gap-3 border-b border-border/70 bg-surface-2/20 px-5 py-3.5">
		<div class="flex items-center gap-2.5">
			<div class="flex h-7 w-7 items-center justify-center rounded-lg border border-accent/20 bg-accent/10">
				<Layers class="h-4 w-4 text-accent" />
			</div>
			<div>
				<h3 class="text-xs font-bold tracking-wide uppercase text-text">Options Chain & Greeks Monitor</h3>
				<p class="text-[11px] text-text-muted">Real-time strike ladder, greeks sensitivity, and open interest</p>
			</div>
		</div>

		<!-- Filter Bar -->
		<div class="flex flex-wrap items-center gap-2">
			<!-- Search -->
			<div class="relative flex items-center">
				<Search class="absolute left-2.5 h-3.5 w-3.5 text-text-muted" />
				<input
					type="text"
					placeholder="Search strike..."
					bind:value={searchQuery}
					class="h-8 w-32 rounded-lg border border-border bg-surface pl-8 pr-2.5 font-mono text-xs text-text placeholder-text-muted focus:border-accent focus:outline-none shadow-xs"
				/>
			</div>

			<!-- Type Filter -->
			<select
				bind:value={selectedType}
				class="h-8 rounded-lg border border-border bg-surface px-2.5 text-xs font-medium text-text focus:border-accent focus:outline-none shadow-xs"
			>
				<option value="all">All Types</option>
				<option value="call">Calls Only</option>
				<option value="put">Puts Only</option>
			</select>

			<!-- Expiration Dropdown -->
			<div class="relative flex items-center">
				<Calendar class="absolute left-2.5 h-3.5 w-3.5 pointer-events-none text-text-muted" />
				<select
					bind:value={selectedExpiration}
					class="h-8 rounded-lg border border-border bg-surface pl-8 pr-3 text-xs font-medium text-text focus:border-accent focus:outline-none shadow-xs"
				>
					<option value="all">All Expirations ({expirationDates.length})</option>
					{#each expirationDates as expDate}
						<option value={expDate}>{expDate}</option>
					{/each}
				</select>
			</div>
		</div>
	</div>

	<!-- Table Area -->
	<div class="overflow-x-auto">
		<table class="w-full text-left text-xs">
			<thead class="border-b border-border/70 bg-surface-2/40 font-semibold text-text-muted text-[11px]">
				<tr>
					<th class="px-4 py-2.5">Strike</th>
					<th class="px-3 py-2.5">Type</th>
					<th class="px-3 py-2.5">Expiration</th>
					<th class="px-3 py-2.5 text-right">Mark</th>
					<th class="px-3 py-2.5 text-right">Bid / Ask</th>
					<th class="px-3 py-2.5 text-right">IV</th>
					<th class="px-3 py-2.5 text-right">Delta</th>
					<th class="px-3 py-2.5 text-right">Gamma</th>
					<th class="px-3 py-2.5 text-right">Theta</th>
					<th class="px-3 py-2.5 text-right">Vega</th>
					<th class="px-3 py-2.5 text-right">OI</th>
					<th class="px-3 py-2.5 text-right">Vol</th>
					<th class="px-4 py-2.5 text-right">GEX</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-border/50">
				{#if sortedContracts.length === 0}
					<tr>
						<td colspan="13" class="px-4 py-12 text-center text-text-muted">
							No option contracts match selected filters.
						</td>
					</tr>
				{:else}
					{#each sortedContracts as contract (contract.contract_symbol || `${contract.strike}-${contract.option_type}-${contract.expiration_date}`)}
						{@const isCall = getType(contract) === 'call'}
						{@const itm = isItm(contract, underlyingPrice)}
						<tr
							class="transition-colors hover:bg-surface-2/60 {itm && isCall ? 'bg-emerald-500/5' : itm && !isCall ? 'bg-rose-500/5' : ''}"
						>
							<td class="px-4 py-2 font-mono font-bold text-text">
								${contract.strike}
							</td>
							<td class="px-3 py-2">
								<span
									class="inline-block rounded-md px-1.5 py-0.5 text-[10px] font-bold uppercase font-mono {isCall ? 'bg-emerald-500/15 text-emerald-400' : 'bg-rose-500/15 text-rose-400'}"
								>
									{contract.option_type}
								</span>
							</td>
							<td class="px-3 py-2 text-text-muted font-mono text-[11px]">{contract.expiration_date}</td>
							<td class="px-3 py-2 text-right font-mono font-semibold text-text">
								{formatCurrency(contract.mark_price)}
							</td>
							<td class="px-3 py-2 text-right font-mono text-text-dim text-[11px]">
								{formatCurrency(contract.bid)} / {formatCurrency(contract.ask)}
							</td>
							<td class="px-3 py-2 text-right font-mono text-text">{formatIv(contract.implied_volatility)}</td>
							<td class="px-3 py-2 text-right font-mono text-text">{formatGreek(contract.delta)}</td>
							<td class="px-3 py-2 text-right font-mono text-text">{formatGreek(contract.gamma)}</td>
							<td class="px-3 py-2 text-right font-mono text-text">{formatGreek(contract.theta)}</td>
							<td class="px-3 py-2 text-right font-mono text-text">{formatGreek(contract.vega)}</td>
							<td class="px-3 py-2 text-right font-mono text-text-muted">{formatCompact(contract.open_interest)}</td>
							<td class="px-3 py-2 text-right font-mono text-text-muted">{formatCompact(contract.volume)}</td>
							<td
								class="px-4 py-2 text-right font-mono font-semibold"
								class:text-emerald-400={contract.gex > 0}
								class:text-rose-400={contract.gex < 0}
								class:text-text-muted={contract.gex === 0}
							>
								{formatCurrency(contract.gex)}
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>
</div>
