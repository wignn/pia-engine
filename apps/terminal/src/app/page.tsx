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
import { CalendarPanel } from "@/components/CalendarPanel";
import { OrderBookPanel } from "@/components/OrderBookPanel";
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
  const [snapshotTrigger, setSnapshotTrigger] = useState(0);

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

  const handleSelectSymbol = useCallback(
    (item: WatchlistItem) => {
      setPanes((curr) =>
        curr.map((p) => (p.id === activePaneId ? { ...p, symbol: item.symbol } : p))
      );
      setTabs((curr) =>
        curr.map((t) =>
          t.id === activeTabId ? { ...t, symbol: item.symbol, name: item.name } : t
        )
      );
    },
    [activePaneId, activeTabId]
  );

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

  const handleTimeframe = useCallback(
    (tf: Timeframe) => {
      setPanes((curr) =>
        curr.map((p) => (p.id === activePaneId ? { ...p, timeframe: tf } : p))
      );
      setTabs((curr) =>
        curr.map((t) => (t.id === activeTabId ? { ...t, timeframe: tf } : t))
      );
    },
    [activePaneId, activeTabId]
  );

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

  // Pro Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Scoped cleanly away from inputs, textareas, or modals
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName?.toLowerCase();
      if (tag === "input" || tag === "textarea" || tag === "select" || target?.isContentEditable) {
        return;
      }

      // Quick Search Modal: '/'
      if (e.key === "/") {
        e.preventDefault();
        setIsSearchOpen(true);
        return;
      }

      // Drawing Tool Shortcuts
      if (e.key === "Escape" || e.key.toLowerCase() === "v") {
        setActiveTool("cursor");
        return;
      }
      if (e.key.toLowerCase() === "t" && !e.ctrlKey && !e.metaKey) {
        setActiveTool("trendline");
        return;
      }
      if (e.key.toLowerCase() === "h" && !e.ctrlKey && !e.metaKey) {
        setActiveTool("horizontal");
        return;
      }
      if (e.key.toLowerCase() === "f" && !e.ctrlKey && !e.metaKey) {
        setActiveTool("fibonacci");
        return;
      }
      if (e.key.toLowerCase() === "m" && !e.ctrlKey && !e.metaKey) {
        setActiveTool("measure");
        return;
      }

      // Space -> Next Symbol in Watchlist
      if (e.code === "Space") {
        e.preventDefault();
        setWatchlist((list) => {
          const currentIndex = list.findIndex((w) => w.symbol === activePane.symbol);
          const nextIndex = (currentIndex + 1) % list.length;
          const nextItem = list[nextIndex];
          if (nextItem) {
            handleSelectSymbol(nextItem);
          }
          return list;
        });
        return;
      }

      // Quick Timeframe Keys
      if (e.key === "1") handleTimeframe("1m");
      else if (e.key === "5") handleTimeframe("5m");
      else if (e.key === "0") handleTimeframe("15m");
      else if (e.key === "6") handleTimeframe("1h");
      else if (e.key.toLowerCase() === "d" && !e.ctrlKey && !e.metaKey) handleTimeframe("1D");
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activePane.symbol, handleSelectSymbol, handleTimeframe]);

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
        onSnapshot={() => setSnapshotTrigger((c) => c + 1)}
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
                  snapshotTrigger={pane.id === activePaneId ? snapshotTrigger : 0}
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
          {rightSidebarTab === "orderbook" && (
            <OrderBookPanel
              symbol={selectedItem.symbol}
              livePrice={selectedItem.price}
              digits={selectedItem.digits}
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
          {rightSidebarTab === "calendar" && <CalendarPanel />}
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
