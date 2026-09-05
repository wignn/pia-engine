<script lang="ts">
	let {
		tone = 'neutral',
		label
	}: {
		tone?: 'green' | 'red' | 'amber' | 'blue' | 'neutral';
		label: string;
	} = $props();

	const tones: Record<string, string> = {
		green: 'border-green/25 bg-green/10 text-green',
		red: 'border-red/25 bg-red/10 text-red',
		amber: 'border-amber/25 bg-amber/10 text-amber',
		blue: 'border-accent/25 bg-accent/10 text-accent',
		neutral: 'border-border bg-surface-2 text-text-muted'
	};
</script>

<span
	class="inline-flex items-center rounded-lg border px-2 py-0.5 text-xs font-bold {tones[tone]}"
>
	{label}
</span>
