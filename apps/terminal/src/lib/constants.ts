import { WatchlistItem } from "@/types";

export const INITIAL_WATCHLIST: WatchlistItem[] = [
  // Commodities
  { symbol: "XAUUSD", name: "Gold Spot / U.S. Dollar", price: 4429.825, change: -43.140, changePercent: -0.96, category: "commodities", provider: "OANDA", digits: 3 },
  { symbol: "XAGUSD", name: "Silver Spot / U.S. Dollar", price: 34.120, change: -0.450, changePercent: -1.30, category: "commodities", provider: "TVC", digits: 3 },
  { symbol: "USOIL", name: "Crude Oil WTI", price: 71.45, change: 0.85, changePercent: 1.20, category: "commodities", provider: "NYMEX", digits: 2 },
  { symbol: "UKOIL", name: "Brent Crude Oil", price: 74.80, change: 0.72, changePercent: 0.97, category: "commodities", provider: "ICEEUR", digits: 2 },

  // Indices
  { symbol: "SPX", name: "S&P 500 Index", price: 5985.40, change: 18.20, changePercent: 0.31, category: "indices", provider: "SP", digits: 2 },
  { symbol: "NDX", name: "Nasdaq 100", price: 21120.50, change: -45.10, changePercent: -0.21, category: "indices", provider: "NASDAQ", digits: 2 },
  { symbol: "DJI", name: "Dow Jones Industrial", price: 43910.20, change: 120.40, changePercent: 0.28, category: "indices", provider: "DJ", digits: 2 },
  { symbol: "DXY", name: "US Dollar Index", price: 104.250, change: 0.180, changePercent: 0.17, category: "indices", provider: "TVC", digits: 3 },
  { symbol: "IHSG", name: "Jakarta Composite Index", price: 7285.50, change: 42.10, changePercent: 0.58, category: "indices", provider: "IDX", digits: 2 },

  // Forex
  { symbol: "EURUSD", name: "Euro / U.S. Dollar", price: 1.05420, change: -0.00150, changePercent: -0.14, category: "forex", provider: "FX", digits: 5 },
  { symbol: "GBPUSD", name: "British Pound / U.S. Dollar", price: 1.26180, change: 0.00220, changePercent: 0.17, category: "forex", provider: "FX", digits: 5 },
  { symbol: "USDJPY", name: "U.S. Dollar / Japanese Yen", price: 154.650, change: 0.420, changePercent: 0.27, category: "forex", provider: "FX", digits: 3 },

  // Crypto
  { symbol: "BTCUSDT", name: "Bitcoin / Tether", price: 96450.00, change: 1250.00, changePercent: 1.31, category: "crypto", provider: "BINANCE", digits: 2 },
  { symbol: "ETHUSDT", name: "Ethereum / Tether", price: 2740.50, change: -35.20, changePercent: -1.27, category: "crypto", provider: "BINANCE", digits: 2 },

  // Stocks
  { symbol: "AAPL", name: "Apple Inc.", price: 232.50, change: 1.80, changePercent: 0.78, category: "stocks", provider: "NASDAQ", digits: 2 },
  { symbol: "NVDA", name: "NVIDIA Corporation", price: 138.40, change: -2.10, changePercent: -1.49, category: "stocks", provider: "NASDAQ", digits: 2 },
  { symbol: "BBCA", name: "Bank Central Asia Tbk", price: 9850.00, change: 150.00, changePercent: 1.55, category: "stocks", provider: "IDX", digits: 2 },
];
