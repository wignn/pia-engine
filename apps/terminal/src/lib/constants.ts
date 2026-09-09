import { WatchlistItem } from "@/types";
import { KNOWN_INSTRUMENTS } from "./instruments";

export const INITIAL_WATCHLIST: WatchlistItem[] = Object.entries(KNOWN_INSTRUMENTS).map(([symbol, meta]) => {
  let defaultPrice = 100.0;
  if (symbol === "BTCUSDT") defaultPrice = 78840.0;
  else if (symbol === "ETHUSDT") defaultPrice = 2499.66;
  else if (symbol === "XAUUSD") defaultPrice = 4376.19;
  else if (symbol === "XAGUSD") defaultPrice = 66.22;
  else if (symbol === "SPX") defaultPrice = 7703.31;
  else if (symbol === "AAPL") defaultPrice = 319.97;
  else if (symbol === "NVDA") defaultPrice = 230.36;
  else if (symbol === "MSFT") defaultPrice = 499.70;
  else if (symbol === "AMZN") defaultPrice = 258.51;
  else if (symbol === "GOOGL") defaultPrice = 338.46;
  else if (symbol === "TSLA") defaultPrice = 354.08;
  else if (symbol === "META") defaultPrice = 616.77;
  else if (symbol === "BBCA") defaultPrice = 6675.0;
  else if (symbol === "BBRI") defaultPrice = 3410.0;
  else if (symbol === "BMRI") defaultPrice = 4430.0;
  else if (symbol === "EURUSD") defaultPrice = 1.16213;
  else if (symbol === "USDJPY") defaultPrice = 153.841;

  return {
    symbol,
    name: meta.name,
    price: defaultPrice,
    change: 0,
    changePercent: 0,
    category: meta.category,
    provider: meta.provider,
    digits: meta.digits,
  };
});
