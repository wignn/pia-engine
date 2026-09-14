"use client";

import React, { useEffect, useRef, useState, useMemo, useCallback } from "react";
import {
  createChart,
  IChartApi,
  ISeriesApi,
  CandlestickData,
  LineData,
  HistogramData,
  Time,
  PriceScaleMode,
} from "lightweight-charts";
import { CandleData, Timeframe, DrawingTool, DrawingItem, DrawingPoint, IndicatorState } from "@/types";
import { Wifi, WifiOff, Loader2, Trash2, X, Eye, EyeOff } from "lucide-react";
import { OscillatorPane } from "./OscillatorPane";

interface ChartAreaProps {
  paneId?: string;
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
  onSnapshotDone?: () => void;
  onToggleIndicator?: (indicator: keyof IndicatorState) => void;
  isDrawingsHidden?: boolean;
  isDrawingModeLocked?: boolean;
  onDrawingFinished?: () => void;
  onCanUndoRedoChange?: (canUndo: boolean, canRedo: boolean) => void;
  undoTrigger?: number;
  redoTrigger?: number;
  theme?: "dark" | "light";
  settings?: import("@/types").TerminalSettings;
}

export const ChartArea: React.FC<ChartAreaProps> = ({
  paneId,
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
  onSnapshotDone,
  onToggleIndicator,
  isDrawingsHidden = false,
  isDrawingModeLocked = false,
  onDrawingFinished,
  onCanUndoRedoChange,
  undoTrigger = 0,
  redoTrigger = 0,
  theme = "dark",
  settings,
}) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const lineRef = useRef<ISeriesApi<"Line"> | null>(null);
  const areaRef = useRef<ISeriesApi<"Area"> | null>(null);
  const barRef = useRef<ISeriesApi<"Bar"> | null>(null);
  const volumeRef = useRef<ISeriesApi<"Histogram"> | null>(null);

  // Technical Indicators Series
  const smaRef = useRef<ISeriesApi<"Line"> | null>(null);
  const emaRef = useRef<ISeriesApi<"Line"> | null>(null);
  const bbUpperRef = useRef<ISeriesApi<"Line"> | null>(null);
  const bbLowerRef = useRef<ISeriesApi<"Line"> | null>(null);
  const bbBasisRef = useRef<ISeriesApi<"Line"> | null>(null);
  const vwapRef = useRef<ISeriesApi<"Line"> | null>(null);

  const hoveringRef = useRef(false);
  const hasInitializedDataRef = useRef(false);
  const previousSymbolRef = useRef(symbol);
  const previousTimeframeRef = useRef(timeframe);
  const previousCandleCountRef = useRef(0);
  const previousFirstTimeRef = useRef<number | null>(null);

  // Re-projection tick to keep SVG drawings locked to candlestick coordinates
  const [, setProjectionTick] = useState(0);

  // Price Scale Modes: Normal, Logarithmic, Percentage
  const [scaleMode, setScaleMode] = useState<"normal" | "log" | "percent">("normal");

  const [ohlc, setOhlc] = useState({ open: 0, high: 0, low: 0, close: 0, change: 0, changePercent: 0 });

  // Drawings State & Undo/Redo Stacks
  const [drawings, setDrawings] = useState<DrawingItem[]>([]);
  const [undoStack, setUndoStack] = useState<DrawingItem[][]>([]);
  const [redoStack, setRedoStack] = useState<DrawingItem[][]>([]);
  const [selectedDrawingId, setSelectedDrawingId] = useState<string | null>(null);
  const [draggingAnchor, setDraggingAnchor] = useState<{ id: string; point: "p1" | "p2" } | null>(null);
  const [currentDrawing, setCurrentDrawing] = useState<{
    type: DrawingItem["type"];
    p1: DrawingPoint;
    p2?: DrawingPoint;
  } | null>(null);
  const isDrawingRef = useRef(false);

  // Load drawings from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem(`atlsd_drawings_${symbol}`);
      const items: DrawingItem[] = stored ? JSON.parse(stored) : [];
      setDrawings(items);
      setUndoStack([]);
      setRedoStack([]);
      onDrawingsCountChange?.(items.length);
      onCanUndoRedoChange?.(false, false);
    } catch {
      setDrawings([]);
      onDrawingsCountChange?.(0);
      onCanUndoRedoChange?.(false, false);
    }
  }, [symbol, onDrawingsCountChange, onCanUndoRedoChange]);

  const saveDrawings = useCallback(
    (newDrawings: DrawingItem[], recordUndo = true) => {
      if (recordUndo) {
        setUndoStack((prev) => [...prev, drawings]);
        setRedoStack([]);
        onCanUndoRedoChange?.(true, false);
      }
      setDrawings(newDrawings);
      onDrawingsCountChange?.(newDrawings.length);
      try {
        localStorage.setItem(`atlsd_drawings_${symbol}`, JSON.stringify(newDrawings));
      } catch {
        /* ignore */
      }
    },
    [drawings, symbol, onDrawingsCountChange, onCanUndoRedoChange]
  );

  // Undo / Redo triggers & handlers
  const handleUndo = useCallback(() => {
    if (undoStack.length === 0) return;
    const previous = undoStack[undoStack.length - 1];
    setRedoStack((prev) => [...prev, drawings]);
    setUndoStack((prev) => prev.slice(0, prev.length - 1));
    setDrawings(previous);
    setSelectedDrawingId(null);
    onDrawingsCountChange?.(previous.length);
    onCanUndoRedoChange?.(undoStack.length > 1, true);
    try {
      localStorage.setItem(`atlsd_drawings_${symbol}`, JSON.stringify(previous));
    } catch {
      /* ignore */
    }
  }, [undoStack, drawings, symbol, onDrawingsCountChange, onCanUndoRedoChange]);

  const handleRedo = useCallback(() => {
    if (redoStack.length === 0) return;
    const next = redoStack[redoStack.length - 1];
    setUndoStack((prev) => [...prev, drawings]);
    setRedoStack((prev) => prev.slice(0, prev.length - 1));
    setDrawings(next);
    setSelectedDrawingId(null);
    onDrawingsCountChange?.(next.length);
    onCanUndoRedoChange?.(true, redoStack.length > 1);
    try {
      localStorage.setItem(`atlsd_drawings_${symbol}`, JSON.stringify(next));
    } catch {
      /* ignore */
    }
  }, [redoStack, drawings, symbol, onDrawingsCountChange, onCanUndoRedoChange]);

  const lastHandledUndoRef = useRef(undoTrigger);
  const lastHandledRedoRef = useRef(redoTrigger);
  const lastHandledClearRef = useRef(clearDrawingsTrigger);
  const lastHandledSnapshotRef = useRef(snapshotTrigger);
  const symbolRef = useRef(symbol);
  symbolRef.current = symbol;
  const timeframeRef = useRef(timeframe);
  timeframeRef.current = timeframe;

  useEffect(() => {
    if (undoTrigger > 0 && undoTrigger !== lastHandledUndoRef.current) {
      lastHandledUndoRef.current = undoTrigger;
      handleUndo();
    } else if (undoTrigger === 0) {
      lastHandledUndoRef.current = 0;
    }
  }, [undoTrigger, handleUndo]);

  useEffect(() => {
    if (redoTrigger > 0 && redoTrigger !== lastHandledRedoRef.current) {
      lastHandledRedoRef.current = redoTrigger;
      handleRedo();
    } else if (redoTrigger === 0) {
      lastHandledRedoRef.current = 0;
    }
  }, [redoTrigger, handleRedo]);

  // Clear drawings trigger
  useEffect(() => {
    if (clearDrawingsTrigger > 0 && clearDrawingsTrigger !== lastHandledClearRef.current) {
      lastHandledClearRef.current = clearDrawingsTrigger;
      if (drawings.length > 0) {
        saveDrawings([], true);
      }
      setSelectedDrawingId(null);
      localStorage.removeItem(`atlsd_drawings_${symbolRef.current}`);
    } else if (clearDrawingsTrigger === 0) {
      lastHandledClearRef.current = 0;
    }
  }, [clearDrawingsTrigger, drawings.length, saveDrawings]);

  // Project chart (time, price, logical) -> current viewport (x, y) pixels
  const projectPoint = useCallback(
    (p: DrawingPoint): { x: number; y: number } => {
      const chart = chartRef.current;
      const series = seriesRef.current;
      let x = p.x;
      let y = p.y;

      if (chart && series) {
        if (p.logical !== undefined) {
          const cx = chart.timeScale().logicalToCoordinate(p.logical as any);
          if (cx !== null && !isNaN(cx)) x = cx;
        } else if (p.time !== undefined) {
          const cx = chart.timeScale().timeToCoordinate(p.time as Time);
          if (cx !== null && !isNaN(cx)) x = cx;
        }

        if (p.price !== undefined) {
          const cy = series.priceToCoordinate(p.price);
          if (cy !== null && !isNaN(cy)) y = cy;
        }
      }

      return { x, y };
    },
    []
  );

  // Snapshot trigger effect
  useEffect(() => {
    if (
      snapshotTrigger > 0 &&
      snapshotTrigger !== lastHandledSnapshotRef.current &&
      chartRef.current
    ) {
      lastHandledSnapshotRef.current = snapshotTrigger;
      const currentSymbol = symbolRef.current;
      const currentTimeframe = timeframeRef.current;

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
          ctx.fillText("PIA TERMINAL", 20, canvas.height - 16);

          // Meta watermark
          ctx.font = "12px monospace";
          ctx.fillStyle = "#d1d4dc";
          ctx.fillText(
            `${currentSymbol} · ${currentTimeframe} · ${new Date().toISOString().replace("T", " ").substring(0, 19)} UTC`,
            150,
            canvas.height - 16
          );

          const url = watermarked.toDataURL("image/png");
          const a = document.createElement("a");
          a.href = url;
          a.download = `PIA_${currentSymbol}_${currentTimeframe}_${Date.now()}.png`;
          a.click();
        }
      } catch (err) {
        console.warn("[ChartArea] Snapshot export failed:", err);
      }

      onSnapshotDone?.();
    } else if (snapshotTrigger === 0) {
      lastHandledSnapshotRef.current = 0;
    }
  }, [snapshotTrigger, onSnapshotDone]);

  // Delete selected drawing on Backspace / Delete or Ctrl+Z / Ctrl+Y
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Avoid firing when typing inside an input or modal
      const target = e.target as HTMLElement | null;
      if (target?.tagName === "INPUT" || target?.tagName === "TEXTAREA") return;

      if ((e.key === "Delete" || e.key === "Backspace") && selectedDrawingId) {
        saveDrawings(drawings.filter((d) => d.id !== selectedDrawingId));
        setSelectedDrawingId(null);
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
        e.preventDefault();
        if (e.shiftKey) handleRedo();
        else handleUndo();
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
        e.preventDefault();
        handleRedo();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedDrawingId, drawings, saveDrawings, handleUndo, handleRedo]);

  // Initialize Lightweight Charts
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const isLight = (settings?.theme || theme) === "light";
    const chart = createChart(chartContainerRef.current, {
      width: chartContainerRef.current.clientWidth,
      height: chartContainerRef.current.clientHeight,
      layout: {
        background: { color: isLight ? "#ffffff" : "#131722" },
        textColor: isLight ? "#131722" : "#787b86",
        fontFamily: "-apple-system, BlinkMacSystemFont, 'Trebuchet MS', Roboto, sans-serif",
        attributionLogo: false,
      },
      grid: {
        vertLines: { color: settings?.gridVisible !== false ? (isLight ? "#f0f3fa" : "#1f2431") : "transparent" },
        horzLines: { color: settings?.gridVisible !== false ? (isLight ? "#f0f3fa" : "#1f2431") : "transparent" },
      },
      crosshair: {
        mode: 1, // Normal Crosshair
        vertLine: { color: isLight ? "#b2b5be" : "#787b86", width: 1, style: 3, labelBackgroundColor: isLight ? "#f0f3fa" : "#2a2e39" },
        horzLine: { color: isLight ? "#b2b5be" : "#787b86", width: 1, style: 3, labelBackgroundColor: isLight ? "#f0f3fa" : "#2a2e39" },
      },
      rightPriceScale: {
        borderColor: isLight ? "#e0e3eb" : "#2a2e39",
        visible: true,
        scaleMargins: { top: 0.1, bottom: 0.2 },
      },
      timeScale: {
        borderColor: isLight ? "#e0e3eb" : "#2a2e39",
        timeVisible: true,
        secondsVisible: false,
      },
      handleScale: {
        mouseWheel: true,
        pinch: true,
        axisPressedMouseMove: true,
      },
      handleScroll: {
        mouseWheel: true,
        pressedMouseMove: true,
        horzTouchDrag: true,
        vertTouchDrag: true,
      },
    });

    const candlestickSeries = chart.addCandlestickSeries({
      upColor: settings?.upColor || "#089981",
      downColor: settings?.downColor || "#f23645",
      borderVisible: false,
      wickUpColor: settings?.upColor || "#089981",
      wickDownColor: settings?.downColor || "#f23645",
    });
    const lineSeries = chart.addLineSeries({ color: isLight ? "#2962ff" : "#d1d4dc", lineWidth: 2, priceLineVisible: false });
    const areaSeries = chart.addAreaSeries({
      lineColor: "#2962ff",
      topColor: "#2962ff55",
      bottomColor: "#2962ff05",
      lineWidth: 2,
      priceLineVisible: false,
    });
    const barSeries = chart.addBarSeries({
      upColor: settings?.upColor || "#089981",
      downColor: settings?.downColor || "#f23645",
      openVisible: true,
      thinBars: false,
    });

    // Volume Histogram (TradingView signature bottom 20% overlay)
    const volumeSeries = chart.addHistogramSeries({
      priceFormat: { type: "volume" },
      priceScaleId: "", // Overlay on separate internal scale
    });
    volumeSeries.priceScale().applyOptions({
      scaleMargins: {
        top: 0.8, // Sits strictly in bottom 20%
        bottom: 0,
      },
    });

    // Indicators
    const smaSeries = chart.addLineSeries({
      color: "#f5b942",
      lineWidth: 2,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    const emaSeries = chart.addLineSeries({
      color: "#2962ff",
      lineWidth: 2,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    const bbUpperSeries = chart.addLineSeries({
      color: "rgba(8, 153, 129, 0.7)",
      lineWidth: 1,
      lineStyle: 2,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    const bbBasisSeries = chart.addLineSeries({
      color: "rgba(245, 185, 66, 0.6)",
      lineWidth: 1,
      lineStyle: 0,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    const bbLowerSeries = chart.addLineSeries({
      color: "rgba(242, 54, 69, 0.7)",
      lineWidth: 1,
      lineStyle: 2,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    const vwapSeries = chart.addLineSeries({
      color: "#a855f7",
      lineWidth: 2,
      priceLineVisible: false,
      lastValueVisible: false,
    });

    chartRef.current = chart;
    seriesRef.current = candlestickSeries;
    lineRef.current = lineSeries;
    areaRef.current = areaSeries;
    barRef.current = barSeries;
    volumeRef.current = volumeSeries;
    smaRef.current = smaSeries;
    emaRef.current = emaSeries;
    bbUpperRef.current = bbUpperSeries;
    bbBasisRef.current = bbBasisSeries;
    bbLowerRef.current = bbLowerSeries;
    vwapRef.current = vwapSeries;

    // Crosshair hover synchronization
    chart.subscribeCrosshairMove((param) => {
      if (!param || !param.point || param.time === undefined) {
        hoveringRef.current = false;
        if (paneId) {
          window.dispatchEvent(
            new CustomEvent("terminal-crosshair-sync", {
              detail: { sourceId: paneId, clear: true },
            })
          );
        }
        return;
      }
      hoveringRef.current = true;
      const data = param.seriesData?.get(candlestickSeries) as CandlestickData | undefined;
      if (data) {
        const ch = data.close - data.open;
        const chp = data.open ? (ch / data.open) * 100 : 0;
        setOhlc({
          open: data.open,
          high: data.high,
          low: data.low,
          close: data.close,
          change: ch,
          changePercent: chp,
        });
      }

      if (paneId) {
        window.dispatchEvent(
          new CustomEvent("terminal-crosshair-sync", {
            detail: {
              sourceId: paneId,
              time: param.time,
            },
          })
        );
      }
    });

    // Continuously re-project SVG drawing coordinates whenever user scrolls or zooms
    const handleRangeChange = () => {
      setProjectionTick((t) => t + 1);
    };
    chart.timeScale().subscribeVisibleLogicalRangeChange(handleRangeChange);

    const resizeObserver = new ResizeObserver((entries) => {
      if (!entries || entries.length === 0) return;
      const { width, height } = entries[0].contentRect;
      chart.applyOptions({ width, height });
      setProjectionTick((t) => t + 1);
    });
    resizeObserver.observe(chartContainerRef.current);

    return () => {
      chart.timeScale().unsubscribeVisibleLogicalRangeChange(handleRangeChange);
      resizeObserver.disconnect();
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
      lineRef.current = null;
      areaRef.current = null;
      barRef.current = null;
      volumeRef.current = null;
      smaRef.current = null;
      emaRef.current = null;
      bbUpperRef.current = null;
      bbBasisRef.current = null;
      bbLowerRef.current = null;
      vwapRef.current = null;
    };
  }, []);

  // Apply live theme & styling updates
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;
    const isLight = (settings?.theme || theme) === "light";
    chart.applyOptions({
      layout: {
        background: { color: isLight ? "#ffffff" : "#131722" },
        textColor: isLight ? "#131722" : "#787b86",
      },
      grid: {
        vertLines: { color: settings?.gridVisible !== false ? (isLight ? "#f0f3fa" : "#1f2431") : "transparent" },
        horzLines: { color: settings?.gridVisible !== false ? (isLight ? "#f0f3fa" : "#1f2431") : "transparent" },
      },
      rightPriceScale: {
        borderColor: isLight ? "#e0e3eb" : "#2a2e39",
      },
      timeScale: {
        borderColor: isLight ? "#e0e3eb" : "#2a2e39",
      },
      crosshair: {
        vertLine: { color: isLight ? "#b2b5be" : "#787b86", labelBackgroundColor: isLight ? "#f0f3fa" : "#2a2e39" },
        horzLine: { color: isLight ? "#b2b5be" : "#787b86", labelBackgroundColor: isLight ? "#f0f3fa" : "#2a2e39" },
      },
    });

    seriesRef.current?.applyOptions({
      upColor: settings?.upColor || "#089981",
      downColor: settings?.downColor || "#f23645",
      wickUpColor: settings?.upColor || "#089981",
      wickDownColor: settings?.downColor || "#f23645",
    });
    barRef.current?.applyOptions({
      upColor: settings?.upColor || "#089981",
      downColor: settings?.downColor || "#f23645",
    });
    lineRef.current?.applyOptions({
      color: isLight ? "#2962ff" : "#d1d4dc",
    });
  }, [settings, theme]);

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

  // Multi-chart Crosshair Synchronization Listener
  useEffect(() => {
    if (settings?.syncCrosshair === false || !paneId) return;

    const handleCrosshairSync = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (!detail || detail.sourceId === paneId || !chartRef.current || !seriesRef.current) return;

      if (detail.clear) {
        chartRef.current.clearCrosshairPosition();
      } else if (detail.time !== undefined) {
        const hit = candles.find((c) => c.time === detail.time);
        const targetPrice = hit ? hit.close : (candles[candles.length - 1]?.close || 100);
        try {
          chartRef.current.setCrosshairPosition(targetPrice, detail.time as any, seriesRef.current);
        } catch {
          // ignore if out of range
        }
      }
    };

    window.addEventListener("terminal-crosshair-sync", handleCrosshairSync);
    return () => window.removeEventListener("terminal-crosshair-sync", handleCrosshairSync);
  }, [paneId, settings?.syncCrosshair, candles]);

  // Multi-chart Time Scale / Zoom Range Synchronization Listener
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart || settings?.syncTime === false || !paneId) return;

    let isSyncing = false;

    const handleRangeSync = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (!detail || detail.sourceId === paneId || isSyncing) return;

      isSyncing = true;
      try {
        chart.timeScale().setVisibleLogicalRange({ from: detail.from, to: detail.to });
      } catch {
        // ignore
      } finally {
        setTimeout(() => {
          isSyncing = false;
        }, 50);
      }
    };

    window.addEventListener("terminal-time-sync", handleRangeSync);

    const onRangeChange = (range: { from: number; to: number } | null) => {
      if (!range || isSyncing || settings?.syncTime === false) return;
      window.dispatchEvent(
        new CustomEvent("terminal-time-sync", {
          detail: { sourceId: paneId, from: range.from, to: range.to },
        })
      );
    };

    chart.timeScale().subscribeVisibleLogicalRangeChange(onRangeChange);

    return () => {
      window.removeEventListener("terminal-time-sync", handleRangeSync);
      chart.timeScale().unsubscribeVisibleLogicalRangeChange(onRangeChange);
    };
  }, [paneId, settings?.syncTime]);

  // Update chart data, volume, and technical overlays
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
    const symbolChanged =
      previousSymbolRef.current !== symbol || previousTimeframeRef.current !== timeframe;
    const rangeBeforeUpdate =
      chart && hasInitializedDataRef.current ? chart.timeScale().getVisibleLogicalRange() : null;
    const firstTime = candles[0]?.time ?? null;
    const prependedCount =
      !symbolChanged && previousFirstTimeRef.current !== null && firstTime !== null && firstTime < previousFirstTimeRef.current
        ? Math.max(0, candles.length - previousCandleCountRef.current)
        : 0;

    const lineData: LineData<Time>[] = candles.map((c) => ({ time: c.time as Time, value: c.close }));
    const heikinData: CandlestickData<Time>[] = candles.reduce<CandlestickData<Time>[]>(
      (result, c, index) => {
        const close = (c.open + c.high + c.low + c.close) / 4;
        const open =
          index === 0
            ? (c.open + c.close) / 2
            : ((result[index - 1].open as number) + (result[index - 1].close as number)) / 2;
        result.push({
          time: c.time as Time,
          open,
          high: Math.max(c.high, open, close),
          low: Math.min(c.low, open, close),
          close,
        });
        return result;
      },
      []
    );

    seriesRef.current.setData(
      chartType === "heikin_ashi" ? heikinData : chartType === "candlestick" ? data : []
    );
    barRef.current?.setData(chartType === "bar" ? data : []);
    lineRef.current?.setData(chartType === "line" ? lineData : []);
    areaRef.current?.setData(chartType === "area" ? lineData : []);

    // Set Volume Bars
    const volumeData: HistogramData<Time>[] = candles.map((c) => ({
      time: c.time as Time,
      value: c.volume ?? 0,
      color: c.close >= c.open ? "rgba(8, 153, 129, 0.45)" : "rgba(242, 54, 69, 0.45)",
    }));
    volumeRef.current?.setData(volumeData);

    // Moving Averages
    const movingAverage = (period: number, exponential: boolean): LineData<Time>[] => {
      const output: LineData<Time>[] = [];
      let previous: number | null = null;
      candles.forEach((c, index) => {
        if (index + 1 < period) return;
        if (exponential) {
          const alpha = 2 / (period + 1);
          previous =
            previous == null
              ? candles.slice(index + 1 - period, index + 1).reduce((sum, row) => sum + row.close, 0) / period
              : c.close * alpha + previous * (1 - alpha);
        } else {
          previous =
            candles.slice(index + 1 - period, index + 1).reduce((sum, row) => sum + row.close, 0) / period;
        }
        output.push({ time: c.time as Time, value: previous });
      });
      return output;
    };

    const smaPeriod = settings?.indicatorParams?.smaPeriod || 20;
    const emaPeriod = settings?.indicatorParams?.emaPeriod || 50;
    const bbPeriod = settings?.indicatorParams?.bollingerPeriod || 20;
    const bbStd = settings?.indicatorParams?.bollingerStdDev || 2.0;

    smaRef.current?.setData(indicators.sma20 ? movingAverage(smaPeriod, false) : []);
    emaRef.current?.setData(indicators.ema50 ? movingAverage(emaPeriod, true) : []);

    // VWAP (Volume-Weighted Average Price, anchored to each UTC calendar day)
    const calculateVWAP = (): LineData<Time>[] => {
      const result: LineData<Time>[] = [];
      let cumVol = 0;
      let cumTypicalVol = 0;
      let lastDay = -1;

      for (let i = 0; i < candles.length; i++) {
        const c = candles[i];
        const date = new Date(c.time * 1000);
        const day = date.getUTCDate();

        if (day !== lastDay) {
          cumVol = 0;
          cumTypicalVol = 0;
          lastDay = day;
        }

        const vol = typeof c.volume === "number" && c.volume > 0 ? c.volume : 1;
        const typicalPrice = (c.high + c.low + c.close) / 3;
        cumVol += vol;
        cumTypicalVol += typicalPrice * vol;

        const vwapVal = cumVol > 0 ? cumTypicalVol / cumVol : c.close;
        result.push({ time: c.time as Time, value: vwapVal });
      }

      return result;
    };

    vwapRef.current?.setData(indicators.vwap ? calculateVWAP() : []);

    // Bollinger Bands (period, mult)
    if (indicators.bollinger && candles.length >= bbPeriod) {
      const upper: LineData<Time>[] = [];
      const basis: LineData<Time>[] = [];
      const lower: LineData<Time>[] = [];
      const period = bbPeriod;
      const mult = bbStd;

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
      hasInitializedDataRef.current = candles.length > 0;
    } else if (prependedCount > 0 && rangeBeforeUpdate && chart) {
      chart.timeScale().setVisibleLogicalRange({
        from: rangeBeforeUpdate.from + prependedCount,
        to: rangeBeforeUpdate.to + prependedCount,
      });
    }

    previousSymbolRef.current = symbol;
    previousTimeframeRef.current = timeframe;
    previousCandleCountRef.current = candles.length;
    previousFirstTimeRef.current = firstTime;

    if (!hoveringRef.current && candles.length > 0) {
      const last = candles[candles.length - 1];
      const ch = last.close - last.open;
      const chp = last.open ? (ch / last.open) * 100 : 0;
      setOhlc({ open: last.open, high: last.high, low: last.low, close: last.close, change: ch, changePercent: chp });
    }
  }, [candles, chartType, indicators, symbol, timeframe, settings?.indicatorParams]);

  // Live price streaming update
  useEffect(() => {
    if (!seriesRef.current || !livePrice || candles.length === 0) return;
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
      setOhlc({
        open: last.open,
        high: Math.max(last.high, livePrice),
        low: Math.min(last.low, livePrice),
        close: livePrice,
        change: ch,
        changePercent: chp,
      });
    }
  }, [livePrice, candles]);

  // Interactive Drawing Handlers (Bound dynamically to Time & Price)
  const handleMouseDown = (e: React.MouseEvent<SVGSVGElement>) => {
    if (draggingAnchor) return;
    if (activeTool === "cursor") {
      setSelectedDrawingId(null);
      return;
    }

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const chart = chartRef.current;
    const series = seriesRef.current;
    const time = (chart?.timeScale().coordinateToTime(x) as number) ?? undefined;
    const logical = (chart?.timeScale().coordinateToLogical(x) as number) ?? undefined;
    const price = (series?.coordinateToPrice(y) as number) ?? undefined;

    const pt: DrawingPoint = { x, y, time, logical, price };

    isDrawingRef.current = true;
    setCurrentDrawing({
      type: activeTool,
      p1: pt,
      p2: pt,
    });
  };

  const handleMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const chart = chartRef.current;
    const series = seriesRef.current;
    const time = (chart?.timeScale().coordinateToTime(x) as number) ?? undefined;
    const logical = (chart?.timeScale().coordinateToLogical(x) as number) ?? undefined;
    const price = (series?.coordinateToPrice(y) as number) ?? undefined;

    const pt: DrawingPoint = { x, y, time, logical, price };

    if (draggingAnchor) {
      setDrawings((prev) =>
        prev.map((d) => {
          if (d.id !== draggingAnchor.id) return d;
          if (draggingAnchor.point === "p1") {
            return { ...d, p1: pt };
          } else {
            return { ...d, p2: pt };
          }
        })
      );
      return;
    }

    if (!isDrawingRef.current || !currentDrawing) return;

    setCurrentDrawing({
      ...currentDrawing,
      p2: pt,
    });
  };

  const handleMouseUp = () => {
    if (draggingAnchor) {
      setDraggingAnchor(null);
      saveDrawings(drawings, true);
      return;
    }

    if (!isDrawingRef.current || !currentDrawing) return;
    isDrawingRef.current = false;

    if (currentDrawing.p2) {
      const dx = Math.abs(currentDrawing.p1.x - currentDrawing.p2.x);
      const dy = Math.abs(currentDrawing.p1.y - currentDrawing.p2.y);
      if (dx > 4 || dy > 4 || currentDrawing.type === "horizontal") {
        const item: DrawingItem = {
          id: "draw_" + Date.now(),
          type: currentDrawing.type,
          symbol,
          p1: currentDrawing.p1,
          p2: currentDrawing.p2,
          color: currentDrawing.type === "horizontal" ? "#f5b942" : "#2962ff",
          strokeWidth: 2,
        };
        saveDrawings([...drawings, item], true);
        setSelectedDrawingId(item.id);

        // Auto-revert to cursor tool unless locked (TradingView principle)
        if (!isDrawingModeLocked) {
          onDrawingFinished?.();
        }
      }
    }
    setCurrentDrawing(null);
  };

  const handleUpdateDrawingStyle = (updates: Partial<DrawingItem>) => {
    if (!selectedDrawingId) return;
    const updated = drawings.map((d) => (d.id === selectedDrawingId ? { ...d, ...updates } : d));
    saveDrawings(updated, true);
  };

  // Price Scale Mode Handlers
  const handleToggleScale = (mode: "normal" | "log" | "percent") => {
    const chart = chartRef.current;
    if (!chart) return;

    if (mode === "normal") {
      chart.priceScale("right").applyOptions({ mode: PriceScaleMode.Normal });
      setScaleMode("normal");
    } else if (mode === "log") {
      const next = scaleMode === "log" ? "normal" : "log";
      chart.priceScale("right").applyOptions({
        mode: next === "log" ? PriceScaleMode.Logarithmic : PriceScaleMode.Normal,
      });
      setScaleMode(next);
    } else if (mode === "percent") {
      const next = scaleMode === "percent" ? "normal" : "percent";
      chart.priceScale("right").applyOptions({
        mode: next === "percent" ? PriceScaleMode.Percentage : PriceScaleMode.Normal,
      });
      setScaleMode(next);
    }
  };

  // Quick Range Selector Handlers
  const handleQuickRange = (range: "1D" | "5D" | "1M" | "3M" | "6M" | "1Y" | "ALL") => {
    const chart = chartRef.current;
    if (!chart || candles.length === 0) return;

    if (range === "ALL") {
      chart.timeScale().fitContent();
      return;
    }

    const lastTime = candles[candles.length - 1].time;
    let seconds = 86400; // 1D
    if (range === "5D") seconds = 5 * 86400;
    else if (range === "1M") seconds = 30 * 86400;
    else if (range === "3M") seconds = 90 * 86400;
    else if (range === "6M") seconds = 180 * 86400;
    else if (range === "1Y") seconds = 365 * 86400;

    const fromTime = (lastTime - seconds) as Time;
    chart.timeScale().setVisibleRange({
      from: fromTime,
      to: lastTime as Time,
    });
  };

  const selectedDrawing = drawings.find((d) => d.id === selectedDrawingId);
  const actionPos = useMemo(() => {
    if (!selectedDrawing) return null;
    const p1 = projectPoint(selectedDrawing.p1);
    const p2 = selectedDrawing.p2 ? projectPoint(selectedDrawing.p2) : p1;
    const top = Math.max(12, Math.min(p1.y, p2.y) - 44);
    const left = Math.max(12, (p1.x + p2.x) / 2 - 80);
    return { top, left };
  }, [selectedDrawing, projectPoint]);

  const isUp = ohlc.close >= ohlc.open;
  const hasData = candles.length > 0;

  const isLight = (settings?.theme || theme) === "light";

  return (
    <div
      className={`relative w-full h-full flex flex-col ${
        isLight ? "bg-white text-[#131722]" : "bg-[#131722] text-[#d1d4dc]"
      } overflow-hidden select-none`}
    >
      {/* Chart Legend Overlay */}
      <div className="absolute top-3 left-3 z-10 flex flex-col gap-1.5 pointer-events-none">
        <div className="flex flex-wrap items-center gap-1.5 pointer-events-auto">
          <span className={`font-bold text-sm tracking-wide ${isLight ? "text-[#131722]" : "text-white"}`}>
            {symbol}
          </span>
          <span className="text-xs text-[#787b86] font-medium">{timeframe}</span>
          <span
            className={`text-[10px] text-[#787b86] font-mono border px-1.5 py-0.5 rounded ${
              isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}
          >
            {provider}
          </span>
          <span
            className={`flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded ${
              connected ? "bg-[#089981]/15 text-[#089981]" : "bg-[#787b86]/15 text-[#787b86]"
            }`}
          >
            {connected ? <Wifi className="w-2.5 h-2.5" /> : <WifiOff className="w-2.5 h-2.5" />}
            {connected ? "LIVE" : "DISCONNECTED"}
          </span>

          {/* Quick Indicator Pills */}
          {indicators.sma20 && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#f5b942] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>SMA {settings?.indicatorParams?.smaPeriod || 20}</span>
              <button
                onClick={() => onToggleIndicator?.("sma20")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove SMA"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
          {indicators.ema50 && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#2962ff] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>EMA {settings?.indicatorParams?.emaPeriod || 50}</span>
              <button
                onClick={() => onToggleIndicator?.("ema50")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove EMA"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
          {indicators.vwap && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#a855f7] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>VWAP</span>
              <button
                onClick={() => onToggleIndicator?.("vwap")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove VWAP"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
          {indicators.bollinger && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#089981] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>BB({settings?.indicatorParams?.bollingerPeriod || 20},{settings?.indicatorParams?.bollingerStdDev || 2})</span>
              <button
                onClick={() => onToggleIndicator?.("bollinger")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove Bollinger Bands"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
          {indicators.rsi && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#ab47bc] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>RSI {settings?.indicatorParams?.rsiPeriod || 14}</span>
              <button
                onClick={() => onToggleIndicator?.("rsi")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove RSI"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
          {indicators.atr && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#f59e0b] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>ATR {settings?.indicatorParams?.atrPeriod || 14}</span>
              <button
                onClick={() => onToggleIndicator?.("atr")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove ATR"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
          {indicators.macd && (
            <div className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#2962ff] ${
              isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}>
              <span>MACD</span>
              <button
                onClick={() => onToggleIndicator?.("macd")}
                className="opacity-0 group-hover:opacity-100 hover:text-white cursor-pointer"
                title="Remove MACD"
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </div>
          )}
        </div>

        {/* Quick Order Placement Floating Widget (TradingView Signature) */}
        <div className="flex items-center gap-1.5 my-0.5 pointer-events-auto select-none">
          {/* Sell Box */}
          <button
            className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-[#f23645]/15 border border-[#f23645]/40 text-[#f23645] font-mono text-[10px] font-bold cursor-pointer hover:bg-[#f23645]/25 transition-all shadow-xs"
            title="Instant Quick Sell"
          >
            <span className="w-1.5 h-1.5 rounded-full bg-[#f23645]" />
            <span>{(livePrice ? livePrice * 0.9998 : ohlc.close * 0.9998).toFixed(digits)}</span>
            <span className="uppercase text-[8px] tracking-wider">SELL</span>
          </button>

          {/* Spread Pill */}
          <div className={`px-1.5 py-0.5 rounded text-[9px] font-mono border ${isLight ? "bg-white border-[#e0e3eb] text-[#787b86]" : "bg-[#1e222d] border-[#2a2e39] text-[#787b86]"}`}>
            <span>{(digits === 3 || digits === 2 ? "0.4" : "1.2")}</span>
          </div>

          {/* Buy Box */}
          <button
            className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-[#2962ff]/15 border border-[#2962ff]/40 text-[#2962ff] font-mono text-[10px] font-bold cursor-pointer hover:bg-[#2962ff]/25 transition-all shadow-xs"
            title="Instant Quick Buy"
          >
            <span className="w-1.5 h-1.5 rounded-full bg-[#2962ff]" />
            <span>{(livePrice ? livePrice * 1.0002 : ohlc.close * 1.0002).toFixed(digits)}</span>
            <span className="uppercase text-[8px] tracking-wider">BUY</span>
          </button>
        </div>

        {/* OHLC Bar Metrics */}
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
              {isUp ? "+" : ""}
              {ohlc.change.toFixed(digits)} ({isUp ? "+" : ""}
              {ohlc.changePercent.toFixed(2)}%)
            </span>
          </div>
        </div>
      </div>

      {/* Loading Overlays */}
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

      {/* Oscillator Sub-pane (RSI / MACD / ATR) */}
      <OscillatorPane candles={candles} indicators={indicators} settings={settings} theme={theme} />

      {/* Quick Timeframe Range Bar & Scale Mode Controls (TradingView Signature) */}
      <div
        className={`h-7 border-t flex items-center justify-between px-2 text-[11px] font-mono select-none shrink-0 z-20 ${
          isLight ? "bg-[#f0f3fa] border-[#e0e3eb] text-[#5d606b]" : "bg-[#1e222d] border-[#2a2e39] text-[#787b86]"
        }`}
      >
        {/* Left: Quick Range Fit */}
        <div className="flex items-center gap-1 text-[#787b86]">
          {(["1D", "5D", "1M", "3M", "6M", "1Y", "ALL"] as const).map((rng) => (
            <button
              key={rng}
              onClick={() => handleQuickRange(rng)}
              className="px-1.5 py-0.5 rounded hover:text-white hover:bg-[#2a2e39] transition-colors cursor-pointer"
            >
              {rng}
            </button>
          ))}
        </div>

        {/* Right: Auto, Log, % Scale Controls */}
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => handleToggleScale("normal")}
            className="px-1.5 py-0.5 rounded text-[#787b86] hover:text-white hover:bg-[#2a2e39] font-bold cursor-pointer"
            title="Auto Fit Scale"
          >
            auto
          </button>
          <button
            onClick={() => handleToggleScale("log")}
            className={`px-1.5 py-0.5 rounded transition-colors cursor-pointer ${
              scaleMode === "log" ? "bg-[#2962ff] text-white font-bold" : "text-[#787b86] hover:text-white"
            }`}
            title="Logarithmic Scale"
          >
            log
          </button>
          <button
            onClick={() => handleToggleScale("percent")}
            className={`px-1.5 py-0.5 rounded transition-colors cursor-pointer ${
              scaleMode === "percent" ? "bg-[#2962ff] text-white font-bold" : "text-[#787b86] hover:text-white"
            }`}
            title="Percentage Scale"
          >
            %
          </button>
        </div>
      </div>

      {/* Floating Action Pill for Selected Drawing */}
      {selectedDrawing && actionPos && !isDrawingsHidden && (
        <div
          style={{ top: `${actionPos.top}px`, left: `${actionPos.left}px` }}
          className="absolute z-30 flex items-center gap-2 rounded-lg border border-[#2a2e39] bg-[#1e222d] px-2.5 py-1.5 shadow-2xl"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Colors */}
          <div className="flex items-center gap-1 pr-1 border-r border-[#2a2e39]">
            {["#2962ff", "#f5b942", "#089981", "#f23645", "#ffffff"].map((col) => (
              <button
                key={col}
                onClick={() => handleUpdateDrawingStyle({ color: col })}
                style={{ backgroundColor: col }}
                className={`w-3.5 h-3.5 rounded-full border border-black/40 transition-transform cursor-pointer ${
                  selectedDrawing.color === col ? "scale-125 ring-1 ring-white" : "hover:scale-110"
                }`}
              />
            ))}
          </div>

          {/* Width */}
          <div className="flex items-center gap-1 pr-1 border-r border-[#2a2e39]">
            {[1, 2, 3].map((w) => (
              <button
                key={w}
                onClick={() => handleUpdateDrawingStyle({ strokeWidth: w })}
                className={`px-1.5 py-0.5 rounded font-mono text-[9px] font-bold cursor-pointer ${
                  (selectedDrawing.strokeWidth || 2) === w
                    ? "bg-[#2962ff] text-white"
                    : "text-[#787b86] hover:text-white"
                }`}
              >
                {w}px
              </button>
            ))}
          </div>

          {/* Delete */}
          <button
            onClick={() => {
              saveDrawings(drawings.filter((d) => d.id !== selectedDrawing.id), true);
              setSelectedDrawingId(null);
            }}
            className="p-1 rounded text-[#787b86] hover:text-[#f23645] hover:bg-[#2a2e39] cursor-pointer"
            title="Delete Drawing (Del)"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Interactive SVG Drawing Overlay */}
      <svg
        className={`absolute inset-0 w-full h-full z-15 ${
          isDrawingsHidden
            ? "pointer-events-none opacity-0"
            : activeTool === "cursor"
            ? "pointer-events-none"
            : "pointer-events-auto cursor-crosshair"
        }`}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
      >
        {/* Render Saved Drawings dynamically projected to current candles */}
        {drawings.map((d) => {
          const isSelected = d.id === selectedDrawingId;
          const strokeColor = d.color || (d.type === "horizontal" ? "#f5b942" : "#2962ff");
          const width = d.strokeWidth || 2;
          const pt1 = projectPoint(d.p1);
          const pt2 = d.p2 ? projectPoint(d.p2) : pt1;

          if (d.type === "trendline" && d.p2) {
            return (
              <g
                key={d.id}
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedDrawingId(d.id);
                }}
                className="pointer-events-auto cursor-pointer"
              >
                <line
                  x1={pt1.x}
                  y1={pt1.y}
                  x2={pt2.x}
                  y2={pt2.y}
                  stroke="transparent"
                  strokeWidth="16"
                  className="cursor-pointer"
                />
                <line
                  x1={pt1.x}
                  y1={pt1.y}
                  x2={pt2.x}
                  y2={pt2.y}
                  stroke={strokeColor}
                  strokeWidth={width}
                  strokeLinecap="round"
                />
                {isSelected ? (
                  <>
                    <circle
                      cx={pt1.x}
                      cy={pt1.y}
                      r="5.5"
                      fill="#ffffff"
                      stroke="#2962ff"
                      strokeWidth="2"
                      className="cursor-move pointer-events-auto"
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setDraggingAnchor({ id: d.id, point: "p1" });
                      }}
                    />
                    <circle
                      cx={pt2.x}
                      cy={pt2.y}
                      r="5.5"
                      fill="#ffffff"
                      stroke="#2962ff"
                      strokeWidth="2"
                      className="cursor-move pointer-events-auto"
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setDraggingAnchor({ id: d.id, point: "p2" });
                      }}
                    />
                  </>
                ) : (
                  <>
                    <circle cx={pt1.x} cy={pt1.y} r="3" fill={strokeColor} />
                    <circle cx={pt2.x} cy={pt2.y} r="3" fill={strokeColor} />
                  </>
                )}
              </g>
            );
          }

          if (d.type === "horizontal") {
            const price = d.p1.price;
            const containerWidth = chartContainerRef.current?.clientWidth || 800;
            return (
              <g
                key={d.id}
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedDrawingId(d.id);
                }}
                className="pointer-events-auto cursor-pointer"
              >
                <line
                  x1={0}
                  y1={pt1.y}
                  x2="100%"
                  y2={pt1.y}
                  stroke="transparent"
                  strokeWidth="16"
                  className="cursor-pointer"
                />
                <line
                  x1={0}
                  y1={pt1.y}
                  x2="100%"
                  y2={pt1.y}
                  stroke={strokeColor}
                  strokeWidth={width}
                  strokeDasharray="4 2"
                />
                {/* Horizontal price pill on right price axis */}
                {price !== undefined && (
                  <g transform={`translate(${containerWidth - 68}, ${pt1.y - 9})`}>
                    <rect width="64" height="18" rx="2" fill={strokeColor} />
                    <text
                      x="32"
                      y="13"
                      textAnchor="middle"
                      fill="#ffffff"
                      fontSize="10"
                      fontFamily="monospace"
                      fontWeight="bold"
                    >
                      {price.toFixed(digits)}
                    </text>
                  </g>
                )}
                {isSelected ? (
                  <circle
                    cx={pt1.x}
                    cy={pt1.y}
                    r="5.5"
                    fill="#ffffff"
                    stroke="#f5b942"
                    strokeWidth="2"
                    className="cursor-move pointer-events-auto"
                    onMouseDown={(e) => {
                      e.stopPropagation();
                      setDraggingAnchor({ id: d.id, point: "p1" });
                    }}
                  />
                ) : (
                  <circle cx={pt1.x} cy={pt1.y} r="3" fill={strokeColor} />
                )}
              </g>
            );
          }

          if (d.type === "fibonacci" && d.p2) {
            const price1 = d.p1.price ?? 0;
            const price2 = d.p2.price ?? 0;
            const priceDiff = price2 - price1;
            const levels = [
              { lvl: 0.0, color: strokeColor },
              { lvl: 0.236, color: "#90caf9" },
              { lvl: 0.382, color: "#ffe082" },
              { lvl: 0.5, color: "#81c784" },
              { lvl: 0.618, color: "#f5b942" },
              { lvl: 0.786, color: "#ff8a80" },
              { lvl: 1.0, color: strokeColor },
            ];

            const series = seriesRef.current;
            return (
              <g
                key={d.id}
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedDrawingId(d.id);
                }}
                className="pointer-events-auto cursor-pointer"
              >
                {/* Shaded bands */}
                {levels.map((item, idx) => {
                  if (idx === levels.length - 1) return null;
                  const lvlNext = levels[idx + 1];
                  const pA = price1 + priceDiff * item.lvl;
                  const pB = price1 + priceDiff * lvlNext.lvl;
                  const yA = series?.priceToCoordinate(pA) ?? pt1.y;
                  const yB = series?.priceToCoordinate(pB) ?? pt2.y;
                  const minY = Math.min(yA, yB);
                  const h = Math.abs(yB - yA);

                  // Golden pocket highlight between 0.5 and 0.618
                  const isGolden = item.lvl === 0.5 || item.lvl === 0.382;
                  return (
                    <rect
                      key={`band-${idx}`}
                      x={0}
                      y={minY}
                      width="100%"
                      height={Math.max(1, h)}
                      fill={isGolden ? "rgba(245, 185, 66, 0.12)" : "rgba(41, 98, 255, 0.06)"}
                    />
                  );
                })}

                {/* Level lines and labels */}
                {levels.map((item) => {
                  const lvlPrice = price1 + priceDiff * item.lvl;
                  const y = series?.priceToCoordinate(lvlPrice) ?? pt1.y;
                  return (
                    <g key={item.lvl}>
                      <line
                        x1={0}
                        y1={y}
                        x2="100%"
                        y2={y}
                        stroke={item.color}
                        strokeWidth={item.lvl === 0 || item.lvl === 1 ? 1.5 : 1}
                        strokeOpacity={0.85}
                      />
                      <text
                        x={12}
                        y={y - 3}
                        fill="#d1d4dc"
                        fontSize="10"
                        fontFamily="monospace"
                        fontWeight="600"
                      >
                        {item.lvl.toFixed(3)} ({lvlPrice.toFixed(digits)})
                      </text>
                    </g>
                  );
                })}

                {isSelected && (
                  <>
                    <circle
                      cx={pt1.x}
                      cy={pt1.y}
                      r="5.5"
                      fill="#ffffff"
                      stroke="#2962ff"
                      strokeWidth="2"
                      className="cursor-move pointer-events-auto"
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setDraggingAnchor({ id: d.id, point: "p1" });
                      }}
                    />
                    <circle
                      cx={pt2.x}
                      cy={pt2.y}
                      r="5.5"
                      fill="#ffffff"
                      stroke="#2962ff"
                      strokeWidth="2"
                      className="cursor-move pointer-events-auto"
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setDraggingAnchor({ id: d.id, point: "p2" });
                      }}
                    />
                  </>
                )}
              </g>
            );
          }

          if (d.type === "measure" && d.p2) {
            const xMin = Math.min(pt1.x, pt2.x);
            const yMin = Math.min(pt1.y, pt2.y);
            const w = Math.max(1, Math.abs(pt2.x - pt1.x));
            const h = Math.max(1, Math.abs(pt2.y - pt1.y));
            const price1 = d.p1.price ?? 0;
            const price2 = d.p2.price ?? 0;
            const deltaPrice = price2 - price1;
            const pct = price1 !== 0 ? (deltaPrice / price1) * 100 : 0;
            const isPositive = deltaPrice >= 0;
            const bars = Math.abs(Math.round((d.p2.logical ?? 0) - (d.p1.logical ?? 0)));

            return (
              <g
                key={d.id}
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedDrawingId(d.id);
                }}
                className="pointer-events-auto cursor-pointer"
              >
                <rect
                  x={xMin}
                  y={yMin}
                  width={w}
                  height={h}
                  fill={isPositive ? "rgba(8, 153, 129, 0.15)" : "rgba(242, 54, 69, 0.15)"}
                  stroke={isPositive ? "#089981" : "#f23645"}
                  strokeWidth="1"
                  strokeDasharray="3 3"
                />
                {/* Measurement badge */}
                <g transform={`translate(${xMin + w / 2 - 60}, ${yMin + h / 2 - 18})`}>
                  <rect
                    width="120"
                    height="36"
                    rx="4"
                    fill={isPositive ? "#089981" : "#f23645"}
                    opacity="0.95"
                  />
                  <text
                    x="60"
                    y="15"
                    textAnchor="middle"
                    fill="#ffffff"
                    fontSize="11"
                    fontFamily="monospace"
                    fontWeight="bold"
                  >
                    {isPositive ? "+" : ""}
                    {deltaPrice.toFixed(digits)} ({isPositive ? "+" : ""}
                    {pct.toFixed(2)}%)
                  </text>
                  <text
                    x="60"
                    y="28"
                    textAnchor="middle"
                    fill="#ffffff"
                    fontSize="9"
                    fontFamily="sans-serif"
                    opacity="0.9"
                  >
                    {bars} bars
                  </text>
                </g>
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
                fill="rgba(41, 98, 255, 0.15)"
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
                fill="rgba(41, 98, 255, 0.15)"
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
