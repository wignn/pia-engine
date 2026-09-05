"use client";

import React, { useEffect, useRef, useState } from "react";
import { createChart, IChartApi, ISeriesApi, CandlestickData, Time } from "lightweight-charts";
import { CandleData, Timeframe } from "@/types";

interface ChartAreaProps {
  symbol: string;
  provider: string;
  timeframe: Timeframe;
  digits: number;
  initialCandles: CandleData[];
  livePrice?: number;
}

export const ChartArea: React.FC<ChartAreaProps> = ({
  symbol,
  provider,
  timeframe,
  digits,
  initialCandles,
  livePrice
}) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const [ohlc, setOhlc] = useState<{ open: number; high: number; low: number; close: number; change: number; changePercent: number }>({
    open: initialCandles[initialCandles.length - 1]?.open ?? 0,
    high: initialCandles[initialCandles.length - 1]?.high ?? 0,
    low: initialCandles[initialCandles.length - 1]?.low ?? 0,
    close: initialCandles[initialCandles.length - 1]?.close ?? 0,
    change: 0,
    changePercent: 0
  });

  // Initialize Chart
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      width: chartContainerRef.current.clientWidth,
      height: chartContainerRef.current.clientHeight,
      layout: {
        background: { color: "#131722" },
        textColor: "#787b86",
        fontFamily: "-apple-system, BlinkMacSystemFont, 'Trebuchet MS', Roboto, sans-serif",
      },
      grid: {
        vertLines: { color: "#1f2431" },
        horzLines: { color: "#1f2431" },
      },
      crosshair: {
        mode: 1,
        vertLine: {
          color: "#787b86",
          width: 1,
          style: 3,
          labelBackgroundColor: "#2a2e39",
        },
        horzLine: {
          color: "#787b86",
          width: 1,
          style: 3,
          labelBackgroundColor: "#2a2e39",
        },
      },
      rightPriceScale: {
        borderColor: "#2a2e39",
        visible: true,
        scaleMargins: {
          top: 0.12,
          bottom: 0.15,
        },
      },
      timeScale: {
        borderColor: "#2a2e39",
        timeVisible: true,
        secondsVisible: false,
      },
    });

    const candlestickSeries = chart.addCandlestickSeries({
      upColor: "#089981",
      downColor: "#f23645",
      borderVisible: false,
      wickUpColor: "#089981",
      wickDownColor: "#f23645",
      priceFormat: {
        type: "price",
        precision: digits,
        minMove: 1 / Math.pow(10, digits),
      },
    });

    const formattedData: CandlestickData<Time>[] = initialCandles.map((c) => ({
      time: c.time as Time,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }));

    candlestickSeries.setData(formattedData);
    chartRef.current = chart;
    seriesRef.current = candlestickSeries;

    // Crosshair move handler
    chart.subscribeCrosshairMove((param) => {
      if (!param || !param.time || !param.seriesData) {
        const last = initialCandles[initialCandles.length - 1];
        if (last) {
          const ch = last.close - last.open;
          const chp = (ch / last.open) * 100;
          setOhlc({ open: last.open, high: last.high, low: last.low, close: last.close, change: ch, changePercent: chp });
        }
        return;
      }
      const data = param.seriesData.get(candlestickSeries) as CandlestickData | undefined;
      if (data) {
        const ch = data.close - data.open;
        const chp = (ch / data.open) * 100;
        setOhlc({ open: data.open, high: data.high, low: data.low, close: data.close, change: ch, changePercent: chp });
      }
    });

    // Resize observer
    const resizeObserver = new ResizeObserver((entries) => {
      if (!entries || entries.length === 0) return;
      const { width, height } = entries[0].contentRect;
      chart.applyOptions({ width, height });
    });
    resizeObserver.observe(chartContainerRef.current);

    return () => {
      resizeObserver.disconnect();
      chart.remove();
    };
  }, [symbol, digits]);

  // Handle live tick update
  useEffect(() => {
    if (!seriesRef.current || !livePrice) return;
    const last = initialCandles[initialCandles.length - 1];
    if (!last) return;

    const updatedCandle: CandlestickData<Time> = {
      time: last.time as Time,
      open: last.open,
      high: Math.max(last.high, livePrice),
      low: Math.min(last.low, livePrice),
      close: livePrice,
    };
    seriesRef.current.update(updatedCandle);
  }, [livePrice]);

  const isUp = ohlc.close >= ohlc.open;

  return (
    <div className="relative w-full h-full flex flex-col bg-[#131722] overflow-hidden select-none">
      {/* Chart Legend / Metadata Overlay (TradingView signature top-left) */}
      <div className="absolute top-3 left-3 z-10 pointer-events-none flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <span className="font-bold text-sm text-white tracking-wide">{symbol}</span>
          <span className="text-xs text-[#787b86] font-medium">{timeframe}</span>
          <span className="text-[10px] text-[#787b86] font-mono bg-[#1e222d] border border-[#2a2e39] px-1.5 py-0.5 rounded">
            {provider}
          </span>
        </div>

        {/* OHLCV readout */}
        <div className="flex flex-wrap items-center gap-3 text-[11px] font-mono">
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">O</span>
            <span className="text-[#d1d4dc]">{ohlc.open.toFixed(digits)}</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">H</span>
            <span className="text-[#d1d4dc]">{ohlc.high.toFixed(digits)}</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">L</span>
            <span className="text-[#d1d4dc]">{ohlc.low.toFixed(digits)}</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">C</span>
            <span className={isUp ? "text-[#089981]" : "text-[#f23645]"}>{ohlc.close.toFixed(digits)}</span>
          </div>
          <div className="flex items-center gap-1 font-semibold">
            <span className={isUp ? "text-[#089981]" : "text-[#f23645]"}>
              {isUp ? "+" : ""}{ohlc.change.toFixed(digits)} ({isUp ? "+" : ""}{ohlc.changePercent.toFixed(2)}%)
            </span>
          </div>
        </div>
      </div>

      {/* Main Lightweight Canvas Container */}
      <div ref={chartContainerRef} className="w-full flex-1" />
    </div>
  );
};
