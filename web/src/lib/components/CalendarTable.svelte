<script lang="ts">
	import { calendarEvents, calendarLoading } from '$lib/stores/calendar';

	function formatDate(iso: string): string {
		if (!iso) return '—';
		try {
			const d = new Date(iso);
			return d.toLocaleString('id-ID', {
				weekday: 'short',
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit',
				timeZone: 'Asia/Jakarta'
			}) + ' WIB';
		} catch {
			return iso;
		}
	}

	function impactColor(impact: string): string {
		const lower = impact.toLowerCase();
		if (lower.includes('high') || lower === 'red') return 'text-red';
		if (lower.includes('medium') || lower === 'orange') return 'text-amber';
		return 'text-text-dim';
	}

	function impactBg(impact: string): string {
		const lower = impact.toLowerCase();
		if (lower.includes('high') || lower === 'red') return 'bg-red/10 border-red/20';
		if (lower.includes('medium') || lower === 'orange') return 'bg-amber/10 border-amber/20';
		return 'bg-surface border-border-subtle';
	}
</script>

<section id="calendar" class="px-4 py-12 md:px-8 lg:px-16">
	<div class="mx-auto max-w-7xl">
		<div class="mb-8">
			<h2 class="text-2xl font-bold tracking-tight">Economic Calendar</h2>
			<p class="mt-1 text-sm text-text-muted">High-impact events this week · Source: Forex Factory</p>
		</div>

		{#if $calendarLoading}
			<div class="space-y-2">
				{#each Array(5) as _}
					<div class="h-14 animate-pulse rounded-lg bg-surface"></div>
				{/each}
			</div>
		{:else if $calendarEvents.length === 0}
			<div class="rounded-lg border border-border bg-surface p-12 text-center">
				<p class="text-text-muted">No high-impact events scheduled this week</p>
			</div>
		{:else}
			<div class="overflow-x-auto rounded-lg border border-border bg-surface">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b border-border text-left text-xs font-medium uppercase tracking-wider text-text-dim">
							<th class="px-4 py-3">Currency</th>
							<th class="px-4 py-3">Event</th>
							<th class="hidden px-4 py-3 sm:table-cell">Date</th>
							<th class="px-4 py-3 text-right">Forecast</th>
							<th class="px-4 py-3 text-right">Previous</th>
							<th class="hidden px-4 py-3 text-right md:table-cell">Actual</th>
							<th class="hidden px-4 py-3 text-center lg:table-cell">Impact</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-border-subtle">
						{#each $calendarEvents as ev, i (ev.title + i)}
							<tr class="transition-colors hover:bg-surface-2">
								<td class="px-4 py-3">
									<span class="font-mono font-semibold text-text">{ev.currency}</span>
								</td>
								<td class="max-w-xs px-4 py-3 text-text">{ev.title}</td>
								<td class="hidden px-4 py-3 whitespace-nowrap text-xs text-text-muted sm:table-cell">{formatDate(ev.date)}</td>
								<td class="px-4 py-3 text-right font-mono text-text-muted">{ev.forecast || '—'}</td>
								<td class="px-4 py-3 text-right font-mono text-text-muted">{ev.previous || '—'}</td>
								<td class="hidden px-4 py-3 text-right font-mono text-text-muted md:table-cell">{ev.actual || '—'}</td>
								<td class="hidden px-4 py-3 text-center lg:table-cell">
									<span class="inline-block rounded border px-2 py-0.5 text-[10px] font-semibold uppercase {impactBg(ev.impact)} {impactColor(ev.impact)}">
										{ev.impact}
									</span>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</section>
