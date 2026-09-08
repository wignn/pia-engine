"use client";

import React, { useState, useEffect, useCallback, useMemo } from "react";
import { TopBar } from "@/components/TopBar";
import { LeftToolbar } from "@/components/LeftToolbar";
import { ChartPaneWrapper } from "@/components/ChartPaneWrapper";
import { RightWatchlist } from "@/components/RightWatchlist";
import { RightDock, SidebarTab } from "@/components/RightDock";
import { NewsPanel } from "@/components/NewsPanel";
import { MarketIntelligencePanel } from "@/components/MarketIntelligencePanel";
import { SocialPanel } from "@/components/SocialPanel";
import { AlertsPanel } from "@/components/AlertsPanel";
import { ChartTabs } from "@/components/ChartTabs";
import { SymbolSearchModal } from "@/components/SymbolSearchModal";
import { INITIAL_WATCHLIST } from "@/lib/constants";
import {
  WatchlistItem,
  Timeframe,
  TabItem,
  ChartType,
  DrawingTool,
  IndicatorState,
  ChartLayout,
  ChartPaneConfig,
} from "@/types";

export default function TerminalPage() {
  const [watchlist, setWatchlist] = useState<WatchlistItem[]>(INITIAL_WATCHLIST);
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [rightSidebarTab, setRightSidebarTab] = useState<SidebarTab>("watchlist");

  // Multi-Chart Grid Layout State
  const [layout, setLayout] = useState<ChartLayout>("1x1");
  const [activePaneId, setActivePaneId] = useState<string>("pane-1");

  const [panes, setPanes] = useState<ChartPaneConfig[]>([
    {
      id: "pane-1",
      symbol: "XAUUSD",
      timeframe: "15m",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
    },
    {
      id: "pane-2",
      symbol: "BTCUSDT",
      timeframe: "1h",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
    },
    {
      id: "pane-3",
      symbol: "SPX",
      timeframe: "1D",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
    },
    {
      id: "pane-4",
      symbol: "DXY",
      timeframe: "1h",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, bollinger: false, rsi: false, macd: false },
    },
  ]);

  const findItem = useCallback(
    (sym: string) =>
      watchlist.find((w) => w.symbol === sym) ??
      INITIAL_WATCHLIST.find((w) => w.symbol === sym) ??
      watchlist[0],
    [watchlist]
  );

  const activePane = useMemo(
    () => panes.find((p) => p.id === activePaneId) || panes[0],
    [panes, activePaneId]
  );

  const selectedItem = findItem(activePane.symbol);

  // Interactive Drawing Tools
  const [activeTool, setActiveTool] = useState<DrawingTool>("cursor");
  const [drawingsCount, setDrawingsCount] = useState(0);
  const [clearDrawingsTrigger, setClearDrawingsTrigger] = useState(0);

  // Visible panes based on current layout
  const visiblePanes = useMemo(() => {
    if (layout === "1x1") return [activePane];
    if (layout === "1x2" || layout === "2x1") return panes.slice(0, 2);
    return panes.slice(0, 4);
  }, [layout, activePane, panes]);

  const gridClass = useMemo(() => {
    switch (layout) {
      case "1x2":
        return "grid grid-cols-2 gap-0.5 h-full w-full bg-[#1e222d]";
      case "2x1":
        return "grid grid-rows-2 gap-0.5 h-full w-full bg-[#1e222d]";
      case "2x2":
        return "grid grid-cols-2 grid-rows-2 gap-0.5 h-full w-full bg-[#1e222d]";
      default:
        return "h-full w-full";
    }
  }, [layout]);

  // Tabbed charts state
  const [tabs, setTabs] = useState<TabItem[]>([
    { id: "tab-1", symbol: "XAUUSD", timeframe: "15m", name: "Gold Spot / U.S. Dollar" },
    { id: "tab-2", symbol: "BTCUSDT", timeframe: "1h", name: "Bitcoin / TetherUS" },
    { id: "tab-3", symbol: "SPX", timeframe: "1D", name: "S&P 500 Index" },
  ]);
  const [activeTabId, setActiveTabId] = useState<string>("tab-1");

  const handleSelectTab = (tabId: string) => {
    setActiveTabId(tabId);
    const t = tabs.find((x) => x.id === tabId);
    if (!t) return;
    setPanes((curr) =>
      curr.map((p) =>
        p.id === activePaneId ? { ...p, symbol: t.symbol, timeframe: t.timeframe } : p
      )
    );
  };

  const handleCloseTab = (tabId: string) => {
    if (tabs.length <= 1) return;
    const remaining = tabs.filter((t) => t.id !== tabId);
    setTabs(remaining);
    if (activeTabId === tabId) handleSelectTab(remaining[remaining.length - 1].id);
  };

  const handleSelectSymbol = (item: WatchlistItem) => {
    setPanes((curr) =>
      curr.map((p) => (p.id === activePaneId ? { ...p, symbol: item.symbol } : p))
    );
    setTabs((curr) =>
      curr.map((t) =>
        t.id === activeTabId ? { ...t, symbol: item.symbol, name: item.name } : t
      )
    );
  };

  const handleCreateNewTab = () => {
    const newId = `tab-${Date.now()}`;
    const item = findItem("ETHUSDT");
    setTabs((curr) => [
      ...curr,
      { id: newId, symbol: item.symbol, timeframe: "15m", name: item.name },
    ]);
    setActiveTabId(newId);
    setPanes((curr) =>
      curr.map((p) => (p.id === activePaneId ? { ...p, symbol: item.symbol, timeframe: "15m" } : p))
    );
  };

  const handleTimeframe = (tf: Timeframe) => {
    setPanes((curr) =>
      curr.map((p) => (p.id === activePaneId ? { ...p, timeframe: tf } : p))
    );
    setTabs((curr) =>
      curr.map((t) => (t.id === activeTabId ? { ...t, timeframe: tf } : t))
    );
  };

  const handleChartTypeChange = (ct: ChartType) => {
    setPanes((curr) =>
      curr.map((p) => (p.id === activePaneId ? { ...p, chartType: ct } : p))
    );
  };

  const handleToggleIndicator = (indicator: keyof IndicatorState) => {
    setPanes((curr) =>
      curr.map((p) =>
        p.id === activePaneId
          ? {
              ...p,
              indicators: {
                ...p.indicators,
                [indicator]: !p.indicators[indicator],
              },
            }
          : p
      )
    );
  };

  const toggleFullscreen = useCallback(async () => {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  }, []);

  // Poll real prices for the whole watchlist every 5s
  useEffect(() => {
    let cancelled = false;
    const pull = async () => {
      try {
        const res = await fetch("/api/market/prices", { cache: "no-store" });
        if (!res.ok) return;
        const data = await res.json();
        if (cancelled || !Array.isArray(data.items)) return;
        const bySym: Record<string, any> = {};
        for (const it of data.items) bySym[String(it.symbol ?? "").toUpperCase()] = it;

        setWatchlist((list) =>
          list.map((w) => {
            const hit = bySym[w.symbol.toUpperCase()];
            if (!hit || typeof hit.price !== "number") return w;
            const change = w.price ? hit.price - (w.price - w.change) : 0;
            return {
              ...w,
              price: hit.price,
              change,
              changePercent: w.price
                ? Number(((change / (hit.price - change)) * 100).toFixed(2))
                : 0,
            };
          })
        );
      } catch {
        /* ignore */
      }
    };
    pull();
    const id = setInterval(pull, 5000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  return (
    <div className="flex flex-col h-full w-full bg-[#131722] overflow-hidden">
      <TopBar
        symbol={selectedItem.symbol}
        timeframe={activePane.timeframe}
        setTimeframe={handleTimeframe}
        price={selectedItem.price}
        change={selectedItem.change}
        changePercent={selectedItem.changePercent}
        digits={selectedItem.digits}
        onSearchClick={() => setIsSearchOpen(true)}
        indicators={activePane.indicators}
        chartType={activePane.chartType}
        onChartTypeChange={handleChartTypeChange}
        onToggleIndicator={handleToggleIndicator}
        onFullscreen={toggleFullscreen}
        onAlertClick={() => setRightSidebarTab("alerts")}
        layout={layout}
        onLayoutChange={setLayout}
      />

      <ChartTabs
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewTab={handleCreateNewTab}
      />

      <div className="flex-1 flex w-full overflow-hidden">
        <LeftToolbar
          activeTool={activeTool}
          setActiveTool={setActiveTool}
          drawingsCount={drawingsCount}
          onClearDrawings={() => setClearDrawingsTrigger((c) => c + 1)}
        />

        {/* Main Grid View */}
        <main className="flex-1 h-full overflow-hidden relative">
          <div className={gridClass}>
            {visiblePanes.map((pane) => {
              const meta = findItem(pane.symbol);
              return (
                <ChartPaneWrapper
                  key={pane.id}
                  pane={pane}
                  isActive={pane.id === activePaneId}
                  onActivate={() => setActivePaneId(pane.id)}
                  activeTool={activeTool}
                  digits={meta.digits}
                  provider={meta.provider}
                  clearDrawingsTrigger={clearDrawingsTrigger}
                  onDrawingsCountChange={setDrawingsCount}
                />
              );
            })}
          </div>
        </main>

        {/* Right Dock Sidebar */}
        <div className="w-[330px] h-full flex flex-col overflow-hidden">
          {rightSidebarTab === "watchlist" && (
            <RightWatchlist
              items={watchlist}
              selectedSymbol={selectedItem.symbol}
              onSelectSymbol={handleSelectSymbol}
            />
          )}
          {rightSidebarTab === "news" && <NewsPanel symbol={selectedItem.symbol} />}
          {rightSidebarTab === "intelligence" && (
            <MarketIntelligencePanel symbol={selectedItem.symbol} />
          )}
          {rightSidebarTab === "social" && <SocialPanel />}
          {rightSidebarTab === "alerts" && (
            <AlertsPanel
              symbol={selectedItem.symbol}
              livePrice={selectedItem.price}
              digits={selectedItem.digits}
            />
          )}
          {rightSidebarTab === "calendar" && (
            <div className="h-full bg-[#1e222d] border-l border-[#2a2e39] p-4 flex flex-col gap-3">
              <span className="font-bold text-white text-sm">Upcoming Macro Events</span>
              <div className="flex flex-col gap-2 text-xs">
                <div className="p-2.5 rounded bg-[#181b27] border border-[#2a2e39]">
                  <div className="flex justify-between font-bold text-[#f23645]">
                    <span>USD Non-Farm Payrolls</span>
                    <span>19:30 UTC</span>
                  </div>
                  <div className="text-[11px] text-[#787b86] mt-1">Forecast: 165K | Previous: 142K</div>
                </div>
                <div className="p-2.5 rounded bg-[#181b27] border border-[#2a2e39]">
                  <div className="flex justify-between font-bold text-[#2962ff]">
                    <span>USD CPI (MoM)</span>
                    <span>Tomorrow</span>
                  </div>
                  <div className="text-[11px] text-[#787b86] mt-1">Forecast: 0.2% | Previous: 0.2%</div>
                </div>
                <div className="p-2.5 rounded bg-[#181b27] border border-[#2a2e39]">
                  <div className="flex justify-between font-bold text-[#f5b942]">
                    <span>FOMC Rate Decision</span>
                    <span>Wed 18:00 UTC</span>
                  </div>
                  <div className="text-[11px] text-[#787b86] mt-1">Target: 5.25% - 5.50%</div>
                </div>
              </div>
            </div>
          )}
        </div>

        <RightDock activeTab={rightSidebarTab} setActiveTab={setRightSidebarTab} />
      </div>

      <SymbolSearchModal
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
        items={watchlist}
        onSelect={handleSelectSymbol}
      />
    </div>
  );
}
