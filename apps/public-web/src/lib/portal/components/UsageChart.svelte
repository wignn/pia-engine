<script lang="ts">
	import type { DailyUsage } from '$lib/portal/types';

	let { data, limit = 0 }: { data: DailyUsage[]; limit?: number } = $props();

	const maxValue = $derived(Math.max(limit, ...data.map((d) => d.count), 1));
</script>

<div class="rounded-lg border border-border bg-surface p-5 shadow-sm">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-lg font-black text-text">Daily usage</h2>
			<p class="mt-1 text-sm text-text-muted">Requests over the selected period.</p>
		</div>
		{#if limit > 0}
			<span class="rounded-full bg-surface-2 px-3 py-1 font-mono text-xs font-bold text-text-dim">
				Limit {limit.toLocaleString()}/day
			</span>
		{/if}
	</div>

	{#if data.length === 0}
		<div class="mt-6 rounded-xl bg-surface-2 p-6 text-center text-sm text-text-muted">
			No usage has been recorded for this period.
		</div>
	{:else}
		<div class="mt-6 flex h-64 items-end gap-1 sm:gap-2">
			{#each data as item}
				<div class="group flex min-w-0 flex-1 flex-col items-center gap-2">
					<div class="relative flex h-52 w-full items-end rounded-t bg-surface-2">
						<div
							class="w-full rounded-t bg-accent transition-all duration-300 group-hover:opacity-85"
							style="height: {Math.max((item.count / maxValue) * 100, item.count > 0 ? 3 : 0)}%"
						></div>
						<div
							class="pointer-events-none absolute bottom-full left-1/2 mb-2 hidden -translate-x-1/2 rounded-md border border-border bg-surface px-2.5 py-1 text-xs font-medium text-text shadow-sm group-hover:block"
						>
							{item.count.toLocaleString()} requests
						</div>
					</div>
					<span class="hidden truncate text-[10px] font-semibold text-text-dim sm:block">
						{new Date(item.day).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}
					</span>
				</div>
			{/each}
		</div>
	{/if}
</div>
