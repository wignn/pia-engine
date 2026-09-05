<script lang="ts">
	let {
		label,
		value,
		help,
		tone = 'blue',
		icon: Icon
	}: {
		label: string;
		value: string | number;
		help?: string;
		tone?: 'blue' | 'green' | 'amber' | 'red' | 'neutral';
		icon?: any;
	} = $props();

	const toneClass: Record<string, string> = {
		blue: 'from-accent/15 text-accent ring-accent/15',
		green: 'from-green/15 text-green ring-green/15',
		amber: 'from-amber/15 text-amber ring-amber/15',
		red: 'from-red/15 text-red ring-red/15',
		neutral: 'from-text/10 text-text-muted ring-border'
	};
</script>

<div
	class="group relative overflow-hidden rounded-2xl border border-border bg-surface p-4 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
>
	<div class="flex items-start justify-between gap-3">
		<div class="min-w-0">
			<p class="text-xs font-semibold text-text-muted">{label}</p>
			<p class="mt-1.5 truncate text-2xl font-black tracking-tight text-text">{value}</p>
			{#if help}
				<p class="mt-1.5 text-xs text-text-dim">{help}</p>
			{/if}
		</div>
		{#if Icon}
			<div class="rounded-xl bg-gradient-to-br {toneClass[tone]} to-transparent p-2.5 ring-1">
				<Icon class="h-4 w-4" />
			</div>
		{/if}
	</div>
</div>
