<script lang="ts">
	import { goto } from '$app/navigation';
	import { UserPlus, Mail, Lock, User } from 'lucide-svelte';
	import { register } from '$lib/portal/api/auth';
	import { ApiError } from '$lib/portal/api/client';
	import { session } from '$lib/portal/stores/session.svelte';
	import { toastStore } from '$lib/portal/stores/toast.svelte';
	import { validateEmail, sanitizeInput } from '$lib/portal/utils';

	let name = $state('');
	let email = $state('');
	let password = $state('');
	let confirm = $state('');
	let error = $state('');
	let loading = $state(false);

	async function submit() {
		error = '';
		const cleanName = sanitizeInput(name, 100);
		const cleanEmail = sanitizeInput(email).toLowerCase();

		if (cleanName.length < 1) {
			error = 'Name is required.';
			return;
		}
		if (!validateEmail(cleanEmail)) {
			error = 'Please enter a valid email address.';
			return;
		}
		if (password.length < 6) {
			error = 'Password must be at least 6 characters.';
			return;
		}
		if (password !== confirm) {
			error = 'Passwords do not match.';
			return;
		}

		loading = true;
		try {
			const data = await register(cleanEmail, cleanName, password);
			session.setFromLogin(data);

			if (data.api_key) {
				toastStore.success('Account created! Your API key has been generated.');
			}

			void goto('/portal');
		} catch (e) {
			if (e instanceof ApiError && e.status === 409) {
				error = 'An account with this email already exists.';
			} else {
				error = e instanceof ApiError ? e.message : 'Registration failed. Please try again.';
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-bg px-4 py-12 text-text">
	<div class="animate-fade-in w-full max-w-md">
		<div class="mb-8 text-center">
			<div
				class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-accent/10 text-accent"
			>
				<UserPlus class="h-7 w-7" />
			</div>
			<h1 class="text-3xl font-bold tracking-tight">Create account</h1>
			<p class="mt-2 text-sm text-text-muted">Start using Fio market intelligence APIs</p>
		</div>

		<form
			class="rounded-2xl border border-border bg-surface p-6 shadow-sm"
			onsubmit={(e) => {
				e.preventDefault();
				void submit();
			}}
		>
			<div class="space-y-4">
				<div>
					<label for="name" class="text-sm font-semibold text-text">Name</label>
					<div
						class="mt-1.5 flex items-center gap-2 rounded-xl border border-border bg-surface-2 px-3 py-2.5 focus-within:border-accent focus-within:bg-surface"
					>
						<User class="h-4 w-4 text-text-dim" />
						<input
							id="name"
							type="text"
							bind:value={name}
							placeholder="Your name"
							class="min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim"
							autocomplete="name"
							maxlength="100"
						/>
					</div>
				</div>

				<div>
					<label for="reg-email" class="text-sm font-semibold text-text">Email</label>
					<div
						class="mt-1.5 flex items-center gap-2 rounded-xl border border-border bg-surface-2 px-3 py-2.5 focus-within:border-accent focus-within:bg-surface"
					>
						<Mail class="h-4 w-4 text-text-dim" />
						<input
							id="reg-email"
							type="email"
							bind:value={email}
							placeholder="you@example.com"
							class="min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim"
							autocomplete="email"
						/>
					</div>
				</div>

				<div>
					<label for="reg-password" class="text-sm font-semibold text-text">Password</label>
					<div
						class="mt-1.5 flex items-center gap-2 rounded-xl border border-border bg-surface-2 px-3 py-2.5 focus-within:border-accent focus-within:bg-surface"
					>
						<Lock class="h-4 w-4 text-text-dim" />
						<input
							id="reg-password"
							type="password"
							bind:value={password}
							placeholder="Min. 6 characters"
							class="min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim"
							autocomplete="new-password"
							minlength="6"
						/>
					</div>
				</div>

				<div>
					<label for="reg-confirm" class="text-sm font-semibold text-text">Confirm password</label>
					<div
						class="mt-1.5 flex items-center gap-2 rounded-xl border border-border bg-surface-2 px-3 py-2.5 focus-within:border-accent focus-within:bg-surface"
					>
						<Lock class="h-4 w-4 text-text-dim" />
						<input
							id="reg-confirm"
							type="password"
							bind:value={confirm}
							placeholder="Repeat your password"
							class="min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim"
							autocomplete="new-password"
						/>
					</div>
				</div>
			</div>

			{#if error}
				<p class="mt-4 rounded-lg border border-red/20 bg-red/10 px-3 py-2 text-sm text-red">
					{error}
				</p>
			{/if}

			<button
				type="submit"
				class="mt-5 w-full rounded-xl bg-accent px-4 py-3 text-sm font-bold text-white shadow-sm transition hover:bg-accent-glow disabled:opacity-60"
				disabled={loading}
			>
				{loading ? 'Creating account...' : 'Create account'}
			</button>
		</form>

		<p class="mt-4 text-center text-sm text-text-muted">
			Already have an account?
			<a href="/portal/login" class="font-bold text-accent hover:text-accent-glow">Sign in</a>
		</p>
	</div>
</div>
