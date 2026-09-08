"use client";

import React from "react";
import { ChartArea } from "./ChartArea";
import { useMarketFeed } from "@/lib/useMarketFeed";
import { ChartPaneConfig, DrawingTool } from "@/types";

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

  return (
    <div
      onClick={onActivate}
      className={`relative h-full w-full overflow-hidden transition-all ${
        isActive
          ? "ring-1 ring-[#2962ff] shadow-sm z-10"
          : "hover:ring-1 hover:ring-[#363a45]"
      }`}
    >
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
      />
    </div>
  );
};
