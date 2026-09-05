export interface PriceData {
	symbol: string;
	price: number;
	bid: number | null;
	ask: number | null;
	volume: number | null;
	source: string;
	asset_type: string;
	received_at: string | null;
	session?: {
		exchange: string;
		timezone: string;
		state: 'open' | 'closed' | 'break' | 'unknown' | string;
		is_open: boolean;
		reason: string;
	};
	direction: 'up' | 'down' | 'none';
	prev_price: number;
	updated_at: number; // timestamp ms
}

export interface NewsItem {
	id: string;
	title: string;
	original_title?: string;
	translated_title?: string;
	summary: string | null;
	source_name: string;
	original_url?: string;
	url?: string;
	sentiment: string | null;
	impact_level: string | null;
	published_at: string | null;
	processed_at: string | null;
	currency_pairs?: string | null;
	tickers?: string | null;
	category?: string;
}

export interface CalendarEvent {
	title: string;
	currency: string;
	date: string;
	impact: string;
	forecast: string;
	previous: string;
	actual: string;
}

export interface WhyMoveCause {
	kind: string;
	title: string;
	summary: string | null;
	source_name: string | null;
	url: string | null;
	published_at: string | null;
	processed_at: string | null;
	sentiment: string | null;
	impact_level: string | null;
	matched_terms: string[];
	score: number;
	reason: string;
}

export interface WhyMoveNarrative {
	headline: string;
	explanation: string;
	drivers: string[];
	confidence: string;
	caveats: string[];
}

export interface WhyMoveCrossAsset {
	symbol: string;
	asset_type: string;
	move_pct: number;
	direction: 'up' | 'down' | string;
	latest_price: number;
	tick_count: number;
	latest_at: string;
	relationship: string;
}

export interface WhyMoveDriver {
	name: string;
	score: number;
	evidence: string[];
}

export interface WhyMoveConfidence {
	label: 'low' | 'medium' | 'high' | string;
	score: number;
	breakdown: Record<string, number>;
}

export interface WhyMoveResponse {
	symbol: string;
	window: string;
	lookback_minutes?: number;
	move: {
		latest_price?: number | null;
		baseline_price?: number | null;
		move_pct?: number | null;
		direction: 'up' | 'down' | 'none' | string;
		severity?: 'medium' | 'high' | string | null;
		threshold_pct?: number | null;
		tick_count: number;
		latest_at?: string | null;
		is_active_spike?: boolean;
	} | null;
	headline?: string;
	explanation?: string;
	summary?: string;
	confidence: WhyMoveConfidence | 'low' | 'medium' | 'high' | string;
	matched_terms?: string[];
	drivers: WhyMoveDriver[] | string[];
	news_clusters?: Array<{ theme: string; score: number; sentiment: string; headlines: string[] }>;
	cross_assets: WhyMoveCrossAsset[];
	llm: {
		provider: string;
		model: string | null;
		status: 'generated' | 'disabled' | 'failed' | 'fallback' | string;
		narrative: WhyMoveNarrative | null;
	};
	engine?: { status: string; version: string };
	cache?: { status: string; evidence_hash?: string };
	causes: {
		news: WhyMoveCause[];
		calendar: unknown[];
	};
	evidence?: unknown;
	generated_at: string;
}

export interface Feature {
	icon: string;
	title: string;
	description: string;
	command: string;
}

export interface Command {
	name: string;
	description: string;
	permission?: string;
	category: string;
}

export interface OptionsSnapshot {
	id: string;
	symbol: string;
	underlying_price: number;
	put_call_ratio: number;
	max_pain_strike: number;
	total_open_interest: number;
	total_volume: number;
	total_gex: number;
	iv_atm: number | null;
	updated_at: string;
}

export interface OptionsContract {
	contract_symbol: string;
	symbol: string;
	option_type: 'call' | 'put' | string;
	strike: number;
	expiration_date: string;
	mark_price: number;
	bid: number | null;
	ask: number | null;
	implied_volatility: number;
	delta: number;
	gamma: number;
	theta: number;
	vega: number;
	gex: number;
	open_interest: number;
	volume: number;
	updated_at: string;
}

export interface OptionsGexItem {
	strike: number;
	call_gex: number;
	put_gex: number;
	total_gex: number;
}

export interface RateObservation {
	source: string;
	country: string;
	tenor: string;
	date: string;
	value: number;
	unit: string;
	raw_series_id: string;
	updated_at: string;
}

export interface RateSpreadObservation {
	country: string;
	spread: string;
	date: string;
	value: number;
	updated_at: string;
}

export interface YieldCurveData {
	country: string;
	source: string;
	date: string | null;
	points: RateObservation[];
	spreads: RateSpreadObservation[];
	stale: boolean;
	updated_at: string | null;
}

export interface FearGreedRecord {
	id: string;
	scope: string;
	date: string;
	score: number;
	label: string;
	components: Record<string, number>;
	source_status: Record<string, string>;
	created_at: string;
}

export interface FearGreedComponent {
	name: string;
	score: number | null;
	base_weight: number;
	status: string;
	description: string;
}

export interface FearGreedComponentsResponse {
	scope: string;
	score: number;
	label: string;
	updated_at: string;
	components: FearGreedComponent[];
}

export interface FearGreedHistoryResponse {
	data: FearGreedRecord[];
	scope: string;
	total: number;
}

export interface BondSnapshot {
	symbol: string;
	name: string;
	yield: number;
	dayChange: number;
	monthChange: number;
	yearChange: number;
	date: string;
}

export interface BondHistoryPoint {
	date: string;
	value: number;
}

export interface BondHistorySeries {
	symbol: string;
	name: string;
	points: BondHistoryPoint[];
}

export interface ScrapedBondYieldCurve {
	country: string;
	source: string;
	as_of: string;
	fetched_at: string | null;
	updated_at: string;
	window: string;
	window_from: string;
	window_to: string;
	stale: boolean;
	history_available: boolean;
	history_kind: 'provider' | 'unavailable' | string;
	history_message: string | null;
	bonds: BondSnapshot[];
	history: BondHistorySeries[];
}

export interface MacroState {
	loading: boolean;
	yieldCurve: YieldCurveData | null;
	scrapedBondCurve: ScrapedBondYieldCurve | null;
	fearGreed: FearGreedRecord | null;
	fearGreedComponents: FearGreedComponentsResponse | null;
	fearGreedHistory: FearGreedHistoryResponse | null;
	errors: {
		yieldCurve: string | null;
		scrapedBondCurve: string | null;
		fearGreed: string | null;
		history: string | null;
	};
	lastFetchedAt: string | null;
}

export const FEAR_GREED_COMPONENTS = [
	'momentum',
	'volatility',
	'safe_haven',
	'news_risk',
	'positioning'
] as const;

export const YIELD_CURVE_TENORS = ['3M', '2Y', '5Y', '10Y', '30Y'] as const;

export const YIELD_CONTEXT_TENORS = ['10Y_REAL', '10Y_BREAKEVEN'] as const;
