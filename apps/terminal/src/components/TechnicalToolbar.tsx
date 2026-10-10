"use client";

import {
  Camera,
  Maximize2,
  Minimize2,
  Pencil,
  Redo2,
  SlidersHorizontal,
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
  indicatorsCount?: number;
  isDrawingToolbarOpen?: boolean;
  onToggleDrawingToolbar?: () => void;
  onSnapshot?: () => void;
  onFullscreen?: () => void;
  isFullscreen?: boolean;
  canUndo?: boolean;
  canRedo?: boolean;
  onUndo?: () => void;
  onRedo?: () => void;
}

const TIMEFRAMES: Timeframe[] = ["1m", "5m", "15m", "1h", "4h", "1D", "1W"];
const CHART_TYPES: { id: ChartType; label: string }[] = [
  { id: "candlestick", label: "Candles" },
  { id: "hollow", label: "Hollow" },
  { id: "heikin_ashi", label: "Heikin Ashi" },
  { id: "bar", label: "OHLC bars" },
  { id: "line", label: "Line" },
  { id: "area", label: "Area" },
];

const selectClass = "h-8 rounded border border-border bg-background px-2 text-[11px] font-medium text-foreground outline-none hover:border-primary/60";
const iconButton = "flex size-8 shrink-0 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-35";

export function TechnicalToolbar({
  timeframe,
  setTimeframe,
  chartType,
  onChartTypeChange,
  isSidebarOpen,
  onToggleSidebar,
  indicatorsCount = 0,
  isDrawingToolbarOpen = false,
  onToggleDrawingToolbar,
  onSnapshot,
  onFullscreen,
  isFullscreen = false,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
}: TechnicalToolbarProps) {
  return (
    <div className="flex min-h-[42px] flex-wrap items-center gap-1.5 border-b border-border bg-card px-3 py-1 lg:h-[42px] lg:flex-nowrap lg:gap-2 lg:py-0 sm:px-4">
      <select aria-label="Chart interval" value={timeframe} onChange={(event) => setTimeframe(event.target.value as Timeframe)} className={`${selectClass} w-[68px] shrink-0 font-mono`}>
        {TIMEFRAMES.map((interval) => <option key={interval}>{interval}</option>)}
      </select>
      <select aria-label="Chart type" value={chartType} onChange={(event) => onChartTypeChange(event.target.value as ChartType)} className={`${selectClass} w-[112px] shrink-0`}>
        {CHART_TYPES.map((type) => <option key={type.id} value={type.id}>{type.label}</option>)}
      </select>
      <span className="h-5 w-px shrink-0 bg-border" />
      <button type="button" onClick={onToggleSidebar} className={`flex h-8 shrink-0 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium transition-colors ${isSidebarOpen ? "bg-primary/15 text-primary" : "text-muted-foreground hover:bg-muted hover:text-foreground"}`} aria-pressed={isSidebarOpen} title="Indicators">
        <SlidersHorizontal className="size-3.5" /><span>Indicators</span>{indicatorsCount > 0 && <span className="font-mono text-[10px] opacity-70">{indicatorsCount}</span>}
      </button>
      {onToggleDrawingToolbar && (
        <button type="button" onClick={onToggleDrawingToolbar} className={`flex h-8 shrink-0 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium transition-colors ${isDrawingToolbarOpen ? "bg-primary/15 text-primary" : "text-muted-foreground hover:bg-muted hover:text-foreground"}`} aria-pressed={isDrawingToolbarOpen} title="Drawing tools">
          <Pencil className="size-3.5" /><span>Draw</span>
        </button>
      )}
      <div className="ml-auto flex shrink-0 items-center gap-0.5 border-l border-border pl-2">
        <button type="button" onClick={onUndo} disabled={!canUndo} className={iconButton} title="Undo" aria-label="Undo"><Undo2 className="size-4" /></button>
        <button type="button" onClick={onRedo} disabled={!canRedo} className={iconButton} title="Redo" aria-label="Redo"><Redo2 className="size-4" /></button>
        <button type="button" onClick={onSnapshot} className={iconButton} title="Capture chart" aria-label="Capture chart"><Camera className="size-4" /></button>
        <button type="button" onClick={onFullscreen} className={iconButton} title={isFullscreen ? "Exit fullscreen" : "Fullscreen"} aria-label="Toggle fullscreen">
          {isFullscreen ? <Minimize2 className="size-4" /> : <Maximize2 className="size-4" />}
        </button>
      </div>
    </div>
  );
}
