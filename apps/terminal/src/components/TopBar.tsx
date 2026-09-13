"use client";

import React, { useState } from "react";
import Image from "next/image";
import { 
  Search, 
  BarChart2, 
  Settings, 
  Camera, 
  Maximize2, 
  SlidersHorizontal,
  ChevronDown,
  Clock,
  TrendingUp,
  LayoutGrid,
  Share2,
  Undo2,
  Redo2,
  Check,
  Moon,
  Sun,
  Plus,
  Crosshair,
  Link2,
  Sliders,
  BarChart3,
  Layers,
  Newspaper,
  Radio,
  Brain,
  Calendar
} from "lucide-react";
import { Timeframe, IndicatorState, ChartLayout, IndicatorParameters, PaneContentType } from "@/types";

import { VisualLayoutPicker } from "./VisualLayoutPicker";

interface TopBarProps {
  symbol: string;
  timeframe: Timeframe;
  setTimeframe: (tf: Timeframe) => void;
  price: number;
  change: number;
  changePercent: number;
  digits: number;
  onSearchClick: () => void;
  paneType?: PaneContentType;
  onChangePaneType?: (type: PaneContentType) => void;
  chartType?: "candlestick" | "bar" | "line" | "area" | "heikin_ashi";
  onChartTypeChange?: (type: "candlestick" | "bar" | "line" | "area" | "heikin_ashi") => void;
  onToggleIndicator?: (indicator: keyof IndicatorState) => void;
  indicators?: IndicatorState;
  onFullscreen?: () => void;
  onAlertClick?: () => void;
  layout?: ChartLayout;
  onLayoutChange?: (layout: ChartLayout) => void;
  onSnapshot?: () => void;
  onToggleSidebar?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  canUndo?: boolean;
  canRedo?: boolean;
  theme?: "dark" | "light";
  onToggleTheme?: () => void;
  onOpenSettings?: () => void;
  onOpenIndicatorSettings?: () => void;
  indicatorParams?: IndicatorParameters;
  syncCrosshair?: boolean;
  onToggleSyncCrosshair?: () => void;
  syncTime?: boolean;
  onToggleSyncTime?: () => void;
  onSave?: () => void;
}

const TIMEFRAMES: Timeframe[] = ["1m", "5m", "15m", "1h", "4h", "1D", "1W"];

