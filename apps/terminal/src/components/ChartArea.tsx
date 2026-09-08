"use client";

import React, { useEffect, useRef, useState, useMemo } from "react";
import {
  createChart,
  IChartApi,
  ISeriesApi,
  CandlestickData,
  LineData,
  Time
} from "lightweight-charts";
import { CandleData, Timeframe, DrawingTool, DrawingItem, IndicatorState } from "@/types";
import { Wifi, WifiOff, Loader2 } from "lucide-react";
import { OscillatorPane } from "./OscillatorPane";

interface ChartAreaProps {
  symbol: string;
  provider: string;
  timeframe: Timeframe;
  chartType?: "candlestick" | "bar" | "line" | "area" | "heikin_ashi";
  indicators?: IndicatorState;
  activeTool?: DrawingTool;
  digits: number;
  candles: CandleData[];
  livePrice?: number | null;
  connected?: boolean;
  loading?: boolean;
  loadingOlder?: boolean;
  hasMoreHistory?: boolean;
  usingRealData?: boolean;
  onLoadOlder?: () => Promise<CandleData[]>;
  onDrawingsCountChange?: (count: number) => void;
  clearDrawingsTrigger?: number;
  snapshotTrigger?: number;
}

export const ChartArea: React.FC<ChartAreaProps> = ({
  symbol,
  provider,
  timeframe,
  chartType = "candlestick",
  indicators = { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
  activeTool = "cursor",
  digits,
  candles,
  livePrice,
  connected = false,
  loading = false,
  loadingOlder = false,
  hasMoreHistory = true,
  onLoadOlder,
  onDrawingsCountChange,
  clearDrawingsTrigger = 0,
  snapshotTrigger = 0,
}) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const lineRef = useRef<ISeriesApi<"Line"> | null>(null);
  const areaRef = useRef<ISeriesApi<"Area"> | null>(null);
  const barRef = useRef<ISeriesApi<"Bar"> | null>(null);

  // Technical Indicators Series
  const smaRef = useRef<ISeriesApi<"Line"> | null>(null);
  const emaRef = useRef<ISeriesApi<"Line"> | null>(null);
  const bbUpperRef = useRef<ISeriesApi<"Line"> | null>(null);
  const bbLowerRef = useRef<ISeriesApi<"Line"> | null>(null);
  const bbBasisRef = useRef<ISeriesApi<"Line"> | null>(null);

  const hoveringRef = useRef(false);
  const hasInitializedDataRef = useRef(false);
  const previousSymbolRef = useRef(symbol);
  const previousTimeframeRef = useRef(timeframe);
  const previousCandleCountRef = useRef(0);
  const previousFirstTimeRef = useRef<number | null>(null);

  const [ohlc, setOhlc] = useState({ open: 0, high: 0, low: 0, close: 0, change: 0, changePercent: 0 });

  // Drawings State
  const [drawings, setDrawings] = useState<DrawingItem[]>([]);
  const [currentDrawing, setCurrentDrawing] = useState<{
    type: DrawingItem["type"];
    p1: { x: number; y: number };
    p2?: { x: number; y: number };
  } | null>(null);
  const isDrawingRef = useRef(false);

  // Load drawings from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem(`atlsd_drawings_${symbol}`);
      const items: DrawingItem[] = stored ? JSON.parse(stored) : [];
      setDrawings(items);
      onDrawingsCountChange?.(items.length);
    } catch {
      setDrawings([]);
      onDrawingsCountChange?.(0);
    }
  }, [symbol, onDrawingsCountChange]);

  // Clear drawings trigger
  useEffect(() => {
    if (clearDrawingsTrigger > 0) {
      setDrawings([]);
      localStorage.removeItem(`atlsd_drawings_${symbol}`);
      onDrawingsCountChange?.(0);
    }
  }, [clearDrawingsTrigger, symbol, onDrawingsCountChange]);

  // Snapshot trigger effect
  useEffect(() => {
    if (snapshotTrigger > 0 && chartRef.current) {
      try {
        const canvas = chartRef.current.takeScreenshot();
        const watermarked = document.createElement("canvas");
        watermarked.width = canvas.width;
        watermarked.height = canvas.height;
        const ctx = watermarked.getContext("2d");
        if (ctx) {
          ctx.drawImage(canvas, 0, 0);

          // Dark footer bar
          ctx.fillStyle = "rgba(19, 23, 34, 0.88)";
          ctx.fillRect(0, canvas.height - 40, canvas.width, 40);

          // Brand Watermark
          ctx.font = "bold 15px -apple-system, sans-serif";
          ctx.fillStyle = "#2962ff";
          ctx.fillText("ATLSD TERMINAL", 20, canvas.height - 16);

          // Meta watermark
          ctx.font = "12px monospace";
          ctx.fillStyle = "#d1d4dc";
          ctx.fillText(`${symbol} · ${timeframe} · ${new Date().toISOString().replace("T", " ").substring(0, 19)} UTC`, 175, canvas.height - 16);

          const url = watermarked.toDataURL("image/png");
          const a = document.createElement("a");
          a.href = url;
          a.download = `ATLSD_${symbol}_${timeframe}_${Date.now()}.png`;
          a.click();
        }
      } catch (err) {
        console.warn("[ChartArea] Snapshot export failed:", err);
      }
    }
  }, [snapshotTrigger, symbol, timeframe]);

  const saveDrawings = (newDrawings: DrawingItem[]) => {
    setDrawings(newDrawings);
    onDrawingsCountChange?.(newDrawings.length);
    try {
      localStorage.setItem(`atlsd_drawings_${symbol}`, JSON.stringify(newDrawings));
    } catch {
      // Ignore write errors
    }
  };

  // Initialize chart
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
    const lineSeries = chart.addLineSeries({ color: "#d1d4dc", lineWidth: 2, priceLineVisible: false });
    const areaSeries = chart.addAreaSeries({
      lineColor: "#2962ff",
      topColor: "#2962ff55",
      bottomColor: "#2962ff05",
      lineWidth: 2,
      priceLineVisible: false,
    });
    const barSeries = chart.addBarSeries({ upColor: "#089981", downColor: "#f23645", openVisible: true, thinBars: false });

    // Indicators
    const smaSeries = chart.addLineSeries({ color: "#f5b942", lineWidth: 2, priceLineVisible: false, lastValueVisible: false });
    const emaSeries = chart.addLineSeries({ color: "#2962ff", lineWidth: 2, priceLineVisible: false, lastValueVisible: false });
    const bbUpperSeries = chart.addLineSeries({ color: "rgba(8, 153, 129, 0.7)", lineWidth: 1, lineStyle: 2, priceLineVisible: false, lastValueVisible: false });
    const bbBasisSeries = chart.addLineSeries({ color: "rgba(245, 185, 66, 0.6)", lineWidth: 1, lineStyle: 0, priceLineVisible: false, lastValueVisible: false });
    const bbLowerSeries = chart.addLineSeries({ color: "rgba(242, 54, 69, 0.7)", lineWidth: 1, lineStyle: 2, priceLineVisible: false, lastValueVisible: false });

    chartRef.current = chart;
    seriesRef.current = candlestickSeries;
    lineRef.current = lineSeries;
    areaRef.current = areaSeries;
    barRef.current = barSeries;
    smaRef.current = smaSeries;
    emaRef.current = emaSeries;
    bbUpperRef.current = bbUpperSeries;
    bbBasisRef.current = bbBasisSeries;
    bbLowerRef.current = bbLowerSeries;

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
      lineRef.current = null;
      areaRef.current = null;
      barRef.current = null;
      smaRef.current = null;
      emaRef.current = null;
      bbUpperRef.current = null;
      bbBasisRef.current = null;
      bbLowerRef.current = null;
    };
  }, []);

  // Infinite history handler
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart || !onLoadOlder) return;
    let fetching = false;
    const handler = async (range: { from: number; to: number } | null) => {
      if (!range || fetching || range.from > 20) return;
      fetching = true;
      try {
        await onLoadOlder();
      } finally {
        fetching = false;
      }
    };
    chart.timeScale().subscribeVisibleLogicalRangeChange(handler);
    return () => chart.timeScale().unsubscribeVisibleLogicalRangeChange(handler);
  }, [onLoadOlder]);

  // Digits precision
  useEffect(() => {
    seriesRef.current?.applyOptions({
      priceFormat: { type: "price", precision: digits, minMove: 1 / Math.pow(10, digits) },
    });
  }, [digits]);

  // Calculate live RSI
  const rsiValue = useMemo(() => {
    if (!indicators.rsi || candles.length < 15) return null;
    const period = 14;
    let gains = 0;
    let losses = 0;
    for (let i = 1; i <= period; i++) {
      const diff = candles[i].close - candles[i - 1].close;
      if (diff >= 0) gains += diff;
      else losses += Math.abs(diff);
    }
    let avgGain = gains / period;
    let avgLoss = losses / period;
    for (let i = period + 1; i < candles.length; i++) {
      const diff = candles[i].close - candles[i - 1].close;
      if (diff >= 0) {
        avgGain = (avgGain * (period - 1) + diff) / period;
        avgLoss = (avgLoss * (period - 1)) / period;
      } else {
        avgGain = (avgGain * (period - 1)) / period;
        avgLoss = (avgLoss * (period - 1) + Math.abs(diff)) / period;
      }
    }
    if (avgLoss === 0) return 100;
    const rs = avgGain / avgLoss;
    return 100 - (100 / (1 + rs));
  }, [candles, indicators.rsi]);

  // Update chart data & indicators
  useEffect(() => {
    if (!seriesRef.current) return;
    const data: CandlestickData<Time>[] = candles.map((c) => ({
      time: c.time as Time,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }));
    const chart = chartRef.current;
    const symbolChanged = previousSymbolRef.current !== symbol || previousTimeframeRef.current !== timeframe;
    const rangeBeforeUpdate = chart && hasInitializedDataRef.current ? chart.timeScale().getVisibleLogicalRange() : null;
    const firstTime = candles[0]?.time ?? null;
    const prependedCount = !symbolChanged && previousFirstTimeRef.current !== null && firstTime !== null && firstTime < previousFirstTimeRef.current
      ? Math.max(0, candles.length - previousCandleCountRef.current)
      : 0;

    const lineData: LineData<Time>[] = candles.map((c) => ({ time: c.time as Time, value: c.close }));
    const heikinData: CandlestickData<Time>[] = candles.reduce<CandlestickData<Time>[]>((result, c, index) => {
      const close = (c.open + c.high + c.low + c.close) / 4;
      const open = index === 0 ? (c.open + c.close) / 2 : ((result[index - 1].open as number) + (result[index - 1].close as number)) / 2;
      result.push({ time: c.time as Time, open, high: Math.max(c.high, open, close), low: Math.min(c.low, open, close), close });
      return result;
    }, []);

    seriesRef.current.setData(chartType === "heikin_ashi" ? heikinData : chartType === "candlestick" ? data : []);
    barRef.current?.setData(chartType === "bar" ? data : []);
    lineRef.current?.setData(chartType === "line" ? lineData : []);
    areaRef.current?.setData(chartType === "area" ? lineData : []);

    // Moving Averages
    const movingAverage = (period: number, exponential: boolean): LineData<Time>[] => {
      const output: LineData<Time>[] = [];
      let previous: number | null = null;
      candles.forEach((c, index) => {
        if (index + 1 < period) return;
        if (exponential) {
          const alpha = 2 / (period + 1);
          previous = previous == null ? candles.slice(index + 1 - period, index + 1).reduce((sum, row) => sum + row.close, 0) / period : c.close * alpha + previous * (1 - alpha);
        } else {
          previous = candles.slice(index + 1 - period, index + 1).reduce((sum, row) => sum + row.close, 0) / period;
        }
        output.push({ time: c.time as Time, value: previous });
      });
      return output;
    };

    smaRef.current?.setData(indicators.sma20 ? movingAverage(20, false) : []);
    emaRef.current?.setData(indicators.ema50 ? movingAverage(50, true) : []);

    // Bollinger Bands (20, 2)
    if (indicators.bollinger && candles.length >= 20) {
      const upper: LineData<Time>[] = [];
      const basis: LineData<Time>[] = [];
      const lower: LineData<Time>[] = [];
      const period = 20;
      const mult = 2;

      for (let i = period - 1; i < candles.length; i++) {
        const slice = candles.slice(i - period + 1, i + 1);
        const mean = slice.reduce((sum, c) => sum + c.close, 0) / period;
        const variance = slice.reduce((sum, c) => sum + Math.pow(c.close - mean, 2), 0) / period;
        const std = Math.sqrt(variance);
        const t = candles[i].time as Time;
        basis.push({ time: t, value: mean });
        upper.push({ time: t, value: mean + mult * std });
        lower.push({ time: t, value: mean - mult * std });
      }
      bbUpperRef.current?.setData(upper);
      bbBasisRef.current?.setData(basis);
      bbLowerRef.current?.setData(lower);
    } else {
      bbUpperRef.current?.setData([]);
      bbBasisRef.current?.setData([]);
      bbLowerRef.current?.setData([]);
    }

    if (!hasInitializedDataRef.current || symbolChanged) {
      chart?.timeScale().fitContent();
      hasInitializedDataRef.current = true;
      previousSymbolRef.current = symbol;
      previousTimeframeRef.current = timeframe;
    } else if (rangeBeforeUpdate) {
      const offset = prependedCount > 0 ? prependedCount : 0;
      chart?.timeScale().setVisibleLogicalRange({
        from: rangeBeforeUpdate.from + offset,
        to: rangeBeforeUpdate.to + offset,
      });
    }
    previousCandleCountRef.current = candles.length;
    previousFirstTimeRef.current = firstTime;

    const last = candles[candles.length - 1];
    if (last && !hoveringRef.current) {
      const ch = last.close - last.open;
      const chp = last.open ? (ch / last.open) * 100 : 0;
      setOhlc({ open: last.open, high: last.high, low: last.low, close: last.close, change: ch, changePercent: chp });
    }
  }, [candles, chartType, indicators.sma20, indicators.ema50, indicators.bollinger, symbol, timeframe]);

  // Live price tick update
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

  // Drawing Canvas Handlers
  const handleMouseDown = (e: React.MouseEvent<SVGSVGElement>) => {
    if (activeTool === "cursor") return;
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    isDrawingRef.current = true;
    setCurrentDrawing({
      type: activeTool,
      p1: { x, y },
      p2: { x, y },
    });
  };

  const handleMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    if (!isDrawingRef.current || !currentDrawing) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    setCurrentDrawing({
      ...currentDrawing,
      p2: { x, y },
    });
  };

  const handleMouseUp = () => {
    if (!isDrawingRef.current || !currentDrawing) return;
    isDrawingRef.current = false;

    if (currentDrawing.p2) {
      const dx = Math.abs(currentDrawing.p1.x - currentDrawing.p2.x);
      const dy = Math.abs(currentDrawing.p1.y - currentDrawing.p2.y);
      // Ignore accidental micro clicks
      if (dx > 4 || dy > 4 || currentDrawing.type === "horizontal") {
        const item: DrawingItem = {
          id: "draw_" + Date.now(),
          type: currentDrawing.type,
          symbol,
          p1: currentDrawing.p1,
          p2: currentDrawing.p2,
        };
        saveDrawings([...drawings, item]);
      }
    }
    setCurrentDrawing(null);
  };

  const isUp = ohlc.close >= ohlc.open;
  const hasData = candles.length > 0;

  return (
    <div className="relative w-full h-full flex flex-col bg-[#131722] overflow-hidden select-none">
      {/* Chart Legend Overlay */}
      <div className="absolute top-3 left-3 z-10 pointer-events-none flex flex-col gap-1.5">
        <div className="flex items-center gap-2">
          <span className="font-bold text-sm text-white tracking-wide">{symbol}</span>
          <span className="text-xs text-[#787b86] font-medium">{timeframe}</span>
          <span className="text-[10px] text-[#787b86] font-mono bg-[#1e222d] border border-[#2a2e39] px-1.5 py-0.5 rounded">
            {provider}
          </span>
          <span
            className={`flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded ${
              connected ? "bg-[#089981]/15 text-[#089981]" : "bg-[#787b86]/15 text-[#787b86]"
            }`}
          >
            {connected ? <Wifi className="w-2.5 h-2.5" /> : <WifiOff className="w-2.5 h-2.5" />}
            {connected ? "LIVE" : "OFFLINE"}
          </span>

          {/* Indicators Badges */}
          {indicators.bollinger && (
            <span className="text-[10px] font-mono bg-[#1e222d] border border-[#2a2e39] px-1.5 py-0.5 rounded text-[#f5b942]">
              BB(20,2)
            </span>
          )}
          {indicators.rsi && rsiValue !== null && (
            <span
              className={`text-[10px] font-mono font-bold px-1.5 py-0.5 rounded border ${
                rsiValue >= 70
                  ? "bg-[#f23645]/20 border-[#f23645]/40 text-[#f23645]"
                  : rsiValue <= 30
                  ? "bg-[#089981]/20 border-[#089981]/40 text-[#089981]"
                  : "bg-[#1e222d] border-[#2a2e39] text-[#2962ff]"
              }`}
            >
              RSI(14): {rsiValue.toFixed(1)} {rsiValue >= 70 ? "OVERBOUGHT" : rsiValue <= 30 ? "OVERSOLD" : "NEUTRAL"}
            </span>
          )}
        </div>

        {/* OHLC Bar Metrics */}
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

      {/* Loading & Empty Overlays */}
      {loading && !loadingOlder && (
        <div className="absolute inset-0 z-20 flex items-center justify-center bg-[#131722]/60 pointer-events-none">
          <div className="flex items-center gap-2 text-[#787b86] text-sm">
            <Loader2 className="w-4 h-4 animate-spin" /> Loading {symbol} history…
          </div>
        </div>
      )}
      {loadingOlder && (
        <div className="absolute top-3 left-1/2 z-20 -translate-x-1/2 flex items-center gap-2 rounded bg-[#1e222d] border border-[#2a2e39] px-3 py-1.5 text-[11px] text-[#d1d4dc] shadow-lg pointer-events-none">
          <Loader2 className="w-3 h-3 animate-spin" /> Loading older candles…
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

      {/* Chart Canvas */}
      <div ref={chartContainerRef} className="w-full flex-1" />

      {/* Oscillator Sub-pane (RSI / MACD) */}
      <OscillatorPane candles={candles} indicators={indicators} />

      {/* Interactive SVG Drawing Overlay */}
      <svg
        className={`absolute inset-0 w-full h-full z-15 ${
          activeTool === "cursor" ? "pointer-events-none" : "pointer-events-auto cursor-crosshair"
        }`}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
      >
        {/* Render Saved Drawings */}
        {drawings.map((d) => {
          if (d.type === "trendline" && d.p2) {
            return (
              <g key={d.id}>
                <line
                  x1={d.p1.x}
                  y1={d.p1.y}
                  x2={d.p2.x}
                  y2={d.p2.y}
                  stroke="#2962ff"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
                <circle cx={d.p1.x} cy={d.p1.y} r="3.5" fill="#2962ff" />
                <circle cx={d.p2.x} cy={d.p2.y} r="3.5" fill="#2962ff" />
              </g>
            );
          }
          if (d.type === "horizontal") {
            return (
              <g key={d.id}>
                <line
                  x1={0}
                  y1={d.p1.y}
                  x2="100%"
                  y2={d.p1.y}
                  stroke="#f5b942"
                  strokeWidth="1.5"
                  strokeDasharray="4 2"
                />
                <circle cx={d.p1.x} cy={d.p1.y} r="3.5" fill="#f5b942" />
              </g>
            );
          }
          if (d.type === "fibonacci" && d.p2) {
            const yMin = Math.min(d.p1.y, d.p2.y);
            const yMax = Math.max(d.p1.y, d.p2.y);
            const height = yMax - yMin;
            const levels = [0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];
            return (
              <g key={d.id}>
                {levels.map((lvl) => {
                  const y = yMin + height * lvl;
                  return (
                    <g key={lvl}>
                      <line x1={0} y1={y} x2="100%" y2={y} stroke="#2962ff88" strokeWidth="1" />
                      <text x={10} y={y - 3} fill="#787b86" fontSize="9" fontFamily="monospace">
                        {(lvl * 100).toFixed(1)}%
                      </text>
                    </g>
                  );
                })}
              </g>
            );
          }
          if (d.type === "measure" && d.p2) {
            const width = Math.abs(d.p2.x - d.p1.x);
            const height = Math.abs(d.p2.y - d.p1.y);
            const x = Math.min(d.p1.x, d.p2.x);
            const y = Math.min(d.p1.y, d.p2.y);
            return (
              <g key={d.id}>
                <rect x={x} y={y} width={width} height={height} fill="#2962ff15" stroke="#2962ff" strokeWidth="1" strokeDasharray="3 3" />
              </g>
            );
          }
          return null;
        })}

        {/* Render Current In-Progress Drawing */}
        {currentDrawing && currentDrawing.p2 && (
          <g>
            {currentDrawing.type === "trendline" && (
              <line
                x1={currentDrawing.p1.x}
                y1={currentDrawing.p1.y}
                x2={currentDrawing.p2.x}
                y2={currentDrawing.p2.y}
                stroke="#2962ff"
                strokeWidth="2"
                strokeDasharray="4 2"
              />
            )}
            {currentDrawing.type === "horizontal" && (
              <line
                x1={0}
                y1={currentDrawing.p1.y}
                x2="100%"
                y2={currentDrawing.p1.y}
                stroke="#f5b942"
                strokeWidth="1.5"
                strokeDasharray="3 3"
              />
            )}
            {currentDrawing.type === "fibonacci" && (
              <rect
                x={Math.min(currentDrawing.p1.x, currentDrawing.p2.x)}
                y={Math.min(currentDrawing.p1.y, currentDrawing.p2.y)}
                width={Math.abs(currentDrawing.p2.x - currentDrawing.p1.x)}
                height={Math.abs(currentDrawing.p2.y - currentDrawing.p1.y)}
                fill="#2962ff20"
                stroke="#2962ff"
                strokeWidth="1"
              />
            )}
            {currentDrawing.type === "measure" && (
              <rect
                x={Math.min(currentDrawing.p1.x, currentDrawing.p2.x)}
                y={Math.min(currentDrawing.p1.y, currentDrawing.p2.y)}
                width={Math.abs(currentDrawing.p2.x - currentDrawing.p1.x)}
                height={Math.abs(currentDrawing.p2.y - currentDrawing.p1.y)}
                fill="#2962ff20"
                stroke="#2962ff"
                strokeWidth="1"
                strokeDasharray="3 3"
              />
            )}
          </g>
        )}
      </svg>
    </div>
  );
};
