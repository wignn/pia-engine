/**
 * Comprehensive Market Companies Metadata
 * Provides market cap weights, company names, sectors, and vector SVG logos
 * for S&P 500 equities, IDX, crypto, forex, commodities, and global indices.
 */

export interface CompanyInfo {
	symbol: string;
	name: string;
	sector: string;
	marketCapBillion: number;
	displaySymbol?: string;
	svgLogo?: string;
}

export const US_EQUITIES: Record<string, CompanyInfo> = {
	AAPL: {
		symbol: 'AAPL',
		name: 'Apple Inc.',
		sector: 'Technology',
		marketCapBillion: 3500,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full fill-white"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M15.97 6.4c.67-.82 1.13-1.96.99-3.1-.98.04-2.18.66-2.88 1.48-.62.72-1.16 1.88-1.01 2.99 1.1.08 2.22-.55 2.9-1.37z"/></svg>`
	},
	NVDA: {
		symbol: 'NVDA',
		name: 'NVIDIA Corporation',
		sector: 'Technology',
		marketCapBillion: 3200,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full fill-[#76B900]"><path d="M8.9 14.7c0-2.3 1.9-4.2 4.2-4.2s4.2 1.9 4.2 4.2c0 2.3-1.9 4.2-4.2 4.2-2.3.1-4.2-1.8-4.2-4.2zm-4.7 0c0 4.9 4 8.9 8.9 8.9 4.9 0 8.9-4 8.9-8.9s-4-8.9-8.9-8.9c-4.9 0-8.9 4-8.9 8.9zm15.4-8.5C17.6 4.3 15 3.3 12 3.3 5.7 3.3.6 8.4.6 14.7c0 4.1 2.2 7.7 5.5 9.8l1.4-2.1c-2.7-1.8-4.5-4.8-4.5-8.2 0-5.3 4.3-9.6 9.6-9.6 2.5 0 4.8 1 6.5 2.5l1.5-1.9z"/></svg>`
	},
	MSFT: {
		symbol: 'MSFT',
		name: 'Microsoft Corporation',
		sector: 'Technology',
		marketCapBillion: 3100,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><path fill="#f25022" d="M1 1h10v10H1z"/><path fill="#7fba00" d="M13 1h10v10H13z"/><path fill="#00a4ef" d="M1 13h10v10H1z"/><path fill="#ffb900" d="M13 13h10v10H13z"/></svg>`
	},
	GOOGL: {
		symbol: 'GOOGL',
		name: 'Alphabet Inc.',
		sector: 'Communication Services',
		marketCapBillion: 2150,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><path fill="#4285F4" d="M23.7 12.3c0-.7-.06-1.4-.19-2.1H12v4.5h6.6c-.3 1.5-1.1 2.8-2.4 3.7v3.1h3.9c2.3-2.1 3.6-5.2 3.6-9.2z"/><path fill="#34A853" d="M12 24c3.2 0 6-1.1 7.9-2.9l-3.9-3.1c-1.1.7-2.5 1.2-4 1.2-3.1 0-5.8-2.1-6.7-4.9H1.2v3.1C3.3 21.5 7.3 24 12 24z"/><path fill="#FBBC05" d="M5.3 14.3c-.3-.7-.4-1.5-.4-2.3s.1-1.6.4-2.3V6.6H1.2C.5 8.2 0 10 0 12s.5 3.8 1.2 5.4l4.1-3.1z"/><path fill="#EA4335" d="M12 4.8c1.8 0 3.4.6 4.6 1.8l3.4-3.4C18 1.2 15.2 0 12 0 7.3 0 3.3 2.5 1.2 6.6l4.1 3.1c.9-2.8 3.6-4.9 6.7-4.9z"/></svg>`
	},
	AMZN: {
		symbol: 'AMZN',
		name: 'Amazon.com Inc.',
		sector: 'Consumer Cyclical',
		marketCapBillion: 2000,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><path fill="#FFFFFF" d="M13.7 14.9c-1.9 0-3.3-.4-4.7-1.2l.5-1.5c1.2.7 2.5 1.1 4.1 1.1 2.5 0 3.9-1.2 3.9-3 0-1.6-1-2.6-3.2-2.9l-1.9-.3c-2.9-.5-4.2-1.9-4.2-4.1 0-2.7 2.1-4.5 5.5-4.5 1.6 0 3 .3 4.1.8l-.5 1.5c-1-.5-2.2-.7-3.6-.7-2.4 0-3.6 1.2-3.6 2.8 0 1.5 1 2.4 3 2.7l1.9.3c3.1.5 4.5 2 4.5 4.3 0 2.8-2.1 4.8-5.8 4.8z"/><path fill="#FF9900" d="M2.1 19.5c5.4 3.3 12.3 3.3 17.7 0 .4-.3.9.1.5.5-3.8 4.5-10.7 5.4-16.1 2.2-.5-.3-.2-.9.3-.7z"/></svg>`
	},
	META: {
		symbol: 'META',
		name: 'Meta Platforms Inc.',
		sector: 'Communication Services',
		marketCapBillion: 1350,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#0866FF"><path d="M12 10.3c-1.3-1.8-2.9-3-4.7-3C3.9 7.3 1.5 9.8 1.5 13s2.4 5.7 5.8 5.7c2.2 0 4-1.2 5.1-3 1.1 1.8 2.9 3 5.1 3 3.4 0 5.8-2.5 5.8-5.7s-2.4-5.7-5.8-5.7c-1.8 0-3.4 1.2-4.7 3zm-4.7 6.4c-2.2 0-3.8-1.6-3.8-3.7s1.6-3.7 3.8-3.7c1.4 0 2.7.9 3.5 2.3-.9 1.4-2.1 2.3-3.5 2.3zm9.4 0c-1.4 0-2.6-.9-3.5-2.3.9-1.4 2.1-2.3 3.5-2.3 2.2 0 3.8 1.6 3.8 3.7s-1.6 3.7-3.8 3.7z"/></svg>`
	},
	BRKB: {
		symbol: 'BRKB',
		displaySymbol: 'BRK.B',
		name: 'Berkshire Hathaway',
		sector: 'Financial Services',
		marketCapBillion: 1000,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#0A2240"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="9" text-anchor="middle" letter-spacing="-0.5">BRK</text></svg>`
	},
	LLY: {
		symbol: 'LLY',
		name: 'Eli Lilly and Co.',
		sector: 'Healthcare',
		marketCapBillion: 880,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#D52B1E"/><text x="12" y="16" fill="#FFFFFF" font-family="serif" font-style="italic" font-weight="900" font-size="10" text-anchor="middle">Lilly</text></svg>`
	},
	TSLA: {
		symbol: 'TSLA',
		name: 'Tesla Inc.',
		sector: 'Consumer Cyclical',
		marketCapBillion: 750,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#E82127"><path d="M12 4.4c2.8 0 5.3.8 7.3 2.2l1.2-2.1C18 2.8 15.1 1.8 12 1.8S6 2.8 3.5 4.5l1.2 2.1c2-1.4 4.5-2.2 7.3-2.2zm0 4.5c1.8 0 3.5.5 4.8 1.4l1-1.8C16.2 7.4 14.2 6.8 12 6.8S7.8 7.4 6.2 8.5l1 1.8c1.3-.9 3-1.4 4.8-1.4zm0 2.2c-.4 0-.8.3-.8.8v9.9c0 .4.4.8.8.8s.8-.4.8-.8V11.9c0-.5-.4-.8-.8-.8z"/></svg>`
	},
	AVGO: {
		symbol: 'AVGO',
		name: 'Broadcom Inc.',
		sector: 'Technology',
		marketCapBillion: 750,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#CC092F"/><path d="M6 12h3l3-5 3 10 3-5h3" stroke="#FFFFFF" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>`
	},
	JPM: {
		symbol: 'JPM',
		name: 'JPMorgan Chase & Co.',
		sector: 'Financial Services',
		marketCapBillion: 650,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><path fill="#0A2540" d="M12 2L2 12l10 10 10-10L12 2z"/><circle cx="12" cy="12" r="4.5" fill="#FFFFFF"/></svg>`
	},
	WMT: {
		symbol: 'WMT',
		name: 'Walmart Inc.',
		sector: 'Consumer Defensive',
		marketCapBillion: 650,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#FFC220"><path d="M12 0l1.8 6.9L12 6l-1.8.9L12 0zm0 24l-1.8-6.9L12 18l1.8-.9L12 24zm12-12l-6.9-1.8.9 1.8-.9 1.8L24 12zM0 12l6.9 1.8-.9-1.8.9-1.8L0 12zm20.5 8.5l-6.2-3.6 1.9-.3 1.9 1.3 2.4 2.6zm-17-17l6.2 3.6-1.9.3-1.9-1.3L3.5 3.5zm17 0l-2.4 2.6-1.9-1.3 1.9-.3 6.2-3.6zm-17 17l2.4-2.6 1.9 1.3-1.9.3-6.2 3.6z"/></svg>`
	},
	V: {
		symbol: 'V',
		name: 'Visa Inc.',
		sector: 'Financial Services',
		marketCapBillion: 580,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><path fill="#1A1F71" d="M3.2 5.5l5.5 13h3.5l7.5-13h-3.6l-4.5 9-2.6-9H3.2z"/><path fill="#F7B600" d="M1.5 5.5L0 12.5l4.8-7h-3.3z"/></svg>`
	},
	UNH: {
		symbol: 'UNH',
		name: 'UnitedHealth Group',
		sector: 'Healthcare',
		marketCapBillion: 540,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#002677"/><path d="M12 5l6 3v5c0 4-3 7-6 8-3-1-6-4-6-8V8l6-3z" fill="#FFFFFF"/></svg>`
	},
	XOM: {
		symbol: 'XOM',
		name: 'Exxon Mobil Corporation',
		sector: 'Energy',
		marketCapBillion: 480,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#ED1B2D"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="8" text-anchor="middle">XOM</text></svg>`
	},
	MA: {
		symbol: 'MA',
		name: 'Mastercard Inc.',
		sector: 'Financial Services',
		marketCapBillion: 460,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="8" cy="12" r="7" fill="#EB001B"/><circle cx="16" cy="12" r="7" fill="#F79E1B"/><path fill="#FF5F00" d="M12 7.2a7 7 0 000 9.6 7 7 0 000-9.6z"/></svg>`
	},
	COST: {
		symbol: 'COST',
		name: 'Costco Wholesale Corp.',
		sector: 'Consumer Defensive',
		marketCapBillion: 400,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#005DAA"/><text x="12" y="16" fill="#E31837" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">COST</text></svg>`
	},
	HD: {
		symbol: 'HD',
		name: 'The Home Depot Inc.',
		sector: 'Consumer Cyclical',
		marketCapBillion: 380,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><rect x="1" y="1" width="22" height="22" rx="3" fill="#F96302"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="10" text-anchor="middle">HD</text></svg>`
	},
	JNJ: {
		symbol: 'JNJ',
		name: 'Johnson & Johnson',
		sector: 'Healthcare',
		marketCapBillion: 380,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#D51900"/><text x="12" y="16" fill="#FFFFFF" font-family="serif" font-weight="900" font-size="8.5" text-anchor="middle">J&amp;J</text></svg>`
	},
	ABBV: {
		symbol: 'ABBV',
		name: 'AbbVie Inc.',
		sector: 'Healthcare',
		marketCapBillion: 350,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#001489"/><path d="M7 16l5-8 5 8h-2.5l-2.5-4-2.5 4H7z" fill="#00D2D2"/></svg>`
	},
	BAC: {
		symbol: 'BAC',
		name: 'Bank of America',
		sector: 'Financial Services',
		marketCapBillion: 330,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#012169"/><path d="M6 10h12v2H6zm0 3h12v2H6z" fill="#E31837"/></svg>`
	},
	CRM: {
		symbol: 'CRM',
		name: 'Salesforce Inc.',
		sector: 'Technology',
		marketCapBillion: 310,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#00A1E0"><path d="M19.4 9.6c-.6-2.5-2.8-4.4-5.4-4.4-1.9 0-3.6 1-4.6 2.5C8.8 7.3 8 7 7.1 7 5 7 3.3 8.7 3.3 10.8c0 .3 0 .5.1.8C1.5 12.3.1 14.1.1 16.3c0 2.7 2.2 4.9 4.9 4.9h14.1c2.6 0 4.8-2.1 4.8-4.8 0-2.4-1.8-4.4-4.2-4.7l-.3-2.1z"/></svg>`
	},
	NFLX: {
		symbol: 'NFLX',
		name: 'Netflix Inc.',
		sector: 'Communication Services',
		marketCapBillion: 310,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#E50914"><path d="M5 2h3.5l5.5 14.5V2H19v20h-3.5L10 7.5V22H5V2z"/></svg>`
	},
	KO: {
		symbol: 'KO',
		name: 'The Coca-Cola Company',
		sector: 'Consumer Defensive',
		marketCapBillion: 290,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#F40009"/><text x="12" y="16" fill="#FFFFFF" font-family="serif" font-style="italic" font-weight="900" font-size="10" text-anchor="middle">Coke</text></svg>`
	},
	CVX: {
		symbol: 'CVX',
		name: 'Chevron Corporation',
		sector: 'Energy',
		marketCapBillion: 280,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><path d="M4 6l8 6 8-6v4l-8 6-8-6V6z" fill="#0054A6"/><path d="M4 12l8 6 8-6v4l-8 6-8-6v-4z" fill="#ED1C24"/></svg>`
	},
	AMD: {
		symbol: 'AMD',
		name: 'Advanced Micro Devices',
		sector: 'Technology',
		marketCapBillion: 240,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#009A66"><path d="M2 2h20v20H2V2zm16 4h-5v5h5V6zm-6 0H7v5h5V6zm0 6H7v5h5v-5zm6 0h-5v5h5v-5z"/></svg>`
	},
	PEP: {
		symbol: 'PEP',
		name: 'PepsiCo Inc.',
		sector: 'Consumer Defensive',
		marketCapBillion: 240,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#004B93"/><path d="M1 12a11 11 0 0122 0c-4-4-10 4-22 0z" fill="#FFFFFF"/><path d="M1 12a11 11 0 0022 0c-4 4-10-4-22 0z" fill="#E32934"/></svg>`
	},
	ADBE: {
		symbol: 'ADBE',
		name: 'Adobe Inc.',
		sector: 'Technology',
		marketCapBillion: 230,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#FF0000"><path d="M13.9 2h8.1v20l-8.1-20zM2 2h8.1l-8.1 20V2zm10 8.3L16.2 22h-3.3l-1.7-4.4h-2.9L12 10.3z"/></svg>`
	},
	ORCL: {
		symbol: 'ORCL',
		name: 'Oracle Corporation',
		sector: 'Technology',
		marketCapBillion: 380,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#C74634"/><ellipse cx="12" cy="12" rx="7" ry="4" stroke="#FFFFFF" stroke-width="2" fill="none"/></svg>`
	},
	MRK: {
		symbol: 'MRK',
		name: 'Merck & Co. Inc.',
		sector: 'Healthcare',
		marketCapBillion: 220,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#00857C"/><circle cx="9" cy="9" r="3" fill="#FFFFFF"/><circle cx="15" cy="9" r="3" fill="#FFFFFF"/><circle cx="12" cy="15" r="3" fill="#FFFFFF"/></svg>`
	},
	MCD: {
		symbol: 'MCD',
		name: "McDonald's Corp.",
		sector: 'Consumer Cyclical',
		marketCapBillion: 210,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#BD0000"/><path d="M6 18c0-5 2-9 4-9s3 3 3 6c0-3 1-6 3-6s4 4 4 9h-2c0-4-1-7-2-7s-2 3-2 6v1h-2v-1c0-3-1-6-2-6s-2 3-2 7H6z" fill="#FFC72C"/></svg>`
	},
	CSCO: {
		symbol: 'CSCO',
		name: 'Cisco Systems Inc.',
		sector: 'Technology',
		marketCapBillion: 200,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#1BA0D7"><path d="M4 14v4h2v-4H4zm4-3v7h2v-7H8zm4-3v10h2V8h-2zm4 3v7h2v-7h-2zm4 3v4h2v-4h-2z"/></svg>`
	},
	NOW: {
		symbol: 'NOW',
		name: 'ServiceNow Inc.',
		sector: 'Technology',
		marketCapBillion: 200,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#293E40"/><circle cx="12" cy="12" r="4.5" fill="#81B5A1"/></svg>`
	},
	QCOM: {
		symbol: 'QCOM',
		name: 'QUALCOMM Inc.',
		sector: 'Technology',
		marketCapBillion: 190,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#003B64"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="8" text-anchor="middle">QCOM</text></svg>`
	},
	IBM: {
		symbol: 'IBM',
		name: 'IBM Corporation',
		sector: 'Technology',
		marketCapBillion: 200,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#052FAD"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="9" text-anchor="middle">IBM</text></svg>`
	},
	DIS: {
		symbol: 'DIS',
		name: 'The Walt Disney Co.',
		sector: 'Communication Services',
		marketCapBillion: 180,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#113CCF"/><text x="12" y="16" fill="#FFFFFF" font-family="serif" font-weight="900" font-size="9" text-anchor="middle">DIS</text></svg>`
	},
	TXN: {
		symbol: 'TXN',
		name: 'Texas Instruments',
		sector: 'Technology',
		marketCapBillion: 180,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><rect x="1" y="1" width="22" height="22" rx="3" fill="#CC0000"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="9" text-anchor="middle">TI</text></svg>`
	},
	AMGN: {
		symbol: 'AMGN',
		name: 'Amgen Inc.',
		sector: 'Healthcare',
		marketCapBillion: 160,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#00629B"/><circle cx="12" cy="12" r="4" fill="#FFFFFF"/></svg>`
	},
	PLTR: {
		symbol: 'PLTR',
		name: 'Palantir Technologies',
		sector: 'Technology',
		marketCapBillion: 180,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#FFFFFF"><circle cx="12" cy="12" r="10" stroke="#FFFFFF" stroke-width="2" fill="none"/><circle cx="12" cy="12" r="5" fill="#FFFFFF"/></svg>`
	},
	INTC: {
		symbol: 'INTC',
		name: 'Intel Corporation',
		sector: 'Technology',
		marketCapBillion: 100,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#0071C5"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="7.5" text-anchor="middle">intel</text></svg>`
	},
	WFC: {
		symbol: 'WFC',
		name: 'Wells Fargo & Co.',
		sector: 'Financial Services',
		marketCapBillion: 200,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><rect x="1" y="1" width="22" height="22" rx="3" fill="#CD1409"/><text x="12" y="15" fill="#FFCD00" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">WELLS</text></svg>`
	},
	GS: {
		symbol: 'GS',
		name: 'Goldman Sachs Group',
		sector: 'Financial Services',
		marketCapBillion: 170,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><rect x="1" y="1" width="22" height="22" rx="3" fill="#7399C6"/><text x="12" y="16" fill="#FFFFFF" font-family="serif" font-weight="900" font-size="9" text-anchor="middle">GS</text></svg>`
	},
	MS: {
		symbol: 'MS',
		name: 'Morgan Stanley',
		sector: 'Financial Services',
		marketCapBillion: 170,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#002D62"/><text x="12" y="16" fill="#FFFFFF" font-family="serif" font-weight="900" font-size="9" text-anchor="middle">MS</text></svg>`
	},
	C: {
		symbol: 'C',
		name: 'Citigroup Inc.',
		sector: 'Financial Services',
		marketCapBillion: 130,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#003B70"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="10" text-anchor="middle">citi</text></svg>`
	},
	BLK: {
		symbol: 'BLK',
		name: 'BlackRock Inc.',
		sector: 'Financial Services',
		marketCapBillion: 150,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#111111"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">BLK</text></svg>`
	},
	AXP: {
		symbol: 'AXP',
		name: 'American Express Co.',
		sector: 'Financial Services',
		marketCapBillion: 180,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><rect x="1" y="1" width="22" height="22" rx="3" fill="#006FCF"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">AMEX</text></svg>`
	},
	PFE: {
		symbol: 'PFE',
		name: 'Pfizer Inc.',
		sector: 'Healthcare',
		marketCapBillion: 160,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#0000FF"/><path d="M8 12c0-2.2 1.8-4 4-4s4 1.8 4 4-1.8 4-4 4-4-1.8-4-4z" fill="#FFFFFF"/></svg>`
	},
	NKE: {
		symbol: 'NKE',
		name: 'NIKE Inc.',
		sector: 'Consumer Cyclical',
		marketCapBillion: 120,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#FFFFFF"><path d="M21.7 8.3L9.5 16.5c-2.4 1.6-5.2 1.4-6.4-.4-1.1-1.7-.5-4.3 1.5-6.1l4-3.5c-.3.7-.4 1.5-.2 2.3.4 1.6 2 2.7 3.6 2.5l9.7-3z"/></svg>`
	},
	MU: {
		symbol: 'MU',
		name: 'Micron Technology',
		sector: 'Technology',
		marketCapBillion: 120,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#005596"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="9" text-anchor="middle">MU</text></svg>`
	},
	PANW: {
		symbol: 'PANW',
		name: 'Palo Alto Networks',
		sector: 'Technology',
		marketCapBillion: 120,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full" fill="#FA582D"><rect x="2" y="2" width="9" height="9" rx="2"/><rect x="13" y="2" width="9" height="9" rx="2"/><rect x="2" y="13" width="9" height="9" rx="2"/><rect x="13" y="13" width="9" height="9" rx="2"/></svg>`
	},
	AMAT: {
		symbol: 'AMAT',
		name: 'Applied Materials',
		sector: 'Technology',
		marketCapBillion: 180,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#004F9F"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">AMAT</text></svg>`
	},
	LRCX: {
		symbol: 'LRCX',
		name: 'Lam Research Corp.',
		sector: 'Technology',
		marketCapBillion: 110,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#003865"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">LAM</text></svg>`
	},
	OXY: {
		symbol: 'OXY',
		name: 'Occidental Petroleum',
		sector: 'Energy',
		marketCapBillion: 60,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#003366"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="8" text-anchor="middle">OXY</text></svg>`
	},
	SLB: {
		symbol: 'SLB',
		name: 'Schlumberger N.V.',
		sector: 'Energy',
		marketCapBillion: 60,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#001489"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="8" text-anchor="middle">SLB</text></svg>`
	},
	COP: {
		symbol: 'COP',
		name: 'ConocoPhillips',
		sector: 'Energy',
		marketCapBillion: 130,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#C8102E"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="8" text-anchor="middle">COP</text></svg>`
	},
	ASML: {
		symbol: 'ASML',
		name: 'ASML Holding N.V.',
		sector: 'Technology',
		marketCapBillion: 350,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#002D62"/><text x="12" y="16" fill="#00A3E0" font-family="sans-serif" font-weight="900" font-size="7.5" text-anchor="middle">ASML</text></svg>`
	},
	TSM: {
		symbol: 'TSM',
		name: 'Taiwan Semiconductor',
		sector: 'Technology',
		marketCapBillion: 800,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#000000"/><circle cx="12" cy="12" r="9" stroke="#E31B23" stroke-width="2" fill="none"/><text x="12" y="15" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="6.5" text-anchor="middle">TSMC</text></svg>`
	},
	NVO: {
		symbol: 'NVO',
		name: 'Novo Nordisk A/S',
		sector: 'Healthcare',
		marketCapBillion: 500,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#00205B"/><text x="12" y="16" fill="#FFFFFF" font-family="sans-serif" font-weight="900" font-size="7" text-anchor="middle">NOVO</text></svg>`
	},
	AZN: {
		symbol: 'AZN',
		name: 'AstraZeneca PLC',
		sector: 'Healthcare',
		marketCapBillion: 250,
		svgLogo: `<svg viewBox="0 0 24 24" class="w-full h-full"><circle cx="12" cy="12" r="11" fill="#3C1053"/><text x="12" y="16" fill="#E8B923" font-family="sans-serif" font-weight="900" font-size="8" text-anchor="middle">AZ</text></svg>`
	}
};

