"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { CandleData, Timeframe } from "@/types";

// Map terminal timeframe -> core market-data resolution string.
const RESOLUTION_MAP: Record<Timeframe, string> = {
  "1m": "1m",
  "5m": "5m",
  "15m": "15m",
  "1h": "1h",
  "4h": "4h",
  "1D": "1D",
  "1W": "1W",
};

interface HistoryRow {
  time: number | string;
  open: number;
  high: number;
  low: number;
  close: number;
  value?: number;
  volume?: number;
}

export interface MarketFeedState {
  candles: CandleData[];
  livePrice: number | null;
  connected: boolean;
  loading: boolean;
  usingRealData: boolean;
}

/**
 * Loads real OHLC history for a symbol from the core market-data service and
 * keeps the last candle updated from the realtime WebSocket. Falls back
 * gracefully (empty history / no live) when the backend is unreachable.
 */
export function useMarketFeed(symbol: string, timeframe: Timeframe): MarketFeedState {
  const [candles, setCandles] = useState<CandleData[]>([]);
  const [livePrice, setLivePrice] = useState<number | null>(null);
  const [connected, setConnected] = useState(false);
  const [loading, setLoading] = useState(true);
  const [usingRealData, setUsingRealData] = useState(false);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectDelay = useRef(1000);
  const symbolRef = useRef(symbol);
  symbolRef.current = symbol;
  const intervalSecRef = useRef(900);

  // Interval in seconds for the active timeframe (used to bucket live ticks).
  const tfSeconds = (tf: Timeframe): number => {
    switch (tf) {
      case "1m": return 60;
      case "5m": return 300;
      case "15m": return 900;
      case "1h": return 3600;
      case "4h": return 14400;
      case "1D": return 86400;
      case "1W": return 604800;
      default: return 900;
    }
  };

  // ---- History load ----
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    intervalSecRef.current = tfSeconds(timeframe);

    (async () => {
      try {
        const res = await fetch(
          `/api/market/history/${encodeURIComponent(symbol)}?resolution=${RESOLUTION_MAP[timeframe]}`,
          { cache: "no-store" }
        );
        if (!res.ok) throw new Error(`history ${res.status}`);
        const rows: HistoryRow[] = await res.json();

        if (cancelled) return;

        if (Array.isArray(rows) && rows.length > 0) {
          const mapped: CandleData[] = rows
            .map((r) => ({
              time: Math.floor(Number(r.time)),
              open: Number(r.open ?? r.value),
              high: Number(r.high ?? r.value),
              low: Number(r.low ?? r.value),
              close: Number(r.close ?? r.value),
              volume: Number(r.volume ?? 0),
            }))
            .filter((c) => Number.isFinite(c.time) && c.time > 0 && Number.isFinite(c.close))
            .sort((a, b) => a.time - b.time);

          setCandles(mapped);
          setLivePrice(mapped[mapped.length - 1]?.close ?? null);
          setUsingRealData(true);
        } else {
          setCandles([]);
          setUsingRealData(false);
        }
      } catch {
        if (!cancelled) {
          setCandles([]);
          setUsingRealData(false);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [symbol, timeframe]);

  // ---- Apply a live tick to the last candle (or roll a new one) ----
  const applyTick = useCallback((price: number, tsMs: number) => {
    setLivePrice(price);
    setCandles((prev) => {
      if (prev.length === 0) return prev;
      const intervalSec = intervalSecRef.current;
      const tsSec = Math.floor(tsMs / 1000);
      const bucket = Math.floor(tsSec / intervalSec) * intervalSec;
      const last = prev[prev.length - 1];

      if (bucket <= last.time) {
        // update in-place: extend high/low, move close
        const updated = { ...last };
        updated.high = Math.max(last.high, price);
        updated.low = Math.min(last.low, price);
        updated.close = price;
        return [...prev.slice(0, -1), updated];
      }

      // roll a new candle
      const fresh: CandleData = {
        time: bucket,
        open: last.close,
        high: Math.max(last.close, price),
        low: Math.min(last.close, price),
        close: price,
        volume: 0,
      };
      return [...prev.slice(-599), fresh];
    });
  }, []);

  // ---- WebSocket realtime connection (lives across symbol changes) ----
  useEffect(() => {
    let disposed = false;

    const scheduleReconnect = () => {
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      const delay = reconnectDelay.current + Math.floor(Math.random() * 250);
      reconnectRef.current = setTimeout(() => {
        reconnectDelay.current = Math.min(reconnectDelay.current * 1.5, 30000);
        void connect();
      }, delay);
    };

    const connect = async () => {
      if (disposed) return;
      if (
        wsRef.current &&
        (wsRef.current.readyState === WebSocket.OPEN ||
          wsRef.current.readyState === WebSocket.CONNECTING)
      )
        return;

      let ticket: string;
      let wsUrl: string;
      try {
        const res = await fetch("/api/realtime/session", { method: "POST", cache: "no-store" });
        if (!res.ok) throw new Error(`session ${res.status}`);
        const data = await res.json();
        if (!data?.ticket) throw new Error("no ticket");
        ticket = data.ticket;
        wsUrl = data.wsUrl;
      } catch {
        setConnected(false);
        scheduleReconnect();
        return;
      }

      const url = `${wsUrl}/ws/v1?bot_id=terminal_client&ticket=${encodeURIComponent(ticket)}`;
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        reconnectDelay.current = 1000;
        setConnected(true);
        ws.send(JSON.stringify({ method: "SUBSCRIBE", params: ["market_data"], id: 1 }));
      };

      ws.onmessage = (evt) => {
        try {
          const msg = JSON.parse(evt.data);
          if (msg.event === "market.trade" && msg.data?.tick) {
            const tick = msg.data.tick;
            const tickSym = String(tick.symbol ?? "").toUpperCase();
            if (tickSym === symbolRef.current.toUpperCase() && Number.isFinite(tick.price)) {
              const tsMs = tick.received_at ? Date.parse(tick.received_at) : Date.now();
              applyTick(Number(tick.price), tsMs);
            }
          }
        } catch {
          /* ignore non-JSON frames */
        }
      };

      ws.onclose = () => {
        setConnected(false);
        if (!disposed) scheduleReconnect();
      };

      ws.onerror = () => {
        ws.close();
      };
    };

    void connect();

    return () => {
      disposed = true;
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      wsRef.current?.close();
      wsRef.current = null;
    };
  }, [applyTick]);

  return { candles, livePrice, connected, loading, usingRealData };
}
