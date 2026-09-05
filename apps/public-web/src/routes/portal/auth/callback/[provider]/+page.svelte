<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { oauthCallback } from '$lib/portal/api/auth';
	import { ApiError } from '$lib/portal/api/client';
	import { session } from '$lib/portal/stores/session.svelte';
	import { toastStore } from '$lib/portal/stores/toast.svelte';

	let error = $state('');
	let processing = $state(true);

	onMount(async () => {
		const provider = page.params.provider;
		const code = page.url.searchParams.get('code');
		const state = page.url.searchParams.get('state');

		if (!code || !state || !provider) {
			error = 'Missing OAuth parameters. Please try again.';
			processing = false;
			return;
		}

		try {
			const data = await oauthCallback(provider, code, state);

			if ('error' in data && (data as any).error) {
				error = (data as any).error;
				processing = false;
				return;
			}

			session.setFromLogin(data);

			if (data.api_key) {
				toastStore.success('Welcome! Your first API key has been generated.');
			}

			void goto('/portal');
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'OAuth authentication failed.';
			processing = false;
		}
	});
</script>

<div class="flex min-h-screen items-center justify-center bg-bg px-4 py-12 text-text">
	<div class="animate-fade-in w-full max-w-sm text-center">
		{#if processing}
			<div
				class="mx-auto h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent"
			></div>
			<p class="mt-4 text-sm font-semibold text-text-muted">Completing authentication...</p>
		{:else if error}
			<div class="rounded-2xl border border-red/20 bg-red/10 p-6">
				<p class="text-sm font-semibold text-red">{error}</p>
				<a
					href="/portal/login"
					class="mt-4 inline-block rounded-lg bg-accent px-4 py-2 text-sm font-bold text-white hover:bg-accent-glow"
				>
					Back to login
				</a>
			</div>
		{/if}
	</div>
</div>
