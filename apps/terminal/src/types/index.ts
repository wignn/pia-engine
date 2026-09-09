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

export type DrawingTool = "cursor" | "trendline" | "horizontal" | "fibonacci" | "measure";

export interface DrawingPoint {
  x: number;
  y: number;
  time?: number;
  price?: number;
  logical?: number;
}

export interface DrawingItem {
  id: string;
  type: "trendline" | "horizontal" | "fibonacci" | "measure";
  symbol: string;
  p1: DrawingPoint;
  p2?: DrawingPoint;
  color?: string;
  strokeWidth?: number;
}

export interface IndicatorState {
  sma20: boolean;
  ema50: boolean;
  bollinger: boolean;
  rsi: boolean;
  macd: boolean;
}

export type ChartLayout = "1x1" | "1x2" | "2x1" | "2x2";

export interface ChartPaneConfig {
  id: string;
  symbol: string;
  timeframe: Timeframe;
  chartType: ChartType;
  indicators: IndicatorState;
}

export interface PriceAlert {
  id: string;
  symbol: string;
  targetPrice: number;
  condition: "crossing_up" | "crossing_down";
  createdPrice: number;
  createdAt: number;
  triggered: boolean;
  triggeredAt?: number;
}

export interface TabItem {
  id: string;
  symbol: string;
  timeframe: Timeframe;
  name: string;
}

export interface NewsArticle {
  id: string;
  title: string;
  source: string;
  url: string;
  published_at: string;
  impact_level?: "low" | "medium" | "high";
  sentiment?: "bullish" | "bearish" | "neutral";
  summary?: string;
}
