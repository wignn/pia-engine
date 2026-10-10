"use client";

import React, { useState } from "react";
import {
  BarChart3,
  Newspaper,
  Radio,
  Layers,
  Brain,
  Calendar,
  ChevronDown,
  Columns,
  Rows,
  X,
  Maximize2,
  PieChart,
  Globe2,
  TrendingUp,
  ShieldAlert,
  Zap,
  FileText,
  Briefcase,
  LayoutDashboard,
  Map,
  LayoutGrid
} from "lucide-react";
import { ChartArea } from "./ChartArea";
import { useMarketFeed } from "@/lib/useMarketFeed";
import { NewsPanel } from "./NewsPanel";
import { SocialPanel } from "./SocialPanel";
import { OrderBookPanel } from "./OrderBookPanel";
import { MarketIntelligencePanel } from "./MarketIntelligencePanel";
import { CalendarPanel } from "./CalendarPanel";
import { LiveStreamPanel } from "./LiveStreamPanel";
import { OptionsPanel } from "./OptionsPanel";
import { MacroPanel } from "./MacroPanel";
import { YieldsPanel } from "./YieldsPanel";
import { GeoSignalsPanel } from "./GeoSignalsPanel";
import { EnergyPanel } from "./EnergyPanel";
import { SecFilingsPanel } from "./SecFilingsPanel";
import { PaperTradingPanel } from "./PaperTradingPanel";
import { ControlPanelView } from "./controlpanel/ControlPanelView";
import { MacroMapsView } from "./macromaps/MacroMapsView";
import { SuperchartsHub } from "./hub/SuperchartsHub";
import {
  ChartPaneConfig,
  DrawingTool,
  TerminalSettings,
  PaneContentType,
} from "@/types";

interface ChartPaneWrapperProps {
  pane: ChartPaneConfig;
  isActive: boolean;
  onActivate: () => void;
  activeTool: DrawingTool;
  clearDrawingsTrigger?: number;
  snapshotTrigger?: number;
  onSnapshotDone?: () => void;
  onDrawingsCountChange?: (count: number) => void;
  digits: number;
  provider: string;
  onToggleIndicator?: (indicator: keyof ChartPaneConfig["indicators"]) => void;
  isDrawingsHidden?: boolean;
  isDrawingModeLocked?: boolean;
  onDrawingFinished?: () => void;
  onCanUndoRedoChange?: (canUndo: boolean, canRedo: boolean) => void;
  undoTrigger?: number;
  redoTrigger?: number;
  theme?: "dark" | "light";
  settings?: TerminalSettings;
  onChangePaneType?: (type: PaneContentType) => void;
  onSplitHorizontal?: () => void;
  onSplitVertical?: () => void;
  onClosePane?: () => void;
  canClosePane?: boolean;
  showPaneHeader?: boolean;
}

