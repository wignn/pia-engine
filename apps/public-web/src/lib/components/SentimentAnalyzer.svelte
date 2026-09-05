<script lang="ts">
	import { apiFetch } from '$lib/api';

	let text = $state('');
	let loading = $state(false);
	let error = $state<string | null>(null);
	let result = $state<any>(null);

	async function handleAnalyze() {
		if (!text.trim()) return;
		loading = true;
		error = null;
		result = null;

		try {
			const res = await apiFetch('/api/v1/analyze', {
				method: 'POST',
				body: JSON.stringify({ text })
			});

			if (!res.ok) {
				throw new Error(`Analisis gagal: ${res.status} ${res.statusText}`);
			}

			const data = await res.json();
			if (data.error) {
				throw new Error(data.error);
			}

			result = data;
		} catch (e: any) {
			error = e.message || 'Terjadi kesalahan saat menganalisis teks.';
		} finally {
			loading = false;
		}
	}

	function handleClear() {
		text = '';
		result = null;
		error = null;
	}

	function getSentimentBadgeColor(sentiment: string) {
		const s = sentiment.toLowerCase();
		if (s === 'positive' || s === 'bullish') return 'bg-green/10 text-green border-green/20';
		if (s === 'negative' || s === 'bearish') return 'bg-red/10 text-red border-red/20';
		if (s === 'mixed') return 'bg-amber/10 text-amber border-amber/20';
		return 'bg-text-dim/10 text-text border-text-dim/20';
	}

	function getHighlightBg(sentiment: string) {
		const s = sentiment.toLowerCase();
		if (s === 'positive' || s === 'bullish')
			return 'bg-green/10 text-green border-l-2 border-green px-2 py-1 rounded-r';
		if (s === 'negative' || s === 'bearish')
			return 'bg-red/10 text-red border-l-2 border-red px-2 py-1 rounded-r';
		if (s === 'mixed') return 'bg-amber/10 text-amber border-l-2 border-amber px-2 py-1 rounded-r';
		return 'bg-surface border-l-2 border-text-dim/40 px-2 py-1 rounded-r text-text';
	}
</script>

