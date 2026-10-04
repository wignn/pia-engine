export interface XauusdForecast {
  status: "active" | "expired" | "unavailable";
  symbol: "XAUUSD";
  timeframe: "15m";
  decision_at?: string;
  horizon_end?: string;
  reference_price?: number;
  probabilities?: { down: number; flat: number; up: number };
  expected_return?: number;
  return_quantiles?: { q10: number; q50: number; q90: number };
  uncertainty?: number;
  model_version?: string;
  data_version?: string;
  feature_version?: string;
  source_masks?: Record<string, boolean>;
  source_age_seconds?: Record<string, number>;
  paper_only?: boolean;
}

export type XauusdForecastState = "loading" | "unavailable" | "error" | "active" | "expired";
