<script lang="ts">
	import {
		ArrowRight,
		BookOpen,
		Bot,
		Braces,
		CheckCircle2,
		ChevronRight,
		Cloud,
		Code2,
		Globe2,
		KeyRound,
		LayoutDashboard,
		LockKeyhole,
		Menu,
		Radio,
		ServerCog,
		ShieldCheck,
		Terminal,
		X
	} from 'lucide-svelte';

	let menuOpen = $state(false);

	const sections = [
		{ id: 'overview', label: 'Overview' },
		{ id: 'quickstart', label: 'Getting Started' },
		{ id: 'portal', label: 'Portal Guide' },
		{ id: 'api', label: 'API Reference' },
		{ id: 'websocket', label: 'WebSocket Streams' },
		{ id: 'bot', label: 'Discord Bot' },
		{ id: 'deployment', label: 'Deployment' },
		{ id: 'operations', label: 'Operations' },
		{ id: 'security', label: 'Security' },
		{ id: 'faq', label: 'FAQ' }
	];

	const capabilities = [
		{
			title: 'Market data aggregation',
			description: 'Live price cache fed by WebSocket ingress workers for forex, crypto, and selected market symbols.'
		},
		{
			title: 'News intelligence',
			description: 'RSS ingestion, article scraping, deduplication, analysis metadata, and real-time distribution.'
		},
		{
			title: 'Tenant-aware access',
			description: 'JWT and API-key authentication with plan limits, usage tracking, and centralized tenant configuration.'
		},
		{
			title: 'Realtime delivery',
			description: 'Dedicated WebSocket channels for market data, news, calendar reminders, X posts, and system events.'
		}
	];

	const quickstartSteps = [
		[
			'1',
			'Create or sign in to an account',
			'Use the portal registration or OAuth flow. Registration returns a JWT session and an initial API key.'
		],
		[
			'2',
			'Store the API key once',
			'Raw keys are only displayed during creation. Keep the key in server-side environment variables.'
		],
		[
			'3',
			'Call the core API',
			'Start with market prices, latest forex news, or the economic calendar before moving to WebSocket streams.'
		]
	];

	const apiGroups = [
		{
			name: 'Core data API',
			base: 'CORE_REST_URL',
			auth: 'API key for protected routes; public market/news routes can be used by the web client.',
			endpoints: [
				['GET', '/api/v1/market/prices', 'List all cached live prices.'],
				['GET', '/api/v1/market/prices/{symbol}', 'Fetch a single cached symbol, case-insensitive.'],
				['GET', '/api/v1/forex/news?page=1&page_size=20', 'Paginated forex article history.'],
				['GET', '/api/v1/forex/news/latest?limit=10', 'Latest processed forex news with analysis fields.'],
				['GET', '/api/v1/forex/news/{id}', 'Fetch one stored forex article.'],
				['GET', '/api/v1/forex/calendar?impact=high&limit=10', 'Upcoming Forex Factory calendar events.'],
				['GET', '/api/v1/equity/news?limit=10', 'Latest processed equity news.'],
				['POST', '/api/v1/content/scrape', 'Private article scraping endpoint.']
			]
		},
		{
			name: 'Control plane API',
			base: 'CONTROL_PLANE_URL',
			auth: 'JWT Bearer token after login or registration.',
			endpoints: [
				['POST', '/api/v1/auth/register', 'Create an account and receive the first API key.'],
				['POST', '/api/v1/auth/login', 'Exchange email and password for a JWT session.'],
				['GET', '/api/v1/auth/me', 'Read profile, plan limits, active keys, and OAuth links.'],
				['GET', '/api/v1/keys', 'List API keys for the authenticated user.'],
				['POST', '/api/v1/keys', 'Create a new API key. The raw key is shown once.'],
				['PATCH', '/api/v1/keys/{id}', 'Rename an API key.'],
				['DELETE', '/api/v1/keys/{id}', 'Revoke an API key.'],
				['GET', '/api/v1/config', 'Read tenant configuration and plan limits.'],
				['PUT', '/api/v1/config/{key}', 'Update tenant config such as tv_symbols or custom_rss_feeds.'],
				['GET', '/api/v1/usage', 'Usage summary for the current account.'],
				['GET', '/api/v1/plans', 'Public plan catalog.']
			]
		}
	];

	const wsChannels = [
		['/api/v1/ws', 'General stream with query-based subscriptions.'],
		['/api/v1/ws/market', 'All live market trade events.'],
		['/api/v1/ws/market/{symbol}', 'Market data for one symbol route.'],
		['/api/v1/ws/news', 'Forex news events.'],
		['/api/v1/ws/equity', 'Equity news events.'],
		['/api/v1/ws/calendar', 'Economic calendar reminder events.'],
		['/api/v1/ws/x', 'Configured X/Twitter feed events.'],
		['/api/v1/ws/x/{symbol}', 'Symbol-oriented X stream route.']
	];

	const events = [
		'market.trade',
		'news.new',
		'news.high_impact',
		'equity.news.new',
		'calendar.reminder',
		'gold.volatility_spike',
		'x.new',
		'heartbeat',
		'system.status'
	];

	const botCommands = [
		['/market_alert XAUUSD 3400', 'Create a direct price alert for a symbol.'],
		['/forex_news_setup #channel', 'Send forex news updates to a Discord channel.'],
		['/stocknews subscribe', 'Subscribe to Indonesian equity news.'],
		['/calendar_setup #channel', 'Enable economic calendar reminders.'],
		['/twitter_setup #channel', 'Forward configured X account posts.'],
		['/volatility_setup #channel', 'Enable volatility spike alerts.']
	];

	const faqs = [
		['Is this only a Discord bot?', 'No. The Discord bot is one delivery surface. The platform also exposes HTTP APIs, WebSocket streams, a control plane, and a management portal.'],
		['How are API keys stored?', 'Raw API keys are shown once, then stored as SHA-256 hashes. Revocation and label updates happen through the control plane.'],
		['What happens if Redis is unavailable?', 'The system keeps core service availability as the priority. Quota counters can fail open, while database-backed identity and configuration remain authoritative.'],
		['Can tenants configure their own sources?', 'Tenant configuration supports TradingView symbols and custom RSS feeds, subject to plan capabilities and limits.'],
		['Which services are required in production?', 'PostgreSQL is required. Redis is recommended for tenant synchronization, quota counters, and low-latency coordination. RSSHub is required for X feed aggregation.']
	];
