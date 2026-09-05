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
  Plus
} from "lucide-react";
import { Timeframe } from "@/types";

interface TopBarProps {
  symbol: string;
  timeframe: Timeframe;
  setTimeframe: (tf: Timeframe) => void;
  price: number;
  change: number;
  changePercent: number;
  digits: number;
  onSearchClick: () => void;
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
  onSearchClick
}) => {
  const isPositive = change >= 0;

  return (
    <header className="h-[46px] bg-[#1e222d] border-b border-[#2a2e39] flex items-center justify-between px-3 text-[#d1d4dc] select-none text-xs">
      {/* Left Segment: Symbol, Interval, Indicators */}
      <div className="flex items-center gap-1.5 h-full">
        {/* Brand / Logo */}
        <div className="flex items-center gap-2 pr-2 border-r border-[#2a2e39] h-6 mr-1">
          <div className="w-6 h-6 rounded bg-[#2962ff] flex items-center justify-center font-black text-white text-xs">
            TV
          </div>
          <span className="font-bold text-white tracking-wider text-[13px] hidden md:inline">ATLSD</span>
        </div>

        {/* Symbol Search Button */}
        <button
          onClick={onSearchClick}
          className="flex items-center gap-2 px-2.5 py-1 rounded hover:bg-[#2a2e39] transition-colors border border-transparent hover:border-[#363a45]"
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
              className={`px-2 py-1 rounded font-semibold text-xs transition-colors ${
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
        <button className="flex items-center gap-1 px-2 py-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]">
          <BarChart2 className="w-3.5 h-3.5" />
          <span className="hidden lg:inline text-xs font-medium">Candles</span>
          <ChevronDown className="w-3 h-3" />
        </button>

        {/* Indicators */}
        <button className="flex items-center gap-1.5 px-2.5 py-1 rounded hover:bg-[#2a2e39] text-[#d1d4dc] font-medium">
          <TrendingUp className="w-3.5 h-3.5 text-[#2962ff]" />
          <span>Indicators</span>
        </button>

        {/* Templates / Alerts */}
        <button className="hidden xl:flex items-center gap-1.5 px-2 py-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]">
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

      {/* Right Segment: Layout, Settings, Fullscreen, Publish */}
      <div className="flex items-center gap-1">
        {/* Undo / Redo */}
        <div className="hidden md:flex items-center gap-0.5 pr-2 border-r border-[#2a2e39]">
          <button className="p-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Undo">
            <Undo2 className="w-3.5 h-3.5" />
          </button>
          <button className="p-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Redo">
            <Redo2 className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Layout Mode */}
        <button className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Select Layout">
          <LayoutGrid className="w-3.5 h-3.5" />
        </button>

        {/* Quick Save */}
        <button className="hidden sm:flex items-center gap-1 px-2.5 py-1 rounded text-xs font-semibold text-[#d1d4dc] hover:bg-[#2a2e39]">
          <span>Save</span>
          <ChevronDown className="w-3 h-3 text-[#787b86]" />
        </button>

        {/* Settings */}
        <button className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Chart Settings">
          <Settings className="w-3.5 h-3.5" />
        </button>

        {/* Fullscreen */}
        <button className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Fullscreen">
          <Maximize2 className="w-3.5 h-3.5" />
        </button>

        {/* Snapshot */}
        <button className="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Take a snapshot">
          <Camera className="w-3.5 h-3.5" />
        </button>

        {/* Publish / Action button */}
        <button className="ml-1 px-3 py-1 rounded bg-[#2962ff] text-white font-bold text-xs hover:bg-[#1e53e5] transition-colors shadow-sm">
          Publish
        </button>
      </div>
    </header>
  );
};
