"use client";

import { useEffect, useState } from "react";
import type { XauusdForecast, XauusdForecastState } from "@/types/forecast";

export function useXauusdForecast(enabled: boolean, symbol: string, timeframe: string) {
  const [forecast, setForecast] = useState<XauusdForecast | null>(null);
  const [state, setState] = useState<XauusdForecastState>("unavailable");

  useEffect(() => {
    if (!enabled || symbol !== "XAUUSD" || timeframe !== "15m") {
      setForecast(null);
      setState("unavailable");
      return;
    }

    let disposed = false;
    const refresh = async () => {
      try {
        const response = await fetch("/api/market/forecasts/XAUUSD?timeframe=15m", { cache: "no-store" });
        if (!response.ok) throw new Error(`Forecast request failed (${response.status})`);
        const result = (await response.json()) as XauusdForecast;
        if (disposed) return;
        setForecast(result.status === "unavailable" ? null : result);
        setState(result.status);
      } catch {
        if (disposed) return;
        setForecast(null);
        setState("error");
      }
    };

    setState("loading");
    void refresh();
    const timer = window.setInterval(refresh, 60_000);
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, [enabled, symbol, timeframe]);

  return { forecast, state };
}
