import { WatchlistItem } from "@/types";

export interface KnownInstrumentMeta {
  name: string;
  category: "commodities" | "indices" | "forex" | "crypto" | "stocks";
  provider: string;
  digits: number;
}

export const KNOWN_INSTRUMENTS: Record<string, KnownInstrumentMeta> = {
  // --- Commodities ---
  XAUUSD: { name: "Gold Spot / U.S. Dollar", category: "commodities", provider: "OANDA", digits: 3 },
  XAGUSD: { name: "Silver Spot / U.S. Dollar", category: "commodities", provider: "TVC", digits: 3 },
  USOIL: { name: "Crude Oil WTI Cash", category: "commodities", provider: "NYMEX", digits: 2 },
  UKOIL: { name: "Brent Crude Oil Cash", category: "commodities", provider: "ICEEUR", digits: 2 },
  WTI: { name: "WTI Light Sweet Crude", category: "commodities", provider: "NYMEX", digits: 2 },
  BRENT: { name: "Brent Crude Oil", category: "commodities", provider: "ICEEUR", digits: 2 },
  NATGAS: { name: "Natural Gas Cash", category: "commodities", provider: "NYMEX", digits: 3 },

  // --- Crypto ---
  BTCUSDT: { name: "Bitcoin / TetherUS", category: "crypto", provider: "BINANCE", digits: 2 },
  ETHUSDT: { name: "Ethereum / TetherUS", category: "crypto", provider: "BINANCE", digits: 2 },
  SOLUSDT: { name: "Solana / TetherUS", category: "crypto", provider: "BINANCE", digits: 2 },
  BNBUSDT: { name: "BNB / TetherUS", category: "crypto", provider: "BINANCE", digits: 2 },
  XRPUSDT: { name: "XRP / TetherUS", category: "crypto", provider: "BINANCE", digits: 4 },
  ADAUSDT: { name: "Cardano / TetherUS", category: "crypto", provider: "BINANCE", digits: 4 },
  DOGEUSDT: { name: "Dogecoin / TetherUS", category: "crypto", provider: "BINANCE", digits: 5 },

  // --- Forex ---
  EURUSD: { name: "Euro / U.S. Dollar", category: "forex", provider: "FX", digits: 5 },
  GBPUSD: { name: "British Pound / U.S. Dollar", category: "forex", provider: "FX", digits: 5 },
  USDJPY: { name: "U.S. Dollar / Japanese Yen", category: "forex", provider: "FX", digits: 3 },
  EURJPY: { name: "Euro / Japanese Yen", category: "forex", provider: "FX", digits: 3 },
  GBPJPY: { name: "British Pound / Japanese Yen", category: "forex", provider: "FX", digits: 3 },
  AUDJPY: { name: "Australian Dollar / Japanese Yen", category: "forex", provider: "FX", digits: 3 },
  NZDUSD: { name: "New Zealand Dollar / U.S. Dollar", category: "forex", provider: "FX", digits: 5 },
  EURGBP: { name: "Euro / British Pound", category: "forex", provider: "FX", digits: 5 },
  AUDUSD: { name: "Australian Dollar / U.S. Dollar", category: "forex", provider: "FX", digits: 5 },
  USDCAD: { name: "U.S. Dollar / Canadian Dollar", category: "forex", provider: "FX", digits: 5 },
  USDCHF: { name: "U.S. Dollar / Swiss Franc", category: "forex", provider: "FX", digits: 5 },

  // --- Indices ---
  SPX: { name: "S&P 500 Index", category: "indices", provider: "SP", digits: 2 },
  NDX: { name: "Nasdaq 100 Index", category: "indices", provider: "NASDAQ", digits: 2 },
  DJI: { name: "Dow Jones Industrial Average", category: "indices", provider: "DJ", digits: 2 },
  RUT: { name: "Russell 2000 Index", category: "indices", provider: "RUSSELL", digits: 2 },
  VIX: { name: "CBOE Volatility Index", category: "indices", provider: "CBOE", digits: 2 },
  DXY: { name: "U.S. Dollar Currency Index", category: "indices", provider: "TVC", digits: 3 },
  IHSG: { name: "Jakarta Composite Index", category: "indices", provider: "IDX", digits: 2 },
  JCI: { name: "Jakarta Composite Index (JCI)", category: "indices", provider: "IDX", digits: 2 },
  FTSE: { name: "FTSE 100 Index", category: "indices", provider: "FTSE", digits: 2 },
  GDAXI: { name: "DAX 40 Performance Index", category: "indices", provider: "XETRA", digits: 2 },
  FCHI: { name: "CAC 40 Index Paris", category: "indices", provider: "EURONEXT", digits: 2 },
  N225: { name: "Nikkei 225 Tokyo", category: "indices", provider: "OSE", digits: 2 },
  HSI: { name: "Hang Seng Index Hong Kong", category: "indices", provider: "HKEX", digits: 2 },
  SSEC: { name: "Shanghai Composite Index", category: "indices", provider: "SSE", digits: 2 },
  STI: { name: "Straits Times Index Singapore", category: "indices", provider: "SGX", digits: 2 },
  KOSPI: { name: "Korea Composite Stock Price Index", category: "indices", provider: "KRX", digits: 2 },
  ASX200: { name: "S&P/ASX 200 Australia", category: "indices", provider: "ASX", digits: 2 },
  NIFTY50: { name: "Nifty 50 National Stock Exchange India", category: "indices", provider: "NSE", digits: 2 },
  SENSEX: { name: "BSE SENSEX India", category: "indices", provider: "BSE", digits: 2 },

  // --- Stocks: US Mega Caps & Tech ---
  AAPL: { name: "Apple Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  MSFT: { name: "Microsoft Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  NVDA: { name: "NVIDIA Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  GOOGL: { name: "Alphabet Inc. (Google)", category: "stocks", provider: "NASDAQ", digits: 2 },
  AMZN: { name: "Amazon.com Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  META: { name: "Meta Platforms Inc. (Facebook)", category: "stocks", provider: "NASDAQ", digits: 2 },
  TSLA: { name: "Tesla Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  AVGO: { name: "Broadcom Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  ORCL: { name: "Oracle Corporation", category: "stocks", provider: "NYSE", digits: 2 },
  ADBE: { name: "Adobe Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  CRM: { name: "Salesforce Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  AMD: { name: "Advanced Micro Devices", category: "stocks", provider: "NASDAQ", digits: 2 },
  INTC: { name: "Intel Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  QCOM: { name: "Qualcomm Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  TSM: { name: "Taiwan Semiconductor Manufacturing", category: "stocks", provider: "NYSE", digits: 2 },
  ASML: { name: "ASML Holding N.V.", category: "stocks", provider: "NASDAQ", digits: 2 },
  NOW: { name: "ServiceNow Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  PANW: { name: "Palo Alto Networks", category: "stocks", provider: "NASDAQ", digits: 2 },
  LRCX: { name: "Lam Research Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  MU: { name: "Micron Technology Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  AMAT: { name: "Applied Materials Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  IBM: { name: "International Business Machines", category: "stocks", provider: "NYSE", digits: 2 },
  NFLX: { name: "Netflix Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },

  // --- Stocks: US Financials ---
  JPM: { name: "JPMorgan Chase & Co.", category: "stocks", provider: "NYSE", digits: 2 },
  V: { name: "Visa Inc. Class A", category: "stocks", provider: "NYSE", digits: 2 },
  MA: { name: "Mastercard Incorporated", category: "stocks", provider: "NYSE", digits: 2 },
  BAC: { name: "Bank of America Corporation", category: "stocks", provider: "NYSE", digits: 2 },
  WFC: { name: "Wells Fargo & Company", category: "stocks", provider: "NYSE", digits: 2 },
  MS: { name: "Morgan Stanley", category: "stocks", provider: "NYSE", digits: 2 },
  GS: { name: "The Goldman Sachs Group", category: "stocks", provider: "NYSE", digits: 2 },
  BLK: { name: "BlackRock Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  AXP: { name: "American Express Company", category: "stocks", provider: "NYSE", digits: 2 },
  C: { name: "Citigroup Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  BRKB: { name: "Berkshire Hathaway Inc. Class B", category: "stocks", provider: "NYSE", digits: 2 },

  // --- Stocks: US Healthcare ---
  LLY: { name: "Eli Lilly and Company", category: "stocks", provider: "NYSE", digits: 2 },
  UNH: { name: "UnitedHealth Group Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  JNJ: { name: "Johnson & Johnson", category: "stocks", provider: "NYSE", digits: 2 },
  ABBV: { name: "AbbVie Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  MRK: { name: "Merck & Co. Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  NVO: { name: "Novo Nordisk A/S", category: "stocks", provider: "NYSE", digits: 2 },
  AZN: { name: "AstraZeneca PLC", category: "stocks", provider: "NASDAQ", digits: 2 },
  PFE: { name: "Pfizer Inc.", category: "stocks", provider: "NYSE", digits: 2 },

  // --- Stocks: US Consumer & Energy ---
  WMT: { name: "Walmart Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  COST: { name: "Costco Wholesale Corporation", category: "stocks", provider: "NASDAQ", digits: 2 },
  HD: { name: "The Home Depot Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  MCD: { name: "McDonald's Corporation", category: "stocks", provider: "NYSE", digits: 2 },
  PEP: { name: "PepsiCo Inc.", category: "stocks", provider: "NASDAQ", digits: 2 },
  KO: { name: "The Coca-Cola Company", category: "stocks", provider: "NYSE", digits: 2 },
  DIS: { name: "The Walt Disney Company", category: "stocks", provider: "NYSE", digits: 2 },
  NKE: { name: "NIKE Inc.", category: "stocks", provider: "NYSE", digits: 2 },
  XOM: { name: "Exxon Mobil Corporation", category: "stocks", provider: "NYSE", digits: 2 },
  CVX: { name: "Chevron Corporation", category: "stocks", provider: "NYSE", digits: 2 },
  COP: { name: "ConocoPhillips", category: "stocks", provider: "NYSE", digits: 2 },
  SLB: { name: "Schlumberger Limited", category: "stocks", provider: "NYSE", digits: 2 },
  OXY: { name: "Occidental Petroleum Corporation", category: "stocks", provider: "NYSE", digits: 2 },

  // --- Stocks: Indonesian Equities (IDX) ---
  BBCA: { name: "Bank Central Asia Tbk", category: "stocks", provider: "IDX", digits: 2 },
  BBRI: { name: "Bank Rakyat Indonesia Tbk", category: "stocks", provider: "IDX", digits: 2 },
  BMRI: { name: "Bank Mandiri (Persero) Tbk", category: "stocks", provider: "IDX", digits: 2 },
  BBNI: { name: "Bank Negara Indonesia Tbk", category: "stocks", provider: "IDX", digits: 2 },
  TLKM: { name: "Telkom Indonesia (Persero) Tbk", category: "stocks", provider: "IDX", digits: 2 },
  ASII: { name: "Astra International Tbk", category: "stocks", provider: "IDX", digits: 2 },
  UNVR: { name: "Unilever Indonesia Tbk", category: "stocks", provider: "IDX", digits: 2 },
  ICBP: { name: "Indofood CBP Sukses Makmur Tbk", category: "stocks", provider: "IDX", digits: 2 },
  INDF: { name: "Indofood Sukses Makmur Tbk", category: "stocks", provider: "IDX", digits: 2 },
  ADRO: { name: "Adaro Energy Indonesia Tbk", category: "stocks", provider: "IDX", digits: 2 },
  ANTM: { name: "Aneka Tambang Tbk", category: "stocks", provider: "IDX", digits: 2 },
  PTBA: { name: "Bukit Asam Tbk", category: "stocks", provider: "IDX", digits: 2 },
  MDKA: { name: "Merdeka Copper Gold Tbk", category: "stocks", provider: "IDX", digits: 2 },
  GOTO: { name: "GoTo Gojek Tokopedia Tbk", category: "stocks", provider: "IDX", digits: 2 },
};

export function resolveInstrument(symbol: string, rawAssetType?: string | null): KnownInstrumentMeta {
  const sym = symbol.toUpperCase();
  if (KNOWN_INSTRUMENTS[sym]) {
    return KNOWN_INSTRUMENTS[sym];
  }

  // Fallback category detection
  let category: "commodities" | "indices" | "forex" | "crypto" | "stocks" = "stocks";
  const at = (rawAssetType || "").toLowerCase();

  if (at.includes("crypto") || sym.endsWith("USDT") || sym.endsWith("BTC")) {
    category = "crypto";
  } else if (at.includes("commodity") || ["WTI", "BRENT", "OIL", "GOLD", "SILVER"].some((s) => sym.includes(s))) {
    category = "commodities";
  } else if (at.includes("forex") || at.includes("fx") || /^[A-Z]{6}$/.test(sym)) {
    category = "forex";
  } else if (at.includes("index") || at.includes("indices")) {
    category = "indices";
  }

  let provider = "MARKET";
  if (category === "crypto") provider = "BINANCE";
  else if (category === "forex") provider = "FX";
  else if (category === "commodities") provider = "OANDA";
  else if (category === "indices") provider = "GLOBAL";

  let digits = 2;
  if (category === "forex") digits = sym.includes("JPY") ? 3 : 5;
  else if (sym === "XAUUSD" || sym === "XAGUSD" || sym === "DXY") digits = 3;

  return {
    name: sym,
    category,
    provider,
    digits,
  };
}
