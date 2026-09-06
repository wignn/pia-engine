"use client";

import React, { useEffect, useRef, useState } from "react";
import { createChart, IChartApi, ISeriesApi, CandlestickData, Time } from "lightweight-charts";
import { CandleData, Timeframe } from "@/types";
import { Wifi, WifiOff, Loader2 } from "lucide-react";

interface ChartAreaProps {
  symbol: string;
  provider: string;
  timeframe: Timeframe;
  digits: number;
  candles: CandleData[];
  livePrice?: number | null;
  connected?: boolean;
  loading?: boolean;
  usingRealData?: boolean;
  onLoadOlder?: () => Promise<CandleData[]>;
}

export const ChartArea: React.FC<ChartAreaProps> = ({
  symbol,
  provider,
  timeframe,
  digits,
  candles,
  livePrice,
  connected = false,
  loading = false,
  usingRealData = false,
  onLoadOlder,
}) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const hoveringRef = useRef(false);
  const [ohlc, setOhlc] = useState({ open: 0, high: 0, low: 0, close: 0, change: 0, changePercent: 0 });

  // Initialize chart once.
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      width: chartContainerRef.current.clientWidth,
      height: chartContainerRef.current.clientHeight,
      layout: {
        background: { color: "#131722" },
        textColor: "#787b86",
        fontFamily: "-apple-system, BlinkMacSystemFont, 'Trebuchet MS', Roboto, sans-serif",
        attributionLogo: false,
      },
      grid: {
        vertLines: { color: "#1f2431" },
        horzLines: { color: "#1f2431" },
      },
      crosshair: {
        mode: 1,
        vertLine: { color: "#787b86", width: 1, style: 3, labelBackgroundColor: "#2a2e39" },
        horzLine: { color: "#787b86", width: 1, style: 3, labelBackgroundColor: "#2a2e39" },
      },
      rightPriceScale: {
        borderColor: "#2a2e39",
        visible: true,
        scaleMargins: { top: 0.12, bottom: 0.15 },
      },
      timeScale: { borderColor: "#2a2e39", timeVisible: true, secondsVisible: false },
    });

    const candlestickSeries = chart.addCandlestickSeries({
      upColor: "#089981",
      downColor: "#f23645",
      borderVisible: false,
      wickUpColor: "#089981",
      wickDownColor: "#f23645",
    });

    chartRef.current = chart;
    seriesRef.current = candlestickSeries;

    chart.subscribeCrosshairMove((param) => {
      if (!param || !param.time || !param.seriesData) {
        hoveringRef.current = false;
        return;
      }
      hoveringRef.current = true;
      const data = param.seriesData.get(candlestickSeries) as CandlestickData | undefined;
      if (data) {
        const ch = data.close - data.open;
        const chp = data.open ? (ch / data.open) * 100 : 0;
        setOhlc({ open: data.open, high: data.high, low: data.low, close: data.close, change: ch, changePercent: chp });
      }
    });

    const resizeObserver = new ResizeObserver((entries) => {
      if (!entries || entries.length === 0) return;
      const { width, height } = entries[0].contentRect;
      chart.applyOptions({ width, height });
    });
    resizeObserver.observe(chartContainerRef.current);

    return () => {
      resizeObserver.disconnect();
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
    };
  }, []);

  // Infinite left history: fetch older pages as the user pans/zooms to the left.
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart || !onLoadOlder) return;
    let fetching = false;
    const handler = async (range: { from: number; to: number } | null) => {
      if (!range || fetching || range.from > 20) return;
      fetching = true;
      const previousFrom = range.from;
      const older = await onLoadOlder();
      if (older.length > 0) {
        window.setTimeout(() => {
          chart.timeScale().setVisibleLogicalRange({
            from: previousFrom + older.length,
            to: range.to + older.length,
          });
        }, 0);
      }
      fetching = false;
    };
    chart.timeScale().subscribeVisibleLogicalRangeChange(handler);
    return () => chart.timeScale().unsubscribeVisibleLogicalRangeChange(handler);
  }, [onLoadOlder]);

  // Update precision when symbol/digits change.
  useEffect(() => {
    seriesRef.current?.applyOptions({
      priceFormat: { type: "price", precision: digits, minMove: 1 / Math.pow(10, digits) },
    });
  }, [digits]);

  // Push full dataset whenever candles array changes (symbol / timeframe / history load).
  useEffect(() => {
    if (!seriesRef.current) return;
    const data: CandlestickData<Time>[] = candles.map((c) => ({
      time: c.time as Time,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }));
    seriesRef.current.setData(data);
    chartRef.current?.timeScale().fitContent();

    const last = candles[candles.length - 1];
    if (last && !hoveringRef.current) {
      const ch = last.close - last.open;
      const chp = last.open ? (ch / last.open) * 100 : 0;
      setOhlc({ open: last.open, high: last.high, low: last.low, close: last.close, change: ch, changePercent: chp });
    }
  }, [candles]);

  // Live tick: update the last candle in-place.
  useEffect(() => {
    if (!seriesRef.current || livePrice == null || candles.length === 0) return;
    const last = candles[candles.length - 1];
    seriesRef.current.update({
      time: last.time as Time,
      open: last.open,
      high: Math.max(last.high, livePrice),
      low: Math.min(last.low, livePrice),
      close: livePrice,
    });
    if (!hoveringRef.current) {
      const ch = livePrice - last.open;
      const chp = last.open ? (ch / last.open) * 100 : 0;
      setOhlc({ open: last.open, high: Math.max(last.high, livePrice), low: Math.min(last.low, livePrice), close: livePrice, change: ch, changePercent: chp });
    }
  }, [livePrice]);

  const isUp = ohlc.close >= ohlc.open;
  const hasData = candles.length > 0;

  return (
    <div className="relative w-full h-full flex flex-col bg-[#131722] overflow-hidden select-none">
      {/* Chart legend overlay */}
      <div className="absolute top-3 left-3 z-10 pointer-events-none flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <span className="font-bold text-sm text-white tracking-wide">{symbol}</span>
          <span className="text-xs text-[#787b86] font-medium">{timeframe}</span>
          <span className="text-[10px] text-[#787b86] font-mono bg-[#1e222d] border border-[#2a2e39] px-1.5 py-0.5 rounded">
            {provider}
          </span>
          {/* Live / connection badge */}
          <span
            className={`flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded ${
              connected ? "bg-[#089981]/15 text-[#089981]" : "bg-[#787b86]/15 text-[#787b86]"
            }`}
          >
            {connected ? <Wifi className="w-2.5 h-2.5" /> : <WifiOff className="w-2.5 h-2.5" />}
            {connected ? "LIVE" : "OFFLINE"}
          </span>
        </div>

        <div className="flex flex-wrap items-center gap-3 text-[11px] font-mono">
          <div className="flex items-center gap-1"><span className="text-[#787b86]">O</span><span className="text-[#d1d4dc]">{ohlc.open.toFixed(digits)}</span></div>
          <div className="flex items-center gap-1"><span className="text-[#787b86]">H</span><span className="text-[#d1d4dc]">{ohlc.high.toFixed(digits)}</span></div>
          <div className="flex items-center gap-1"><span className="text-[#787b86]">L</span><span className="text-[#d1d4dc]">{ohlc.low.toFixed(digits)}</span></div>
          <div className="flex items-center gap-1"><span className="text-[#787b86]">C</span><span className={isUp ? "text-[#089981]" : "text-[#f23645]"}>{ohlc.close.toFixed(digits)}</span></div>
          <div className="flex items-center gap-1 font-semibold">
            <span className={isUp ? "text-[#089981]" : "text-[#f23645]"}>
              {isUp ? "+" : ""}{ohlc.change.toFixed(digits)} ({isUp ? "+" : ""}{ohlc.changePercent.toFixed(2)}%)
            </span>
          </div>
        </div>
      </div>

      {/* Loading / empty overlays */}
      {loading && (
        <div className="absolute inset-0 z-20 flex items-center justify-center bg-[#131722]/60 pointer-events-none">
          <div className="flex items-center gap-2 text-[#787b86] text-sm">
            <Loader2 className="w-4 h-4 animate-spin" /> Loading {symbol} history…
          </div>
        </div>
      )}
      {!loading && !hasData && (
        <div className="absolute inset-0 z-20 flex items-center justify-center pointer-events-none">
          <div className="flex flex-col items-center gap-1 text-center">
            <span className="text-[#d1d4dc] font-semibold">No historical data for {symbol}</span>
            <span className="text-[11px] text-[#787b86]">
              Market may be closed, or this symbol isn't ingested yet.
            </span>
          </div>
        </div>
      )}

      <div ref={chartContainerRef} className="w-full flex-1" />
    </div>
  );
};
