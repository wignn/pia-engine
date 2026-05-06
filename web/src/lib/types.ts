export interface PriceData {
	symbol: string;
	price: number;
	bid: number | null;
	ask: number | null;
	volume: number | null;
	source: string;
	asset_type: string;
	received_at: string | null;
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