export const ChartPaneWrapper: React.FC<ChartPaneWrapperProps> = ({
  pane,
  isActive,
  onActivate,
  activeTool,
  clearDrawingsTrigger,
  snapshotTrigger,
  onSnapshotDone,
  onDrawingsCountChange,
  digits,
  provider,
  onToggleIndicator,
  isDrawingsHidden,
  isDrawingModeLocked,
  onDrawingFinished,
  onCanUndoRedoChange,
  undoTrigger,
  redoTrigger,
  theme = "dark",
  settings,
  onChangePaneType,
  onSplitHorizontal,
  onSplitVertical,
  onClosePane,
  canClosePane = false,
  showPaneHeader = false,
}) => {
  const [isTypeMenuOpen, setIsTypeMenuOpen] = useState(false);
  const paneType = pane.type || "chart";
  const {
    candles,
    livePrice,
    connected,
    loading,
    loadingOlder,
    hasMoreHistory,
    loadOlder,
  } = useMarketFeed(pane.symbol, pane.timeframe);

  const paneTypes: { id: PaneContentType; label: string; icon: React.ReactNode }[] = [
    { id: "chart", label: "Chart", icon: <BarChart3 className="size-3.5 text-muted-foreground" /> },
    { id: "orderbook", label: "Order book", icon: <Layers className="size-3.5 text-muted-foreground" /> },
    { id: "news", label: "News", icon: <Newspaper className="size-3.5 text-muted-foreground" /> },
    { id: "social", label: "Social pulse", icon: <Radio className="size-3.5 text-muted-foreground" /> },
    { id: "intelligence", label: "Market intelligence", icon: <Brain className="size-3.5 text-muted-foreground" /> },
    { id: "calendar", label: "Economic calendar", icon: <Calendar className="size-3.5 text-muted-foreground" /> },
    { id: "options", label: "Options & GEX", icon: <PieChart className="size-3.5 text-muted-foreground" /> },
    { id: "macro", label: "Macro & central banks", icon: <Globe2 className="size-3.5 text-muted-foreground" /> },
    { id: "yields", label: "Treasury yields", icon: <TrendingUp className="size-3.5 text-muted-foreground" /> },
    { id: "geosignals", label: "Geopolitical signals", icon: <ShieldAlert className="size-3.5 text-muted-foreground" /> },
    { id: "energy", label: "Energy markets", icon: <Zap className="size-3.5 text-muted-foreground" /> },
    { id: "sec", label: "SEC filings", icon: <FileText className="size-3.5 text-muted-foreground" /> },
    { id: "paper", label: "Paper trading", icon: <Briefcase className="size-3.5 text-muted-foreground" /> },
    { id: "controlpanel", label: "Market dashboard", icon: <LayoutDashboard className="size-3.5 text-muted-foreground" /> },
    { id: "macromaps", label: "Macro map", icon: <Map className="size-3.5 text-muted-foreground" /> },
    { id: "hub", label: "Workspace gallery", icon: <LayoutGrid className="size-3.5 text-muted-foreground" /> },
  ];

  const currentTypeMeta = paneTypes.find((t) => t.id === paneType) || paneTypes[0];

  return (
    <div
      onClick={onActivate}
      className={`relative flex h-full w-full flex-col overflow-visible ${isActive ? "z-10" : "z-0"}`}
    >
      {/* Dynamic Pane Header Bar (Custom Arrangement Toolbar) */}
      {showPaneHeader && (
        <div className={`z-20 flex h-8 shrink-0 items-center justify-between border-b border-l-2 border-border bg-card px-2 font-mono text-[11px] ${isActive ? "border-l-primary text-foreground" : "border-l-transparent text-muted-foreground"}`}>
          {/* Left: Content Type Switcher */}
          <div className="relative flex min-w-0 items-center gap-1.5">
            <button
              onClick={(e) => {
                e.stopPropagation();
                setIsTypeMenuOpen((v) => !v);
              }}
              className="flex min-w-0 items-center gap-1 rounded px-1.5 py-0.5 font-mono text-[11px] font-semibold text-foreground transition-colors hover:bg-muted"
              title="Change pane view"
            >
              <span className="truncate">{paneType === "chart" ? pane.symbol : currentTypeMeta.label}</span>
              <ChevronDown className="size-3 shrink-0 text-muted-foreground" />
            </button>
            <span className="font-mono text-[10px] text-muted-foreground">{pane.timeframe}</span>

            {/* Dropdown Menu to change tool in this pane */}
            {isTypeMenuOpen && (
              <>
                <div
                  className="fixed inset-0 z-30"
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsTypeMenuOpen(false);
                  }}
                />
                <div
                  className="absolute left-0 top-full z-40 mt-1 flex max-h-[min(440px,70vh)] w-52 flex-col gap-0.5 overflow-y-auto rounded border border-border bg-card p-1 shadow-xl"
                  onClick={(e) => e.stopPropagation()}
                >
                  <div className="text-[9px] uppercase tracking-wider font-bold px-2 py-1 opacity-50">
                    Switch Pane View
                  </div>
                  {paneTypes.map((t) => (
                    <button
                      key={t.id}
                      onClick={() => {
                        onChangePaneType?.(t.id);
                        setIsTypeMenuOpen(false);
                      }}
                      className={`flex items-center gap-2 rounded px-2 py-1.5 text-left font-sans text-[11px] transition-colors ${paneType === t.id ? "bg-primary/10 font-semibold text-primary" : "text-foreground hover:bg-muted"}`}
                    >
                      {t.icon}
                      <span>{t.label}</span>
                    </button>
                  ))}
                </div>
              </>
            )}
          </div>

          {/* Right: Custom Split & Action Controls */}
          <div className="flex items-center gap-1">
            {/* Split Horizontal Button */}
            {onSplitHorizontal && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onSplitHorizontal();
                }}
                className="rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                title="Split Pane Horizontally (2 Columns)"
              >
                <Columns className="w-3 h-3" />
              </button>
            )}

            {/* Split Vertical Button */}
            {onSplitVertical && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onSplitVertical();
                }}
                className="rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                title="Split Pane Vertically (2 Rows)"
              >
                <Rows className="w-3 h-3" />
              </button>
            )}

            {/* Close Pane Button */}
            {canClosePane && onClosePane && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onClosePane();
                }}
                className="rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-down"
                title="Close this Pane"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>
        </div>
      )}

      {/* Pane Content Rendering */}
      <div className="flex-1 h-full w-full overflow-hidden">
        {paneType === "news" ? (
          <NewsPanel symbol={pane.symbol} theme={theme} />
        ) : paneType === "social" ? (
          <SocialPanel theme={theme} />
        ) : paneType === "orderbook" ? (
          <OrderBookPanel symbol={pane.symbol} livePrice={livePrice} digits={digits} theme={theme} />
        ) : paneType === "intelligence" ? (
          <MarketIntelligencePanel symbol={pane.symbol} theme={theme} />
        ) : paneType === "calendar" ? (
          <CalendarPanel theme={theme} />
        ) : paneType === "live" ? (
          <LiveStreamPanel theme={theme} />
        ) : paneType === "options" ? (
          <OptionsPanel />
        ) : paneType === "macro" ? (
          <MacroPanel />
        ) : paneType === "yields" ? (
          <YieldsPanel />
        ) : paneType === "geosignals" ? (
          <GeoSignalsPanel />
        ) : paneType === "energy" ? (
          <EnergyPanel />
        ) : paneType === "sec" ? (
          <SecFilingsPanel />
        ) : paneType === "paper" ? (
          <PaperTradingPanel />
        ) : paneType === "controlpanel" ? (
          <ControlPanelView />
        ) : paneType === "macromaps" ? (
          <MacroMapsView />
        ) : paneType === "hub" ? (
          <SuperchartsHub />
        ) : (
          <ChartArea
            paneId={pane.id}
            symbol={pane.symbol}
            provider={provider}
            timeframe={pane.timeframe}
            chartType={pane.chartType}
            indicators={pane.indicators}
            activeTool={isActive ? activeTool : "cursor"}
            digits={digits}
            candles={candles}
            livePrice={livePrice}
            connected={connected}
            loading={loading}
            loadingOlder={loadingOlder}
            hasMoreHistory={hasMoreHistory}
            onLoadOlder={loadOlder}
            clearDrawingsTrigger={isActive ? clearDrawingsTrigger : 0}
            snapshotTrigger={isActive ? snapshotTrigger : 0}
            onSnapshotDone={isActive ? onSnapshotDone : undefined}
            onDrawingsCountChange={onDrawingsCountChange}
            onToggleIndicator={isActive ? onToggleIndicator : undefined}
            isDrawingsHidden={isDrawingsHidden}
            isDrawingModeLocked={isDrawingModeLocked}
            onDrawingFinished={isActive ? onDrawingFinished : undefined}
            onCanUndoRedoChange={isActive ? onCanUndoRedoChange : undefined}
            undoTrigger={isActive ? undoTrigger : 0}
            redoTrigger={isActive ? redoTrigger : 0}
            theme={theme}
            settings={settings}
          />
        )}
      </div>
    </div>
  );
};