export const TopBar: React.FC<TopBarProps> = ({
  symbol,
  timeframe,
  setTimeframe,
  price,
  change,
  changePercent,
  digits,
  onSearchClick,
  paneType = "chart",
  onChangePaneType,
  chartType = "candlestick",
  onChartTypeChange,
  onToggleIndicator,
  indicators = { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
  onFullscreen,
  onAlertClick,
  layout = "1x1",
  onLayoutChange,
  onSnapshot,
  onToggleSidebar,
  onUndo,
  onRedo,
  canUndo = false,
  canRedo = false,
  theme = "dark",
  onToggleTheme,
  onOpenSettings,
  onOpenIndicatorSettings,
  indicatorParams,
  syncCrosshair = true,
  onToggleSyncCrosshair,
  syncTime = true,
  onToggleSyncTime,
  onSave,
}) => {
  const isPositive = change >= 0;
  const isLight = theme === "light";
  const [savedFeedback, setSavedFeedback] = useState(false);
  const [isLayoutPickerOpen, setIsLayoutPickerOpen] = useState(false);

  const handleSave = () => {
    try {
      onSave?.();
      localStorage.setItem("atlsd_terminal_saved_layout", JSON.stringify({ symbol, timeframe, chartType, layout }));
      setSavedFeedback(true);
      setTimeout(() => setSavedFeedback(false), 2000);
    } catch {
      // ignore
    }
  };

  return (
    <header
      className={`h-[38px] min-h-[38px] border-b flex items-center justify-between px-2 sm:px-3 select-none text-xs shrink-0 transition-colors ${
        isLight
          ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
          : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Left Segment: Brand, View Dropdown, Symbol, Interval, Indicators */}
      <div className="flex items-center gap-1.5 h-full">
        {/* Brand / Logo (PIA Logo) */}
        <div className={`flex items-center gap-2 pr-2 border-r h-6 mr-0.5 ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
          <div className="w-5 h-5 rounded overflow-hidden flex items-center justify-center shrink-0">
            <Image
              src="/logo.png"
              alt="PIA Logo"
              width={20}
              height={20}
              className="w-full h-full object-contain"
              priority
            />
          </div>
          <span className={`font-black tracking-wider text-[13px] hidden sm:inline ${isLight ? "text-[#131722]" : "text-white"}`}>
            PIA
          </span>
        </div>

        {/* Workspace View Dropdown (Chart, Order Book, News, Social, Intel, Calendar) */}
        <div className="relative group">
          <button
            className={`flex items-center gap-1.5 px-2 py-1 rounded cursor-pointer transition-colors ${
              isLight
                ? "hover:bg-[#f0f3fa] text-[#131722]"
                : "hover:bg-[#2a2e39] text-[#d1d4dc]"
            }`}
          >
            {paneType === "orderbook" && <Layers className="w-3.5 h-3.5 text-[#089981]" />}
            {paneType === "news" && <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" />}
            {paneType === "social" && <Radio className="w-3.5 h-3.5 text-[#e040fb]" />}
            {paneType === "intelligence" && <Brain className="w-3.5 h-3.5 text-[#00e5ff]" />}
            {paneType === "calendar" && <Calendar className="w-3.5 h-3.5 text-[#ff5252]" />}
            {(!paneType || paneType === "chart") && <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" />}
            <span className="font-bold text-xs">
              {paneType === "orderbook"
                ? "Order Book"
                : paneType === "intelligence"
                ? "Market Intel"
                : paneType === "social"
                ? "Social Pulse"
                : paneType === "calendar"
                ? "Calendar"
                : paneType === "news"
                ? "News"
                : "Chart"}
            </span>
            <ChevronDown className="w-3 h-3 text-[#787b86]" />
          </button>
          <div
            className={`hidden group-hover:flex absolute top-full left-0 z-40 mt-1 w-56 flex-col rounded-lg border p-1 shadow-2xl ${
              isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}
          >
            <div className="text-[10px] uppercase font-bold tracking-wider px-2 py-1 text-[#787b86]">
              Workspace View
            </div>
            {([
              { id: "chart", label: "Chart View", icon: <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" />, desc: "Interactive Candlesticks" },
              { id: "orderbook", label: "DOM & Order Book", icon: <Layers className="w-3.5 h-3.5 text-[#089981]" />, desc: "Market Depth & Level 2" },
              { id: "news", label: "News Stream", icon: <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" />, desc: "Financial Breaking News" },
              { id: "social", label: "Social Pulse", icon: <Radio className="w-3.5 h-3.5 text-[#e040fb]" />, desc: "𝕏 Live Posts & Sentiment" },
              { id: "intelligence", label: "Market Intelligence", icon: <Brain className="w-3.5 h-3.5 text-[#00e5ff]" />, desc: "AI Macro & COT Analysis" },
              { id: "calendar", label: "Economic Calendar", icon: <Calendar className="w-3.5 h-3.5 text-[#ff5252]" />, desc: "Central Bank & Macro Events" }
            ] as const).map((item) => (
              <button
                key={item.id}
                onClick={() => onChangePaneType?.(item.id)}
                className={`flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-xs cursor-pointer transition-colors ${
                  paneType === item.id || (!paneType && item.id === "chart")
                    ? "bg-[#2962ff]/10 text-[#2962ff] font-bold"
                    : isLight
                    ? "text-[#131722] hover:bg-[#f0f3fa]"
                    : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                }`}
              >
                {item.icon}
                <div className="flex flex-col">
                  <span>{item.label}</span>
                  <span className="text-[10px] font-normal text-[#787b86]">{item.desc}</span>
                </div>
              </button>
            ))}
          </div>
        </div>

        <div className={`h-4 w-px mx-0.5 ${isLight ? "bg-[#e0e3eb]" : "bg-[#2a2e39]"}`} />

        {/* Symbol Search Button */}
        <button
          onClick={onSearchClick}
          className={`flex items-center gap-2 px-2.5 py-1 rounded transition-colors border border-transparent cursor-pointer ${
            isLight
              ? "hover:bg-[#f0f3fa] hover:border-[#e0e3eb]"
              : "hover:bg-[#2a2e39] hover:border-[#363a45]"
          }`}
        >
          <Search className="w-3.5 h-3.5 text-[#787b86]" />
          <span className={`font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>{symbol}</span>
          <span
            className={`text-[10px] font-mono uppercase px-1.5 py-0.5 rounded ${
              isLight ? "bg-[#f0f3fa] text-[#5d606b]" : "bg-[#131722] text-[#787b86]"
            }`}
          >
            OANDA
          </span>
        </button>

        <div className={`h-4 w-px mx-1 ${isLight ? "bg-[#e0e3eb]" : "bg-[#2a2e39]"}`} />

        {/* Timeframe selector with dropdown */}
        <div className="flex items-center gap-0.5">
          <div className="flex items-center gap-0.5 max-w-[42vw] overflow-x-auto scrollbar-hide">
            {TIMEFRAMES.map((tf) => (
              <button
                key={tf}
                onClick={() => setTimeframe(tf)}
                className={`px-1.5 py-0.5 rounded font-semibold text-xs transition-colors cursor-pointer ${
                  timeframe === tf
                    ? "text-[#2962ff] bg-[#2962ff]/10 font-bold"
                    : isLight
                    ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
                    : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
                }`}
              >
                {tf}
              </button>
            ))}
          </div>
          <div className="relative group">
            <button
              className={`p-1 rounded cursor-pointer transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#5d606b]" : "hover:bg-[#2a2e39] text-[#787b86]"
              }`}
              title="All Intervals"
            >
              <ChevronDown className="w-3 h-3" />
            </button>
            <div
              className={`hidden group-hover:flex absolute top-full left-0 z-40 mt-1 w-36 flex-col rounded-lg border p-1 shadow-2xl ${
                isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
              }`}
            >
              <div className="text-[9px] uppercase font-bold tracking-wider px-2 py-1 text-[#787b86]">Minutes</div>
              {(["1m", "5m", "15m"] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setTimeframe(t)}
                  className={`rounded px-2 py-1 text-left text-xs cursor-pointer ${
                    timeframe === t ? "text-[#2962ff] font-bold bg-[#2962ff]/10" : isLight ? "text-[#131722] hover:bg-[#f0f3fa]" : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                  }`}
                >
                  {t}
                </button>
              ))}
              <div className="text-[9px] uppercase font-bold tracking-wider px-2 py-1 text-[#787b86] border-t border-[#2a2e39]/20 mt-1 pt-1">Hours</div>
              {(["1h", "4h"] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setTimeframe(t)}
                  className={`rounded px-2 py-1 text-left text-xs cursor-pointer ${
                    timeframe === t ? "text-[#2962ff] font-bold bg-[#2962ff]/10" : isLight ? "text-[#131722] hover:bg-[#f0f3fa]" : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                  }`}
                >
                  {t}
                </button>
              ))}
              <div className="text-[9px] uppercase font-bold tracking-wider px-2 py-1 text-[#787b86] border-t border-[#2a2e39]/20 mt-1 pt-1">Days</div>
              {(["1D", "1W"] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setTimeframe(t)}
                  className={`rounded px-2 py-1 text-left text-xs cursor-pointer ${
                    timeframe === t ? "text-[#2962ff] font-bold bg-[#2962ff]/10" : isLight ? "text-[#131722] hover:bg-[#f0f3fa]" : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                  }`}
                >
                  {t}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className={`h-4 w-px mx-1 ${isLight ? "bg-[#e0e3eb]" : "bg-[#2a2e39]"}`} />

        {/* Chart Style (Candles) */}
        <div className="relative group">
          <button
            className={`flex items-center gap-1 px-2 py-1 rounded cursor-pointer transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            <BarChart2 className="w-3.5 h-3.5" />
            <span className="hidden lg:inline text-xs font-medium">
              {chartType === "heikin_ashi" ? "Heikin Ashi" : chartType[0].toUpperCase() + chartType.slice(1)}
            </span>
            <ChevronDown className="w-3 h-3" />
          </button>
          <div
            className={`hidden group-hover:flex absolute top-full left-0 z-30 mt-1 w-40 flex-col rounded-lg border p-1 shadow-2xl ${
              isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}
          >
            {([
              { id: "candlestick", label: "Candles", icon: "🕯️" },
              { id: "bar", label: "Bars", icon: "📊" },
              { id: "line", label: "Line", icon: "📈" },
              { id: "area", label: "Area", icon: "⛰️" },
              { id: "heikin_ashi", label: "Heikin Ashi", icon: "⛩️" },
            ] as const).map(({ id, label, icon }) => (
              <button
                key={id}
                onClick={() => onChartTypeChange?.(id)}
                className={`flex items-center gap-2 rounded px-2 py-1.5 text-left text-xs cursor-pointer ${
                  chartType === id
                    ? "text-[#2962ff] font-bold bg-[#2962ff]/10"
                    : isLight
                    ? "text-[#131722] hover:bg-[#f0f3fa]"
                    : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                }`}
              >
                <span>{icon}</span>
                <span>{label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Indicators */}
        <div className="relative group">
          <button
            className={`flex items-center gap-1.5 px-2 py-1 rounded font-medium cursor-pointer transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
            }`}
          >
            <span className="font-serif font-bold text-[#2962ff] text-xs">fx</span>
            <span className="hidden sm:inline">Indicators</span>
          </button>
          <div
            className={`hidden group-hover:flex absolute top-full left-0 z-30 mt-1 w-60 flex-col rounded-xl border p-1.5 shadow-2xl ${
              isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}
          >
            <div className="text-[10px] uppercase font-bold tracking-wider px-2 py-1 text-[#787b86]">
              Technical Indicators
            </div>
            {([
              ['sma20', `SMA ${indicatorParams?.smaPeriod || 20} (Moving Average)`],
              ['ema50', `EMA ${indicatorParams?.emaPeriod || 50} (Exponential)`],
              ['vwap', 'VWAP (Volume Weighted)'],
              ['bollinger', `Bollinger Bands (${indicatorParams?.bollingerPeriod || 20}, ${indicatorParams?.bollingerStdDev || 2})`],
              ['rsi', `RSI (${indicatorParams?.rsiPeriod || 14}) Oscillator`],
              ['atr', `ATR (${indicatorParams?.atrPeriod || 14}) Volatility`],
              ['macd', `MACD (${indicatorParams?.macdFast || 12}, ${indicatorParams?.macdSlow || 26}, ${indicatorParams?.macdSignal || 9})`]
            ] as const).map(([id, label]) => (
              <button
                key={id}
                onClick={() => onToggleIndicator?.(id as any)}
                className={`flex items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-xs cursor-pointer transition-colors ${
                  isLight ? "text-[#131722] hover:bg-[#f0f3fa]" : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                }`}
              >
                <span>{label}</span>
                <span className={indicators[id as keyof IndicatorState] ? "text-[#2962ff] font-bold" : "text-[#787b86]"}>
                  {indicators[id as keyof IndicatorState] ? "ON" : "OFF"}
                </span>
              </button>
            ))}

            <div className={`mt-1 pt-1 border-t ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onOpenIndicatorSettings?.();
                }}
                className={`w-full flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-left text-xs font-semibold cursor-pointer transition-colors ${
                  isLight ? "text-[#2962ff] hover:bg-[#f0f3fa]" : "text-[#2962ff] hover:bg-[#2a2e39]"
                }`}
              >
                <Sliders className="w-3.5 h-3.5" />
                <span>Configure Parameters...</span>
              </button>
            </div>
          </div>
        </div>

        {/* Alerts Button */}
        <button
          onClick={onAlertClick}
          className={`hidden xl:flex items-center gap-1.5 px-2 py-1 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
          }`}
        >
          <Clock className="w-3.5 h-3.5" />
          <span>Alert</span>
        </button>

        {/* Live Ticker Header stats */}
        <div className={`hidden 2xl:flex items-center gap-3 pl-3 border-l ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
          <div className="flex items-baseline gap-1.5">
            <span className={`font-mono text-sm font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
              {price.toFixed(digits)}
            </span>
            <span className={`font-mono text-xs font-semibold ${isPositive ? "text-[#089981]" : "text-[#f23645]"}`}>
              {isPositive ? "+" : ""}{change.toFixed(digits)} ({isPositive ? "+" : ""}{changePercent.toFixed(2)}%)
            </span>
          </div>
        </div>
      </div>

      {/* Right Segment: Undo/Redo, Layout, Save, Theme Toggle, Settings, Fullscreen, Snapshot */}
      <div className="flex items-center gap-1">
        {/* Interactive Undo / Redo */}
        <div className={`hidden md:flex items-center gap-0.5 pr-2 border-r ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
          <button
            onClick={onUndo}
            disabled={!canUndo}
            className={`p-1.5 rounded transition-colors cursor-pointer ${
              canUndo
                ? isLight
                  ? "text-[#131722] hover:bg-[#f0f3fa]"
                  : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                : "text-[#787b86]/40 cursor-not-allowed"
            }`}
            title="Undo Drawing (Ctrl+Z)"
          >
            <Undo2 className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={onRedo}
            disabled={!canRedo}
            className={`p-1.5 rounded transition-colors cursor-pointer ${
              canRedo
                ? isLight
                  ? "text-[#131722] hover:bg-[#f0f3fa]"
                  : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                : "text-[#787b86]/40 cursor-not-allowed"
            }`}
            title="Redo Drawing (Ctrl+Y)"
          >
            <Redo2 className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Multi-Chart Synchronization Controls */}
        {layout !== "1x1" && (
          <div className={`flex items-center gap-0.5 px-1 py-0.5 rounded-lg border ${isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"}`}>
            <button
              onClick={onToggleSyncCrosshair}
              className={`p-1 rounded cursor-pointer transition-colors ${
                syncCrosshair
                  ? "bg-[#2962ff]/20 text-[#2962ff]"
                  : isLight
                  ? "text-[#787b86] hover:text-[#131722]"
                  : "text-[#787b86] hover:text-white"
              }`}
              title={`Sync Crosshair across Charts: ${syncCrosshair ? "ON" : "OFF"}`}
            >
              <Crosshair className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={onToggleSyncTime}
              className={`p-1 rounded cursor-pointer transition-colors ${
                syncTime
                  ? "bg-[#2962ff]/20 text-[#2962ff]"
                  : isLight
                  ? "text-[#787b86] hover:text-[#131722]"
                  : "text-[#787b86] hover:text-white"
              }`}
              title={`Sync Time / Zoom Range across Charts: ${syncTime ? "ON" : "OFF"}`}
            >
              <Link2 className="w-3.5 h-3.5" />
            </button>
          </div>
        )}

        {/* Visual Layout Mode Picker (TradingView-style) */}
        <div className="relative">
          <button
            onClick={() => setIsLayoutPickerOpen((v) => !v)}
            className={`p-1.5 rounded transition-colors cursor-pointer ${
              isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
            }`}
            title={`Select Layout (${layout})`}
          >
            <LayoutGrid className="w-3.5 h-3.5" />
          </button>

          {isLayoutPickerOpen && (
            <>
              <div
                className="fixed inset-0 z-30"
                onClick={() => setIsLayoutPickerOpen(false)}
              />
              <div className="absolute top-full right-0 z-40 mt-1">
                <VisualLayoutPicker
                  currentLayout={layout}
                  onSelectLayout={(ly) => {
                    onLayoutChange?.(ly);
                    setIsLayoutPickerOpen(false);
                  }}
                  onClose={() => setIsLayoutPickerOpen(false)}
                  theme={theme}
                />
              </div>
            </>
          )}
        </div>

        {/* Quick Save */}
        <button
          onClick={handleSave}
          className={`hidden sm:flex items-center gap-1 px-2.5 py-1 rounded text-xs font-semibold cursor-pointer transition-colors ${
            isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
          }`}
          title="Save Layout State"
        >
          {savedFeedback ? (
            <>
              <Check className="w-3 h-3 text-[#089981]" />
              <span className="text-[#089981]">Saved</span>
            </>
          ) : (
            <>
              <span>Save</span>
              <ChevronDown className="w-3 h-3 text-[#787b86]" />
            </>
          )}
        </button>

        {/* Quick Theme Toggle (Sun / Moon) */}
        <button
          onClick={onToggleTheme}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
          }`}
          title={isLight ? "Switch to Dark Mode" : "Switch to Light Mode"}
        >
          {isLight ? <Moon className="w-3.5 h-3.5" /> : <Sun className="w-3.5 h-3.5" />}
        </button>

        {/* Settings Dialog Button */}
        <button
          onClick={onOpenSettings}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
          }`}
          title="Chart & Terminal Settings"
        >
          <Settings className="w-3.5 h-3.5" />
        </button>

        {/* Fullscreen */}
        <button
          onClick={onFullscreen}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
          }`}
          title="Fullscreen (F11)"
        >
          <Maximize2 className="w-3.5 h-3.5" />
        </button>

        {/* Snapshot */}
        <button
          onClick={onSnapshot}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
          }`}
          title="Take a snapshot (PNG)"
        >
          <Camera className="w-3.5 h-3.5" />
        </button>
      </div>
    </header>
  );
};
