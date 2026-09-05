<script lang="ts">
	import { Inbox } from 'lucide-svelte';

	let { title = 'Nothing here', description = '' }: { title?: string; description?: string } =
		$props();
</script>

<div
	class="flex flex-col items-center justify-center rounded-2xl border border-border bg-surface p-8 text-center"
>
	<Inbox class="h-10 w-10 text-text-dim" />
	<h3 class="mt-4 text-sm font-bold text-text">{title}</h3>
	{#if description}
		<p class="mt-1 max-w-sm text-sm text-text-muted">{description}</p>
	{/if}
</div>
