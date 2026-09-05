<script lang="ts">
	import { goto } from '$app/navigation';
	import { LogIn, Mail, Lock } from 'lucide-svelte';
	import { login, getOAuthUrl } from '$lib/portal/api/auth';
	import { ApiError } from '$lib/portal/api/client';
	import { session } from '$lib/portal/stores/session.svelte';
	import { validateEmail, sanitizeInput } from '$lib/portal/utils';

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);
	let oauthLoading = $state('');

	async function submit() {
		error = '';
		const cleanEmail = sanitizeInput(email).toLowerCase();
		const cleanPassword = password;

		if (!validateEmail(cleanEmail)) {
			error = 'Please enter a valid email address.';
			return;
		}
		if (cleanPassword.length < 6) {
			error = 'Password must be at least 6 characters.';
			return;
		}

		loading = true;
		try {
			const data = await login(cleanEmail, cleanPassword);
			session.setFromLogin(data);
			void goto('/portal');
		} catch (e) {
			error = e instanceof ApiError ? e.message : 'Login failed. Please try again.';
		} finally {
			loading = false;
		}
	}

	async function startOAuth(provider: string) {
		error = '';
		oauthLoading = provider;
		try {
			const data = await getOAuthUrl(provider);
			if (data.error) {
				error = data.error;
				return;
			}
			window.location.href = data.url;
		} catch (e) {
			error = e instanceof ApiError ? e.message : `${provider} login is not available.`;
		} finally {
			oauthLoading = '';
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-bg px-4 py-12 text-text">
	<div class="animate-fade-in w-full max-w-md">
		<div class="mb-8 text-center">
			<div
				class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-accent/10 text-accent"
			>
				<LogIn class="h-7 w-7" />
			</div>
			<h1 class="text-3xl font-bold tracking-tight">Welcome back</h1>
			<p class="mt-2 text-sm text-text-muted">Sign in to your Fio account</p>
		</div>

		<!-- OAuth buttons -->
		<div class="space-y-3">
			<button
				type="button"
				class="flex w-full items-center justify-center gap-3 rounded-xl border border-border bg-surface px-4 py-3 text-sm font-semibold text-text transition hover:bg-surface-2 disabled:opacity-60"
				onclick={() => startOAuth('google')}
				disabled={!!oauthLoading}
			>
				<svg class="h-5 w-5" viewBox="0 0 24 24">
					<path
						fill="#4285F4"
						d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"
					/>
					<path
						fill="#34A853"
						d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
					/>
					<path
						fill="#FBBC05"
						d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
					/>
					<path
						fill="#EA4335"
						d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
					/>
				</svg>
				{oauthLoading === 'google' ? 'Redirecting...' : 'Continue with Google'}
			</button>

			<button
				type="button"
				class="flex w-full items-center justify-center gap-3 rounded-xl border border-border bg-surface px-4 py-3 text-sm font-semibold text-text transition hover:bg-surface-2 disabled:opacity-60"
				onclick={() => startOAuth('github')}
				disabled={!!oauthLoading}
			>
				<svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
					<path
						d="M12 2C6.48 2 2 6.58 2 12.26c0 4.53 2.87 8.37 6.84 9.73.5.1.68-.22.68-.49v-1.9c-2.78.62-3.37-1.21-3.37-1.21-.45-1.19-1.11-1.5-1.11-1.5-.91-.64.07-.63.07-.63 1 .07 1.53 1.06 1.53 1.06.9 1.57 2.35 1.12 2.92.86.09-.67.35-1.12.63-1.38-2.22-.26-4.56-1.14-4.56-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.71 0 0 .84-.28 2.75 1.05A9.3 9.3 0 0 1 12 6.98c.85 0 1.7.12 2.5.34 1.9-1.33 2.74-1.05 2.74-1.05.55 1.41.2 2.45.1 2.71.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.8-4.57 5.06.36.32.68.95.68 1.91v2.83c0 .27.18.59.69.49A10.2 10.2 0 0 0 22 12.26C22 6.58 17.52 2 12 2Z"
					/>
				</svg>
				{oauthLoading === 'github' ? 'Redirecting...' : 'Continue with GitHub'}
			</button>
		</div>

		<div class="my-6 flex items-center gap-3">
			<div class="h-px flex-1 bg-border"></div>
			<span class="text-xs font-semibold text-text-dim">or sign in with email</span>
			<div class="h-px flex-1 bg-border"></div>
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
					<label for="email" class="text-sm font-semibold text-text">Email</label>
					<div
						class="mt-1.5 flex items-center gap-2 rounded-xl border border-border bg-surface-2 px-3 py-2.5 focus-within:border-accent focus-within:bg-surface"
					>
						<Mail class="h-4 w-4 text-text-dim" />
						<input
							id="email"
							type="email"
							bind:value={email}
							placeholder="you@example.com"
							class="min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim"
							autocomplete="email"
						/>
					</div>
				</div>

				<div>
					<label for="password" class="text-sm font-semibold text-text">Password</label>
					<div
						class="mt-1.5 flex items-center gap-2 rounded-xl border border-border bg-surface-2 px-3 py-2.5 focus-within:border-accent focus-within:bg-surface"
					>
						<Lock class="h-4 w-4 text-text-dim" />
						<input
							id="password"
							type="password"
							bind:value={password}
							placeholder="Enter your password"
							class="min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim"
							autocomplete="current-password"
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
				{loading ? 'Signing in...' : 'Sign in'}
			</button>
		</form>

		<p class="mt-4 text-center text-sm text-text-muted">
			Don't have an account?
			<a href="/portal/register" class="font-bold text-accent hover:text-accent-glow">Sign up</a>
		</p>
	</div>
</div>
