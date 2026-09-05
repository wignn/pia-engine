<script lang="ts">
	import { Check, Copy } from 'lucide-svelte';

	let { value, label = 'Copy' }: { value: string; label?: string } = $props();

	let copied = $state(false);

	async function copy() {
		try {
			await navigator.clipboard.writeText(value);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			// Fallback for older browsers
			const textarea = document.createElement('textarea');
			textarea.value = value;
			textarea.style.position = 'fixed';
			textarea.style.opacity = '0';
			document.body.appendChild(textarea);
			textarea.select();
			document.execCommand('copy');
			document.body.removeChild(textarea);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}
</script>

<button
	type="button"
	class="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1.5 text-xs font-semibold text-text-muted transition hover:bg-surface-2 hover:text-text"
	onclick={copy}
	title={label}
>
	{#if copied}
		<Check class="h-3.5 w-3.5 text-green" />
		<span class="text-green">Copied</span>
	{:else}
		<Copy class="h-3.5 w-3.5" />
		<span>{label}</span>
	{/if}
</button>
