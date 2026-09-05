<script lang="ts">
	let {
		open = $bindable(false),
		title,
		description,
		confirmLabel = 'Confirm',
		confirmTone = 'blue',
		onconfirm
	}: {
		open: boolean;
		title: string;
		description: string;
		confirmLabel?: string;
		confirmTone?: 'blue' | 'red';
		onconfirm: () => void | Promise<void>;
	} = $props();

	let busy = $state(false);

	async function confirm() {
		busy = true;
		try {
			await onconfirm();
			open = false;
		} finally {
			busy = false;
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 px-4 backdrop-blur-sm"
		onkeydown={(e) => e.key === 'Escape' && (open = false)}
	>
		<div
			class="animate-fade-in w-full max-w-md rounded-2xl border border-border bg-surface p-5 shadow-xl"
		>
			<h2 class="text-lg font-bold text-text">{title}</h2>
			<p class="mt-2 text-sm leading-relaxed text-text-muted">{description}</p>
			<div class="mt-6 flex justify-end gap-2">
				<button
					type="button"
					class="rounded-lg border border-border bg-surface px-4 py-2 text-sm font-semibold text-text-muted transition hover:bg-surface-2 hover:text-text"
					onclick={() => (open = false)}
					disabled={busy}
				>
					Cancel
				</button>
				<button
					type="button"
					class="rounded-lg px-4 py-2 text-sm font-semibold text-white transition disabled:opacity-60 {confirmTone ===
					'red'
						? 'bg-red hover:bg-red/90'
						: 'bg-accent hover:bg-accent-glow'}"
					onclick={confirm}
					disabled={busy}
				>
					{busy ? 'Working...' : confirmLabel}
				</button>
			</div>
		</div>
	</div>
{/if}