</script>

<svelte:head>
	<title>Documentation — Fio Real-time Market Intelligence</title>
	<meta
		name="description"
		content="Professional documentation for Fio and the ATLSD platform: getting started, API reference, WebSocket streams, portal guide, Discord bot, deployment, operations, and security."
	/>
</svelte:head>

<div class="min-h-screen bg-bg text-text">
	<header class="sticky top-0 z-50 border-b border-border bg-bg/95 backdrop-blur">
		<div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 md:px-8 lg:px-16">
			<a href="/" class="flex items-center gap-3">
				<div class="flex h-9 w-9 items-center justify-center rounded-lg border border-accent/25 bg-accent/10 text-accent">
					<BookOpen class="h-4 w-4" />
				</div>
				<div>
					<div class="text-sm font-semibold">Fio Documentation</div>
					<div class="text-xs text-text-dim">ATLSD Platform</div>
				</div>
			</a>

			<nav class="hidden items-center gap-6 text-sm text-text-muted md:flex">
				<a href="#api" class="transition-colors hover:text-text">API</a>
				<a href="#websocket" class="transition-colors hover:text-text">WebSocket</a>
				<a href="#deployment" class="transition-colors hover:text-text">Deploy</a>
				<a
					href="/"
					class="inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-text transition-colors hover:border-accent/50 hover:text-accent"
				>
					Back to Fio
					<ArrowRight class="h-4 w-4" />
				</a>
			</nav>

			<button
				type="button"
				class="inline-flex h-10 w-10 items-center justify-center rounded-md border border-border text-text md:hidden"
				aria-label="Toggle documentation navigation"
				aria-expanded={menuOpen}
				onclick={() => (menuOpen = !menuOpen)}
			>
				{#if menuOpen}
					<X class="h-5 w-5" />
				{:else}
					<Menu class="h-5 w-5" />
				{/if}
			</button>
		</div>

		{#if menuOpen}
			<div class="border-t border-border bg-surface md:hidden">
				<nav class="grid gap-1 px-4 py-3">
					{#each sections as section}
						<a
							href={`#${section.id}`}
							class="rounded-md px-3 py-2 text-sm text-text-muted hover:bg-surface-2 hover:text-text"
							onclick={() => (menuOpen = false)}
						>
							{section.label}
						</a>
					{/each}
				</nav>
			</div>
		{/if}
	</header>

	<div class="mx-auto grid max-w-7xl gap-10 px-4 py-10 md:px-8 lg:grid-cols-[260px_minmax(0,1fr)] lg:px-16">
		<aside class="hidden lg:block">
			<div class="sticky top-24">
				<div class="mb-4 text-xs font-semibold uppercase tracking-[0.18em] text-text-dim">Documentation</div>
				<nav class="space-y-1">
					{#each sections as section}
						<a
							href={`#${section.id}`}
							class="flex items-center justify-between rounded-md px-3 py-2 text-sm text-text-muted transition-colors hover:bg-surface hover:text-text"
						>
							<span>{section.label}</span>
							<ChevronRight class="h-3.5 w-3.5" />
						</a>
					{/each}
				</nav>
			</div>
		</aside>

		<main class="min-w-0">
			<section id="overview" class="scroll-mt-24 pb-14">
				<div class="mb-6 inline-flex items-center gap-2 rounded-full border border-border bg-surface px-3 py-1 text-xs font-medium text-text-muted">
					<span class="h-1.5 w-1.5 rounded-full bg-green"></span>
					Production-oriented market intelligence platform
				</div>

				<h1 class="max-w-3xl text-4xl font-bold leading-tight tracking-tight text-text md:text-6xl">
					Build with real-time market data, news intelligence, and tenant-aware APIs.
				</h1>
				<p class="mt-6 max-w-3xl text-base leading-8 text-text-muted md:text-lg">
					Fio is the public product experience for the ATLSD platform: a service-oriented market information system
					combining Rust data pipelines, HTTP APIs, WebSocket streams, a SaaS control plane, a management portal,
					and Discord delivery workflows.
				</p>

				<div class="mt-8 flex flex-col gap-3 sm:flex-row">
					<a
						href="#quickstart"
						class="inline-flex items-center justify-center gap-2 rounded-md bg-accent px-4 py-3 text-sm font-semibold text-white transition-colors hover:bg-accent-glow"
					>
						Start integrating
						<ArrowRight class="h-4 w-4" />
					</a>
					<a
						href="#api"
						class="inline-flex items-center justify-center gap-2 rounded-md border border-border px-4 py-3 text-sm font-semibold text-text transition-colors hover:border-accent/50 hover:text-accent"
					>
						View API reference
						<Code2 class="h-4 w-4" />
					</a>
				</div>

				<div class="mt-10 grid gap-4 md:grid-cols-2">
					{#each capabilities as item}
						<article class="rounded-lg border border-border bg-surface p-5">
							<div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text">
								<CheckCircle2 class="h-4 w-4 text-green" />
								{item.title}
							</div>
							<p class="text-sm leading-6 text-text-muted">{item.description}</p>
						</article>
					{/each}
				</div>
			</section>

			<section id="quickstart" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('terminal', 'Getting Started', 'First request in minutes')}
				<div class="grid gap-6 lg:grid-cols-[1fr_1fr]">
					<div class="space-y-4">
						{#each quickstartSteps as step}
							{@render Step(step[0], step[1], step[2])}
						{/each}
					</div>

					{@render CodeBlock(
						'Example request',
						`curl "$CORE_REST_URL/api/v1/market/prices/XAUUSD" \\
  -H "x-api-key: $FIO_API_KEY"

curl "$CORE_REST_URL/api/v1/forex/news/latest?limit=5" \\
  -H "x-api-key: $FIO_API_KEY"`
					)}
				</div>
			</section>

			<section id="portal" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('dashboard', 'Portal Guide', 'Manage access, plans, and tenant configuration')}
				<div class="grid gap-4 md:grid-cols-3">
					{@render FeatureCard('Dashboard', 'Review account status, current plan, usage summary, and service activity from a single operational view.')}
					{@render FeatureCard('API keys', 'Create, rename, and revoke API keys. The platform allows up to 10 active keys per user.')}
					{@render FeatureCard('Tenant config', 'Configure TradingView symbols and custom RSS feeds. Validation follows the active plan limits.')}
				</div>
				<p class="mt-6 rounded-lg border border-blue/25 bg-blue/10 p-4 text-sm leading-6 text-text-muted">
					The control plane is the source of truth for user identity, plans, API keys, configuration, and usage history.
					Core services consume synchronized tenant state so HTTP and WebSocket access remain consistent.
				</p>
			</section>

			<section id="api" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('braces', 'API Reference', 'HTTP endpoints and authentication model')}
				<div class="space-y-8">
					{#each apiGroups as group}
						<article class="rounded-lg border border-border bg-surface">
							<div class="border-b border-border p-5">
								<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
									<h3 class="text-lg font-semibold">{group.name}</h3>
									<code class="rounded bg-bg px-2 py-1 font-mono text-xs text-accent">{group.base}</code>
								</div>
								<p class="mt-2 text-sm text-text-muted">{group.auth}</p>
							</div>
							<div class="divide-y divide-border">
								{#each group.endpoints as endpoint}
									<div class="grid gap-3 p-4 text-sm md:grid-cols-[90px_minmax(0,1fr)_minmax(220px,0.8fr)]">
										<span class="font-mono text-xs font-semibold text-green">{endpoint[0]}</span>
										<code class="break-words font-mono text-xs text-text">{endpoint[1]}</code>
										<span class="leading-6 text-text-muted">{endpoint[2]}</span>
									</div>
								{/each}
							</div>
						</article>
					{/each}
				</div>

				<div class="mt-8 grid gap-6 lg:grid-cols-2">
					{@render CodeBlock(
						'Market price response',
						`{
  "symbol": "XAUUSD",
  "price": 3400.25,
  "bid": 3399.9,
  "ask": 3400.6,
  "volume": null,
  "source": "tiingo",
  "asset_type": "forex",
  "received_at": "2026-05-15T10:00:00Z"
}`
					)}
					{@render CodeBlock(
						'Tenant config update',
						`PUT /api/v1/config/tv_symbols
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "value": ["XAUUSD", "EURUSD", "BTCUSDT"]
}`
					)}
				</div>
			</section>

			<section id="websocket" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('radio', 'WebSocket Streams', 'Low-latency event delivery')}
				<div class="grid gap-8 lg:grid-cols-[1fr_0.9fr]">
					<div class="rounded-lg border border-border bg-surface">
						<div class="border-b border-border p-5">
							<h3 class="text-lg font-semibold">Stream routes</h3>
							<p class="mt-2 text-sm text-text-muted">Each specialized route auto-subscribes to its corresponding channel.</p>
						</div>
						<div class="divide-y divide-border">
							{#each wsChannels as channel}
								<div class="grid gap-3 p-4 text-sm md:grid-cols-[220px_minmax(0,1fr)]">
									<code class="break-words font-mono text-xs text-accent">{channel[0]}</code>
									<span class="leading-6 text-text-muted">{channel[1]}</span>
								</div>
							{/each}
						</div>
					</div>

					<div class="space-y-6">
						{@render CodeBlock(
							'Browser connection',
							`const ws = new WebSocket(
  \`\${CORE_WS_URL}/api/v1/ws/market?api_key=\${apiKey}\`
);

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log(message.event, message.data);
};`
						)}

						<div class="rounded-lg border border-border bg-surface p-5">
							<h3 class="mb-4 text-sm font-semibold uppercase tracking-[0.16em] text-text-dim">Event names</h3>
							<div class="flex flex-wrap gap-2">
								{#each events as event}
									<code class="rounded-md border border-border bg-bg px-2.5 py-1.5 font-mono text-xs text-text-muted">{event}</code>
								{/each}
							</div>
						</div>
					</div>
				</div>
			</section>

			<section id="bot" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('bot', 'Discord Bot', 'Operational alerts inside Discord')}
				<div class="grid gap-4 md:grid-cols-2">
					{#each botCommands as command}
						<div class="rounded-lg border border-border bg-surface p-5">
							<code class="font-mono text-sm text-accent">{command[0]}</code>
							<p class="mt-3 text-sm leading-6 text-text-muted">{command[1]}</p>
						</div>
					{/each}
				</div>
			</section>

			<section id="deployment" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('cloud', 'Deployment', 'Containerized platform services')}
				<div class="grid gap-6 lg:grid-cols-[0.85fr_1fr]">
					<div class="space-y-4">
						{@render FeatureCard('Required services', 'PostgreSQL, core service, control plane, portal, and RSSHub for X feed ingestion.')}
						{@render FeatureCard('Recommended services', 'Redis for tenant synchronization, quota counters, pub/sub coordination, and news ingest streams.')}
						{@render FeatureCard('Build artifacts', 'The infrastructure layer provides component Dockerfiles and Compose orchestration for local or server deployment.')}
					</div>
					{@render CodeBlock(
						'Local compose workflow',
						`cd infrastructure
docker compose up --build

# Service health
curl "$CORE_REST_URL/health"
curl "$CONTROL_PLANE_URL/health"`
					)}
				</div>
			</section>

			<section id="operations" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('server', 'Operations', 'Run with visibility and graceful degradation')}
				<div class="grid gap-4 md:grid-cols-2">
					{@render FeatureCard('Structured logs', 'Rust services emit structured JSON logs for pipeline status, HTTP behavior, WebSocket connections, and worker failures.')}
					{@render FeatureCard('Tenant sync', 'Control-plane updates publish Redis notifications so core services refresh API keys, plans, and tenant configuration.')}
					{@render FeatureCard('Usage tracking', 'API requests are recorded through batched usage logging, with Redis-backed quota counters for low-latency enforcement.')}
					{@render FeatureCard('Graceful shutdown', 'HTTP servers and scheduled tasks are designed to preserve state consistency during process termination.')}
				</div>
			</section>

			<section id="security" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('shield', 'Security', 'Access control is enforced at tenant boundaries')}
				<div class="grid gap-6 lg:grid-cols-3">
					<div class="rounded-lg border border-border bg-surface p-5">
						<KeyRound class="mb-4 h-5 w-5 text-accent" />
						<h3 class="font-semibold">API keys</h3>
						<p class="mt-2 text-sm leading-6 text-text-muted">Keys are generated once, hashed before storage, and synchronized to the core tenant registry.</p>
					</div>
					<div class="rounded-lg border border-border bg-surface p-5">
						<LockKeyhole class="mb-4 h-5 w-5 text-blue" />
						<h3 class="font-semibold">JWT sessions</h3>
						<p class="mt-2 text-sm leading-6 text-text-muted">Portal and control-plane requests use signed Bearer tokens with user, email, plan, and expiry claims.</p>
					</div>
					<div class="rounded-lg border border-border bg-surface p-5">
						<ShieldCheck class="mb-4 h-5 w-5 text-green" />
						<h3 class="font-semibold">Plan limits</h3>
						<p class="mt-2 text-sm leading-6 text-text-muted">Daily quotas, WebSocket connection caps, rate limits, and feature capabilities are evaluated per tenant.</p>
					</div>
				</div>
			</section>

			<section id="faq" class="scroll-mt-24 border-t border-border py-14">
				{@render SectionHeading('globe', 'FAQ', 'Common implementation questions')}
				<div class="space-y-3">
					{#each faqs as faq}
						<details class="rounded-lg border border-border bg-surface p-5">
							<summary class="cursor-pointer text-sm font-semibold text-text">{faq[0]}</summary>
							<p class="mt-3 text-sm leading-6 text-text-muted">{faq[1]}</p>
						</details>
					{/each}
				</div>
			</section>
		</main>
	</div>
</div>

{#snippet SectionHeading(icon: string, eyebrow: string, title: string)}
	<div class="mb-8">
		<div class="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.18em] text-text-dim">
			{#if icon === 'terminal'}
				<Terminal class="h-4 w-4 text-accent" />
			{:else if icon === 'dashboard'}
				<LayoutDashboard class="h-4 w-4 text-accent" />
			{:else if icon === 'braces'}
				<Braces class="h-4 w-4 text-accent" />
			{:else if icon === 'radio'}
				<Radio class="h-4 w-4 text-accent" />
			{:else if icon === 'bot'}
				<Bot class="h-4 w-4 text-accent" />
			{:else if icon === 'cloud'}
				<Cloud class="h-4 w-4 text-accent" />
			{:else if icon === 'server'}
				<ServerCog class="h-4 w-4 text-accent" />
			{:else if icon === 'shield'}
				<ShieldCheck class="h-4 w-4 text-accent" />
			{:else}
				<Globe2 class="h-4 w-4 text-accent" />
			{/if}
			{eyebrow}
		</div>
		<h2 class="text-2xl font-bold tracking-tight text-text md:text-3xl">{title}</h2>
	</div>
{/snippet}

{#snippet Step(number: string, title: string, description: string)}
	<div class="rounded-lg border border-border bg-surface p-5">
		<div class="mb-3 flex items-center gap-3">
			<div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-accent text-sm font-bold text-white">
				{number}
			</div>
			<h3 class="font-semibold">{title}</h3>
		</div>
		<p class="text-sm leading-6 text-text-muted">{description}</p>
	</div>
{/snippet}

{#snippet FeatureCard(title: string, description: string)}
	<article class="rounded-lg border border-border bg-surface p-5">
		<h3 class="text-sm font-semibold text-text">{title}</h3>
		<p class="mt-2 text-sm leading-6 text-text-muted">{description}</p>
	</article>
{/snippet}

{#snippet CodeBlock(title: string, code: string)}
	<div class="overflow-hidden rounded-lg border border-border bg-[#101014]">
		<div class="flex items-center justify-between border-b border-border px-4 py-3">
			<div class="text-sm font-semibold text-text">{title}</div>
			<Code2 class="h-4 w-4 text-text-dim" />
		</div>
		<pre class="overflow-x-auto p-4 text-xs leading-6 text-text-muted"><code>{code}</code></pre>
	</div>
{/snippet}
