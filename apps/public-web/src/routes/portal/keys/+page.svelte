<script lang="ts">
	import { onMount } from 'svelte';
	import { KeyRound, Plus, RefreshCw, Trash2 } from 'lucide-svelte';
	import { listKeys, createKey, revokeKey, updateKeyLabel } from '$lib/portal/api/keys';
	import { ApiError } from '$lib/portal/api/client';
	import type { ApiKeyInfo } from '$lib/portal/types';
	import { toastStore } from '$lib/portal/stores/toast.svelte';
	import { sanitizeInput, formatDateTime, relativeDate } from '$lib/portal/utils';
	import CopyButton from '$lib/portal/components/CopyButton.svelte';
	import ConfirmDialog from '$lib/portal/components/ConfirmDialog.svelte';
	import LoadingBlock from '$lib/portal/components/LoadingBlock.svelte';
	import EmptyState from '$lib/portal/components/EmptyState.svelte';
	import StatusBadge from '$lib/portal/components/StatusBadge.svelte';

	let keys = $state<ApiKeyInfo[]>([]);
	let loading = $state(true);
	let error = $state('');

	// Create key state
	let showCreate = $state(false);
	let newLabel = $state('');
	let creating = $state(false);
	let newRawKey = $state('');

	// Revoke state
	let revokeTarget = $state<ApiKeyInfo | null>(null);
	let confirmRevokeOpen = $state(false);

	// Rename state
	let editingId = $state<string | null>(null);
	let editLabel = $state('');

	const activeKeys = $derived(keys.filter((k) => k.is_active));
	const revokedKeys = $derived(keys.filter((k) => !k.is_active));

	async function load() {
		loading = true;
		error = '';
		try {
			const data = await listKeys();
			keys = data.keys;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load API keys.';
		} finally {
			loading = false;
		}
	}

	async function handleCreate() {
		const label = sanitizeInput(newLabel, 80) || undefined;
		creating = true;
		try {
			const data = await createKey(label);
			newRawKey = data.api_key;
			toastStore.success("API key created. Copy it now — it won't be shown again.");
			await load();
			newLabel = '';
		} catch (e) {
			toastStore.error(e instanceof ApiError ? e.message : 'Failed to create key.');
		} finally {
			creating = false;
		}
	}

	async function handleRevoke() {
		if (!revokeTarget) return;
		try {
			await revokeKey(revokeTarget.id);
			toastStore.success('API key revoked.');
			await load();
		} catch (e) {
			toastStore.error(e instanceof ApiError ? e.message : 'Failed to revoke key.');
		}
	}

	async function handleRename(keyId: string) {
		const label = sanitizeInput(editLabel, 80);
		if (!label) return;
		try {
			await updateKeyLabel(keyId, label);
			toastStore.success('Key label updated.');
			editingId = null;
			await load();
		} catch (e) {
			toastStore.error(e instanceof ApiError ? e.message : 'Failed to update label.');
		}
	}

	onMount(() => {
		void load();
	});
</script>

