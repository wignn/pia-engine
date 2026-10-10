import { WatchlistItem } from "@/types";

export interface KnownInstrumentMeta {
  name: string;
  category: "commodities" | "indices" | "forex" | "crypto" | "stocks";
  provider: string;
  digits: number;
}

export const KNOWN_INSTRUMENTS: Record<string, KnownInstrumentMeta> = {
  // --- Commodities / Metals / Energies (MT5 Real Feed) ---
  XAUUSD: { name: "Gold Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 3 },
  XAGUSD: { name: "Silver Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 3 },
  USOIL: { name: "Crude Oil WTI Cash Spot", category: "commodities", provider: "MT5", digits: 2 },
  UKOIL: { name: "Brent Crude Oil Cash Spot", category: "commodities", provider: "MT5", digits: 2 },
  XNGUSD: { name: "Natural Gas Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 3 },
  XCUUSD: { name: "Copper Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 3 },
  XPTUSD: { name: "Platinum Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 2 },
  XPDUSD: { name: "Palladium Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 2 },
  XALUSD: { name: "Aluminium Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 2 },
  XNIUSD: { name: "Nickel Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 2 },
  XPBUSD: { name: "Lead Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 2 },
  XZNUSD: { name: "Zinc Spot / U.S. Dollar", category: "commodities", provider: "MT5", digits: 2 },
  XAUEUR: { name: "Gold Spot / Euro", category: "commodities", provider: "MT5", digits: 3 },
  XAUGBP: { name: "Gold Spot / British Pound", category: "commodities", provider: "MT5", digits: 3 },
  XAUAUD: { name: "Gold Spot / Australian Dollar", category: "commodities", provider: "MT5", digits: 3 },
  XAGEUR: { name: "Silver Spot / Euro", category: "commodities", provider: "MT5", digits: 3 },
  XAGGBP: { name: "Silver Spot / British Pound", category: "commodities", provider: "MT5", digits: 3 },
  XAGAUD: { name: "Silver Spot / Australian Dollar", category: "commodities", provider: "MT5", digits: 3 },

  // --- Indices (MT5 Real & Global Feeds) ---
  US30: { name: "Dow Jones Industrial Average 30", category: "indices", provider: "MT5", digits: 2 },
  US500: { name: "S&P 500 Index Cash", category: "indices", provider: "MT5", digits: 2 },
  USTEC: { name: "Nasdaq 100 Index Cash", category: "indices", provider: "MT5", digits: 2 },
  DE30: { name: "Germany DAX 40 Index", category: "indices", provider: "MT5", digits: 2 },
  UK100: { name: "UK FTSE 100 Index", category: "indices", provider: "MT5", digits: 2 },
  JP225: { name: "Japan Nikkei 225 Index", category: "indices", provider: "MT5", digits: 2 },
  AUS200: { name: "Australia S&P/ASX 200", category: "indices", provider: "MT5", digits: 2 },
  HK50: { name: "Hong Kong Hang Seng 50", category: "indices", provider: "MT5", digits: 2 },
  FR40: { name: "France CAC 40 Index", category: "indices", provider: "MT5", digits: 2 },
  STOXX50: { name: "Euro Stoxx 50 Index", category: "indices", provider: "MT5", digits: 2 },
  DXY: { name: "U.S. Dollar Currency Index", category: "indices", provider: "TVC", digits: 3 },
  SPX: { name: "S&P 500 Composite", category: "indices", provider: "GLOBAL", digits: 2 },
  IHSG: { name: "Jakarta Composite Index", category: "indices", provider: "IDX", digits: 2 },
  HSI: { name: "Hang Seng Index Hong Kong", category: "indices", provider: "HKEX", digits: 2 },
  SSEC: { name: "Shanghai Composite Index", category: "indices", provider: "SSE", digits: 2 },
  KOSPI: { name: "Korea Composite Stock Price Index", category: "indices", provider: "KRX", digits: 2 },
  STI: { name: "Straits Times Index Singapore", category: "indices", provider: "SGX", digits: 2 },
  N225: { name: "Nikkei 225 Tokyo Stock Exchange", category: "indices", provider: "OSE", digits: 2 },
  SENSEX: { name: "BSE SENSEX India", category: "indices", provider: "BSE", digits: 2 },
  NIFTY50: { name: "Nifty 50 India", category: "indices", provider: "NSE", digits: 2 },
  ASX200: { name: "S&P/ASX 200 Australia", category: "indices", provider: "ASX", digits: 2 },

  // --- Crypto (MT5 Real Feed) ---
  BTCUSD: { name: "Bitcoin / U.S. Dollar", category: "crypto", provider: "MT5", digits: 2 },
  ETHUSD: { name: "Ethereum / U.S. Dollar", category: "crypto", provider: "MT5", digits: 2 },
  BTCUSDT: { name: "Bitcoin / TetherUS", category: "crypto", provider: "MT5", digits: 2 },
  ETHBTC: { name: "Ethereum / Bitcoin", category: "crypto", provider: "MT5", digits: 5 },
  BTCXAU: { name: "Bitcoin / Gold", category: "crypto", provider: "MT5", digits: 4 },
  BTCXAG: { name: "Bitcoin / Silver", category: "crypto", provider: "MT5", digits: 4 },
  BTCAUD: { name: "Bitcoin / Australian Dollar", category: "crypto", provider: "MT5", digits: 2 },
  BTCJPY: { name: "Bitcoin / Japanese Yen", category: "crypto", provider: "MT5", digits: 0 },
  BTCTHB: { name: "Bitcoin / Thai Baht", category: "crypto", provider: "MT5", digits: 2 },
  BTCZAR: { name: "Bitcoin / South African Rand", category: "crypto", provider: "MT5", digits: 2 },
  BTCCNH: { name: "Bitcoin / Chinese Yuan", category: "crypto", provider: "MT5", digits: 2 },

  // --- Forex: Majors & Top Crosses (MT5 Real Feed) ---
  EURUSD: { name: "Euro / U.S. Dollar", category: "forex", provider: "MT5", digits: 5 },
  GBPUSD: { name: "British Pound / U.S. Dollar", category: "forex", provider: "MT5", digits: 5 },
  USDJPY: { name: "U.S. Dollar / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },
  AUDUSD: { name: "Australian Dollar / U.S. Dollar", category: "forex", provider: "MT5", digits: 5 },
  USDCAD: { name: "U.S. Dollar / Canadian Dollar", category: "forex", provider: "MT5", digits: 5 },
  USDCHF: { name: "U.S. Dollar / Swiss Franc", category: "forex", provider: "MT5", digits: 5 },
  NZDUSD: { name: "New Zealand Dollar / U.S. Dollar", category: "forex", provider: "MT5", digits: 5 },

  EURGBP: { name: "Euro / British Pound", category: "forex", provider: "MT5", digits: 5 },
  EURJPY: { name: "Euro / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },
  GBPJPY: { name: "British Pound / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },
  AUDJPY: { name: "Australian Dollar / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },
  CADJPY: { name: "Canadian Dollar / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },
  CHFJPY: { name: "Swiss Franc / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },
  NZDJPY: { name: "New Zealand Dollar / Japanese Yen", category: "forex", provider: "MT5", digits: 3 },

  EURAUD: { name: "Euro / Australian Dollar", category: "forex", provider: "MT5", digits: 5 },
  EURCAD: { name: "Euro / Canadian Dollar", category: "forex", provider: "MT5", digits: 5 },
  EURCHF: { name: "Euro / Swiss Franc", category: "forex", provider: "MT5", digits: 5 },
  EURNZD: { name: "Euro / New Zealand Dollar", category: "forex", provider: "MT5", digits: 5 },
  GBPAUD: { name: "British Pound / Australian Dollar", category: "forex", provider: "MT5", digits: 5 },
  GBPCAD: { name: "British Pound / Canadian Dollar", category: "forex", provider: "MT5", digits: 5 },
  GBPCHF: { name: "British Pound / Swiss Franc", category: "forex", provider: "MT5", digits: 5 },
  GBPNZD: { name: "British Pound / New Zealand Dollar", category: "forex", provider: "MT5", digits: 5 },
  AUDNZD: { name: "Australian Dollar / New Zealand Dollar", category: "forex", provider: "MT5", digits: 5 },
  AUDCAD: { name: "Australian Dollar / Canadian Dollar", category: "forex", provider: "MT5", digits: 5 },
  AUDCHF: { name: "Australian Dollar / Swiss Franc", category: "forex", provider: "MT5", digits: 5 },
  CADCHF: { name: "Canadian Dollar / Swiss Franc", category: "forex", provider: "MT5", digits: 5 },
  NZDCAD: { name: "New Zealand Dollar / Canadian Dollar", category: "forex", provider: "MT5", digits: 5 },
  NZDCHF: { name: "New Zealand Dollar / Swiss Franc", category: "forex", provider: "MT5", digits: 5 },

  // --- Stocks: Active Mega Caps & Equities ---
  AAPL: { name: "Apple Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  MSFT: { name: "Microsoft Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  NVDA: { name: "NVIDIA Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  GOOGL: { name: "Alphabet Inc. (Google)", category: "stocks", provider: "NASDAQ", digits: 2 },
  AMZN: { name: "Amazon.com Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  META: { name: "Meta Platforms Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  TSLA: { name: "Tesla Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  AVGO: { name: "Broadcom Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  BRKB: { name: "Berkshire Hathaway Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  JPM: { name: "JPMorgan Chase & Co.", category: "stocks", provider: "NYSE", digits: 2 },
  V: { name: "Visa Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  LLY: { name: "Eli Lilly and Company", category: "stocks", provider: "NYSE", digits: 2 },
  WMT: { name: "Walmart Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  UNH: { name: "UnitedHealth Group Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  COST: { name: "Costco Wholesale Corp.", category: "stocks", provider: "NASDAQ", digits: 2 },

  // --- Stocks: Indonesian Equities (IDX) ---
  BBCA: { name: "Bank Central Asia Tbk", category: "stocks", provider: "IDX", digits: 0 },
  BBRI: { name: "Bank Rakyat Indonesia Tbk", category: "stocks", provider: "IDX", digits: 0 },
  BMRI: { name: "Bank Mandiri (Persero) Tbk", category: "stocks", provider: "IDX", digits: 0 },
};

const CURRENCY_NAMES: Record<string, string> = {
  USD: "U.S. Dollar",
  EUR: "Euro",
  GBP: "British Pound",
  JPY: "Japanese Yen",
  AUD: "Australian Dollar",
  CAD: "Canadian Dollar",
  CHF: "Swiss Franc",
  NZD: "New Zealand Dollar",
  SGD: "Singapore Dollar",
  HKD: "Hong Kong Dollar",
  CNH: "Offshore Chinese Yuan",
  THB: "Thai Baht",
  ZAR: "South African Rand",
  TRY: "Turkish Lira",
  MXN: "Mexican Peso",
  PLN: "Polish Zloty",
  SEK: "Swedish Krona",
  NOK: "Norwegian Krone",
  DKK: "Danish Krone",
  ILS: "Israeli Shekel",
};

export function resolveInstrument(symbol: string, rawAssetType?: string | null): KnownInstrumentMeta {
  const sym = symbol.toUpperCase().trim();
  // Strip trailing broker suffix M if base symbol is in known dictionary
  const baseSym = sym.endsWith("M") && sym.length >= 5 ? sym.slice(0, -1) : sym;

  if (KNOWN_INSTRUMENTS[baseSym]) {
    const meta = KNOWN_INSTRUMENTS[baseSym];
    return sym === baseSym ? meta : { ...meta, name: `${meta.name}` };
  }
  if (KNOWN_INSTRUMENTS[sym]) {
    return KNOWN_INSTRUMENTS[sym];
  }

  const at = (rawAssetType || "").toLowerCase();

  // 1. Crypto Detection
  if (
    sym.startsWith("BTC") ||
    sym.startsWith("ETH") ||
    at.includes("crypto") ||
    sym.endsWith("USDT") ||
    sym.endsWith("BTC")
  ) {
    let digits = 2;
    if (sym.includes("BTC") && !sym.startsWith("BTC")) digits = 5;
    if (sym.endsWith("JPY")) digits = 0;
    return {
      name: formatCryptoName(sym),
      category: "crypto",
      provider: "MT5",
      digits,
    };
  }

  // 2. Commodities / Metals / Energies Detection
  if (
    at.includes("commodity") ||
    at.includes("metal") ||
    ["XAU", "XAG", "XPT", "XPD", "XCU", "XNG", "XAL", "XNI", "XPB", "XZN"].some((p) =>
      sym.startsWith(p)
    ) ||
    ["USOIL", "UKOIL", "WTI", "BRENT", "NATGAS"].includes(sym)
  ) {
    const digits = sym.startsWith("XAU") || sym.startsWith("XAG") || sym.startsWith("XNG") ? 3 : 2;
    return {
      name: formatCommodityName(sym),
      category: "commodities",
      provider: "MT5",
      digits,
    };
  }

  // 3. Indices Detection
  if (
    at.includes("index") ||
    ["US30", "US500", "USTEC", "DE30", "UK100", "JP225", "AUS200", "HK50", "FR40", "STOXX50", "DXY", "SPX", "IHSG", "HSI", "SSEC", "KOSPI", "STI", "N225", "SENSEX", "NIFTY50", "ASX200"].some((idx) =>
      sym.startsWith(idx)
    )
  ) {
    return {
      name: formatIndexName(sym),
      category: "indices",
      provider: at.includes("index") ? "GLOBAL" : "MT5",
      digits: sym.includes("DXY") ? 3 : 2,
    };
  }

  // 4. Stocks Detection
  if (at.includes("stock") || ["BBCA", "BBRI", "BMRI"].includes(sym)) {
    const isIdx = ["BBCA", "BBRI", "BMRI"].includes(sym);
    return {
      name: sym,
      category: "stocks",
      provider: isIdx ? "IDX" : "NASDAQ",
      digits: isIdx ? 0 : 2,
    };
  }

  // 5. Forex Detection (Default for standard currency pairs)
  const digits = sym.includes("JPY") ? 3 : 5;
  return {
    name: formatForexName(sym),
    category: "forex",
    provider: "MT5",
    digits,
  };
}

function formatForexName(sym: string): string {
  const clean = sym.endsWith("M") && sym.length >= 7 ? sym.slice(0, -1) : sym;
  if (clean.length === 6) {
    const base = clean.slice(0, 3);
    const quote = clean.slice(3, 6);
    const baseName = CURRENCY_NAMES[base] || base;
    const quoteName = CURRENCY_NAMES[quote] || quote;
    return `${baseName} / ${quoteName}`;
  }
  return `${clean} (FX)`;
}

function formatCryptoName(sym: string): string {
  const clean = sym.endsWith("M") && sym.length >= 7 ? sym.slice(0, -1) : sym;
  if (clean.startsWith("BTC")) {
    const quote = clean.slice(3);
    return `Bitcoin / ${CURRENCY_NAMES[quote] || quote}`;
  }
  if (clean.startsWith("ETH")) {
    const quote = clean.slice(3);
    return `Ethereum / ${CURRENCY_NAMES[quote] || quote}`;
  }
  return clean;
}

function formatCommodityName(sym: string): string {
  if (sym.startsWith("XAU")) return `Gold Spot / ${CURRENCY_NAMES[sym.slice(3)] || sym.slice(3)}`;
  if (sym.startsWith("XAG")) return `Silver Spot / ${CURRENCY_NAMES[sym.slice(3)] || sym.slice(3)}`;
  if (sym.startsWith("XPT")) return "Platinum Spot / U.S. Dollar";
  if (sym.startsWith("XPD")) return "Palladium Spot / U.S. Dollar";
  if (sym.startsWith("XCU")) return "Copper Spot / U.S. Dollar";
  if (sym.startsWith("XNG")) return "Natural Gas Spot / U.S. Dollar";
  if (sym.startsWith("XAL")) return "Aluminium Spot / U.S. Dollar";
  if (sym.startsWith("XNI")) return "Nickel Spot / U.S. Dollar";
  if (sym.startsWith("XPB")) return "Lead Spot / U.S. Dollar";
  if (sym.startsWith("XZN")) return "Zinc Spot / U.S. Dollar";
  if (sym === "USOIL") return "Crude Oil WTI Cash Spot";
  if (sym === "UKOIL") return "Brent Crude Oil Cash Spot";
  return sym;
}

function formatIndexName(sym: string): string {
  if (sym.startsWith("US30")) return "Dow Jones Industrial Average 30";
  if (sym.startsWith("US500")) return "S&P 500 Index Cash";
  if (sym.startsWith("USTEC")) return "Nasdaq 100 Index Cash";
  if (sym.startsWith("DE30")) return "Germany DAX 40 Index";
  if (sym.startsWith("UK100")) return "UK FTSE 100 Index";
  if (sym.startsWith("JP225")) return "Japan Nikkei 225 Index";
  if (sym.startsWith("AUS200")) return "Australia S&P/ASX 200";
  if (sym.startsWith("HK50")) return "Hong Kong Hang Seng 50";
  if (sym.startsWith("FR40")) return "France CAC 40 Index";
  if (sym.startsWith("STOXX50")) return "Euro Stoxx 50 Index";
  return sym;
}
