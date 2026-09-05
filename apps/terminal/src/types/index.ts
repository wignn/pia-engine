export interface WatchlistItem {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  category: "crypto" | "forex" | "indices" | "commodities" | "stocks";
  provider: string;
  digits: number;
}

export interface CandleData {
  time: number; // Unix timestamp in seconds
  open: number;
  high: number;
  low: number;
  close: number;
  volume?: number;
}

export type Timeframe = "1m" | "5m" | "15m" | "1h" | "4h" | "1D" | "1W";
export type ChartType = "candlestick" | "bar" | "line" | "area" | "heikin_ashi";
