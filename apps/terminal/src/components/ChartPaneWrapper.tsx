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
  Maximize2
} from "lucide-react";
import { ChartArea } from "./ChartArea";
import { useMarketFeed } from "@/lib/useMarketFeed";
import { NewsPanel } from "./NewsPanel";
import { SocialPanel } from "./SocialPanel";
import { OrderBookPanel } from "./OrderBookPanel";
import { MarketIntelligencePanel } from "./MarketIntelligencePanel";
import { CalendarPanel } from "./CalendarPanel";
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
  const isLight = theme === "light";
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
    { id: "chart", label: "Chart View", icon: <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" /> },
    { id: "orderbook", label: "DOM & Order Book", icon: <Layers className="w-3.5 h-3.5 text-[#089981]" /> },
    { id: "news", label: "News Stream", icon: <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" /> },
    { id: "social", label: "Social Pulse", icon: <Radio className="w-3.5 h-3.5 text-[#e040fb]" /> },
    { id: "intelligence", label: "Market Intel", icon: <Brain className="w-3.5 h-3.5 text-[#00e5ff]" /> },
    { id: "calendar", label: "Economic Calendar", icon: <Calendar className="w-3.5 h-3.5 text-[#ff5252]" /> },
  ];

  const currentTypeMeta = paneTypes.find((t) => t.id === paneType) || paneTypes[0];

  return (
    <div
      onClick={onActivate}
      className={`relative w-full h-full flex flex-col overflow-hidden transition-all ${
        isActive
          ? "ring-1 ring-[#2962ff] z-10"
          : isLight
          ? "opacity-95 hover:opacity-100"
          : "opacity-90 hover:opacity-100"
      }`}
    >
      {/* Dynamic Pane Header Bar (Custom Arrangement Toolbar) */}
      {showPaneHeader && (
        <div
          className={`h-7 px-2 border-b flex items-center justify-between text-[11px] font-mono shrink-0 select-none z-20 ${
            isActive
              ? isLight
                ? "bg-[#e8f0fe] border-[#2962ff]/40 text-[#131722]"
                : "bg-[#1f293d] border-[#2962ff]/50 text-white"
              : isLight
              ? "bg-[#f8f9fc] border-[#e0e3eb] text-[#5d606b]"
              : "bg-[#181b24] border-[#2a2e39] text-[#787b86]"
          }`}
        >
          {/* Left: Content Type Switcher */}
          <div className="relative flex items-center gap-1.5">
            <button
              onClick={(e) => {
                e.stopPropagation();
                setIsTypeMenuOpen((v) => !v);
              }}
              className={`flex items-center gap-1 px-1.5 py-0.5 rounded cursor-pointer transition-colors ${
                isLight ? "hover:bg-[#e0e3eb] text-[#131722]" : "hover:bg-[#2a2e39] text-white"
              }`}
            >
              {currentTypeMeta.icon}
              <span className="font-bold text-[10px]">{currentTypeMeta.label}</span>
              <ChevronDown className="w-3 h-3 opacity-60" />
            </button>

            {/* Symbol Tag */}
            <span
              className={`text-[10px] font-bold px-1.5 py-0.2 rounded ${
                isLight ? "bg-[#ffffff] text-[#131722] border border-[#e0e3eb]" : "bg-[#141722] text-[#d1d4dc] border border-[#2a2e39]"
              }`}
            >
              {pane.symbol}
            </span>

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
                  className={`absolute top-full left-0 mt-1 w-44 rounded-lg border shadow-xl p-1 z-40 flex flex-col gap-0.5 ${
                    isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
                  }`}
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
                      className={`flex items-center gap-2 px-2 py-1.5 rounded text-[11px] font-sans text-left transition-colors cursor-pointer ${
                        paneType === t.id
                          ? "bg-[#2962ff]/10 text-[#2962ff] font-bold"
                          : isLight
                          ? "hover:bg-[#f0f3fa] text-[#131722]"
                          : "hover:bg-[#2a2e39] text-[#d1d4dc]"
                      }`}
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
                className={`p-1 rounded transition-colors cursor-pointer ${
                  isLight ? "hover:bg-[#e0e3eb] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
                }`}
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
                className={`p-1 rounded transition-colors cursor-pointer ${
                  isLight ? "hover:bg-[#e0e3eb] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
                }`}
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
                className={`p-1 rounded transition-colors cursor-pointer ${
                  isLight ? "hover:bg-[#e0e3eb] text-[#5d606b] hover:text-[#f23645]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#f23645]"
                }`}
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
            onDrawingsCountChange={isActive ? onDrawingsCountChange : undefined}
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
