<script lang="ts">
	import type { NewsItem } from '$lib/types';

	interface Props {
		forexItems: NewsItem[];
		stockItems: NewsItem[];
		selectedSymbol?: string;
	}

	let { forexItems, stockItems, selectedSymbol = 'SPX' }: Props = $props();

	let activeTab = $state<'global' | 'filtered'>('global');

	const synonyms: Record<string, string[]> = {
		SPX: [
			'SPX',
			'S&P 500',
			'S&P500',
			'US500',
			'STOCK',
			'SAHAM',
			'EQUITY',
			'FED',
			'FEDERAL RESERVE',
			'INFLASI',
			'INFLATION'
		],
		XAUUSD: ['XAUUSD', 'XAU', 'GOLD', 'EMAS', 'METAL', 'COMMODITY', 'FED', 'INFLATION'],
		BTCUSDT: ['BTCUSDT', 'BTC', 'BITCOIN', 'CRYPTO', 'KRIPTO', 'USDT'],
		DXY: ['DXY', 'DOLLAR', 'USD', 'GREENBACK', 'FED', 'TREASURY', 'YIELD'],
		EURUSD: ['EURUSD', 'EUR', 'EURO', 'ECB'],
		GBPUSD: ['GBPUSD', 'GBP', 'POUND', 'BOE'],
		USDJPY: ['USDJPY', 'JPY', 'YEN', 'BOJ'],
		AUDUSD: ['AUDUSD', 'AUD', 'AUSSIME', 'RBA']
	};

	let filteredForex = $derived.by(() => {
		const sym = selectedSymbol.toUpperCase();
		const terms = synonyms[sym] || [sym];
		return forexItems.filter((item) => {
			const textToSearch =
				`${item.title} ${item.summary || ''} ${item.tickers || ''} ${item.currency_pairs || ''}`.toUpperCase();
			return terms.some((term) => textToSearch.includes(term));
		});
	});

	let filteredStock = $derived.by(() => {
		const sym = selectedSymbol.toUpperCase();
		const terms = synonyms[sym] || [sym];
		return stockItems.filter((item) => {
			const textToSearch =
				`${item.title} ${item.summary || ''} ${item.tickers || ''} ${item.currency_pairs || ''}`.toUpperCase();
			return terms.some((term) => textToSearch.includes(term));
		});
	});

	let hasFilteredData = $derived(filteredForex.length + filteredStock.length > 0);

	$effect(() => {
		if (selectedSymbol && hasFilteredData) {
			activeTab = 'filtered';
		} else {
			activeTab = 'global';
		}
	});

	let activeForex = $derived(activeTab === 'global' ? forexItems : filteredForex);
	let activeStock = $derived(activeTab === 'global' ? stockItems : filteredStock);
	let activeAll = $derived([...activeForex, ...activeStock]);

	function getStats(items: NewsItem[]) {
		let positive = 0;
		let negative = 0;
		let neutral = 0;
		for (const item of items) {
			const s = item.sentiment?.toLowerCase();
			if (s === 'positive' || s === 'bullish') positive++;
			else if (s === 'negative' || s === 'bearish') negative++;
			else if (s === 'neutral' || s === 'mixed') neutral++;
		}
		const total = positive + negative + neutral;
		const score = total > 0 ? (positive - negative) / total : 0;

		let moodLabel = 'NEUTRAL';
		let moodColor = 'text-text-dim';
		if (total > 0) {
			if (score > 0.6) {
				moodLabel = 'EXTREME GREED';
				moodColor = 'text-green font-bold';
			} else if (score > 0.15) {
				moodLabel = 'GREED';
				moodColor = 'text-green';
			} else if (score < -0.6) {
				moodLabel = 'EXTREME FEAR';
				moodColor = 'text-red font-bold';
			} else if (score < -0.15) {
				moodLabel = 'FEAR';
				moodColor = 'text-red';
			} else {
				moodLabel = 'NEUTRAL';
				moodColor = 'text-text-dim';
			}
		} else {
			moodLabel = 'NO DATA';
			moodColor = 'text-text-dim';
		}

		return { positive, negative, neutral, total, score, moodLabel, moodColor };
	}

	let stats = $derived(getStats(activeAll));
	let forexStats = $derived(getStats(activeForex));
	let stockStats = $derived(getStats(activeStock));

	let needleAngle = $derived(stats.total > 0 ? stats.score * 90 : 0);

	let sourceStats = $derived.by(() => {
		const map = new Map<
			string,
			{ positive: number; negative: number; neutral: number; total: number }
		>();
		for (const item of activeAll) {
			const name = item.source_name || 'Unknown';
			if (!map.has(name)) map.set(name, { positive: 0, negative: 0, neutral: 0, total: 0 });
			const entry = map.get(name)!;
			const s = item.sentiment?.toLowerCase();
			if (s === 'positive' || s === 'bullish') {
				entry.positive++;
				entry.total++;
			} else if (s === 'negative' || s === 'bearish') {
				entry.negative++;
				entry.total++;
			} else if (s === 'neutral' || s === 'mixed') {
				entry.neutral++;
				entry.total++;
			}
		}
		return [...map.entries()].sort((a, b) => b[1].total - a[1].total).slice(0, 5);
	});

	let recentAnalyzed = $derived(
		activeAll
			.filter((item) => item.sentiment)
			.sort((a, b) => {
				const da = a.processed_at || a.published_at || '';
				const db = b.processed_at || b.published_at || '';
				return db.localeCompare(da);
			})
			.slice(0, 5)
	);
	let analyzedCount = $derived(activeAll.filter((item) => item.sentiment).length);

	function sentimentBadge(s: string | null): string {
		if (!s) return 'bg-text-dim/10 text-text-dim border-text-dim/20';
		const v = s.toLowerCase();
		if (v === 'positive' || v === 'bullish') return 'bg-green/10 text-green border-green/20';
		if (v === 'negative' || v === 'bearish') return 'bg-red/10 text-red border-red/20';
		if (v === 'mixed' || v === 'neutral') return 'bg-amber/10 text-amber border-amber/20';
		return 'bg-text-dim/10 text-text border-text-dim/20';
	}

	function sentimentDot(s: string | null): string {
		if (!s) return 'bg-text-dim/50 border-border';
		const v = s.toLowerCase();
		if (v === 'positive' || v === 'bullish') return 'bg-green border-green/30';
		if (v === 'negative' || v === 'bearish') return 'bg-red border-red/30';
		if (v === 'mixed' || v === 'neutral') return 'bg-amber border-amber/30';
		return 'bg-text-dim/50 border-border';
	}

	function getTitle(item: NewsItem): string {
		return item.translated_title || item.original_title || item.title || item.summary || '';
	}

	function formatTime(iso: string | null): string {
		if (!iso) return '';
		try {
			const d = new Date(iso);
			return d.toLocaleString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
		} catch {
			return '';
		}
	}
