<script lang="ts">
	import { toastStore } from '$lib/portal/stores/toast.svelte';
	import { CheckCircle2, Info, X, XCircle } from 'lucide-svelte';

	const icons = { success: CheckCircle2, error: XCircle, info: Info };
	const tones = {
		success: 'border-green/30 bg-green/10 text-green',
		error: 'border-red/30 bg-red/10 text-red',
		info: 'border-accent/30 bg-accent/10 text-accent'
	};
</script>

{#if toastStore.toasts.length > 0}
	<div class="fixed top-4 right-4 z-50 flex flex-col gap-2">
		{#each toastStore.toasts as toast (toast.id)}
			{@const Icon = icons[toast.type]}
			<div
				class="animate-fade-in flex items-center gap-3 rounded-xl border px-4 py-3 shadow-lg backdrop-blur-sm {tones[
					toast.type
				]}"
			>
				<Icon class="h-4 w-4 shrink-0" />
				<p class="text-sm font-semibold">{toast.message}</p>
				<button
					type="button"
					class="ml-2 rounded p-0.5 opacity-60 hover:opacity-100"
					onclick={() => toastStore.dismiss(toast.id)}
				>
					<X class="h-3.5 w-3.5" />
				</button>
			</div>
		{/each}
	</div>
{/if}
