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
  Plus
} from "lucide-react";
import { Timeframe, IndicatorState, ChartLayout } from "@/types";

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
      className={`h-[46px] border-b flex items-center justify-between px-3 select-none text-xs shrink-0 transition-colors ${
        isLight
          ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
          : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Left Segment: Symbol, Interval, Indicators */}
      <div className="flex items-center gap-1.5 h-full">
        {/* Brand / Logo (PIA Logo) */}
        <div className={`flex items-center gap-2 pr-2.5 border-r h-7 mr-1 ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
          <div className="w-6 h-6 rounded-md overflow-hidden flex items-center justify-center shrink-0">
            <Image
              src="/logo.png"
              alt="PIA Logo"
              width={24}
              height={24}
              className="w-full h-full object-contain"
              priority
            />
          </div>
          <span className={`font-black tracking-wider text-[14px] hidden md:inline ${isLight ? "text-[#131722]" : "text-white"}`}>
            PIA
          </span>
        </div>

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

        {/* Timeframe selector */}
        <div className="flex items-center gap-0.5">
          {TIMEFRAMES.map((tf) => (
            <button
              key={tf}
              onClick={() => setTimeframe(tf)}
              className={`px-2 py-1 rounded font-semibold text-xs transition-colors cursor-pointer ${
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
            className={`hidden group-hover:flex absolute top-full left-0 z-30 mt-1 w-36 flex-col rounded border p-1 shadow-xl ${
              isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}
          >
            {(["candlestick", "bar", "line", "area", "heikin_ashi"] as const).map((type) => (
              <button
                key={type}
                onClick={() => onChartTypeChange?.(type)}
                className={`rounded px-2 py-1.5 text-left text-xs capitalize cursor-pointer ${
                  chartType === type
                    ? "text-[#2962ff] font-bold bg-[#2962ff]/10"
                    : isLight
                    ? "text-[#131722] hover:bg-[#f0f3fa]"
                    : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                }`}
              >
                {type === "heikin_ashi" ? "Heikin Ashi" : type}
              </button>
            ))}
          </div>
        </div>

        {/* Indicators */}
        <div className="relative group">
          <button
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded font-medium cursor-pointer transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
            }`}
          >
            <TrendingUp className="w-3.5 h-3.5 text-[#2962ff]" />
            <span>Indicators</span>
          </button>
          <div
            className={`hidden group-hover:flex absolute top-full left-0 z-30 mt-1 w-52 flex-col rounded border p-1 shadow-xl ${
              isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
            }`}
          >
            {([
              ['sma20', 'SMA 20 (Moving Average)'],
              ['ema50', 'EMA 50 (Exponential)'],
              ['bollinger', 'Bollinger Bands (20, 2)'],
              ['rsi', 'RSI (14) Oscillator'],
              ['macd', 'MACD (12, 26, 9)']
            ] as const).map(([id, label]) => (
              <button
                key={id}
                onClick={() => onToggleIndicator?.(id)}
                className={`flex items-center justify-between rounded px-2.5 py-1.5 text-left text-xs cursor-pointer ${
                  isLight ? "text-[#131722] hover:bg-[#f0f3fa]" : "text-[#d1d4dc] hover:bg-[#2a2e39]"
                }`}
              >
                <span>{label}</span>
                <span className={indicators[id] ? "text-[#2962ff] font-bold" : "text-[#787b86]"}>
                  {indicators[id] ? "ON" : "OFF"}
                </span>
              </button>
            ))}
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
