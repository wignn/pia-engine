"use client";

import React, { useState } from "react";
import { ChartArea } from "./ChartArea";
import { NewsPanel } from "./NewsPanel";
import { SocialPanel } from "./SocialPanel";
import { OrderBookPanel } from "./OrderBookPanel";
import { MarketIntelligencePanel } from "./MarketIntelligencePanel";
import { CalendarPanel } from "./CalendarPanel";
import { useMarketFeed } from "@/lib/useMarketFeed";
import { ChartPaneConfig, DrawingTool, TerminalSettings, PaneContentType } from "@/types";
import { BarChart3, Newspaper, Radio, Layers, Brain, Calendar, ChevronDown } from "lucide-react";

interface ChartPaneWrapperProps {
  pane: ChartPaneConfig;
  isActive: boolean;
  onActivate: () => void;
  activeTool: DrawingTool;
  clearDrawingsTrigger?: number;
  snapshotTrigger?: number;
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
  showPaneHeader?: boolean;
}

export const ChartPaneWrapper: React.FC<ChartPaneWrapperProps> = ({
  pane,
  isActive,
  onActivate,
  activeTool,
  clearDrawingsTrigger = 0,
  snapshotTrigger = 0,
  onDrawingsCountChange,
  digits,
  provider,
  onToggleIndicator,
  isDrawingsHidden = false,
  isDrawingModeLocked = false,
  onDrawingFinished,
  onCanUndoRedoChange,
  undoTrigger = 0,
  redoTrigger = 0,
  theme = "dark",
  settings,
  onChangePaneType,
  showPaneHeader = false,
}) => {
  const {
    candles,
    livePrice,
    connected,
    loading,
    loadingOlder,
    hasMoreHistory,
    loadOlder,
  } = useMarketFeed(pane.symbol, pane.timeframe);

  const [isTypeDropdownOpen, setIsTypeDropdownOpen] = useState(false);
  const isLight = theme === "light";
  const currentType: PaneContentType = pane.type || "chart";

  const renderContent = () => {
    switch (currentType) {
      case "news":
        return (
          <div className="h-full w-full overflow-hidden bg-[#1e222d]">
            <NewsPanel symbol={pane.symbol} />
          </div>
        );
      case "social":
        return (
          <div className="h-full w-full overflow-hidden bg-[#1e222d]">
            <SocialPanel />
          </div>
        );
      case "orderbook":
        return (
          <div className="h-full w-full overflow-hidden bg-[#1e222d]">
            <OrderBookPanel
              symbol={pane.symbol}
              livePrice={livePrice ?? 0}
              digits={digits}
            />
          </div>
        );
      case "intelligence":
        return (
          <div className="h-full w-full overflow-hidden bg-[#1e222d]">
            <MarketIntelligencePanel symbol={pane.symbol} />
          </div>
        );
      case "calendar":
        return (
          <div className="h-full w-full overflow-hidden bg-[#1e222d]">
            <CalendarPanel />
          </div>
        );
      default:
        return (
          <ChartArea
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
            onDrawingsCountChange={isActive ? onDrawingsCountChange : undefined}
            clearDrawingsTrigger={isActive ? clearDrawingsTrigger : 0}
            snapshotTrigger={isActive ? snapshotTrigger : 0}
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
        );
    }
  };

  return (
    <div
      onClick={onActivate}
      className={`relative h-full w-full overflow-hidden flex flex-col transition-all ${
        isActive
          ? "ring-1 ring-[#2962ff] shadow-sm z-10"
          : isLight
          ? "hover:ring-1 hover:ring-[#b2b5be]"
          : "hover:ring-1 hover:ring-[#363a45]"
      }`}
    >
      {/* Optional Pane Switcher Header Bar for Multi-Pane Grids */}
      {showPaneHeader && (
        <div
          className={`h-7 px-2 border-b flex items-center justify-between text-xs select-none shrink-0 z-20 ${
            isLight
              ? "bg-[#f0f3fa] border-[#e0e3eb] text-[#131722]"
              : "bg-[#181b27] border-[#2a2e39] text-[#d1d4dc]"
          }`}
        >
          {/* Pane Type Dropdown Selector */}
          <div className="relative">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                setIsTypeDropdownOpen((v) => !v);
              }}
              className="flex items-center gap-1.5 px-2 py-0.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] font-bold text-[11px] cursor-pointer"
            >
              {currentType === "chart" && <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" />}
              {currentType === "news" && <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" />}
              {currentType === "social" && <Radio className="w-3.5 h-3.5 text-[#00b4d8]" />}
              {currentType === "orderbook" && <Layers className="w-3.5 h-3.5 text-[#089981]" />}
              {currentType === "intelligence" && <Brain className="w-3.5 h-3.5 text-[#a855f7]" />}
              {currentType === "calendar" && <Calendar className="w-3.5 h-3.5 text-[#f23645]" />}
              <span className="capitalize">{currentType}</span>
              <ChevronDown className="w-2.5 h-2.5 opacity-60" />
            </button>

            {isTypeDropdownOpen && (
              <>
                <div
                  className="fixed inset-0 z-30"
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsTypeDropdownOpen(false);
                  }}
                />
                <div
                  className={`absolute top-full left-0 z-40 mt-1 w-44 rounded-lg border shadow-xl p-1 flex flex-col gap-0.5 text-xs select-none ${
                    isLight
                      ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
                      : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
                  }`}
                  onClick={(e) => e.stopPropagation()}
                >
                  {(
                    [
                      { id: "chart", label: "Chart View", icon: <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" /> },
                      { id: "news", label: "News Headlines", icon: <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" /> },
                      { id: "social", label: "Social Pulse", icon: <Radio className="w-3.5 h-3.5 text-[#00b4d8]" /> },
                      { id: "orderbook", label: "Order Book & DOM", icon: <Layers className="w-3.5 h-3.5 text-[#089981]" /> },
                      { id: "intelligence", label: "Market Intel", icon: <Brain className="w-3.5 h-3.5 text-[#a855f7]" /> },
                      { id: "calendar", label: "Economic Calendar", icon: <Calendar className="w-3.5 h-3.5 text-[#f23645]" /> },
                    ] as const
                  ).map((item) => (
                    <button
                      key={item.id}
                      onClick={() => {
                        onChangePaneType?.(item.id);
                        setIsTypeDropdownOpen(false);
                      }}
                      className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
                    >
                      {item.icon}
                      <span>{item.label}</span>
                    </button>
                  ))}
                </div>
              </>
            )}
          </div>

          <div className="flex items-center gap-2">
            <span className="font-mono font-bold text-[11px]">{pane.symbol}</span>
            {currentType === "chart" && (
              <span className="font-mono text-[10px] opacity-60">{pane.timeframe}</span>
            )}
            {isActive && (
              <span className="w-1.5 h-1.5 rounded-full bg-[#2962ff]" title="Active Focused Pane" />
            )}
          </div>
        </div>
      )}

      {/* Pane Content */}
      <div className="flex-1 overflow-hidden relative">{renderContent()}</div>
    </div>
  );
};