</script>

<div class="flex flex-col gap-4 p-5">
	<!-- Tab Toggle Header -->
	<div class="flex items-center justify-between border-b border-border/60 pb-5">
		<div class="flex items-center gap-3">
			<span class="text-xs font-bold tracking-wider text-text-muted uppercase"
				>AI Market Sentiment</span
			>
			{#if activeTab === 'filtered'}
				<span
					class="animate-pulse rounded border border-accent/20 bg-accent/10 px-2 py-0.5 text-xs font-bold tracking-wider text-accent uppercase shadow-sm"
				>
					{selectedSymbol} Filtered
				</span>
			{/if}
		</div>
		<div class="flex gap-1.5 rounded-lg border border-border/50 bg-surface-2/80 p-1">
			<button
				onclick={() => (activeTab = 'global')}
				class="cursor-pointer rounded-md px-3 py-1.5 text-xs font-bold transition-all
				{activeTab === 'global'
					? 'border border-border bg-surface text-text shadow-sm'
					: 'text-text-dim hover:text-text'}"
			>
				Global Mood
			</button>
			<button
				onclick={() => {
					if (hasFilteredData) {
						activeTab = 'filtered';
					}
				}}
				disabled={!hasFilteredData}
				class="cursor-pointer rounded-md px-3 py-1.5 text-xs font-bold transition-all disabled:cursor-not-allowed disabled:opacity-50
				{activeTab === 'filtered'
					? 'border border-border bg-surface text-text shadow-sm'
					: 'text-text-dim hover:text-text'}"
				title={!hasFilteredData
					? `No sentiment data for ${selectedSymbol}`
					: `Show sentiment for ${selectedSymbol}`}
			>
				{selectedSymbol} Mood
			</button>
		</div>
	</div>

	<div
		class="grid grid-cols-1 items-center justify-items-center gap-6 rounded-2xl border border-border/80 bg-surface p-6 shadow-sm md:grid-cols-3"
	>
		<div class="flex w-full max-w-[220px] flex-col items-center justify-center gap-3 text-center">
			<span class="text-[11px] font-bold tracking-wider text-text-dim uppercase">Forex & Macro</span
			>
			<span class="text-sm font-black tracking-tight uppercase {forexStats.moodColor}">
				{forexStats.moodLabel}
			</span>

			<div
				class="relative my-1 h-2.5 w-full overflow-hidden rounded-full border border-border/50 bg-surface-2/80 shadow-inner"
			>
				<!-- Gradient sections -->
				<div
					class="absolute top-0 left-0 h-full w-[40%] bg-red/40 transition-all duration-700"
				></div>
				<div
					class="absolute top-0 left-[40%] h-full w-[20%] bg-text-dim/20 transition-all duration-700"
				></div>
				<div
					class="absolute top-0 left-[60%] h-full w-[40%] bg-green/40 transition-all duration-700"
				></div>
				<!-- Indicator pointer pin -->
				{#if forexStats.total > 0}
					<div
						class="absolute top-1/2 h-4 w-1.5 -translate-y-1/2 rounded-full border border-surface bg-text transition-all duration-1000"
						style="left: calc({(forexStats.score + 1) * 50}% - 3px);"
					></div>
				{/if}
			</div>

			<div class="flex w-full justify-between px-1 font-mono text-[10px] font-bold text-text-dim">
				<span class="text-red/80">{forexStats.negative} Fear</span>
				<span class="text-text-dim/80">{forexStats.neutral} Neu</span>
				<span class="text-green/80">{forexStats.positive} Greed</span>
			</div>
		</div>

		<div
			class="relative flex w-full max-w-[280px] flex-col items-center justify-center border-border/50 px-6 py-2 md:border-r md:border-l"
		>
			<div
				class="relative flex flex-col items-center justify-center overflow-visible"
			>
				<svg width="220" height="120" viewBox="0 0 220 120" class="overflow-visible">
					<defs>
						<linearGradient id="tv-gauge-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
							<stop offset="0%" stop-color="var(--color-red, #F23645)" />
							<stop offset="35%" stop-color="#FF7A85" />
							<stop offset="50%" stop-color="var(--color-text-dim, #787b86)" />
							<stop offset="65%" stop-color="#4CD2B9" />
							<stop offset="100%" stop-color="var(--color-green, #089981)" />
						</linearGradient>
						</defs>

					<path
						d="M 25 110 A 85 85 0 0 1 195 110"
						fill="none"
						stroke="var(--color-surface-2, #f0f3fa)"
						stroke-width="12"
						stroke-linecap="round"
					/>

					<path
						d="M 25 110 A 85 85 0 0 1 195 110"
						fill="none"
						stroke="url(#tv-gauge-gradient)"
						stroke-width="12"
						stroke-linecap="round"
						opacity="1"
					/>

					<!-- Center Pivot -->
					<circle cx="110" cy="110" r="8" fill="var(--color-text, #131722)" />
					<circle cx="110" cy="110" r="3" fill="var(--color-bg, #ffffff)" />

					<!-- Ticks around gauge -->
					<circle cx="25" cy="110" r="2" fill="var(--color-bg)" />
					<circle cx="49.88" cy="49.88" r="2" fill="var(--color-bg)" />
					<circle cx="110" cy="25" r="2" fill="var(--color-bg)" />
					<circle cx="170.12" cy="49.88" r="2" fill="var(--color-bg)" />
					<circle cx="195" cy="110" r="2" fill="var(--color-bg)" />

					<!-- Needle -->
					<g
						style="transform: rotate({needleAngle}deg); transform-origin: 110px 110px; transition: transform 1.2s cubic-bezier(0.175, 0.885, 0.32, 1.1);"
					>
						<path
							d="M 108.5 110 L 110 16 L 111.5 110 Z"
							fill="var(--color-text, #131722)"
							
						/>
					</g>
				</svg>

				<!-- Gauge bottom indicators -->
				<div
					class="absolute top-[95px] flex w-full justify-between px-2 text-[9px] font-black tracking-wider text-text-dim/80 uppercase"
				>
					<span>Fear</span>
					<span>Neutral</span>
					<span>Greed</span>
				</div>
			</div>

			<!-- Mood Text & Numeric Score -->
			<div class="mt-5 flex flex-col items-center">
				<span class="text-xl font-black tracking-tight uppercase {stats.moodColor}">
					{stats.moodLabel}
				</span>
				<span
					class="mt-1 rounded border border-border/50 bg-surface-2/50 px-2 py-0.5 font-mono text-[11px] font-bold text-text-dim"
				>
					Score: {stats.score > 0 ? '+' : ''}{stats.score.toFixed(2)}
				</span>
			</div>
		</div>

		<!-- Right: Stocks / Equities Indicator Gauge -->
		<div class="flex w-full max-w-[220px] flex-col items-center justify-center gap-3 text-center">
			<span class="text-[11px] font-bold tracking-wider text-text-dim uppercase"
				>Stocks & Equities</span
			>
			<span class="text-sm font-black tracking-tight uppercase {stockStats.moodColor}">
				{stockStats.moodLabel}
			</span>

			<div
				class="relative my-1 h-2.5 w-full overflow-hidden rounded-full border border-border/50 bg-surface-2/80 shadow-inner"
			>
				<!-- Gradient sections -->
				<div
					class="absolute top-0 left-0 h-full w-[40%] bg-red/40 transition-all duration-700"
				></div>
				<div
					class="absolute top-0 left-[40%] h-full w-[20%] bg-text-dim/20 transition-all duration-700"
				></div>
				<div
					class="absolute top-0 left-[60%] h-full w-[40%] bg-green/40 transition-all duration-700"
				></div>
				<!-- Indicator pointer pin -->
				{#if stockStats.total > 0}
					<div
						class="absolute top-1/2 h-4 w-1.5 -translate-y-1/2 rounded-full border border-surface bg-text transition-all duration-1000"
						style="left: calc({(stockStats.score + 1) * 50}% - 3px);"
					></div>
				{/if}
			</div>

			<div class="flex w-full justify-between px-1 font-mono text-[10px] font-bold text-text-dim">
				<span class="text-red/80">{stockStats.negative} Fear</span>
				<span class="text-text-dim/80">{stockStats.neutral} Neu</span>
				<span class="text-green/80">{stockStats.positive} Greed</span>
			</div>
		</div>
	</div>

	<!-- Pill Summary Row -->
	<div
		class="mt-1 flex flex-wrap items-center justify-center gap-3 rounded-lg border border-border/60 bg-surface px-4 py-2.5"
	>
		<div
			class="flex items-center gap-2 rounded-md border border-red/20 bg-red/10 px-2.5 py-1 text-xs"
		>
			<span class="h-2 w-2 rounded-full bg-red"></span>
			<span class="font-bold text-text-muted">Fear:</span>
			<span class="font-mono font-bold text-red">{stats.negative}</span>
		</div>
		<div
			class="flex items-center gap-2 rounded-md border border-text-dim/20 bg-text-dim/10 px-2.5 py-1 text-xs"
		>
			<span class="h-2 w-2 rounded-full border border-text-dim/50 bg-text-dim"></span>
			<span class="font-bold text-text-muted">Neutral:</span>
			<span class="font-mono font-bold text-text">{stats.neutral}</span>
		</div>
		<div
			class="flex items-center gap-2 rounded-md border border-green/20 bg-green/10 px-2.5 py-1 text-xs"
		>
			<span class="h-2 w-2 rounded-full bg-green"></span>
			<span class="font-bold text-text-muted">Greed:</span>
			<span class="font-mono font-bold text-green">{stats.positive}</span>
		</div>
		<div class="hidden h-5 w-px bg-border sm:block"></div>
		<div
			class="rounded-lg border border-border/50 bg-surface px-3 py-1.5 text-xs font-medium text-text-muted shadow-sm"
		>
			Based on <span class="font-mono font-bold text-text">{analyzedCount}</span> analyzed articles
		</div>
	</div>

	<!-- Bottom Section: Breakdown & Recent News -->
	<div class="mt-2 grid grid-cols-1 gap-5 lg:grid-cols-12">
		<!-- Source Breakdown (4 cols) -->
		<div
			class="flex flex-col rounded-xl border border-border/80 bg-surface p-5 shadow-sm lg:col-span-4"
		>
			<div
				class="mb-5 flex items-center justify-between border-b border-border/60 pb-3 text-xs font-bold tracking-wider text-text-muted uppercase"
			>
				<span>Sentiment by Source</span>
				<span
					class="rounded border border-border bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-text-dim"
					>Top 5</span
				>
			</div>

			{#if sourceStats.length > 0}
				<div class="flex flex-col gap-5">
					{#each sourceStats as [name, src]}
						<div class="group flex flex-col gap-2">
							<div class="flex items-center justify-between text-xs font-bold">
								<span
									class="max-w-[150px] truncate text-text transition-colors group-hover:text-accent"
									title={name}>{name}</span
								>
								<span
									class="rounded border border-border/50 bg-surface-2 px-1 font-mono text-[11px] text-text-dim"
									>{src.total}</span
								>
							</div>
							<div
								class="flex h-2.5 w-full overflow-hidden rounded-full border border-border/40 bg-surface-2 shadow-inner"
							>
								{#if src.total > 0}
									<div
										class="h-full bg-green transition-all duration-1000 ease-out"
										style="width: {(src.positive / src.total) * 100}%"
										title="Bullish"
									></div>
									<div
										class="h-full bg-text-dim/40 transition-all duration-1000 ease-out"
										style="width: {(src.neutral / src.total) * 100}%"
										title="Neutral"
									></div>
									<div
										class="h-full bg-red transition-all duration-1000 ease-out"
										style="width: {(src.negative / src.total) * 100}%"
										title="Bearish"
									></div>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div
					class="flex flex-1 items-center justify-center rounded-lg border border-dashed border-border/50 bg-surface-2/30 py-8 text-xs text-text-dim"
				>
					No source stats available
				</div>
			{/if}
		</div>

		<!-- Recently Analyzed (8 cols) -->
		<div
			class="flex flex-col rounded-xl border border-border/80 bg-surface p-5 shadow-sm lg:col-span-8"
		>
			<div
				class="mb-5 flex items-center justify-between border-b border-border/60 pb-3 text-xs font-bold tracking-wider text-text-muted uppercase"
			>
				<span>Recently Analyzed Feed</span>
				<span class="flex items-center gap-1.5 font-mono text-[10px] text-text-dim"
					><span class="h-1.5 w-1.5 animate-pulse rounded-full bg-green"></span>Live Stream</span
				>
			</div>

			{#if recentAnalyzed.length > 0}
				<div class="flex flex-col gap-2">
					{#each recentAnalyzed as item, i (item.id ?? i)}
						<div
							class="group flex cursor-pointer items-start gap-4 rounded-xl border border-transparent px-3 py-3 transition-all hover:border-border/60 hover:bg-surface-2/60 hover:shadow-sm"
						>
							<span
								class="mt-1.5 inline-block h-2.5 w-2.5 shrink-0 rounded-full border {sentimentDot(
									item.sentiment
								)} transition-transform duration-300 group-hover:scale-125"
							></span>
							<div class="min-w-0 flex-1">
								<div class="mb-1.5 flex items-center gap-2 text-[11px] text-text-dim">
									<span class="font-bold text-text-muted">{item.source_name || 'Source'}</span>
									<span>·</span>
									<span class="font-mono">{formatTime(item.processed_at ?? item.published_at)}</span
									>
								</div>
								{#if item.original_url || item.url}
									<a
										href={item.original_url ?? item.url}
										target="_blank"
										rel="noopener noreferrer"
										class="line-clamp-2 text-sm leading-relaxed font-medium text-text decoration-accent/50 underline-offset-2 transition-colors group-hover:text-accent hover:underline"
									>
										{getTitle(item)}
									</a>
								{:else}
									<p class="line-clamp-2 text-sm leading-relaxed font-medium text-text">
										{getTitle(item)}
									</p>
								{/if}
							</div>
							<span
								class="shrink-0 rounded-lg border px-2.5 py-1 text-[10px] font-black tracking-wider uppercase {sentimentBadge(
									item.sentiment
								)} shadow-sm transition-colors group-hover:border-current"
							>
								{item.sentiment ?? 'n/a'}
							</span>
						</div>
					{/each}
				</div>
			{:else}
				<div
					class="flex flex-1 items-center justify-center rounded-lg border border-dashed border-border/50 bg-surface-2/30 py-8 text-xs text-text-dim"
				>
					No recent analyzed news articles.
				</div>
			{/if}
		</div>
	</div>
</div>
