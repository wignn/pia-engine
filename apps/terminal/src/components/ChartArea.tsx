"use client";

import React, { useEffect, useRef, useState, useCallback } from "react";
import type { Chart as KLineChartInstance, KLineData, Period, PeriodType } from "klinecharts";
import { CandleData, Timeframe, DrawingTool, IndicatorState } from "@/types";
import { Wifi, WifiOff, Loader2, X } from "lucide-react";
import type { XauusdForecast, XauusdForecastState } from "@/types/forecast";

interface ChartAreaProps {
  paneId?: string;
  symbol: string;
  provider: string;
  timeframe: Timeframe;
  chartType?: "candlestick" | "bar" | "line" | "area" | "heikin_ashi";
  indicators?: IndicatorState;
  forecast?: XauusdForecast | null;
  forecastState?: XauusdForecastState;
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

const mapTimeframeToPeriod = (tf: Timeframe): Period => {
  switch (tf) {
    case "1m":
      return { type: "minute" as PeriodType, span: 1 };
    case "5m":
      return { type: "minute" as PeriodType, span: 5 };
    case "15m":
      return { type: "minute" as PeriodType, span: 15 };
    case "1h":
      return { type: "hour" as PeriodType, span: 1 };
    case "4h":
      return { type: "hour" as PeriodType, span: 4 };
    case "1D":
      return { type: "day" as PeriodType, span: 1 };
    case "1W":
      return { type: "week" as PeriodType, span: 1 };
    default:
      return { type: "minute" as PeriodType, span: 15 };
  }
};

const mapCandleToKLine = (c: CandleData): KLineData => ({
  timestamp: c.time * 1000,
  open: c.open,
  high: c.high,
  low: c.low,
  close: c.close,
  volume: c.volume ?? 0,
});

const computeHeikinAshi = (raw: CandleData[]): KLineData[] => {
  const result: KLineData[] = [];
  for (let i = 0; i < raw.length; i++) {
    const c = raw[i];
    const close = (c.open + c.high + c.low + c.close) / 4;
    const open =
      i === 0
        ? (c.open + c.close) / 2
        : ((result[i - 1].open as number) + (result[i - 1].close as number)) / 2;
    result.push({
      timestamp: c.time * 1000,
      open,
      high: Math.max(c.high, open, close),
      low: Math.min(c.low, open, close),
      close,
      volume: c.volume ?? 0,
    });
  }
  return result;
};

export const ChartArea: React.FC<ChartAreaProps> = ({
  paneId,
  symbol,
  provider,
  timeframe,
  chartType = "candlestick",
  indicators = { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
  forecast = null,
  forecastState = "unavailable",
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
  const chartRef = useRef<KLineChartInstance | null>(null);
  const klinechartsModuleRef = useRef<typeof import("klinecharts") | null>(null);
  const subscribeCallbackRef = useRef<((data: KLineData) => void) | null>(null);

  const candlesRef = useRef(candles);
  candlesRef.current = candles;
  const hasMoreHistoryRef = useRef(hasMoreHistory);
  hasMoreHistoryRef.current = hasMoreHistory;
  const onLoadOlderRef = useRef(onLoadOlder);
  onLoadOlderRef.current = onLoadOlder;
  const chartTypeRef = useRef(chartType);
  chartTypeRef.current = chartType;

  // Scale Mode: normal, log, percent
  const [scaleMode, setScaleMode] = useState<"normal" | "log" | "percent">("normal");

  // Drawings state & undo/redo tracking
  const [undoStack, setUndoStack] = useState<string[]>([]);
  const [redoStack, setRedoStack] = useState<any[]>([]);
  const selectedDrawingIdRef = useRef<string | null>(null);

  // OHLC display state
  const [ohlc, setOhlc] = useState({
    open: 0,
    high: 0,
    low: 0,
    close: 0,
    change: 0,
    changePercent: 0,
  });

  const isLight = (settings?.theme || theme) === "light";

  const forecastDecisionTime = forecast?.decision_at
    ? Math.floor(new Date(forecast.decision_at).getTime() / 1000)
    : null;
  const matchingForecastCandle =
    forecastDecisionTime === null
      ? undefined
      : candles.find((candle) => candle.time + 900 === forecastDecisionTime);
  const hasForecastCandle = Boolean(matchingForecastCandle);

  const updateDrawingsCount = useCallback(() => {
    const chart = chartRef.current;
    if (!chart) return;
    const overlays = chart.getOverlays({ groupId: "user_drawings" });
    onDrawingsCountChange?.(overlays.length);
  }, [onDrawingsCountChange]);

  // Dynamic import and chart initialization
  useEffect(() => {
    if (!chartContainerRef.current) return;
    let chartInstance: KLineChartInstance | null = null;
    let isDisposed = false;

    import("klinecharts").then((klinecharts) => {
      if (isDisposed || !chartContainerRef.current) return;
      klinechartsModuleRef.current = klinecharts;

      // Register custom indicators once
      try {
        if (!klinecharts.getSupportedIndicators().includes("ATR")) {
          klinecharts.registerIndicator({
            name: "ATR",
            shortName: "ATR",
            calcParams: [14],
            figures: [{ key: "atr", title: "ATR: ", type: "line" }],
            calc: (dataList, indicator) => {
              const p = indicator.calcParams[0] || 14;
              let trSum = 0;
              let prevAtr = 0;
              return dataList.map((kLine, i) => {
                if (i === 0) {
                  const tr = kLine.high - kLine.low;
                  trSum += tr;
                  return {};
                }
                const prevClose = dataList[i - 1].close;
                const tr = Math.max(
                  kLine.high - kLine.low,
                  Math.abs(kLine.high - prevClose),
                  Math.abs(kLine.low - prevClose)
                );
                trSum += tr;
                if (i < p - 1) return {};
                if (i === p - 1) {
                  prevAtr = trSum / p;
                  return { atr: prevAtr };
                }
                prevAtr = (prevAtr * (p - 1) + tr) / p;
                return { atr: prevAtr };
              });
            },
          });
        }

        if (!klinecharts.getSupportedIndicators().includes("VWAP")) {
          klinecharts.registerIndicator({
            name: "VWAP",
            shortName: "VWAP",
            series: "price",
            precision: 2,
            shouldOhlc: true,
            figures: [{ key: "vwap", title: "VWAP: ", type: "line" }],
            calc: (dataList) => {
              let cumVol = 0;
              let cumTypicalVol = 0;
              let lastDay = -1;
              return dataList.map((kLine) => {
                const date = new Date(kLine.timestamp);
                const day = date.getUTCDate();
                if (day !== lastDay) {
                  cumVol = 0;
                  cumTypicalVol = 0;
                  lastDay = day;
                }
                const vol =
                  typeof kLine.volume === "number" && kLine.volume > 0 ? kLine.volume : 1;
                const typical = (kLine.high + kLine.low + kLine.close) / 3;
                cumVol += vol;
                cumTypicalVol += typical * vol;
                const val = cumVol > 0 ? cumTypicalVol / cumVol : kLine.close;
                return { vwap: val };
              });
            },
          });
        }

        if (!klinecharts.getSupportedOverlays().includes("measure")) {
          klinecharts.registerOverlay({
            name: "measure",
            totalStep: 3,
            needDefaultPointFigure: true,
            needDefaultXAxisFigure: true,
            needDefaultYAxisFigure: true,
            createPointFigures: ({ coordinates, overlay }) => {
              if (coordinates.length === 2) {
                const p1 = coordinates[0];
                const p2 = coordinates[1];
                const minX = Math.min(p1.x, p2.x);
                const maxX = Math.max(p1.x, p2.x);
                const minY = Math.min(p1.y, p2.y);
                const maxY = Math.max(p1.y, p2.y);
                const w = maxX - minX;
                const h = maxY - minY;
                const v1 = overlay.points[0]?.value ?? 0;
                const v2 = overlay.points[1]?.value ?? 0;
                const diff = v2 - v1;
                const pct = v1 !== 0 ? (diff / v1) * 100 : 0;
                const isPos = diff >= 0;
                const color = isPos ? "#089981" : "#f23645";
                const text = `${isPos ? "+" : ""}${diff.toFixed(2)} (${isPos ? "+" : ""}${pct.toFixed(2)}%)`;
                return [
                  {
                    type: "polygon",
                    attrs: {
                      coordinates: [
                        { x: minX, y: minY },
                        { x: maxX, y: minY },
                        { x: maxX, y: maxY },
                        { x: minX, y: maxY },
                      ],
                    },
                    styles: {
                      style: "stroke_fill",
                      color: isPos ? "rgba(8, 153, 129, 0.15)" : "rgba(242, 54, 69, 0.15)",
                      borderColor: color,
                      borderSize: 1,
                      borderStyle: "dashed",
                    },
                  },
                  {
                    type: "text",
                    attrs: {
                      x: minX + w / 2,
                      y: minY + h / 2,
                      text,
                      align: "center",
                      baseline: "middle",
                    },
                    styles: {
                      color: "#ffffff",
                      backgroundColor: color,
                      paddingLeft: 6,
                      paddingRight: 6,
                      paddingTop: 3,
                      paddingBottom: 3,
                      borderRadius: 4,
                    },
                  },
                ];
              }
              return [];
            },
          });
        }
      } catch (err) {
        console.warn("[ChartArea] indicator/overlay registration warning:", err);
      }

      // Initialize chart instance
      chartInstance = klinecharts.init(chartContainerRef.current, {
        timezone: settings?.timezone || "UTC",
        styles: isLight ? "light" : "dark",
      });
      if (!chartInstance) return;
      chartRef.current = chartInstance;

      // Apply institutional styles
      chartInstance.setStyles({
        grid: {
          show: settings?.gridVisible !== false,
          horizontal: { color: isLight ? "#f0f3fa" : "#1f2431" },
          vertical: { color: isLight ? "#f0f3fa" : "#1f2431" },
        },
        candle: {
          type:
            chartTypeRef.current === "bar"
              ? "ohlc"
              : chartTypeRef.current === "area" || chartTypeRef.current === "line"
              ? "area"
              : "candle_solid",
          bar: {
            upColor: settings?.upColor || "#089981",
            downColor: settings?.downColor || "#f23645",
            upBorderColor: settings?.upColor || "#089981",
            downBorderColor: settings?.downColor || "#f23645",
            upWickColor: settings?.upColor || "#089981",
            downWickColor: settings?.downColor || "#f23645",
          },
          tooltip: {
            showRule: "follow_cross",
          },
        },
      });

      // Configure symbol & period
      chartInstance.setSymbol({
        ticker: symbol,
        pricePrecision: digits,
        volumePrecision: 2,
      });
      chartInstance.setPeriod(mapTimeframeToPeriod(timeframe));

      // Set DataLoader
      chartInstance.setDataLoader({
        getBars: async (params) => {
          if (params.type === "init") {
            const raw = candlesRef.current;
            const data =
              chartTypeRef.current === "heikin_ashi"
                ? computeHeikinAshi(raw)
                : raw.map(mapCandleToKLine);
            params.callback(data, { forward: hasMoreHistoryRef.current });
            if (raw.length > 0) {
              const last = raw[raw.length - 1];
              const ch = last.close - last.open;
              const chp = last.open ? (ch / last.open) * 100 : 0;
              setOhlc({
                open: last.open,
                high: last.high,
                low: last.low,
                close: last.close,
                change: ch,
                changePercent: chp,
              });
            }
          } else if (params.type === "forward") {
            if (onLoadOlderRef.current) {
              try {
                const older = await onLoadOlderRef.current();
                const data =
                  chartTypeRef.current === "heikin_ashi"
                    ? computeHeikinAshi(older)
                    : older.map(mapCandleToKLine);
                params.callback(data, { forward: hasMoreHistoryRef.current });
              } catch {
                params.callback([], { forward: false });
              }
            } else {
              params.callback([], { forward: false });
            }
          }
        },
        subscribeBar: (params) => {
          subscribeCallbackRef.current = params.callback;
        },
        unsubscribeBar: () => {
          subscribeCallbackRef.current = null;
        },
      });

      // Crosshair sync & OHLC updates
      chartInstance.subscribeAction("onCrosshairChange", (param: any) => {
        if (!param || !param.kLineData) {
          if (paneId) {
            window.dispatchEvent(
              new CustomEvent("terminal-crosshair-sync", {
                detail: { sourceId: paneId, clear: true },
              })
            );
          }
          if (candlesRef.current.length > 0) {
            const last = candlesRef.current[candlesRef.current.length - 1];
            const ch = last.close - last.open;
            const chp = last.open ? (ch / last.open) * 100 : 0;
            setOhlc({
              open: last.open,
              high: last.high,
              low: last.low,
              close: last.close,
              change: ch,
              changePercent: chp,
            });
          }
          return;
        }

        const k = param.kLineData;
        const ch = k.close - k.open;
        const chp = k.open ? (ch / k.open) * 100 : 0;
        setOhlc({
          open: k.open,
          high: k.high,
          low: k.low,
          close: k.close,
          change: ch,
          changePercent: chp,
        });

        if (paneId) {
          window.dispatchEvent(
            new CustomEvent("terminal-crosshair-sync", {
              detail: {
                sourceId: paneId,
                time: Math.floor(k.timestamp / 1000),
              },
            })
          );
        }
      });
    });

    const resizeObserver = new ResizeObserver(() => {
      chartRef.current?.resize();
    });
    if (chartContainerRef.current) {
      resizeObserver.observe(chartContainerRef.current);
    }

    return () => {
      isDisposed = true;
      resizeObserver.disconnect();
      if (chartContainerRef.current) {
        klinechartsModuleRef.current?.dispose(chartContainerRef.current);
      }
      chartRef.current = null;
    };
  }, []);

  // Update Symbol, Precision & Period
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;
    chart.setSymbol({
      ticker: symbol,
      pricePrecision: digits,
      volumePrecision: 2,
    });
    chart.setPeriod(mapTimeframeToPeriod(timeframe));
    chart.resetData();
  }, [symbol, timeframe, digits]);

  // Update styles (Theme, Colors, Grid, Chart Type)
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;
    chart.setStyles(isLight ? "light" : "dark");
    chart.setStyles({
      grid: {
        show: settings?.gridVisible !== false,
        horizontal: { color: isLight ? "#f0f3fa" : "#1f2431" },
        vertical: { color: isLight ? "#f0f3fa" : "#1f2431" },
      },
      candle: {
        type:
          chartType === "bar"
            ? "ohlc"
            : chartType === "area" || chartType === "line"
            ? "area"
            : "candle_solid",
        bar: {
          upColor: settings?.upColor || "#089981",
          downColor: settings?.downColor || "#f23645",
          upBorderColor: settings?.upColor || "#089981",
          downBorderColor: settings?.downColor || "#f23645",
          upWickColor: settings?.upColor || "#089981",
          downWickColor: settings?.downColor || "#f23645",
        },
      },
    });
    chart.resetData();
  }, [isLight, settings, chartType]);

  // Sync Technical Indicators
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;

    const smaPeriod = settings?.indicatorParams?.smaPeriod || 20;
    const emaPeriod = settings?.indicatorParams?.emaPeriod || 50;
    const bbPeriod = settings?.indicatorParams?.bollingerPeriod || 20;
    const bbStd = settings?.indicatorParams?.bollingerStdDev || 2.0;
    const rsiPeriod = settings?.indicatorParams?.rsiPeriod || 14;
    const macdFast = settings?.indicatorParams?.macdFast || 12;
    const macdSlow = settings?.indicatorParams?.macdSlow || 26;
    const macdSignal = settings?.indicatorParams?.macdSignal || 9;
    const atrPeriod = settings?.indicatorParams?.atrPeriod || 14;

    // SMA (on candle pane)
    if (indicators.sma20) {
      chart.createIndicator(
        {
          id: "ind_sma",
          name: "MA",
          shortName: "SMA",
          calcParams: [smaPeriod],
          paneId: "candle_pane",
          styles: { lines: [{ color: "#f5b942", size: 2 }] },
        },
        false
      );
    } else {
      chart.removeIndicator({ id: "ind_sma" });
    }

    // EMA (on candle pane)
    if (indicators.ema50) {
      chart.createIndicator(
        {
          id: "ind_ema",
          name: "EMA",
          shortName: "EMA",
          calcParams: [emaPeriod],
          paneId: "candle_pane",
          styles: { lines: [{ color: "#2962ff", size: 2 }] },
        },
        false
      );
    } else {
      chart.removeIndicator({ id: "ind_ema" });
    }

    // VWAP (on candle pane)
    if (indicators.vwap) {
      chart.createIndicator(
        {
          id: "ind_vwap",
          name: "VWAP",
          shortName: "VWAP",
          paneId: "candle_pane",
          styles: { lines: [{ color: "#a855f7", size: 2 }] },
        },
        false
      );
    } else {
      chart.removeIndicator({ id: "ind_vwap" });
    }

    // Bollinger Bands (on candle pane)
    if (indicators.bollinger) {
      chart.createIndicator(
        {
          id: "ind_boll",
          name: "BOLL",
          shortName: "BOLL",
          calcParams: [bbPeriod, bbStd],
          paneId: "candle_pane",
          styles: {
            lines: [
              { color: "rgba(8, 153, 129, 0.7)", size: 1 },
              { color: "rgba(245, 185, 66, 0.6)", size: 1 },
              { color: "rgba(242, 54, 69, 0.7)", size: 1 },
            ],
          },
        },
        false
      );
    } else {
      chart.removeIndicator({ id: "ind_boll" });
    }

    // RSI (sub-pane)
    if (indicators.rsi) {
      chart.createIndicator(
        {
          id: "ind_rsi",
          name: "RSI",
          shortName: "RSI",
          calcParams: [rsiPeriod],
          paneId: "pane_rsi",
          styles: { lines: [{ color: "#ab47bc", size: 2 }] },
        },
        false
      );
      chart.setPaneOptions({ id: "pane_rsi", height: 100, minHeight: 60 });
    } else {
      chart.removeIndicator({ id: "ind_rsi" });
    }

    // MACD (sub-pane)
    if (indicators.macd) {
      chart.createIndicator(
        {
          id: "ind_macd",
          name: "MACD",
          shortName: "MACD",
          calcParams: [macdFast, macdSlow, macdSignal],
          paneId: "pane_macd",
        },
        false
      );
      chart.setPaneOptions({ id: "pane_macd", height: 110, minHeight: 60 });
    } else {
      chart.removeIndicator({ id: "ind_macd" });
    }

    // ATR (sub-pane)
    if (indicators.atr) {
      chart.createIndicator(
        {
          id: "ind_atr",
          name: "ATR",
          shortName: "ATR",
          calcParams: [atrPeriod],
          paneId: "pane_atr",
          styles: { lines: [{ color: "#f59e0b", size: 2 }] },
        },
        false
      );
      chart.setPaneOptions({ id: "pane_atr", height: 90, minHeight: 50 });
    } else {
      chart.removeIndicator({ id: "ind_atr" });
    }
  }, [indicators, settings?.indicatorParams]);

  // Feed candles updates
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;
    chart.resetData();
  }, [candles]);

  // Live Price streaming updates
  useEffect(() => {
    if (!livePrice || candles.length === 0) return;
    const last = candles[candles.length - 1];
    const updated: KLineData = {
      timestamp: last.time * 1000,
      open: last.open,
      high: Math.max(last.high, livePrice),
      low: Math.min(last.low, livePrice),
      close: livePrice,
      volume: last.volume ?? 0,
    };
    subscribeCallbackRef.current?.(updated);
  }, [livePrice, candles]);

  // Active Drawing Tool handler (KLineChart Overlays)
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart || activeTool === "cursor") return;

    let overlayName = "segment";
    if (activeTool === "trendline") overlayName = "segment";
    else if (activeTool === "horizontal") overlayName = "horizontalStraightLine";
    else if (activeTool === "fibonacci") overlayName = "fibonacciLine";
    else if (activeTool === "measure") overlayName = "measure";

    const id = chart.createOverlay({
      name: overlayName,
      groupId: "user_drawings",
      onDrawEnd: () => {
        if (!isDrawingModeLocked) {
          onDrawingFinished?.();
        }
        updateDrawingsCount();
      },
      onRemoved: () => {
        updateDrawingsCount();
      },
      onSelected: (event) => {
        selectedDrawingIdRef.current = event.overlay.id;
      },
      onDeselected: () => {
        if (selectedDrawingIdRef.current === id) {
          selectedDrawingIdRef.current = null;
        }
      },
    });

    if (id && typeof id === "string") {
      setUndoStack((prev) => [...prev, id]);
      setRedoStack([]);
      onCanUndoRedoChange?.(true, false);
    }
  }, [activeTool, isDrawingModeLocked, onDrawingFinished, onCanUndoRedoChange, updateDrawingsCount]);

  // Clear drawings trigger
  const lastHandledClearRef = useRef(clearDrawingsTrigger);
  useEffect(() => {
    if (clearDrawingsTrigger > 0 && clearDrawingsTrigger !== lastHandledClearRef.current) {
      lastHandledClearRef.current = clearDrawingsTrigger;
      chartRef.current?.removeOverlay({ groupId: "user_drawings" });
      setUndoStack([]);
      setRedoStack([]);
      onDrawingsCountChange?.(0);
      onCanUndoRedoChange?.(false, false);
    } else if (clearDrawingsTrigger === 0) {
      lastHandledClearRef.current = 0;
    }
  }, [clearDrawingsTrigger, onDrawingsCountChange, onCanUndoRedoChange]);

  // Hide / Show drawings
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;
    const overlays = chart.getOverlays({ groupId: "user_drawings" });
    overlays.forEach((o) => {
      chart.overrideOverlay({ id: o.id, visible: !isDrawingsHidden });
    });
  }, [isDrawingsHidden]);

  // Undo / Redo handlers
  const handleUndo = useCallback(() => {
    const chart = chartRef.current;
    if (!chart || undoStack.length === 0) return;
    const lastId = undoStack[undoStack.length - 1];
    const overlays = chart.getOverlays({ id: lastId });
    if (overlays.length > 0) {
      const removed = overlays[0];
      chart.removeOverlay({ id: lastId });
      setRedoStack((prev) => [...prev, removed]);
    }
    const newUndo = undoStack.slice(0, -1);
    setUndoStack(newUndo);
    updateDrawingsCount();
    onCanUndoRedoChange?.(newUndo.length > 0, true);
  }, [undoStack, updateDrawingsCount, onCanUndoRedoChange]);

  const handleRedo = useCallback(() => {
    const chart = chartRef.current;
    if (!chart || redoStack.length === 0) return;
    const nextOverlay = redoStack[redoStack.length - 1];
    const id = chart.createOverlay(nextOverlay);
    if (id && typeof id === "string") {
      setUndoStack((prev) => [...prev, id]);
    }
    const newRedo = redoStack.slice(0, -1);
    setRedoStack(newRedo);
    updateDrawingsCount();
    onCanUndoRedoChange?.(true, newRedo.length > 0);
  }, [redoStack, updateDrawingsCount, onCanUndoRedoChange]);

  const lastHandledUndoRef = useRef(undoTrigger);
  const lastHandledRedoRef = useRef(redoTrigger);
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

  // Keyboard Delete / Backspace & Undo/Redo Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (target?.tagName === "INPUT" || target?.tagName === "TEXTAREA") return;

      if ((e.key === "Delete" || e.key === "Backspace") && selectedDrawingIdRef.current) {
        chartRef.current?.removeOverlay({ id: selectedDrawingIdRef.current });
        selectedDrawingIdRef.current = null;
        updateDrawingsCount();
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
  }, [handleUndo, handleRedo, updateDrawingsCount]);

  // Snapshot Trigger export with watermark
  const lastHandledSnapshotRef = useRef(snapshotTrigger);
  useEffect(() => {
    if (
      snapshotTrigger > 0 &&
      snapshotTrigger !== lastHandledSnapshotRef.current &&
      chartRef.current
    ) {
      lastHandledSnapshotRef.current = snapshotTrigger;
      try {
        const bg = isLight ? "#ffffff" : "#131722";
        const base64Url = chartRef.current.getConvertPictureUrl(true, "png", bg);
        if (base64Url) {
          const img = new Image();
          img.onload = () => {
            const canvas = document.createElement("canvas");
            canvas.width = img.width;
            canvas.height = img.height;
            const ctx = canvas.getContext("2d");
            if (ctx) {
              ctx.drawImage(img, 0, 0);

              // Footer watermark banner
              ctx.fillStyle = isLight ? "rgba(240, 243, 250, 0.92)" : "rgba(19, 23, 34, 0.92)";
              ctx.fillRect(0, canvas.height - 38, canvas.width, 38);

              // Brand logo text
              ctx.font = "bold 14px -apple-system, sans-serif";
              ctx.fillStyle = "#2962ff";
              ctx.fillText("PIA TERMINAL", 18, canvas.height - 14);

              // Metadata text
              ctx.font = "12px monospace";
              ctx.fillStyle = isLight ? "#5d606b" : "#d1d4dc";
              ctx.fillText(
                `${symbol} · ${timeframe} · ${new Date().toISOString().replace("T", " ").substring(0, 19)} UTC`,
                145,
                canvas.height - 14
              );

              const watermarkedUrl = canvas.toDataURL("image/png");
              const a = document.createElement("a");
              a.href = watermarkedUrl;
              a.download = `PIA_${symbol}_${timeframe}_${Date.now()}.png`;
              a.click();
            }
          };
          img.src = base64Url;
        }
      } catch (err) {
        console.warn("[ChartArea] Snapshot export error:", err);
      }
      onSnapshotDone?.();
    } else if (snapshotTrigger === 0) {
      lastHandledSnapshotRef.current = 0;
    }
  }, [snapshotTrigger, onSnapshotDone, symbol, timeframe, isLight]);

  // Scale Mode controls (Normal, Logarithmic, Percentage)
  const handleToggleScale = (mode: "normal" | "log" | "percent") => {
    const chart = chartRef.current;
    if (!chart) return;
    if (mode === "normal") {
      chart.overrideYAxis({ paneId: "candle_pane", name: "normal" });
      setScaleMode("normal");
    } else if (mode === "log") {
      const next = scaleMode === "log" ? "normal" : "log";
      chart.overrideYAxis({
        paneId: "candle_pane",
        name: next === "log" ? "logarithm" : "normal",
      });
      setScaleMode(next);
    } else if (mode === "percent") {
      const next = scaleMode === "percent" ? "normal" : "percent";
      chart.overrideYAxis({
        paneId: "candle_pane",
        name: next === "percent" ? "percentage" : "normal",
      });
      setScaleMode(next);
    }
  };

  // Quick Range Selector
  const handleQuickRange = (range: "1D" | "5D" | "1M" | "3M" | "6M" | "1Y" | "ALL") => {
    const chart = chartRef.current;
    if (!chart || candles.length === 0) return;

    if (range === "ALL") {
      chart.scrollToTimestamp(candles[0].time * 1000);
      return;
    }

    const lastTime = candles[candles.length - 1].time;
    let seconds = 86400;
    if (range === "5D") seconds = 5 * 86400;
    else if (range === "1M") seconds = 30 * 86400;
    else if (range === "3M") seconds = 90 * 86400;
    else if (range === "6M") seconds = 180 * 86400;
    else if (range === "1Y") seconds = 365 * 86400;

    const fromTime = lastTime - seconds;
    chart.scrollToTimestamp(fromTime * 1000);
  };

  const isUp = ohlc.close >= ohlc.open;
  const hasData = candles.length > 0;

  return (
    <div
      className={`relative w-full h-full flex flex-col ${
        isLight ? "bg-white text-[#131722]" : "bg-[#131722] text-[#d1d4dc]"
      } overflow-hidden select-none`}
    >
      {/* Chart Legend & Status Bar */}
      <div className="absolute top-3 left-3 z-10 flex flex-col gap-1.5 pointer-events-none">
        <div className="flex flex-wrap items-center gap-1.5 pointer-events-auto">
          <span
            className={`font-bold text-sm tracking-wide ${
              isLight ? "text-[#131722]" : "text-white"
            }`}
          >
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#f5b942] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#2962ff] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#a855f7] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#089981] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
              <span>
                BB({settings?.indicatorParams?.bollingerPeriod || 20},
                {settings?.indicatorParams?.bollingerStdDev || 2})
              </span>
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#ab47bc] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#f59e0b] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
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
            <div
              className={`group flex items-center gap-1 text-[10px] font-mono border px-1.5 py-0.5 rounded text-[#2962ff] ${
                isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
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

        {/* AI Forecast Overlay Card (for XAUUSD 15m) */}
        {indicators.aiForecast && (
          <div
            className={`pointer-events-auto w-72 rounded border px-2.5 py-2 text-[10px] font-mono shadow-lg ${
              isLight
                ? "bg-white/95 border-[#e0e3eb] text-[#131722]"
                : "bg-[#1e222d]/95 border-[#2a2e39] text-[#d1d4dc]"
            }`}
            role="status"
            aria-live="polite"
          >
            <div className="mb-1 flex items-center justify-between font-bold">
              <span>XAUUSD · 1h Forecast</span>
              <span
                className={
                  forecastState === "active"
                    ? "text-[#089981]"
                    : forecastState === "expired"
                    ? "text-[#f5b942]"
                    : "text-[#787b86]"
                }
              >
                {symbol !== "XAUUSD" || timeframe !== "15m"
                  ? "15m XAUUSD only"
                  : !hasForecastCandle && forecast
                  ? "CANDLE NOT LOADED"
                  : forecastState.toUpperCase()}
              </span>
            </div>
            {symbol === "XAUUSD" && timeframe === "15m" && forecast && hasForecastCandle ? (
              <>
                <div className="grid grid-cols-3 gap-1 py-1 text-center">
                  <span>DOWN {(forecast.probabilities!.down * 100).toFixed(0)}%</span>
                  <span>FLAT {(forecast.probabilities!.flat * 100).toFixed(0)}%</span>
                  <span>UP {(forecast.probabilities!.up * 100).toFixed(0)}%</span>
                </div>
                <div>
                  Expected {(forecast.expected_return! * 100).toFixed(3)}% · uncertainty{" "}
                  {(forecast.uncertainty! * 100).toFixed(3)}%
                </div>
                <div className="mt-1 grid grid-cols-3 gap-1 text-center">
                  <span>
                    q10{" "}
                    {(
                      forecast.reference_price! *
                      Math.exp(forecast.return_quantiles!.q10)
                    ).toFixed(digits)}
                  </span>
                  <span>
                    q50{" "}
                    {(
                      forecast.reference_price! *
                      Math.exp(forecast.return_quantiles!.q50)
                    ).toFixed(digits)}
                  </span>
                  <span>
                    q90{" "}
                    {(
                      forecast.reference_price! *
                      Math.exp(forecast.return_quantiles!.q90)
                    ).toFixed(digits)}
                  </span>
                </div>
                <div className="mt-1 truncate text-[#787b86]">
                  {forecast.model_version} ·{" "}
                  {new Date(forecast.decision_at!).toLocaleString()}
                </div>
                <div className="text-[#787b86]">
                  Horizon ends {new Date(forecast.horizon_end!).toLocaleString()} · paper only
                </div>
              </>
            ) : (
              <div className="text-[#787b86]">
                {symbol !== "XAUUSD" || timeframe !== "15m"
                  ? "Switch this pane to XAUUSD 15m."
                  : forecastState === "loading"
                  ? "Loading the latest forecast…"
                  : forecastState === "error"
                  ? "Forecast service is unavailable."
                  : !hasForecastCandle && forecast
                  ? "The decision candle is outside the loaded chart history."
                  : "No evaluated model forecast is available."}
              </div>
            )}
          </div>
        )}

        {/* OHLC Bar Metrics */}
        <div className="flex flex-wrap items-center gap-3 text-[11px] font-mono">
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">O</span>
            <span className={isLight ? "text-[#131722]" : "text-[#d1d4dc]"}>
              {ohlc.open.toFixed(digits)}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">H</span>
            <span className={isLight ? "text-[#131722]" : "text-[#d1d4dc]"}>
              {ohlc.high.toFixed(digits)}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">L</span>
            <span className={isLight ? "text-[#131722]" : "text-[#d1d4dc]"}>
              {ohlc.low.toFixed(digits)}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-[#787b86]">C</span>
            <span className={isUp ? "text-[#089981]" : "text-[#f23645]"}>
              {ohlc.close.toFixed(digits)}
            </span>
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
              Market may be closed, or this symbol isn&apos;t ingested yet.
            </span>
          </div>
        </div>
      )}

      {/* Primary Chart Canvas */}
      <div ref={chartContainerRef} className="w-full flex-1" />

      {/* Quick Timeframe Range Bar & Scale Mode Controls */}
      <div
        className={`h-7 border-t flex items-center justify-between px-2 text-[11px] font-mono select-none shrink-0 z-20 ${
          isLight
            ? "bg-[#f0f3fa] border-[#e0e3eb] text-[#5d606b]"
            : "bg-[#1e222d] border-[#2a2e39] text-[#787b86]"
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
              scaleMode === "log"
                ? "bg-[#2962ff] text-white font-bold"
                : "text-[#787b86] hover:text-white"
            }`}
            title="Logarithmic Scale"
          >
            log
          </button>
          <button
            onClick={() => handleToggleScale("percent")}
            className={`px-1.5 py-0.5 rounded transition-colors cursor-pointer ${
              scaleMode === "percent"
                ? "bg-[#2962ff] text-white font-bold"
                : "text-[#787b86] hover:text-white"
            }`}
            title="Percentage Scale"
          >
            %
          </button>
        </div>
      </div>
    </div>
  );
};
