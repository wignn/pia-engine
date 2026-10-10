import { WatchlistItem } from "@/types";
import { KNOWN_INSTRUMENTS } from "./instruments";

export const INITIAL_WATCHLIST: WatchlistItem[] = Object.entries(KNOWN_INSTRUMENTS).map(([symbol, meta]) => {
  return {
    symbol,
    name: meta.name,
    price: 0,
    change: 0,
    changePercent: 0,
    category: meta.category,
    provider: meta.provider,
    digits: meta.digits,
  };
});
