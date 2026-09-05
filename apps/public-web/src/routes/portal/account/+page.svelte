<script lang="ts">
	import { KeyRound, Link, LogOut, Mail, ShieldCheck, User } from 'lucide-svelte';
	import { session } from '$lib/portal/stores/session.svelte';
	import PlanBadge from '$lib/portal/components/PlanBadge.svelte';
	import StatusBadge from '$lib/portal/components/StatusBadge.svelte';
	import { formatDateTime } from '$lib/portal/utils';
</script>

<div class="animate-fade-in space-y-6">
	<div>
		<h1 class="text-3xl font-black tracking-tight text-text">Account</h1>
		<p class="mt-2 text-sm text-text-muted">
			Review profile, authentication, and plan information.
		</p>
	</div>

	{#if session.user}
		<section class="rounded-2xl border border-border bg-surface p-6 shadow-sm">
			<div class="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
				<div class="flex items-center gap-4">
					{#if session.user.avatar_url}
						<img src={session.user.avatar_url} alt="" class="h-16 w-16 rounded-2xl object-cover" />
					{:else}
						<div
							class="flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/10 text-accent"
						>
							<User class="h-8 w-8" />
						</div>
					{/if}
					<div>
						<h2 class="text-2xl font-black text-text">{session.user.name}</h2>
						<p class="mt-1 flex items-center gap-2 text-sm text-text-muted">
							<Mail class="h-4 w-4" />
							{session.user.email}
						</p>
					</div>
				</div>
				<button
					class="inline-flex items-center justify-center gap-2 rounded-xl border border-border px-4 py-2 text-sm font-bold text-text-muted transition hover:border-red/30 hover:bg-red/10 hover:text-red"
					onclick={() => session.logout()}
				>
					<LogOut class="h-4 w-4" /> Sign out
				</button>
			</div>
		</section>

		<div class="grid gap-5 lg:grid-cols-2">
			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<h2 class="flex items-center gap-2 font-black text-text">
					<ShieldCheck class="h-5 w-5 text-accent" /> Security
				</h2>
				<div class="mt-5 space-y-4">
					<div class="flex items-center justify-between rounded-xl bg-surface-2 p-4">
						<div>
							<p class="text-sm font-bold text-text">Email verification</p>
							<p class="mt-1 text-xs text-text-muted">
								Required for account recovery and notifications.
							</p>
						</div>
						<StatusBadge
							tone={session.user.email_verified ? 'green' : 'amber'}
							label={session.user.email_verified ? 'Verified' : 'Pending'}
						/>
					</div>
					<div class="flex items-center justify-between rounded-xl bg-surface-2 p-4">
						<div>
							<p class="text-sm font-bold text-text">Password login</p>
							<p class="mt-1 text-xs text-text-muted">
								OAuth-only accounts may not have a password.
							</p>
						</div>
						<StatusBadge
							tone={session.user.has_password ? 'green' : 'blue'}
							label={session.user.has_password ? 'Enabled' : 'OAuth only'}
						/>
					</div>
				</div>
			</section>

			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<h2 class="flex items-center gap-2 font-black text-text">
					<Link class="h-5 w-5 text-accent" /> Linked providers
				</h2>
				{#if session.linkedProviders.length > 0}
					<div class="mt-5 flex flex-wrap gap-2">
						{#each session.linkedProviders as provider}
							<span
								class="rounded-full border border-border bg-surface-2 px-3 py-1 text-sm font-bold text-text capitalize"
								>{provider}</span
							>
						{/each}
					</div>
				{:else}
					<p class="mt-5 rounded-xl bg-surface-2 p-4 text-sm text-text-muted">
						No OAuth providers are linked to this account.
					</p>
				{/if}
			</section>
		</div>

		<div class="grid gap-5 lg:grid-cols-3">
			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<p class="text-xs font-bold text-text-dim">Current plan</p>
				<div class="mt-3"><PlanBadge plan={session.user.plan} /></div>
			</section>
			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<p class="text-xs font-bold text-text-dim">Active API keys</p>
				<p class="mt-2 flex items-center gap-2 text-2xl font-black text-text">
					<KeyRound class="h-5 w-5 text-accent" />
					{session.activeKeys}
				</p>
			</section>
			<section class="rounded-2xl border border-border bg-surface p-5 shadow-sm">
				<p class="text-xs font-bold text-text-dim">Member since</p>
				<p class="mt-2 text-sm font-bold text-text">{formatDateTime(session.user.created_at)}</p>
			</section>
		</div>
	{/if}
</div>
