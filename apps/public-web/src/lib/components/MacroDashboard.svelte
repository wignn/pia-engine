<script lang="ts">
	import { RefreshCw, AlertCircle, Database, TrendingUp, Sparkles, Layers } from 'lucide-svelte';
	import { fetchMacroData, macroState } from '$lib/stores/macro';
	import {
		FEAR_GREED_COMPONENTS,
		YIELD_CONTEXT_TENORS,
		YIELD_CURVE_TENORS,
		type FearGreedComponent,
		type RateObservation
	} from '$lib/types';

	const TENOR_LABELS: Record<string, string> = {
		'3M': '3 Month',
		'2Y': '2 Year',
		'5Y': '5 Year',
		'10Y': '10 Year',
		'30Y': '30 Year',
		'10Y_REAL': '10Y Real Yield',
		'10Y_BREAKEVEN': '10Y Breakeven'
	};

	const COMPONENT_LABELS: Record<string, string> = {
		momentum: 'Momentum',
		volatility: 'Volatility',
		safe_haven: 'Safe Haven',
		news_risk: 'News Risk',
		positioning: 'Positioning'
	};

	const COMPONENT_WEIGHTS: Record<string, number> = {
		momentum: 0.25,
		volatility: 0.2,
		safe_haven: 0.2,
		news_risk: 0.2,
		positioning: 0.15
	};

	let state = $derived($macroState);
	let curvePoints = $derived.by(() => {
		const points = state.yieldCurve?.points ?? [];
		return YIELD_CURVE_TENORS.map((tenor) => points.find((point) => point.tenor === tenor)).filter(
			(point): point is RateObservation => point !== undefined
		);
	});

	let contextPoints = $derived.by(() => {
		const points = state.yieldCurve?.points ?? [];
		return YIELD_CONTEXT_TENORS.map((tenor) =>
			points.find((point) => point.tenor === tenor)
		).filter((point): point is RateObservation => point !== undefined);
	});

	let spreadPoints = $derived(state.yieldCurve?.spreads ?? []);
	let curveMin = $derived(
		curvePoints.length ? Math.min(...curvePoints.map((point) => point.value)) : 0
	);
	let curveMax = $derived(
		curvePoints.length ? Math.max(...curvePoints.map((point) => point.value)) : 1
	);
	let curveRange = $derived(Math.max(curveMax - curveMin, 0.4));
	
	let curveShape = $derived.by(() => {
		const short = curvePoints.find((point) => point.tenor === '3M')?.value;
		const long = curvePoints.find((point) => point.tenor === '10Y')?.value;
		if (short === undefined || long === undefined) return 'unknown';
		return long >= short ? 'normal' : 'inverted';
	});

	let fearRecord = $derived(state.fearGreed);
	let fearComponents = $derived(state.fearGreedComponents?.components ?? []);
	let fearScore = $derived(state.fearGreedComponents?.score ?? fearRecord?.score ?? null);
	let fearLabel = $derived(state.fearGreedComponents?.label ?? fearRecord?.label ?? 'neutral');

	let historyValues = $derived(state.fearGreedHistory?.data ?? []);
	let historyPath = $derived.by(() => {
		if (historyValues.length < 2) return '';
		return historyValues
			.map((item, index) => {
				const x = 8 + (index / (historyValues.length - 1)) * 184;
				const y = 58 - (item.score / 100) * 48;
				return `${index === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
			})
			.join(' ');
	});

	const width = 760;
	const height = 240;
	const paddingX = 50;
	const paddingY = 40;

	function getX(index: number, count: number): number {
		if (count <= 1) return width / 2;
		return paddingX + (index / (count - 1)) * (width - paddingX * 2);
	}

	function getY(value: number): number {
		const normalized = (value - curveMin) / curveRange;
		return height - paddingY - normalized * (height - paddingY * 2);
	}

	// Smooth cubic bezier spline
	let splinePath = $derived.by(() => {
		if (curvePoints.length === 0) return '';
		const pts = curvePoints.map((p, i) => ({ x: getX(i, curvePoints.length), y: getY(p.value) }));
		if (pts.length === 1) return `M ${pts[0].x} ${pts[0].y}`;
		
		let d = `M ${pts[0].x.toFixed(1)} ${pts[0].y.toFixed(1)}`;
		for (let i = 0; i < pts.length - 1; i++) {
			const p0 = pts[Math.max(0, i - 1)];
			const p1 = pts[i];
			const p2 = pts[i + 1];
			const p3 = pts[Math.min(pts.length - 1, i + 2)];

			const cp1x = p1.x + (p2.x - p0.x) / 6;
			const cp1y = p1.y + (p2.y - p0.y) / 6;
			const cp2x = p2.x - (p3.x - p1.x) / 6;
			const cp2y = p2.y - (p3.y - p1.y) / 6;

			d += ` C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${p2.x.toFixed(1)} ${p2.y.toFixed(1)}`;
		}
		return d;
	});

	let areaPath = $derived.by(() => {
		if (!splinePath || curvePoints.length === 0) return '';
		const firstX = getX(0, curvePoints.length).toFixed(1);
		const lastX = getX(curvePoints.length - 1, curvePoints.length).toFixed(1);
		const bottomY = (height - paddingY).toFixed(1);
		return `${splinePath} L ${lastX} ${bottomY} L ${firstX} ${bottomY} Z`;
	});

	function formatPercent(value: number | null | undefined, digits = 2): string {
		return value === null || value === undefined || !Number.isFinite(value)
			? '—'
			: `${value.toFixed(digits)}%`;
	}

	function formatSpread(value: number | null | undefined): string {
		if (value === null || value === undefined || !Number.isFinite(value)) return '—';
		return `${value > 0 ? '+' : ''}${value.toFixed(2)}%`;
	}

	function spreadTone(value: number): string {
		return value < 0 ? 'text-red-400' : value > 0 ? 'text-emerald-400' : 'text-text-muted';
	}

	function formatDate(value: string | null | undefined): string {
		if (!value) return 'Date unavailable';
		const parsed = new Date(value);
		if (Number.isNaN(parsed.getTime())) return 'Date unavailable';
		return new Intl.DateTimeFormat('en-US', {
			timeZone: 'UTC',
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		}).format(parsed);
	}

	function scoreColor(score: number | null): string {
		if (score === null) return 'text-text-dim';
		if (score < 25) return 'text-red-400';
		if (score < 45) return 'text-amber-400';
		if (score < 55) return 'text-text-muted';
		return score < 75 ? 'text-emerald-400' : 'text-emerald-300';
	}
</script>

<div class="flex flex-col gap-6">
	<!-- Section Header -->
	<div class="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
		<div>
			<div class="flex items-center gap-2.5">
				<div class="flex h-7 w-7 items-center justify-center rounded-lg border border-accent/20 bg-accent/10">
					<TrendingUp class="h-4 w-4 text-accent" />
				</div>
				<h2 class="text-xl font-bold tracking-tight text-text">Macro Intelligence & Yield Curve</h2>
				<span class="rounded-full border border-border/80 bg-surface px-2.5 py-0.5 font-mono text-[10px] font-semibold text-text-muted">
					US Treasuries
				</span>
			</div>
			<p class="mt-1 text-xs text-text-muted">
				Institutional term structure, yield curve inversions, and cross-asset macroeconomic indicators.
			</p>
		</div>

		<div class="flex items-center gap-3">
			{#if state.loading}
				<span class="flex items-center gap-1.5 font-mono text-xs text-accent">
					<RefreshCw class="h-3.5 w-3.5 animate-spin" /> Live Syncing
				</span>
			{/if}
			<button
				class="flex h-8 items-center gap-2 rounded-lg border border-border bg-surface px-3 text-xs font-semibold text-text-muted shadow-xs transition hover:bg-surface-2 hover:text-text disabled:opacity-50"
				onclick={() => fetchMacroData()}
				disabled={state.loading}
			>
				<RefreshCw class="h-3.5 w-3.5 {state.loading ? 'animate-spin' : ''}" />
				<span>Refresh</span>
			</button>
		</div>
	</div>

	<!-- Top Metrics Strip -->
	<div class="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-6">
		<div class="flex flex-col justify-between rounded-xl border border-border/70 bg-surface/80 p-3.5 backdrop-blur-sm shadow-xs">
			<span class="text-[11px] font-medium tracking-wide text-text-muted">Curve Slope</span>
			<div class="mt-2 flex items-baseline gap-2">
				<span class="text-lg font-bold tracking-tight {curveShape === 'normal' ? 'text-emerald-400' : 'text-red-400'} capitalize">
					{curveShape}
				</span>
				<span class="text-[10px] font-mono text-text-dim">
					{curveShape === 'normal' ? 'Expansionary' : 'Recession Watch'}
				</span>
			</div>
			<span class="mt-1 text-[10px] text-text-dim">3M vs 10Y maturity spread</span>
		</div>

		{#each spreadPoints as sp (sp.spread)}
			<div class="flex flex-col justify-between rounded-xl border border-border/70 bg-surface/80 p-3.5 backdrop-blur-sm shadow-xs">
				<span class="text-[11px] font-medium tracking-wide text-text-muted">Spread {sp.spread}</span>
				<div class="mt-2 flex items-baseline gap-2">
					<span class="font-mono text-lg font-bold tracking-tight {spreadTone(sp.value)}">
						{formatSpread(sp.value)}
					</span>
				</div>
				<span class="mt-1 text-[10px] text-text-dim">
					{sp.value < 0 ? 'Inverted term spread' : 'Positive steepness'}
				</span>
			</div>
		{/each}

		{#each contextPoints as cp (cp.tenor)}
			<div class="flex flex-col justify-between rounded-xl border border-border/70 bg-surface/80 p-3.5 backdrop-blur-sm shadow-xs">
				<span class="text-[11px] font-medium tracking-wide text-text-muted">{TENOR_LABELS[cp.tenor]}</span>
				<div class="mt-2 flex items-baseline gap-2">
					<span class="font-mono text-lg font-bold tracking-tight text-text">
						{formatPercent(cp.value)}
					</span>
				</div>
				<span class="mt-1 text-[10px] text-text-dim">TIPS Market Benchmark</span>
			</div>
		{/each}
	</div>

	<!-- Main Yield Curve Chart & Cards -->
	<div class="overflow-hidden rounded-xl border border-border/80 bg-surface shadow-xs">
		<!-- Chart Header -->
		<div class="flex flex-wrap items-center justify-between gap-3 border-b border-border/70 px-5 py-3.5 bg-surface-2/20">
			<div class="flex items-center gap-2.5">
				<span class="flex h-2 w-2 rounded-full bg-accent animate-pulse"></span>
				<span class="text-xs font-bold tracking-wide uppercase text-text">US Treasury Benchmark Yield Curve</span>
			</div>
			<div class="flex items-center gap-4 text-xs">
				<div class="flex items-center gap-1.5">
					<span class="h-2 w-4 rounded-xs bg-accent"></span>
					<span class="text-text-muted">Nominal Yield</span>
				</div>
				<div class="flex items-center gap-1.5 text-text-dim font-mono text-[11px]">
					<span>Date: {formatDate(state.yieldCurve?.date)}</span>
					<span>•</span>
					<span>Source: FRED / Treasury.gov</span>
				</div>
			</div>
		</div>

		{#if curvePoints.length === 0}
			<div class="flex min-h-[300px] flex-col items-center justify-center p-8 text-center">
				<Database class="h-8 w-8 text-text-dim" />
				<p class="mt-2 text-sm font-semibold text-text-muted">No Yield Curve Data Available</p>
			</div>
		{:else}
			<!-- Interactive SVG Yield Curve Graphic -->
			<div class="relative px-6 pt-6 pb-2">
				<svg viewBox="0 0 {width} {height}" class="h-[260px] w-full overflow-visible" role="img">
					<defs>
						<!-- Soft Gradient Fill -->
						<linearGradient id="yieldAreaGrad" x1="0" y1="0" x2="0" y2="1">
							<stop offset="0%" stop-color="var(--color-accent, #3b82f6)" stop-opacity="0.28" />
							<stop offset="70%" stop-color="var(--color-accent, #3b82f6)" stop-opacity="0.04" />
							<stop offset="100%" stop-color="var(--color-accent, #3b82f6)" stop-opacity="0.0" />
						</linearGradient>
						<!-- Subtle Grid Pattern -->
						<linearGradient id="yieldLineGrad" x1="0" y1="0" x2="1" y2="0">
							<stop offset="0%" stop-color="var(--color-accent, #3b82f6)" />
							<stop offset="100%" stop-color="#60a5fa" />
						</linearGradient>
					</defs>

					<!-- Y-Axis Grid Lines & Ticks -->
					{#each [curveMax, curveMin + curveRange * 0.66, curveMin + curveRange * 0.33, curveMin] as tickValue}
						{@const yPos = getY(tickValue)}
						<line
							x1={paddingX}
							y1={yPos}
							x2={width - paddingX}
							y2={yPos}
							stroke="currentColor"
							stroke-opacity="0.07"
							stroke-dasharray="3 3"
						/>
						<text
							x={paddingX - 12}
							y={yPos + 3.5}
							text-anchor="end"
							fill="currentColor"
							class="text-[10px] font-mono fill-text-dim font-medium"
						>
							{tickValue.toFixed(2)}%
						</text>
					{/each}

					<!-- Shaded Area Under Curve -->
					<path d={areaPath} fill="url(#yieldAreaGrad)" />

					<!-- Smooth Interpolated Yield Line -->
					<path
						d={splinePath}
						fill="none"
						stroke="url(#yieldLineGrad)"
						stroke-width="2.75"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>

					<!-- Yield Tenor Points & Circles -->
					{#each curvePoints as point, i (point.tenor)}
						{@const px = getX(i, curvePoints.length)}
						{@const py = getY(point.value)}

						<!-- Vertical drop dashed line -->
						<line
							x1={px}
							y1={py}
							x2={px}
							y2={height - paddingY}
							stroke="currentColor"
							stroke-opacity="0.12"
							stroke-dasharray="2 2"
						/>

						<!-- Glowing Halo Circle -->
						<circle
							cx={px}
							cy={py}
							r="5.5"
							class="fill-surface stroke-accent"
							stroke-width="2.5"
						/>

						<!-- Value Badge on Top of Point -->
						<g transform="translate({px}, {py - 12})">
							<rect
								x="-21"
								y="-14"
								width="42"
								height="17"
								rx="4"
								class="fill-surface-2/95 stroke-border/70"
								stroke-width="1"
							/>
							<text
								x="0"
								y="-2.5"
								text-anchor="middle"
								class="fill-text font-mono text-[10.5px] font-bold"
							>
								{point.value.toFixed(2)}%
							</text>
						</g>

						<!-- X-Axis Tenor Label -->
						<text
							x={px}
							y={height - paddingY + 22}
							text-anchor="middle"
							class="fill-text-muted text-[11px] font-bold font-mono tracking-wider"
						>
							{point.tenor}
						</text>
					{/each}
				</svg>
			</div>

			<!-- Detailed Tenor Metric Row -->
			<div class="grid grid-cols-2 divide-x divide-y sm:grid-cols-5 border-t border-border/70 divide-border/60 bg-surface-2/10">
				{#each curvePoints as pt (pt.tenor)}
					<div class="flex flex-col px-4 py-3.5 transition hover:bg-surface-2/40">
						<span class="text-[10px] font-bold tracking-wider uppercase text-text-dim">{TENOR_LABELS[pt.tenor]}</span>
						<div class="mt-1 flex items-baseline justify-between">
							<span class="font-mono text-base font-bold text-text">{formatPercent(pt.value)}</span>
							<span class="text-[10px] font-mono text-text-dim">Maturity {pt.tenor}</span>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
