"use client";

import React, { useState } from "react";
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
  Plus
} from "lucide-react";
import { Timeframe, IndicatorState, ChartLayout } from "@/types";

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
}) => {
  const isPositive = change >= 0;
  const [savedFeedback, setSavedFeedback] = useState(false);

  const handleSave = () => {
    try {
      localStorage.setItem("atlsd_terminal_saved_layout", JSON.stringify({ symbol, timeframe, chartType, layout }));
      setSavedFeedback(true);
      setTimeout(() => setSavedFeedback(false), 2000);
    } catch {
      // ignore
    }
  };

  return (
    <header className="h-[46px] bg-[#1e222d] border-b border-[#2a2e39] flex items-center justify-between px-3 text-[#d1d4dc] select-none text-xs shrink-0">
      {/* Left Segment: Symbol, Interval, Indicators */}
      <div className="flex items-center gap-1.5 h-full">
        {/* Brand / Logo */}
        <div className="flex items-center gap-2 pr-2 border-r border-[#2a2e39] h-6 mr-1">
          <div className="w-6 h-6 rounded bg-[#2962ff] flex items-center justify-center font-black text-white text-xs shadow-sm">
            TV
          </div>
          <span className="font-bold text-white tracking-wider text-[13px] hidden md:inline">ATLSD</span>
        </div>

        {/* Symbol Search Button */}
        <button
          onClick={onSearchClick}
          className="flex items-center gap-2 px-2.5 py-1 rounded hover:bg-[#2a2e39] transition-colors border border-transparent hover:border-[#363a45] cursor-pointer"
        >
          <Search className="w-3.5 h-3.5 text-[#787b86]" />
          <span className="font-bold text-white text-sm">{symbol}</span>
          <span className="text-[10px] text-[#787b86] font-mono uppercase bg-[#131722] px-1.5 py-0.5 rounded">OANDA</span>
        </button>

        <div className="h-4 w-px bg-[#2a2e39] mx-1" />

        {/* Timeframe selector */}
        <div className="flex items-center gap-0.5">
          {TIMEFRAMES.map((tf) => (
            <button
              key={tf}
              onClick={() => setTimeframe(tf)}
              className={`px-2 py-1 rounded font-semibold text-xs transition-colors cursor-pointer ${
                timeframe === tf
                  ? "text-[#2962ff] bg-[#2962ff]/10 font-bold"
                  : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
              }`}
            >
              {tf}
            </button>
          ))}
        </div>

        <div className="h-4 w-px bg-[#2a2e39] mx-1" />

        {/* Chart Style (Candles) */}
        <div className="relative group">
          <button className="flex items-center gap-1 px-2 py-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] cursor-pointer">
            <BarChart2 className="w-3.5 h-3.5" />
            <span className="hidden lg:inline text-xs font-medium">
              {chartType === "heikin_ashi" ? "Heikin Ashi" : chartType[0].toUpperCase() + chartType.slice(1)}
            </span>
            <ChevronDown className="w-3 h-3" />
          </button>
          <div className="hidden group-hover:flex absolute top-full left-0 z-30 mt-1 w-36 flex-col rounded border border-[#2a2e39] bg-[#1e222d] p-1 shadow-xl">
            {(["candlestick", "bar", "line", "area", "heikin_ashi"] as const).map((type) => (
              <button
                key={type}
                onClick={() => onChartTypeChange?.(type)}
                className={`rounded px-2 py-1.5 text-left text-xs capitalize hover:bg-[#2a2e39] cursor-pointer ${
                  chartType === type ? "text-[#2962ff] font-bold" : "text-[#d1d4dc]"
                }`}
              >
                {type === "heikin_ashi" ? "Heikin Ashi" : type}
              </button>
            ))}
          </div>
        </div>

        {/* Indicators */}
        <div className="relative group">
          <button className="flex items-center gap-1.5 px-2.5 py-1 rounded hover:bg-[#2a2e39] text-[#d1d4dc] font-medium cursor-pointer">
            <TrendingUp className="w-3.5 h-3.5 text-[#2962ff]" />
            <span>Indicators</span>
          </button>
          <div className="hidden group-hover:flex absolute top-full left-0 z-30 mt-1 w-52 flex-col rounded border border-[#2a2e39] bg-[#1e222d] p-1 shadow-xl">
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
                className="flex items-center justify-between rounded px-2.5 py-1.5 text-left text-xs text-[#d1d4dc] hover:bg-[#2a2e39] cursor-pointer"
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
          className="hidden xl:flex items-center gap-1.5 px-2 py-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors cursor-pointer"
        >
          <Clock className="w-3.5 h-3.5" />
          <span>Alert</span>
        </button>

        {/* Live Ticker Header stats */}
        <div className="hidden 2xl:flex items-center gap-3 pl-3 border-l border-[#2a2e39]">
          <div className="flex items-baseline gap-1.5">
            <span className="font-mono text-sm font-bold text-white">
              {price.toFixed(digits)}
            </span>
            <span className={`font-mono text-xs font-semibold ${isPositive ? "text-[#089981]" : "text-[#f23645]"}`}>
              {isPositive ? "+" : ""}{change.toFixed(digits)} ({isPositive ? "+" : ""}{changePercent.toFixed(2)}%)
            </span>
          </div>
        </div>
      </div>

      {/* Right Segment: Layout, Settings, Fullscreen, Snapshot */}
      <div className="flex items-center gap-1">
        {/* Interactive Undo / Redo */}
        <div className="hidden md:flex items-center gap-0.5 pr-2 border-r border-[#2a2e39]">
          <button
            onClick={onUndo}
            disabled={!canUndo}
            className={`p-1.5 rounded transition-colors cursor-pointer ${
              canUndo ? "text-[#d1d4dc] hover:bg-[#2a2e39]" : "text-[#787b86]/40 cursor-not-allowed"
            }`}
            title="Undo Drawing (Ctrl+Z)"
          >
            <Undo2 className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={onRedo}
            disabled={!canRedo}
            className={`p-1.5 rounded transition-colors cursor-pointer ${
              canRedo ? "text-[#d1d4dc] hover:bg-[#2a2e39]" : "text-[#787b86]/40 cursor-not-allowed"
            }`}
            title="Redo Drawing (Ctrl+Y)"
          >
            <Redo2 className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Layout Mode */}
        <div className="relative group">
          <button className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors cursor-pointer" title={`Select Grid Layout (${layout})`}>
            <LayoutGrid className="w-3.5 h-3.5" />
          </button>
          <div className="hidden group-hover:flex absolute top-full right-0 z-30 mt-1 w-36 flex-col rounded border border-[#2a2e39] bg-[#1e222d] p-1 shadow-xl">
            {([
              ["1x1", "Single (1x1)"],
              ["1x2", "Dual Horizontal (1x2)"],
              ["2x1", "Dual Vertical (2x1)"],
              ["2x2", "Quad Grid (2x2)"],
            ] as const).map(([id, label]) => (
              <button
                key={id}
                onClick={() => onLayoutChange?.(id)}
                className={`rounded px-2 py-1.5 text-left text-xs hover:bg-[#2a2e39] cursor-pointer ${
                  layout === id ? "text-[#2962ff] font-bold" : "text-[#d1d4dc]"
                }`}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        {/* Quick Save with Live Feedback */}
        <button
          onClick={handleSave}
          className="hidden sm:flex items-center gap-1 px-2.5 py-1 rounded text-xs font-semibold text-[#d1d4dc] hover:bg-[#2a2e39] cursor-pointer transition-colors"
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

        {/* Fullscreen */}
        <button
          onClick={onFullscreen}
          className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] cursor-pointer transition-colors"
          title="Fullscreen (F11)"
        >
          <Maximize2 className="w-3.5 h-3.5" />
        </button>

        {/* Snapshot */}
        <button
          onClick={onSnapshot}
          className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors cursor-pointer"
          title="Take a snapshot (PNG)"
        >
          <Camera className="w-3.5 h-3.5" />
        </button>
      </div>
    </header>
  );
};
