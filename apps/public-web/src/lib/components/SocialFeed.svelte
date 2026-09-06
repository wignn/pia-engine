<script lang="ts">
	import { onMount } from 'svelte';
	import { apiFetch } from '$lib/api';

	type Post = {
		event_id: string;
		author_display_name: string;
		author_username: string;
		text: string;
		url: string;
		created_at: string;
		like_count: number;
		retweet_count: number;
		reply_count: number;
	};

	let items = $state<Post[]>([]);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			const response = await apiFetch('/api/v1/social/posts?platform=twitter&limit=20');
			if (!response.ok) throw new Error('Social feed unavailable');
			const payload = await response.json();
			items = Array.isArray(payload.items) ? payload.items : [];
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Social feed unavailable';
		} finally {
			loading = false;
		}
	});
</script>

<div class="p-5">
	<div class="mb-4 flex items-end justify-between gap-3">
		<div>
			<h3 class="text-lg font-black text-text">Social Pulse</h3>
			<p class="text-sm text-text-muted">Verified posts collected by ATLSD</p>
		</div>
		<span class="rounded border border-border bg-surface-2 px-2 py-1 text-[10px] font-bold uppercase tracking-wider text-text-muted">X / Twitter</span>
	</div>
	{#if loading}
		<div class="py-8 text-center text-sm text-text-muted">Loading social posts…</div>
	{:else if error}
		<div class="rounded border border-border bg-surface-2 p-4 text-sm text-text-muted">{error}</div>
	{:else if items.length === 0}
		<div class="rounded border border-border bg-surface-2 p-4 text-sm text-text-muted">No social posts available.</div>
	{:else}
		<div class="grid gap-3 md:grid-cols-2">
			{#each items as post (post.event_id)}
				<article class="rounded border border-border bg-surface-2 p-4 transition-colors hover:border-accent/50">
					<div class="flex items-start justify-between gap-3">
						<div><div class="font-bold text-text">{post.author_display_name || post.author_username}</div><div class="text-xs text-text-muted">@{post.author_username} · {new Date(post.created_at).toLocaleString()}</div></div>
						<a class="text-xs font-bold text-accent hover:underline" href={post.url} target="_blank" rel="noreferrer">Open ↗</a>
					</div>
					<p class="mt-3 whitespace-pre-wrap text-sm leading-6 text-text">{post.text}</p>
					<div class="mt-3 flex gap-4 text-xs text-text-muted"><span>♡ {post.like_count}</span><span>↻ {post.retweet_count}</span><span>▢ {post.reply_count}</span></div>
				</article>
			{/each}
		</div>
	{/if}
</div>
