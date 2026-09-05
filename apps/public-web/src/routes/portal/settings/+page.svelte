<script lang="ts">
	import { onMount } from 'svelte';
	import { Save, Settings, Trash2 } from 'lucide-svelte';
	import { deleteConfig, listConfig, setConfig } from '$lib/portal/api/tenant-config';
	import { ApiError } from '$lib/portal/api/client';
	import type { Plan, TenantConfig } from '$lib/portal/types';
	import { toastStore } from '$lib/portal/stores/toast.svelte';
	import EmptyState from '$lib/portal/components/EmptyState.svelte';
	import LoadingBlock from '$lib/portal/components/LoadingBlock.svelte';

	let configs = $state<TenantConfig>({});
	let planLimits = $state<Plan | null>(null);
	let loading = $state(true);
	let saving = $state('');
	let error = $state('');

	let xUsernames = $state('');
	let tvSymbols = $state('');
	let rssFeeds = $state('');

	function asLines(value: unknown) {
		return Array.isArray(value) ? value.filter((item) => typeof item === 'string').join('\n') : '';
	}

	function parseLines(value: string) {
		return value
			.split('\n')
			.map((item) => item.trim())
			.filter(Boolean);
	}

	function applyConfig(data: { configs: TenantConfig; plan_limits: Plan | null }) {
		configs = data.configs;
		planLimits = data.plan_limits;
		xUsernames = asLines(configs.x_usernames);
		tvSymbols = [
			'BTCUSDT',
			'ETHUSDT',
			'SOLUSDT',
			'PAXGUSDT',
			'EURUSD',
			'GBPUSD',
			'USDJPY',
			'XAUUSD',
			'SPX',
			'DXY'
		].join('\n');
		rssFeeds = ['Forex News RSS Hub Feed (Active)', 'Stock Market Feed (Active)'].join('\n');
	}

	async function load() {
		loading = true;
		error = '';
		try {
			applyConfig(await listConfig());
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load tenant settings.';
		} finally {
			loading = false;
		}
	}

	async function saveConfig(key: string, value: string) {
		saving = key;
		try {
			await setConfig(key, parseLines(value));
			toastStore.success('Settings saved.');
			applyConfig(await listConfig());
		} catch (e) {
			toastStore.error(e instanceof ApiError ? e.message : 'Failed to save settings.');
		} finally {
			saving = '';
		}
	}

	async function clearConfig(key: string) {
		saving = key;
		try {
			await deleteConfig(key);
			toastStore.success('Settings cleared.');
			applyConfig(await listConfig());
		} catch (e) {
			toastStore.error(e instanceof ApiError ? e.message : 'Failed to clear settings.');
		} finally {
			saving = '';
		}
	}

	onMount(() => {
		void load();
	});
</script>

<div class="animate-fade-in space-y-6">
	<div>
		<div class="flex items-center gap-3">
			<div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-accent/10 text-accent">
				<Settings class="h-5 w-5" />
			</div>
			<div>
				<h1 class="text-3xl font-black tracking-tight text-text">Feed settings</h1>
				<p class="mt-1 text-sm text-text-muted">
					Configure tenant-specific market intelligence sources.
				</p>
			</div>
		</div>
	</div>

	{#if loading}
		<LoadingBlock label="Loading settings..." />
	{:else if error}
		<EmptyState title="Settings unavailable" description={error} />
	{:else}
		<div class="grid gap-4 lg:grid-cols-3">
			<div class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<p class="text-xs font-bold text-text-dim">X usernames</p>
				<p class="mt-1 text-2xl font-black text-text">{planLimits?.x_usernames_max ?? 0}</p>
				<p class="mt-1 text-sm text-text-muted">Maximum tracked accounts for your plan.</p>
			</div>
			<div class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<p class="text-xs font-bold text-text-dim">Market symbols</p>
				<p class="mt-1 text-2xl font-black text-text">{planLimits?.tv_symbols_max ?? 0}</p>
				<p class="mt-1 text-sm text-text-muted">Maximum custom symbols for your plan.</p>
			</div>
			<div class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<p class="text-xs font-bold text-text-dim">Custom RSS</p>
				<p class="mt-1 text-2xl font-black text-text">
					{planLimits?.can_custom_rss ? 'Enabled' : 'Disabled'}
				</p>
				<p class="mt-1 text-sm text-text-muted">RSS feeds are validated by the backend.</p>
			</div>
		</div>

		<div class="grid gap-5 xl:grid-cols-3">
			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<h2 class="font-black text-text">X usernames</h2>
				<p class="mt-1 text-sm text-text-muted">One username per line, without @.</p>
				<textarea
					bind:value={xUsernames}
					rows="10"
					class="mt-4 w-full resize-none rounded-xl border border-border bg-surface-2 p-3 font-mono text-sm text-text outline-none placeholder:text-text-dim focus:border-accent"
					placeholder="Reuters\nInvestingcom\nfinancialjuice"
				></textarea>
				<div class="mt-4 flex gap-2">
					<button
						class="inline-flex items-center gap-2 rounded-xl bg-accent px-4 py-2 text-sm font-bold text-white hover:bg-accent-glow disabled:opacity-60"
						onclick={() => saveConfig('x_usernames', xUsernames)}
						disabled={!!saving}
					>
						<Save class="h-4 w-4" />
						{saving === 'x_usernames' ? 'Saving...' : 'Save'}
					</button>
					<button
						class="rounded-xl border border-border px-4 py-2 text-sm font-bold text-text-muted hover:bg-surface-2 disabled:opacity-60"
						onclick={() => clearConfig('x_usernames')}
						disabled={!!saving}
					>
						<Trash2 class="h-4 w-4" />
					</button>
				</div>
			</section>

			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<h2 class="font-black text-text">Market symbols</h2>
				<p class="mt-1 text-sm text-text-muted">
					One public market symbol per line, such as EURUSD, XAUUSD, SPX, or AAPL.
				</p>
				<textarea
					readonly
					bind:value={tvSymbols}
					rows="10"
					class="mt-4 w-full resize-none rounded-xl border border-border bg-surface-2 p-3 font-mono text-sm text-text-muted opacity-80 outline-none"
					placeholder="No symbols configured. Contact administrator."
				></textarea>
				<div class="mt-4">
					<span
						class="inline-flex items-center rounded-xl border border-border bg-surface-2 px-3 py-1.5 text-xs font-bold text-text-muted"
					>
						Managed by Administrator
					</span>
				</div>
			</section>

			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<h2 class="font-black text-text">Custom RSS feeds</h2>
				<p class="mt-1 text-sm text-text-muted">One URL per line. Availability depends on plan.</p>
				<textarea
					readonly
					bind:value={rssFeeds}
					rows="10"
					class="mt-4 w-full resize-none rounded-xl border border-border bg-surface-2 p-3 font-mono text-sm text-text-muted opacity-80 outline-none"
					placeholder="No RSS feeds configured. Contact administrator."
				></textarea>
				<div class="mt-4">
					<span
						class="inline-flex items-center rounded-xl border border-border bg-surface-2 px-3 py-1.5 text-xs font-bold text-text-muted"
					>
						Managed by Administrator
					</span>
				</div>
			</section>
		</div>
	{/if}
</div>
