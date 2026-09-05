<script lang="ts">
	let { label = 'Loading...' }: { label?: string } = $props();
</script>

<div
	class="flex flex-col items-center justify-center rounded-2xl border border-border bg-surface p-8"
>
	<div class="h-6 w-6 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
	<p class="mt-3 text-sm font-semibold text-text-muted">{label}</p>
</div>
