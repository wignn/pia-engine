/**
 * Single source of symbol display metadata (name, badge, format, logo).
 * Previously duplicated across PriceChart, MarketHeatmap, and MarketGrid
 * with per-coin hardcoded hex colors — now one definition with neutral
 * badge classes.
 */

import { getLocalLogo } from '$lib/logo';
import { getCompanyInfo } from '$lib/market-companies';

export type AssetCategory = 'stocks' | 'forex' | 'indices' | 'crypto' | 'commodities' | 'other';

export interface SymbolMeta {
	name: string;
	badge: string;
	badgeClass: string;
	unit: string;
	format: (val: number) => string;
	logo: { type: 'img' | 'svg'; url: string };
	displaySymbol: string;
	sector?: string;
	marketCap?: number;
	svgLogo?: string;
}

export function getAssetCategory(itemOrSymbol: { symbol: string; asset_type?: string | null } | string): AssetCategory {
	const symbol = typeof itemOrSymbol === 'string' ? itemOrSymbol : itemOrSymbol.symbol;
	const assetType = (typeof itemOrSymbol === 'string' ? '' : (itemOrSymbol.asset_type ?? '')).toLowerCase();
	if (['stock', 'stocks', 'equity', 'saham'].includes(assetType)) return 'stocks';
	if (['forex', 'fx', 'currency'].includes(assetType)) return 'forex';
	if (['index', 'indices', 'global_index'].includes(assetType)) return 'indices';
	if (['crypto', 'cryptocurrency'].includes(assetType)) return 'crypto';
	if (['commodity', 'commodities', 'metal', 'energy'].includes(assetType)) return 'commodities';

	const sym = symbol.toUpperCase();
	if (sym.endsWith('USDT')) return 'crypto';
	if (sym === 'XAUUSD' || sym.startsWith('XAU') || sym === 'WTI' || sym === 'BRENT' || sym === 'USOIL' || sym === 'UKOIL' || sym === 'NATGAS') return 'commodities';
	if (/^[A-Z]{6}$/.test(sym)) return 'forex';
	if (['SPX', 'DXY', 'IHSG', 'HSI', 'KOSPI', 'NIFTY', 'NIFTY50', 'SSEC', 'STI', 'ASX', 'ASX200', 'SANSEX', 'SENSEX', 'JCI', 'DJI', 'NDX', 'RUT', 'FTSE', 'GDAXI', 'FCHI', 'N225', 'VIX'].includes(sym))
		return 'indices';
	return 'stocks';
}

function formatPrice(val: number, category: AssetCategory, symbol: string): string {
	if (category === 'crypto') {
		return val.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 2 });
	}
	if (category === 'stocks' || category === 'indices') {
		return val.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 2 });
	}
	if (category === 'forex') {
		if (symbol.includes('JPY')) return val.toFixed(3);
		if (symbol.includes('IDR')) return val.toLocaleString(undefined, { maximumFractionDigits: 2 });
		return val.toFixed(5);
	}
	return val.toLocaleString(undefined, { minimumFractionDigits: 1, maximumFractionDigits: 2 });
}

/** Neutral badge classes — no per-coin neon hex. */
const BADGE_ACCENT = 'bg-accent/10 text-accent border border-accent/20';
const BADGE_GREEN = 'bg-green/10 text-green border border-green/20';
const BADGE_BLUE = 'bg-blue/10 text-blue border border-blue/20';

export function getSymbolMeta(symbol: string): SymbolMeta {
	const sym = symbol.toUpperCase();
	const category = getAssetCategory(sym);
	const company = getCompanyInfo(sym);

	let name = company ? company.name : sym;
	let badge = sym.substring(0, 4);
	let badgeClass = BADGE_ACCENT;
	let unit: string =
		category === 'forex' ? 'RATE' : category === 'stocks' ? 'EQTY' : category === 'indices' ? 'IDX' : 'USD';
	let format = (val: number) => formatPrice(val, category, sym);
	let logo: { type: 'img' | 'svg'; url: string } = { type: 'svg', url: '' };
	let displaySymbol = company?.displaySymbol || sym;
	let sector = company?.sector;
	let marketCap = company?.marketCapBillion;
	let svgLogo = company?.svgLogo;

	const usd = (digits: number) => (val: number) =>
		`$${val.toLocaleString(undefined, { minimumFractionDigits: digits, maximumFractionDigits: digits })}`;

	switch (sym) {
		case 'BTCUSDT':
			badge = 'BTC';
			unit = 'USD';
			format = usd(2);
			break;
		case 'ETHUSDT':
			badge = 'ETH';
			unit = 'USD';
			format = usd(2);
			break;
		case 'SOLUSDT':
			badge = 'SOL';
			unit = 'USD';
			format = usd(2);
			break;
		case 'BNBUSDT':
			badge = 'BNB';
			unit = 'USD';
			format = usd(2);
			break;
		case 'PAXGUSDT':
			badge = 'PAXG';
			displaySymbol = 'PAXG';
			unit = 'USD';
			format = usd(1);
			break;
		case 'XAUUSD':
			badge = 'GOLD';
			badgeClass = BADGE_GREEN;
			unit = 'USD';
			format = usd(2);
			break;
		case 'EURUSD':
			badge = 'EUR';
			badgeClass = BADGE_BLUE;
			unit = 'RATE';
			format = (val) => val.toFixed(5);
			break;
		case 'GBPUSD':
			badge = 'GBP';
			badgeClass = BADGE_BLUE;
			unit = 'RATE';
			format = (val) => val.toFixed(5);
			break;
		case 'USDJPY':
			badge = 'JPY';
			badgeClass = BADGE_BLUE;
			unit = 'JPY';
			format = (val) => val.toFixed(3);
			break;
		case 'AUDUSD':
			badge = 'AUD';
			badgeClass = BADGE_BLUE;
			unit = 'RATE';
			format = (val) => val.toFixed(5);
			break;
		case 'SPX':
			badge = 'SPX';
			badgeClass = BADGE_BLUE;
			unit = 'USD';
			format = (val) =>
				val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
			break;
		case 'DXY':
			badge = 'DXY';
			badgeClass = BADGE_GREEN;
			unit = 'RATE';
			format = (val) => val.toFixed(3);
			break;
		case 'WTI':
		case 'USOIL':
			badge = 'WTI';
			unit = 'USD';
			format = usd(2);
			break;
		default:
			if (category === 'stocks') {
				badge = sym.slice(0, 4);
			} else if (category === 'indices') {
				badge = sym.slice(0, 5);
			} else if (category === 'forex') {
				badge = sym.slice(0, 3);
			}
			break;
	}

	const localLogoUrl = getLocalLogo(sym);
	if (localLogoUrl) {
		logo = { type: 'img', url: localLogoUrl };
	}

	return { name, badge, badgeClass, unit, format, logo, displaySymbol, sector, marketCap, svgLogo };
}
