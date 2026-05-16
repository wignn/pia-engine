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

<div class="flex flex-col">
	<div class="flex h-10 shrink-0 items-center justify-between border-b border-border bg-surface px-4">
		<h3 class="text-xs font-semibold uppercase tracking-wider text-text">Upcoming Events</h3>
	</div>

	{#if $calendarLoading}
		<div class="p-8 text-center text-sm text-text-muted">
			<div class="animate-pulse">Loading calendar...</div>
		</div>
	{:else if $calendarEvents.length === 0}
		<div class="p-8 text-center text-sm text-text-muted">
			No upcoming events
		</div>
	{:else}
		<div class="max-h-[600px] overflow-auto custom-scrollbar">
			<table class="w-full text-sm">
				<thead class="sticky top-0 z-10 bg-surface">
					<tr class="border-b border-border text-left text-[11px] text-text-dim">
						<th class="py-2 pl-4 pr-2 font-normal">Time</th>
						<th class="py-2 px-2 font-normal">Currency</th>
						<th class="py-2 px-2 font-normal">Impact</th>
						<th class="py-2 px-2 font-normal">Event</th>
						<th class="py-2 px-2 text-right font-normal">Actual</th>
						<th class="py-2 px-2 text-right font-normal">Forecast</th>
						<th class="py-2 pl-2 pr-4 text-right font-normal">Previous</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-border">
					{#each $calendarEvents as ev, i (ev.title + i)}
						<tr class="cursor-pointer transition-colors hover:bg-surface-2 text-text">
							<td class="py-2 pl-4 pr-2 text-xs text-text-muted whitespace-nowrap">
								{formatDate(ev.date)}
							</td>
							<td class="py-2 px-2 font-mono font-semibold">
								{ev.currency}
							</td>
							<td class="py-2 px-2">
								<span class="inline-flex items-center gap-1.5 rounded px-2 py-0.5 text-[10px] uppercase font-medium border {impactBg(ev.impact)}">
									<span class="inline-block h-1.5 w-1.5 shrink-0 rounded-full {impactBg(ev.impact).includes('red') ? 'bg-red' : impactBg(ev.impact).includes('amber') ? 'bg-amber' : 'bg-text-dim'}"></span>
									{ev.impact}
								</span>
							</td>
							<td class="py-2 px-2 text-sm">
								{ev.title}
							</td>
							<td class="py-2 px-2 text-right font-mono text-xs">
								{ev.actual || '—'}
							</td>
							<td class="py-2 px-2 text-right font-mono text-xs">
								{ev.forecast || '—'}
							</td>
							<td class="py-2 pl-2 pr-4 text-right font-mono text-xs text-text-muted">
								{ev.previous || '—'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
