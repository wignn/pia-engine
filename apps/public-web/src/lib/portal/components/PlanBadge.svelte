<script lang="ts">
	let { plan }: { plan: string } = $props();

	const colors: Record<string, string> = {
		free: 'border-text-dim/30 bg-text-dim/10 text-text-dim',
		starter: 'border-accent/30 bg-accent/10 text-accent',
		pro: 'border-green/30 bg-green/10 text-green',
		enterprise: 'border-amber/30 bg-amber/10 text-amber'
	};
</script>

<span
	class="inline-flex items-center rounded-lg border px-2 py-0.5 text-xs font-bold capitalize {colors[
		plan
	] ?? colors.free}"
>
	{plan}
</span>
