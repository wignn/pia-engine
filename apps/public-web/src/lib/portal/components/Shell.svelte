<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import {
		BarChart3,
		KeyRound,
		LayoutDashboard,
		LogOut,
		Menu,
		Moon,
		Settings,
		Sun,
		TrendingUp,
		User,
		WalletCards,
		X
	} from 'lucide-svelte';
	import { session } from '$lib/portal/stores/session.svelte';

	let { children }: { children: import('svelte').Snippet } = $props();

	let isDark = $state(false);
	let mobileOpen = $state(false);

	const nav = [
		{ href: '/portal', label: 'Dashboard', icon: LayoutDashboard },
		{ href: '/portal/keys', label: 'API Keys', icon: KeyRound },
		{ href: '/portal/usage', label: 'Usage', icon: BarChart3 },
		{ href: '/portal/settings', label: 'Settings', icon: Settings },
		{ href: '/portal/plans', label: 'Plans', icon: WalletCards },
		{ href: '/portal/account', label: 'Account', icon: User }
	];

	onMount(() => {
		const stored = localStorage.getItem('fio-theme');
		if (
			stored === 'dark' ||
			(!stored && window.matchMedia('(prefers-color-scheme: dark)').matches)
		) {
			isDark = true;
			document.documentElement.classList.add('dark');
		}
	});

	function toggleTheme() {
		isDark = !isDark;
		document.documentElement.classList.toggle('dark', isDark);
		localStorage.setItem('fio-theme', isDark ? 'dark' : 'light');
	}

	function handleLogout() {
		session.logout();
	}

	function isActive(href: string) {
		if (href === '/portal') return page.url.pathname === '/portal';
		return page.url.pathname.startsWith(href);
	}
</script>

<div class="min-h-screen bg-bg text-text">
	<!-- Header -->
	<header class="sticky top-0 z-40 border-b border-border bg-surface/90 shadow-sm backdrop-blur-xl">
		<div class="mx-auto flex h-14 max-w-[1400px] items-center justify-between gap-4 px-4 lg:px-6">
			<div class="flex items-center gap-3">
				<button
					type="button"
					class="rounded-lg p-2 text-text-muted hover:bg-surface-2 lg:hidden"
					onclick={() => (mobileOpen = !mobileOpen)}
				>
					{#if mobileOpen}
						<X class="h-5 w-5" />
					{:else}
						<Menu class="h-5 w-5" />
					{/if}
				</button>
				<a href="/portal" class="flex items-center gap-2">
					<div class="flex h-8 w-8 items-center justify-center rounded-xl bg-accent/10 text-accent">
						<TrendingUp class="h-4 w-4" />
					</div>
					<span class="text-sm font-black tracking-tight">Fio</span>
				</a>
			</div>

			<div class="flex items-center gap-2">
				{#if session.user}
					<span class="hidden text-xs font-semibold text-text-muted md:inline">
						{session.user.email}
					</span>
				{/if}
				<button
					type="button"
					class="rounded-lg border border-border bg-surface p-2 text-text-muted transition hover:bg-surface-2 hover:text-text"
					onclick={toggleTheme}
					title="Toggle theme"
				>
					{#if isDark}
						<Sun class="h-4 w-4" />
					{:else}
						<Moon class="h-4 w-4" />
					{/if}
				</button>
				<button
					type="button"
					class="inline-flex items-center gap-2 rounded-lg border border-border bg-surface px-3 py-2 text-xs font-semibold text-text-muted transition hover:bg-surface-2 hover:text-text"
					onclick={handleLogout}
				>
					<LogOut class="h-3.5 w-3.5" />
					<span class="hidden sm:inline">Logout</span>
				</button>
			</div>
		</div>
	</header>

	<div class="mx-auto grid max-w-[1400px] grid-cols-1 lg:grid-cols-[240px_1fr]">
		<!-- Sidebar -->
		<aside
			class="border-b border-border bg-surface/70 lg:min-h-[calc(100vh-56px)] lg:border-r lg:border-b-0 lg:bg-transparent {mobileOpen
				? 'block'
				: 'hidden lg:block'}"
		>
			<nav class="sticky top-14 flex gap-1 overflow-x-auto p-3 lg:flex-col lg:p-4">
				{#each nav as item}
					{@const active = isActive(item.href)}
					{@const Icon = item.icon}
					<a
						href={item.href}
						class="flex shrink-0 items-center gap-2.5 rounded-xl px-3 py-2.5 text-sm font-semibold transition lg:shrink {active
							? 'bg-accent text-white shadow-md shadow-accent/15'
							: 'text-text-muted hover:bg-surface-2 hover:text-text'}"
						onclick={() => (mobileOpen = false)}
					>
						<Icon class="h-4 w-4" />
						<span>{item.label}</span>
					</a>
				{/each}
			</nav>
		</aside>

		<!-- Main -->
		<main class="min-w-0 p-4 md:p-6 lg:p-8">
			{@render children()}
		</main>
	</div>
</div>
