"use client";

import React, { useState, useEffect, useCallback, useMemo } from "react";
import { X, ChevronLeft, ChevronRight } from "lucide-react";
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
import { MobileBottomNav } from "@/components/MobileBottomNav";
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
  const [initialSearchQuery, setInitialSearchQuery] = useState("");
  const [rightSidebarTab, setRightSidebarTab] = useState<SidebarTab>("watchlist");
  const [isMobileDrawerOpen, setIsMobileDrawerOpen] = useState(false);

  // Desktop Resizable & Collapsible Sidebar State
  const [sidebarWidth, setSidebarWidth] = useState(330);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [isDraggingSplitter, setIsDraggingSplitter] = useState(false);

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

  // Interactive Drawing Tools & State
  const [activeTool, setActiveTool] = useState<DrawingTool>("cursor");
  const [drawingsCount, setDrawingsCount] = useState(0);
  const [clearDrawingsTrigger, setClearDrawingsTrigger] = useState(0);
  const [snapshotTrigger, setSnapshotTrigger] = useState(0);
  const [isDrawingModeLocked, setIsDrawingModeLocked] = useState(false);
  const [isDrawingsHidden, setIsDrawingsHidden] = useState(false);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);
  const [undoTrigger, setUndoTrigger] = useState(0);
  const [redoTrigger, setRedoTrigger] = useState(0);

  // Visible panes based on current layout
  const visiblePanes = useMemo(() => {
    if (layout === "1x1") return [activePane];
    if (layout === "1x2" || layout === "2x1") return panes.slice(0, 2);
    return panes.slice(0, 4);
  }, [layout, activePane, panes]);

  const gridClass = useMemo(() => {
    switch (layout) {
      case "1x2":
        return "grid grid-cols-1 md:grid-cols-2 gap-0.5 h-full w-full bg-[#1e222d]";
      case "2x1":
        return "grid grid-rows-2 gap-0.5 h-full w-full bg-[#1e222d]";
      case "2x2":
        return "grid grid-cols-1 sm:grid-cols-2 grid-rows-2 gap-0.5 h-full w-full bg-[#1e222d]";
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
      setIsMobileDrawerOpen(false);
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

  const handleToggleIndicator = useCallback(
    (indicator: keyof IndicatorState) => {
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
    },
    [activePaneId]
  );

  const toggleFullscreen = useCallback(async () => {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  }, []);

  // TradingView principle: Clicking active tab toggles panel open/close!
  const handleTabChangeFromDock = (tab: SidebarTab) => {
    if (rightSidebarTab === tab && !isSidebarCollapsed) {
      setIsSidebarCollapsed(true);
    } else {
      setRightSidebarTab(tab);
      setIsMobileDrawerOpen(true);
      setIsSidebarCollapsed(false);
    }
  };

  // Draggable Splitter Handler
  const handleSplitterMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDraggingSplitter(true);
  };

  useEffect(() => {
    if (!isDraggingSplitter) return;

    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = window.innerWidth - 45 - e.clientX;
      setSidebarWidth(Math.max(250, Math.min(540, newWidth)));
      if (isSidebarCollapsed) setIsSidebarCollapsed(false);
    };

    const handleMouseUp = () => {
      setIsDraggingSplitter(false);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDraggingSplitter, isSidebarCollapsed]);

  // Pro Keyboard Shortcuts & Type-to-Search
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
        setInitialSearchQuery("");
        setIsSearchOpen(true);
        return;
      }

      // Toggle Sidebar Collapse: Alt+S
      if (e.altKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        setIsSidebarCollapsed((c) => !c);
        return;
      }

      // Undo / Redo Shortcuts
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
        e.preventDefault();
        if (e.shiftKey) setRedoTrigger((c) => c + 1);
        else setUndoTrigger((c) => c + 1);
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
        e.preventDefault();
        setRedoTrigger((c) => c + 1);
        return;
      }

      // Drawing Tool Shortcuts
      if (e.key === "Escape" || e.key.toLowerCase() === "v") {
        setActiveTool("cursor");
        setIsMobileDrawerOpen(false);
        return;
      }
      if (e.key.toLowerCase() === "t" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        setActiveTool("trendline");
        return;
      }
      if (e.key.toLowerCase() === "h" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        setActiveTool("horizontal");
        return;
      }
      if (e.key.toLowerCase() === "f" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        setActiveTool("fibonacci");
        return;
      }
      if (e.key.toLowerCase() === "m" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        setActiveTool("measure");
        return;
      }

      // TradingView Type-to-Search: Any letter key when cursor tool is active
      if (
        /^[a-zA-Z]$/.test(e.key) &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        activeTool === "cursor"
      ) {
        e.preventDefault();
        setInitialSearchQuery(e.key.toUpperCase());
        setIsSearchOpen(true);
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
  }, [activePane.symbol, handleSelectSymbol, handleTimeframe, activeTool]);

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
        onSearchClick={() => {
          setInitialSearchQuery("");
          setIsSearchOpen(true);
        }}
        indicators={activePane.indicators}
        chartType={activePane.chartType}
        onChartTypeChange={handleChartTypeChange}
        onToggleIndicator={handleToggleIndicator}
        onFullscreen={toggleFullscreen}
        onAlertClick={() => handleTabChangeFromDock("alerts")}
        layout={layout}
        onLayoutChange={setLayout}
        onSnapshot={() => setSnapshotTrigger((c) => c + 1)}
        onToggleSidebar={() => setIsMobileDrawerOpen((prev) => !prev)}
        onUndo={() => setUndoTrigger((c) => c + 1)}
        onRedo={() => setRedoTrigger((c) => c + 1)}
        canUndo={canUndo}
        canRedo={canRedo}
      />

      <ChartTabs
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewTab={handleCreateNewTab}
      />

      <div className="flex-1 flex w-full overflow-hidden relative">
        <LeftToolbar
          activeTool={activeTool}
          setActiveTool={setActiveTool}
          drawingsCount={drawingsCount}
          onClearDrawings={() => setClearDrawingsTrigger((c) => c + 1)}
          isDrawingModeLocked={isDrawingModeLocked}
          onToggleDrawingModeLock={() => setIsDrawingModeLocked((v) => !v)}
          isDrawingsHidden={isDrawingsHidden}
          onToggleHideDrawings={() => setIsDrawingsHidden((v) => !v)}
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
                  onToggleIndicator={handleToggleIndicator}
                  isDrawingsHidden={isDrawingsHidden}
                  isDrawingModeLocked={isDrawingModeLocked}
                  onDrawingFinished={() => setActiveTool("cursor")}
                  onCanUndoRedoChange={(u, r) => {
                    setCanUndo(u);
                    setCanRedo(r);
                  }}
                  undoTrigger={pane.id === activePaneId ? undoTrigger : 0}
                  redoTrigger={pane.id === activePaneId ? redoTrigger : 0}
                />
              );
            })}
          </div>
        </main>

        {/* Desktop Splitter & Collapse Button */}
        <div
          onMouseDown={handleSplitterMouseDown}
          className={`hidden lg:flex relative w-1 hover:w-1.5 cursor-col-resize bg-[#2a2e39] hover:bg-[#2962ff] transition-all items-center justify-center select-none z-20 group shrink-0 ${
            isDraggingSplitter ? "bg-[#2962ff] w-1.5" : ""
          }`}
          title="Drag to resize width"
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsSidebarCollapsed((c) => !c);
            }}
            className="absolute -left-2.5 top-1/2 -translate-y-1/2 w-5 h-7 rounded bg-[#1e222d] border border-[#2a2e39] text-[#787b86] hover:text-white hover:border-[#363a45] flex items-center justify-center shadow-md z-30 transition-all cursor-pointer"
            title={isSidebarCollapsed ? "Expand Panel (Alt+S)" : "Collapse Panel (Alt+S)"}
          >
            {isSidebarCollapsed ? (
              <ChevronLeft className="w-3 h-3" />
            ) : (
              <ChevronRight className="w-3 h-3" />
            )}
          </button>
        </div>

        {/* Mobile Backdrop */}
        {isMobileDrawerOpen && (
          <div
            onClick={() => setIsMobileDrawerOpen(false)}
            className="fixed inset-0 bg-black/60 backdrop-blur-xs z-40 lg:hidden"
          />
        )}

        {/* Right Dock Sidebar */}
        <aside
          style={{ width: isSidebarCollapsed ? 0 : `${sidebarWidth}px` }}
          className={`
            fixed inset-y-0 right-0 z-50 w-[85vw] max-w-[360px] bg-[#1e222d] shadow-2xl transition-transform duration-300 ease-in-out
            lg:static lg:z-auto lg:shadow-none lg:translate-x-0 lg:transition-none
            ${isMobileDrawerOpen ? "translate-x-0" : "translate-x-full lg:translate-x-0"}
            ${isSidebarCollapsed ? "lg:hidden" : "lg:flex"}
            flex flex-col h-full overflow-hidden border-l border-[#2a2e39] shrink-0
          `}
        >
          {/* Mobile Drawer Close Header */}
          <div className="flex items-center justify-between px-3 py-2.5 border-b border-[#2a2e39] lg:hidden bg-[#141722] shrink-0">
            <span className="font-bold text-xs uppercase tracking-wider text-white">
              {rightSidebarTab}
            </span>
            <button
              onClick={() => setIsMobileDrawerOpen(false)}
              className="p-1 rounded text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-colors cursor-pointer"
              title="Close Drawer"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          <div className="flex-1 overflow-hidden">
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
        </aside>

        <RightDock activeTab={rightSidebarTab} setActiveTab={handleTabChangeFromDock} />
      </div>

      {/* Mobile Bottom Navigation (< md) */}
      <MobileBottomNav
        activeTab={rightSidebarTab}
        setActiveTab={(tab) => {
          setRightSidebarTab(tab);
          setIsMobileDrawerOpen(true);
        }}
        isDrawerOpen={isMobileDrawerOpen}
        setIsDrawerOpen={setIsMobileDrawerOpen}
        activeTool={activeTool}
        setActiveTool={setActiveTool}
      />

      <SymbolSearchModal
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
        items={watchlist}
        onSelect={handleSelectSymbol}
        initialQuery={initialSearchQuery}
      />
    </div>
  );
}
