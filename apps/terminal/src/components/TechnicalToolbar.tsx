"use client";

import React from "react";
import {
  Camera,
  Maximize2,
  Minimize2,
  PanelLeft,
  Pencil,
  Redo2,
  Undo2,
} from "lucide-react";
import { ChartType, Timeframe } from "@/types";

interface TechnicalToolbarProps {
  timeframe: Timeframe;
  setTimeframe: (timeframe: Timeframe) => void;
  chartType: ChartType;
  onChartTypeChange: (type: ChartType) => void;
  isSidebarOpen: boolean;
  onToggleSidebar: () => void;
  isDrawingToolbarOpen?: boolean;
  onToggleDrawingToolbar?: () => void;
  onSnapshot?: () => void;
  onFullscreen?: () => void;
  isFullscreen?: boolean;
  canUndo?: boolean;
  canRedo?: boolean;
  onUndo?: () => void;
  onRedo?: () => void;
  activeScale?: "Linear" | "Log" | "%";
  onScaleChange?: (scale: "Linear" | "Log" | "%") => void;
  activeRange?: string;
  onRangeSelect?: (range: string) => void;
}

const RANGES = ["1D", "5D", "1M", "3M", "6M", "YTD", "1Y", "3Y", "5Y", "MAX"];
const TIMEFRAMES: Timeframe[] = ["1m", "5m", "15m", "1h", "4h", "1D", "1W"];
const CHART_TYPES: { id: ChartType; label: string }[] = [
  { id: "candlestick", label: "Candles" },
  { id: "hollow", label: "Hollow candles" },
  { id: "heikin_ashi", label: "Heikin Ashi" },
  { id: "bar", label: "OHLC bars" },
  { id: "line", label: "Line" },
  { id: "area", label: "Area" },
];

const selectClassName =
  "h-8 rounded-md border border-border bg-card px-2 text-xs font-medium text-foreground outline-none transition-colors hover:border-primary/40 focus:border-primary";

export const TechnicalToolbar: React.FC<TechnicalToolbarProps> = ({
  timeframe,
  setTimeframe,
  chartType,
  onChartTypeChange,
  isSidebarOpen,
  onToggleSidebar,
  isDrawingToolbarOpen = false,
  onToggleDrawingToolbar,
  onSnapshot,
  onFullscreen,
  isFullscreen = false,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
  activeScale = "Linear",
  onScaleChange,
  activeRange = "1Y",
  onRangeSelect,
}) => (
  <div className="flex h-10 min-h-10 items-center justify-between gap-2 border-b border-border bg-background px-2.5 text-xs sm:px-3">
    <div className="flex min-w-0 items-center gap-1.5 overflow-x-auto no-scrollbar">
      <button
        onClick={onToggleSidebar}
        className={`flex size-8 shrink-0 items-center justify-center rounded-md border transition-colors ${
          isSidebarOpen
            ? "border-primary/40 bg-primary/10 text-primary"
            : "border-border text-muted-foreground hover:bg-muted hover:text-foreground"
        }`}
        title={isSidebarOpen ? "Hide indicators panel" : "Show indicators panel"}
        aria-label={isSidebarOpen ? "Hide indicators panel" : "Show indicators panel"}
      >
        <PanelLeft className="size-4" />
      </button>

      <label className="sr-only" htmlFor="chart-range">Chart range</label>
      <select
        id="chart-range"
        value={activeRange}
        onChange={(event) => onRangeSelect?.(event.target.value)}
        className={`${selectClassName} w-[76px] shrink-0 font-mono`}
      >
        {RANGES.map((range) => <option key={range}>{range}</option>)}
      </select>

      <label className="sr-only" htmlFor="chart-timeframe">Chart timeframe</label>
      <select
        id="chart-timeframe"
        value={timeframe}
        onChange={(event) => setTimeframe(event.target.value as Timeframe)}
        className={`${selectClassName} w-[78px] shrink-0 font-mono`}
      >
        {TIMEFRAMES.map((interval) => <option key={interval}>{interval}</option>)}
      </select>

      <label className="sr-only" htmlFor="chart-type">Chart type</label>
      <select
        id="chart-type"
        value={chartType}
        onChange={(event) => onChartTypeChange(event.target.value as ChartType)}
        className={`${selectClassName} w-[142px] shrink-0`}
      >
        {CHART_TYPES.map((type) => <option key={type.id} value={type.id}>{type.label}</option>)}
      </select>

      <label className="sr-only" htmlFor="chart-scale">Price scale</label>
      <select
        id="chart-scale"
        value={activeScale}
        onChange={(event) => onScaleChange?.(event.target.value as "Linear" | "Log" | "%")}
        className={`${selectClassName} w-[82px] shrink-0`}
      >
        <option>Linear</option>
        <option>Log</option>
        <option>%</option>
      </select>
    </div>

    <div className="flex shrink-0 items-center gap-1">
      {(onUndo || onRedo) && (
        <div className="hidden items-center gap-0.5 border-r border-border pr-1 sm:flex">
          <button onClick={onUndo} disabled={!canUndo} className="flex size-8 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-30" title="Undo (Ctrl+Z)" aria-label="Undo">
            <Undo2 className="size-3.5" />
          </button>
          <button onClick={onRedo} disabled={!canRedo} className="flex size-8 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-30" title="Redo (Ctrl+Y)" aria-label="Redo">
            <Redo2 className="size-3.5" />
          </button>
        </div>
      )}

      {onToggleDrawingToolbar && (
        <button
          onClick={onToggleDrawingToolbar}
          className={`flex size-8 items-center justify-center rounded-md border transition-colors ${
            isDrawingToolbarOpen
              ? "border-primary/40 bg-primary/10 text-primary"
              : "border-border text-muted-foreground hover:bg-muted hover:text-foreground"
          }`}
          title="Drawing tools"
          aria-label="Toggle drawing tools"
        >
          <Pencil className="size-3.5" />
        </button>
      )}

      {onSnapshot && (
        <button onClick={onSnapshot} className="flex size-8 items-center justify-center rounded-md border border-border text-muted-foreground hover:bg-muted hover:text-foreground" title="Capture chart" aria-label="Capture chart">
          <Camera className="size-4" />
        </button>
      )}

      {onFullscreen && (
        <button onClick={onFullscreen} className="flex size-8 items-center justify-center rounded-md border border-border text-muted-foreground hover:bg-muted hover:text-foreground" title={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"} aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}>
          {isFullscreen ? <Minimize2 className="size-4" /> : <Maximize2 className="size-4" />}
        </button>
      )}
    </div>
  </div>
);