<div class="flex flex-col gap-6 p-6 lg:flex-row">
	<!-- Input panel -->
	<div class="flex flex-1 flex-col gap-4">
		<div class="flex flex-col gap-1">
			<label
				for="analyzer-input"
				class="text-xs font-semibold tracking-wider text-text-muted uppercase"
				>Masukkan Teks Finansial</label
			>
			<p class="text-xs text-text-dim">
				Masukkan berita, headline pasar, atau paragraf laporan keuangan untuk dianalisis oleh AI.
			</p>
		</div>
		<textarea
			id="analyzer-input"
			bind:value={text}
			placeholder="Ketik atau tempel berita di sini... (Contoh: 'Federal Reserve rates surged by 25 basis points exceeding expectations, driving stock gains.')"
			class="h-56 w-full resize-none rounded border border-border bg-surface-2 p-4 text-sm text-text placeholder-text-dim transition-colors focus:border-accent focus:outline-none"
			disabled={loading}
		></textarea>

		<div class="flex gap-3">
			<button
				onclick={handleAnalyze}
				disabled={loading || !text.trim()}
				class="flex flex-1 items-center justify-center gap-2 rounded bg-accent px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-accent/90 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if loading}
					<svg class="h-4 w-4 animate-spin text-white" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					Menganalisis...
				{:else}
					Mulai Analisis
				{/if}
			</button>

			{#if text || result}
				<button
					onclick={handleClear}
					class="rounded border border-border px-5 py-2.5 text-sm font-semibold text-text transition-colors hover:bg-surface-2"
				>
					Bersihkan
				</button>
			{/if}
		</div>

		{#if error}
			<div class="rounded border border-red/20 bg-red/10 p-3 text-xs text-red">
				{error}
			</div>
		{/if}
	</div>

	<!-- Results panel -->
	<div
		class="flex min-h-[300px] flex-1 flex-col justify-center rounded border border-border bg-surface-2 p-5"
	>
		{#if !result && !loading}
			<div class="flex flex-col items-center gap-3 text-center text-text-dim">
				<svg
					class="h-12 w-12 text-text-dim/40"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
					/>
				</svg>
				<p class="text-sm">Hasil analisis sentimen FinBERT Anda akan ditampilkan di sini.</p>
			</div>
		{:else if loading}
			<div class="flex flex-col items-center gap-3 text-center text-text-dim">
				<div class="flex w-full animate-pulse space-x-4 px-4">
					<div class="flex-1 space-y-4 py-1">
						<div class="h-4 w-3/4 rounded bg-border"></div>
						<div class="space-y-2">
							<div class="h-4 rounded bg-border"></div>
							<div class="h-4 w-5/6 rounded bg-border"></div>
						</div>
					</div>
				</div>
				<p class="mt-4 animate-pulse text-xs text-text-dim">
					Memproses dan menghitung probabilitas model...
				</p>
			</div>
		{:else if result}
			<div class="flex flex-col gap-4">
				<div class="flex items-center justify-between border-b border-border pb-3">
					<h4 class="text-xs font-semibold tracking-wider text-text-muted uppercase">
						Hasil Analisis
					</h4>
					<span
						class="rounded border px-2.5 py-1 text-xs font-bold uppercase {getSentimentBadgeColor(
							result.sentiment
						)}"
					>
						{result.sentiment}
					</span>
				</div>

				<!-- Distribution list -->
				<div class="flex flex-col gap-2">
					<div class="text-xs font-medium text-text-muted">Distribusi Sentimen:</div>
					{#if result.distribution}
						{#each Object.entries(result.distribution) as [label, prob]}
							{@const pct = ((prob as number) * 100).toFixed(1)}
							<div class="flex items-center gap-3 text-xs">
								<span class="w-16 font-medium text-text-dim capitalize">{label}</span>
								<div class="h-2 flex-1 overflow-hidden rounded-full bg-surface">
									<div
										class="h-full rounded-full transition-all duration-500"
										style="width: {pct}%; background-color: {label === 'positive'
											? 'var(--color-green, #089981)'
											: label === 'negative'
												? 'var(--color-red, #F23645)'
												: 'var(--color-text-dim, #787b86)'}"
									></div>
								</div>
								<span class="w-10 text-right font-mono font-bold text-text">{pct}%</span>
							</div>
						{/each}
					{/if}
				</div>

				<!-- Highlights -->
				{#if result.highlights && result.highlights.length > 0}
					<div class="mt-2 flex flex-col gap-2">
						<div class="text-xs font-medium text-text-muted">Analisis Per Kalimat:</div>
						<div class="flex max-h-48 flex-col gap-1.5 overflow-y-auto pr-1">
							{#each result.highlights as h}
								<div class="text-xs {getHighlightBg(h.sentiment)}">
									<span class="font-medium text-text">{h.sentence}</span>
									<span class="ml-1 font-mono text-[10px] text-text-dim/80"
										>({(h.score * 100).toFixed(0)}%)</span
									>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Entities -->
				{#if result.entities && (result.entities.tickers?.length > 0 || result.entities.currencies?.length > 0)}
					<div class="mt-2 flex flex-col gap-2 border-t border-border pt-3">
						<div class="text-xs font-medium text-text-muted">Entitas Terdeteksi:</div>
						<div class="flex flex-wrap gap-2">
							{#if result.entities.tickers && result.entities.tickers.length > 0}
								{#each result.entities.tickers as ticker}
									<span
										class="rounded border border-border bg-surface px-2 py-0.5 font-mono text-[10px] text-blue"
										>#{ticker}</span
									>
								{/each}
							{/if}
							{#if result.entities.currencies && result.entities.currencies.length > 0}
								{#each result.entities.currencies as curr}
									<span
										class="rounded border border-border bg-surface px-2 py-0.5 font-mono text-[10px] text-accent"
										>${curr}</span
									>
								{/each}
							{/if}
						</div>
					</div>
				{/if}

				<!-- Translation layer details -->
				{#if result.translated}
					<div class="mt-4 flex flex-col gap-2 border-t border-border pt-4">
						<div class="flex items-center justify-between">
							<span class="text-xs font-semibold text-text-muted">Terjemahan Otomatis</span>
							<span
								class="rounded border border-accent/20 bg-accent/15 px-1.5 py-0.5 font-mono text-[10px] text-accent"
							>
								{result.language?.toUpperCase()} → {result.analysis_language?.toUpperCase()}
							</span>
						</div>
						<div
							class="max-h-32 overflow-y-auto rounded border border-border bg-surface p-3 text-xs leading-relaxed text-text-muted italic"
						>
							"{result.translated_text}"
						</div>
					</div>
				{/if}

				<!-- Macro economic event info -->
				{#if result.event && result.event.type}
					<div class="mt-4 flex flex-col gap-2 border-t border-border pt-4">
						<div class="text-xs font-semibold text-text-muted">Detail Data Makroekonomi:</div>
						<div class="flex flex-col gap-2 rounded border border-border bg-surface p-3">
							<div class="flex items-center justify-between border-b border-border/60 pb-2">
								<span class="max-w-[180px] truncate text-xs font-bold text-text"
									>{result.event.type}</span
								>
								<span
									class="rounded border border-border bg-surface-2 px-1.5 py-0.5 text-[10px] font-bold text-text-muted uppercase"
								>
									{result.event.country || 'Global'}
								</span>
							</div>
							<div class="mt-1 grid grid-cols-3 gap-2 text-center text-xs">
								<div class="flex flex-col rounded bg-surface-2/40 p-1.5">
									<span class="text-[10px] text-text-dim">Actual</span>
									<span class="font-mono font-bold text-text"
										>{result.event.actual !== null && result.event.actual !== undefined
											? result.event.actual
											: '-'}
										{result.event.unit || ''}</span
									>
								</div>
								<div class="flex flex-col rounded bg-surface-2/40 p-1.5">
									<span class="text-[10px] text-text-dim">Forecast</span>
									<span class="font-mono font-bold text-text-muted"
										>{result.event.forecast !== null && result.event.forecast !== undefined
											? result.event.forecast
											: '-'}
										{result.event.unit || ''}</span
									>
								</div>
								<div class="flex flex-col rounded bg-surface-2/40 p-1.5">
									<span class="text-[10px] text-text-dim">Previous</span>
									<span class="font-mono font-bold text-text-muted"
										>{result.event.previous !== null && result.event.previous !== undefined
											? result.event.previous
											: '-'}
										{result.event.unit || ''}</span
									>
								</div>
							</div>
						</div>
					</div>
				{/if}

				<!-- Asset market impact grid -->
				{#if result.market_impact && Object.keys(result.market_impact).length > 0}
					<div class="mt-4 flex flex-col gap-2 border-t border-border pt-4">
						<div class="text-xs font-semibold text-text-muted">Proyeksi Dampak Aset:</div>
						<div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
							{#each Object.entries(result.market_impact) as [asset, impact]}
								{@const impLower = String(impact).toLowerCase()}
								<div
									class="flex flex-col items-center justify-center rounded border border-border bg-surface p-2 text-center"
								>
									<span class="text-[10px] font-bold tracking-wider text-text-dim uppercase"
										>{asset}</span
									>
									<span
										class="mt-1 rounded px-1.5 py-0.5 text-[10px] font-black uppercase
										{impLower === 'bullish' || impLower === 'positive'
											? 'border border-green/20 bg-green/10 text-green'
											: impLower === 'bearish' || impLower === 'negative'
												? 'border border-red/20 bg-red/10 text-red'
												: 'border border-text-dim/20 bg-text-dim/10 text-text'}"
									>
										{impact}
									</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- AI trading intelligence -->
				{#if result.final_signal || result.reason}
					<div class="mt-4 flex flex-col gap-3 border-t border-border pt-4">
						<div class="text-xs font-semibold text-text-muted">AI Trading Intelligence:</div>
						<div
							class="flex flex-col gap-3 rounded-lg border bg-surface p-4
							{result.final_signal?.toLowerCase() === 'buy' || result.final_signal?.toLowerCase() === 'bullish'
								? 'border-green/20 bg-green/5'
								: result.final_signal?.toLowerCase() === 'sell' ||
									  result.final_signal?.toLowerCase() === 'bearish'
									? 'border-red/20 bg-red/5'
									: 'border-border'}"
						>
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<span class="text-xs font-semibold text-text-dim">Signal:</span>
									<span
										class="rounded border px-2.5 py-0.5 text-sm font-black uppercase
										{result.final_signal?.toLowerCase() === 'buy' || result.final_signal?.toLowerCase() === 'bullish'
											? 'border-green/20 bg-green/10 text-green'
											: result.final_signal?.toLowerCase() === 'sell' ||
												  result.final_signal?.toLowerCase() === 'bearish'
												? 'border-red/20 bg-red/10 text-red'
												: 'border-text-dim/20 bg-text-dim/10 text-text'}"
									>
										{result.final_signal || 'NO_SIGNAL'}
									</span>
								</div>
								{#if result.confidence !== undefined && result.confidence !== null}
									<div class="flex items-center gap-2 text-xs">
										<span class="font-semibold text-text-dim">Confidence:</span>
										<span class="font-mono font-black text-text"
											>{(result.confidence * 100).toFixed(0)}%</span
										>
									</div>
								{/if}
							</div>

							{#if result.confidence !== undefined && result.confidence !== null}
								<div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
									<div
										class="h-full rounded-full transition-all duration-700
											{result.final_signal?.toLowerCase() === 'buy' || result.final_signal?.toLowerCase() === 'bullish'
											? 'bg-green'
											: result.final_signal?.toLowerCase() === 'sell' ||
												  result.final_signal?.toLowerCase() === 'bearish'
												? 'bg-red'
												: 'bg-text-dim'}"
										style="width: {result.confidence * 100}%"
									></div>
								</div>
							{/if}

							{#if result.reason}
								<p
									class="mt-1 rounded border border-border/50 bg-surface-2/40 p-2.5 text-xs leading-relaxed text-text-muted italic"
								>
									"{result.reason}"
								</p>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>
