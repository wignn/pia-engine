<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { fetchMe } from '$lib/portal/api/auth';
	import { session } from '$lib/portal/stores/session.svelte';
	import PortalShell from '$lib/portal/components/Shell.svelte';
	import Toast from '$lib/portal/components/Toast.svelte';

	let { children } = $props();

	const publicPaths = ['/portal/login', '/portal/register', '/portal/auth'];

	function isPublicPath(path: string) {
		return publicPaths.some((p) => path.startsWith(p));
	}

	onMount(async () => {
		if (isPublicPath(page.url.pathname)) {
			session.setReady(true);
			return;
		}

		session.setLoading(true);
		try {
			const data = await fetchMe();
			session.setFromMe(data);
		} catch {
			session.clear();
			if (!isPublicPath(page.url.pathname)) {
				void goto('/portal/login');
			}
		} finally {
			session.setLoading(false);
		}
	});
</script>

<svelte:head>
	<title>Fio Portal</title>
	<meta name="description" content="Manage your Fio API keys, usage, and settings." />
</svelte:head>

<Toast />

{#if isPublicPath(page.url.pathname)}
	{@render children()}
{:else if !session.ready || session.loading}
	<div class="flex min-h-screen items-center justify-center bg-bg p-4">
		<div class="text-center">
			<div
				class="mx-auto h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent"
			></div>
			<p class="mt-4 text-sm text-text-muted">Loading your session...</p>
		</div>
	</div>
{:else if session.isAuthenticated}
	<PortalShell>
		{@render children()}
	</PortalShell>
{/if}