export const IDX_EQUITIES: Record<string, CompanyInfo> = {
	BBCA: { symbol: 'BBCA', name: 'Bank Central Asia', sector: 'Financials', marketCapBillion: 70 },
	BBRI: { symbol: 'BBRI', name: 'Bank Rakyat Indonesia', sector: 'Financials', marketCapBillion: 40 },
	BMRI: { symbol: 'BMRI', name: 'Bank Mandiri', sector: 'Financials', marketCapBillion: 38 },
	BBNI: { symbol: 'BBNI', name: 'Bank Negara Indonesia', sector: 'Financials', marketCapBillion: 12 },
	TLKM: { symbol: 'TLKM', name: 'Telkom Indonesia', sector: 'Communication Services', marketCapBillion: 18 },
	ASII: { symbol: 'ASII', name: 'Astra International', sector: 'Industrials', marketCapBillion: 12 },
	UNVR: { symbol: 'UNVR', name: 'Unilever Indonesia', sector: 'Consumer Defensive', marketCapBillion: 6 },
	ICBP: { symbol: 'ICBP', name: 'Indofood CBP', sector: 'Consumer Defensive', marketCapBillion: 8 },
	ADRO: { symbol: 'ADRO', name: 'Adaro Energy', sector: 'Energy', marketCapBillion: 5 },
	MDKA: { symbol: 'MDKA', name: 'Merdeka Copper Gold', sector: 'Basic Materials', marketCapBillion: 4 },
	ANTM: { symbol: 'ANTM', name: 'Aneka Tambang', sector: 'Basic Materials', marketCapBillion: 3 },
	GOTO: { symbol: 'GOTO', name: 'GoTo Gojek Tokopedia', sector: 'Technology', marketCapBillion: 4 },
	PTBA: { symbol: 'PTBA', name: 'Bukit Asam', sector: 'Energy', marketCapBillion: 2 },
	INDF: { symbol: 'INDF', name: 'Indofood Sukses Makmur', sector: 'Consumer Defensive', marketCapBillion: 4 }
};

