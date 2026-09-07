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
		media_urls?: string[];
	};

	type SocialPage = {
		items?: Post[];
		next_before?: string | null;
		has_more?: boolean;
		error?: string;
	};

	const pageSize = 20;
	let items = $state<Post[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let error = $state('');
	let hasMore = $state(false);
	let nextBefore = $state<string | null>(null);

	function mergePosts(posts: Post[]) {
		const existing = new Set(items.map((post) => post.event_id));
		items = [...items, ...posts.filter((post) => post.event_id && !existing.has(post.event_id))];
	}

	async function loadPage(append = false) {
		if (append) {
			if (loadingMore || !hasMore) return;
			loadingMore = true;
		} else {
			loading = true;
			error = '';
		}

		try {
			const params = new URLSearchParams({ platform: 'twitter', limit: String(pageSize) });
			if (append && nextBefore) params.set('before', nextBefore);
			const response = await apiFetch(`/api/v1/social/posts?${params}`);
			if (!response.ok) throw new Error('Social history unavailable');
			const payload: SocialPage = await response.json();
			if (payload.error) throw new Error('Social history unavailable');
			mergePosts(Array.isArray(payload.items) ? payload.items : []);
			nextBefore = payload.next_before ?? null;
			hasMore = Boolean(payload.has_more && nextBefore);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : 'Social history unavailable';
		} finally {
			loading = false;
			loadingMore = false;
		}
	}

	onMount(() => {
		void loadPage();
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
		<div class="py-8 text-center text-sm text-text-muted">Loading social history…</div>
	{:else if error && items.length === 0}
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
					<p class="mt-3 whitespace-pre-wrap break-words text-sm leading-6 text-text">{post.text}</p>
					{#if post.media_urls?.length}
						<div class="mt-3 grid gap-2 {post.media_urls.length > 1 ? 'grid-cols-2' : 'grid-cols-1'}">
							{#each post.media_urls.slice(0, 4) as mediaUrl}
								<a href={mediaUrl} target="_blank" rel="noreferrer" class="block overflow-hidden rounded border border-border bg-bg">
									<img src={mediaUrl} alt="Media attached to social post" loading="lazy" class="max-h-72 w-full object-cover transition-transform hover:scale-[1.02]" />
								</a>
							{/each}
						</div>
					{/if}
					<div class="mt-3 flex gap-4 border-t border-border/60 pt-3 text-xs text-text-muted"><span>♡ {post.like_count ?? 0}</span><span>↻ {post.retweet_count ?? 0}</span><span>▢ {post.reply_count ?? 0}</span></div>
				</article>
			{/each}
		</div>
		<div class="mt-5 flex flex-col items-center gap-2">
			{#if error}
				<p class="text-xs text-red-400">{error}</p>
			{/if}
			{#if hasMore}
				<button class="rounded border border-border bg-surface-2 px-4 py-2 text-xs font-bold text-text transition-colors hover:border-accent/50 disabled:opacity-50" disabled={loadingMore} onclick={() => loadPage(true)}>
					{loadingMore ? 'Loading history…' : 'Load older posts'}
				</button>
			{:else}
				<span class="text-xs text-text-dim">End of social history</span>
			{/if}
		</div>
	{/if}
</div>