<div class="animate-fade-in space-y-6">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
		<div>
			<h1 class="text-3xl font-black tracking-tight text-text">API Keys</h1>
			<p class="mt-2 text-sm text-text-muted">
				Create and manage API keys for authenticating with Fio services. Max 10 active keys.
			</p>
		</div>
		<div class="flex gap-2">
			<button
				class="inline-flex items-center gap-2 rounded-xl bg-accent px-4 py-2 text-sm font-bold text-white shadow-md shadow-accent/15 transition hover:bg-accent-glow disabled:opacity-60"
				onclick={() => {
					showCreate = !showCreate;
					newRawKey = '';
				}}
				disabled={activeKeys.length >= 10}
			>
				<Plus class="h-4 w-4" /> New key
			</button>
			<button
				class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-3 py-2 text-sm font-semibold text-text-muted transition hover:bg-surface-2"
				onclick={load}
				disabled={loading}
			>
				<RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
			</button>
		</div>
	</div>

	<!-- Create key form -->
	{#if showCreate}
		<div class="animate-fade-in rounded-2xl border border-accent/20 bg-accent/5 p-5">
			<h3 class="text-sm font-bold text-text">Create new API key</h3>
			<div class="mt-3 flex gap-2">
				<input
					type="text"
					bind:value={newLabel}
					placeholder="Key label (optional)"
					maxlength="80"
					class="flex-1 rounded-xl border border-border bg-surface px-3 py-2 text-sm text-text outline-none placeholder:text-text-dim focus:border-accent"
				/>
				<button
					class="rounded-xl bg-accent px-4 py-2 text-sm font-bold text-white hover:bg-accent-glow disabled:opacity-60"
					onclick={handleCreate}
					disabled={creating}
				>
					{creating ? 'Creating...' : 'Create'}
				</button>
			</div>

			{#if newRawKey}
				<div class="mt-4 rounded-xl border border-green/30 bg-green/10 p-4">
					<p class="text-xs font-bold text-green">Save this key now. It will not be shown again.</p>
					<div class="mt-2 flex items-center gap-2">
						<code class="flex-1 truncate rounded-lg bg-surface p-2 font-mono text-xs text-text">
							{newRawKey}
						</code>
						<CopyButton value={newRawKey} label="Copy key" />
					</div>
				</div>
			{/if}
		</div>
	{/if}

	{#if loading}
		<LoadingBlock label="Loading API keys..." />
	{:else if error}
		<EmptyState title="Keys unavailable" description={error} />
	{:else if keys.length === 0}
		<EmptyState
			title="No API keys"
			description="Create your first API key to start using the API."
		/>
	{:else}
		<!-- Active keys -->
		<section class="overflow-hidden rounded-2xl border border-border bg-surface shadow-sm">
			<div class="border-b border-border bg-surface-2/50 px-4 py-3">
				<p class="text-sm font-bold text-text">
					{activeKeys.length} active key{activeKeys.length !== 1 ? 's' : ''}
				</p>
			</div>
			<div class="divide-y divide-border">
				{#each activeKeys as key}
					<div class="flex items-center justify-between gap-4 p-4 transition hover:bg-surface-2/30">
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2">
								<KeyRound class="h-4 w-4 shrink-0 text-accent" />
								{#if editingId === key.id}
									<form
										class="flex items-center gap-2"
										onsubmit={(e) => {
											e.preventDefault();
											void handleRename(key.id);
										}}
									>
										<input
											type="text"
											bind:value={editLabel}
											class="rounded-lg border border-accent bg-surface px-2 py-1 text-sm outline-none"
											maxlength="80"
										/>
										<button type="submit" class="text-xs font-bold text-accent">Save</button>
										<button
											type="button"
											class="text-xs text-text-dim"
											onclick={() => (editingId = null)}>Cancel</button
										>
									</form>
								{:else}
									<button
										type="button"
										class="truncate text-sm font-bold text-text hover:text-accent"
										onclick={() => {
											editingId = key.id;
											editLabel = key.label;
										}}
										title="Click to rename"
									>
										{key.label}
									</button>
								{/if}
							</div>
							<div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-text-dim">
								<code class="rounded bg-surface-2 px-1.5 py-0.5 font-mono"
									>{key.key_prefix}••••</code
								>
								<span>Created {relativeDate(key.created_at)}</span>
								{#if key.last_used_at}
									<span>· Last used {relativeDate(key.last_used_at)}</span>
								{:else}
									<span>· Never used</span>
								{/if}
							</div>
						</div>
						<button
							class="shrink-0 rounded-lg border border-border p-2 text-text-dim transition hover:border-red/30 hover:bg-red/10 hover:text-red"
							title="Revoke key"
							onclick={() => {
								revokeTarget = key;
								confirmRevokeOpen = true;
							}}
						>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
				{/each}
			</div>
		</section>

		<!-- Revoked keys -->
		{#if revokedKeys.length > 0}
			<section
				class="overflow-hidden rounded-2xl border border-border bg-surface opacity-60 shadow-sm"
			>
				<div class="border-b border-border bg-surface-2/50 px-4 py-3">
					<p class="text-sm font-bold text-text-muted">{revokedKeys.length} revoked</p>
				</div>
				<div class="divide-y divide-border">
					{#each revokedKeys as key}
						<div class="flex items-center gap-3 p-4">
							<KeyRound class="h-4 w-4 text-text-dim" />
							<span class="text-sm text-text-dim line-through">{key.label}</span>
							<StatusBadge tone="red" label="Revoked" />
							<code
								class="ml-auto rounded bg-surface-2 px-1.5 py-0.5 font-mono text-xs text-text-dim"
								>{key.key_prefix}••••</code
							>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{/if}
</div>

<ConfirmDialog
	bind:open={confirmRevokeOpen}
	title="Revoke API key"
	description="Revoke key '{revokeTarget?.label ??
		''}'? This action cannot be undone. Any application using this key will lose access immediately."
	confirmLabel="Revoke key"
	confirmTone="red"
	onconfirm={handleRevoke}
/>