export const OTHER_MARKETS: Record<string, CompanyInfo> = {
	BTCUSDT: { symbol: 'BTCUSDT', displaySymbol: 'BTC', name: 'Bitcoin', sector: 'Cryptocurrency', marketCapBillion: 1350 },
	ETHUSDT: { symbol: 'ETHUSDT', displaySymbol: 'ETH', name: 'Ethereum', sector: 'Cryptocurrency', marketCapBillion: 360 },
	SOLUSDT: { symbol: 'SOLUSDT', displaySymbol: 'SOL', name: 'Solana', sector: 'Cryptocurrency', marketCapBillion: 75 },
	BNBUSDT: { symbol: 'BNBUSDT', displaySymbol: 'BNB', name: 'BNB', sector: 'Cryptocurrency', marketCapBillion: 85 },
	PAXGUSDT: { symbol: 'PAXGUSDT', displaySymbol: 'PAXG', name: 'PAX Gold', sector: 'Cryptocurrency', marketCapBillion: 1 },
	XAUUSD: { symbol: 'XAUUSD', displaySymbol: 'GOLD', name: 'Gold Spot', sector: 'Precious Metals', marketCapBillion: 2500 },
	XAGUSD: { symbol: 'XAGUSD', displaySymbol: 'SILVER', name: 'Silver Spot', sector: 'Precious Metals', marketCapBillion: 500 },
	WTI: { symbol: 'WTI', displaySymbol: 'WTI', name: 'Crude Oil WTI', sector: 'Energy Commodities', marketCapBillion: 800 },
	USOIL: { symbol: 'USOIL', displaySymbol: 'USOIL', name: 'Crude Oil WTI', sector: 'Energy Commodities', marketCapBillion: 800 },
	BRENT: { symbol: 'BRENT', displaySymbol: 'BRENT', name: 'Brent Crude Oil', sector: 'Energy Commodities', marketCapBillion: 900 },
	UKOIL: { symbol: 'UKOIL', displaySymbol: 'UKOIL', name: 'Brent Crude Oil', sector: 'Energy Commodities', marketCapBillion: 900 },
	NATGAS: { symbol: 'NATGAS', displaySymbol: 'NATGAS', name: 'Natural Gas', sector: 'Energy Commodities', marketCapBillion: 200 },
	EURUSD: { symbol: 'EURUSD', displaySymbol: 'EUR/USD', name: 'Euro / US Dollar', sector: 'Majors', marketCapBillion: 1500 },
	GBPUSD: { symbol: 'GBPUSD', displaySymbol: 'GBP/USD', name: 'British Pound / US Dollar', sector: 'Majors', marketCapBillion: 800 },
	USDJPY: { symbol: 'USDJPY', displaySymbol: 'USD/JPY', name: 'US Dollar / Yen', sector: 'Majors', marketCapBillion: 1200 },
	AUDUSD: { symbol: 'AUDUSD', displaySymbol: 'AUD/USD', name: 'Australian Dollar / USD', sector: 'Majors', marketCapBillion: 500 },
	USDCAD: { symbol: 'USDCAD', displaySymbol: 'USD/CAD', name: 'US Dollar / Canadian Dollar', sector: 'Majors', marketCapBillion: 400 },
	USDIDR: { symbol: 'USDIDR', displaySymbol: 'USD/IDR', name: 'US Dollar / Indonesian Rupiah', sector: 'Emerging FX', marketCapBillion: 200 },
	GBPJPY: { symbol: 'GBPJPY', displaySymbol: 'GBP/JPY', name: 'British Pound / Yen', sector: 'Crosses', marketCapBillion: 400 },
	EURJPY: { symbol: 'EURJPY', displaySymbol: 'EUR/JPY', name: 'Euro / Yen', sector: 'Crosses', marketCapBillion: 500 },
	AUDJPY: { symbol: 'AUDJPY', displaySymbol: 'AUD/JPY', name: 'Aussie / Yen', sector: 'Crosses', marketCapBillion: 300 },
	EURGBP: { symbol: 'EURGBP', displaySymbol: 'EUR/GBP', name: 'Euro / Pound', sector: 'Crosses', marketCapBillion: 400 },
	NZDUSD: { symbol: 'NZDUSD', displaySymbol: 'NZD/USD', name: 'New Zealand Dollar / USD', sector: 'Majors', marketCapBillion: 250 },
	SPX: { symbol: 'SPX', displaySymbol: 'S&P 500', name: 'S&P 500 Index', sector: 'Global Indices', marketCapBillion: 4500 },
	DXY: { symbol: 'DXY', displaySymbol: 'DXY', name: 'US Dollar Index', sector: 'Currency Indices', marketCapBillion: 2000 },
	IHSG: { symbol: 'IHSG', displaySymbol: 'IHSG', name: 'Jakarta Composite Index', sector: 'Asia Indices', marketCapBillion: 800 },
	JCI: { symbol: 'JCI', displaySymbol: 'JCI', name: 'Jakarta Composite Index', sector: 'Asia Indices', marketCapBillion: 800 },
	DJI: { symbol: 'DJI', displaySymbol: 'DOW 30', name: 'Dow Jones Industrial', sector: 'US Indices', marketCapBillion: 2000 },
	NDX: { symbol: 'NDX', displaySymbol: 'NASDAQ 100', name: 'NASDAQ 100', sector: 'US Indices', marketCapBillion: 3000 },
	RUT: { symbol: 'RUT', displaySymbol: 'RUSSELL 2000', name: 'Russell 2000', sector: 'US Indices', marketCapBillion: 1000 },
	FTSE: { symbol: 'FTSE', displaySymbol: 'FTSE 100', name: 'FTSE 100 Index', sector: 'Europe Indices', marketCapBillion: 1200 },
	GDAXI: { symbol: 'GDAXI', displaySymbol: 'DAX 40', name: 'DAX Performance Index', sector: 'Europe Indices', marketCapBillion: 1200 },
	FCHI: { symbol: 'FCHI', displaySymbol: 'CAC 40', name: 'CAC 40 Index', sector: 'Europe Indices', marketCapBillion: 1000 },
	N225: { symbol: 'N225', displaySymbol: 'NIKKEI 225', name: 'Nikkei 225', sector: 'Asia Indices', marketCapBillion: 2500 },
	HSI: { symbol: 'HSI', displaySymbol: 'HANG SENG', name: 'Hang Seng Index', sector: 'Asia Indices', marketCapBillion: 1800 },
	KOSPI: { symbol: 'KOSPI', displaySymbol: 'KOSPI', name: 'KOSPI Composite Index', sector: 'Asia Indices', marketCapBillion: 1200 },
	ASX200: { symbol: 'ASX200', displaySymbol: 'ASX 200', name: 'S&P/ASX 200', sector: 'Asia Indices', marketCapBillion: 1400 },
	NIFTY50: { symbol: 'NIFTY50', displaySymbol: 'NIFTY 50', name: 'Nifty 50 Index', sector: 'Asia Indices', marketCapBillion: 1800 },
	SENSEX: { symbol: 'SENSEX', displaySymbol: 'BSE SENSEX', name: 'BSE SENSEX Index', sector: 'Asia Indices', marketCapBillion: 1600 },
	SSEC: { symbol: 'SSEC', displaySymbol: 'SHANGHAI', name: 'Shanghai Composite', sector: 'Asia Indices', marketCapBillion: 2500 },
	STI: { symbol: 'STI', displaySymbol: 'STI', name: 'Straits Times Index', sector: 'Asia Indices', marketCapBillion: 500 },
	VIX: { symbol: 'VIX', displaySymbol: 'VIX', name: 'CBOE Volatility Index', sector: 'Volatility', marketCapBillion: 100 }
};

export function getCompanyInfo(sym: string): CompanyInfo | null {
	const upper = sym.toUpperCase();
	if (US_EQUITIES[upper]) return US_EQUITIES[upper];
	if (IDX_EQUITIES[upper]) return IDX_EQUITIES[upper];
	if (OTHER_MARKETS[upper]) return OTHER_MARKETS[upper];
	return null;
}
