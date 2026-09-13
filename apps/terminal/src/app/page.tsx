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
import { SettingsModal } from "@/components/SettingsModal";
import { IndicatorSettingsModal } from "@/components/IndicatorSettingsModal";
import { INITIAL_WATCHLIST } from "@/lib/constants";
import { resolveInstrument } from "@/lib/instruments";
import {
  WatchlistItem,
  Timeframe,
  TabItem,
  TabContentType,
  PaneContentType,
  ChartType,
  DrawingTool,
  IndicatorState,
  ChartLayout,
  ChartPaneConfig,
  TerminalSettings,
  IndicatorParameters,
  DEFAULT_INDICATOR_PARAMS,
} from "@/types";

function getGridClass(layout: ChartLayout, isLight: boolean): string {
  const borderColor = isLight ? "bg-[#e0e3eb]" : "bg-[#1e222d]";
  switch (layout) {
    case "1x2":
      return `grid grid-cols-1 md:grid-cols-2 gap-0.5 h-full w-full ${borderColor}`;
    case "2x1":
      return `grid grid-rows-2 gap-0.5 h-full w-full ${borderColor}`;
    case "1x3":
      return `grid grid-cols-1 lg:grid-cols-3 gap-0.5 h-full w-full ${borderColor}`;
    case "3x1":
      return `grid grid-rows-3 gap-0.5 h-full w-full ${borderColor}`;
    case "1L-2R":
    case "2L-1R":
      return `grid grid-cols-1 md:grid-cols-2 grid-rows-2 gap-0.5 h-full w-full ${borderColor}`;
    case "1T-2B":
    case "2T-1B":
      return `grid grid-cols-2 grid-rows-2 gap-0.5 h-full w-full ${borderColor}`;
    case "2x2":
      return `grid grid-cols-1 sm:grid-cols-2 grid-rows-2 gap-0.5 h-full w-full ${borderColor}`;
    case "3x2":
      return `grid grid-cols-1 sm:grid-cols-3 grid-rows-2 gap-0.5 h-full w-full ${borderColor}`;
    default:
      return "h-full w-full";
  }
}

function getPaneSpanClass(layout: ChartLayout, index: number): string {
  if (layout === "1L-2R") {
    if (index === 0) return "col-span-1 row-span-2 h-full w-full overflow-hidden";
    return "col-span-1 row-span-1 h-full w-full overflow-hidden";
  }
  if (layout === "2L-1R") {
    if (index === 2) return "col-span-1 row-span-2 h-full w-full overflow-hidden";
    return "col-span-1 row-span-1 h-full w-full overflow-hidden";
  }
  if (layout === "1T-2B") {
    if (index === 0) return "col-span-2 row-span-1 h-full w-full overflow-hidden";
    return "col-span-1 row-span-1 h-full w-full overflow-hidden";
  }
  if (layout === "2T-1B") {
    if (index === 2) return "col-span-2 row-span-1 h-full w-full overflow-hidden";
    return "col-span-1 row-span-1 h-full w-full overflow-hidden";
  }
  return "h-full w-full overflow-hidden";
}

export default function TerminalPage() {
  const [watchlist, setWatchlist] = useState<WatchlistItem[]>(INITIAL_WATCHLIST);
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isIndicatorSettingsOpen, setIsIndicatorSettingsOpen] = useState(false);
  const [initialSearchQuery, setInitialSearchQuery] = useState("");
  const [rightSidebarTab, setRightSidebarTab] = useState<SidebarTab>("watchlist");
  const [isMobileDrawerOpen, setIsMobileDrawerOpen] = useState(false);

  // Terminal Settings State (Dark / Light Theme, Candle Colors, Audio Alerts, Indicator Configs)
  const [settings, setSettings] = useState<TerminalSettings>({
    theme: "dark",
    upColor: "#089981",
    downColor: "#f23645",
    gridVisible: true,
    timezone: "UTC",
    audioAlerts: true,
    defaultTimeframe: "15m",
    syncCrosshair: true,
    syncTime: true,
    indicatorParams: DEFAULT_INDICATOR_PARAMS,
  });

  const handleSaveSettings = (newSettings: TerminalSettings) => {
    setSettings(newSettings);
    try {
      localStorage.setItem("atlsd_terminal_settings", JSON.stringify(newSettings));
    } catch {
      // ignore
    }
  };

  const handleToggleTheme = () => {
    const nextTheme = settings.theme === "light" ? "dark" : "light";
    handleSaveSettings({ ...settings, theme: nextTheme });
  };

  const handleToggleSyncCrosshair = () => {
    const next = settings.syncCrosshair === false;
    handleSaveSettings({ ...settings, syncCrosshair: next });
  };

  const handleToggleSyncTime = () => {
    const next = settings.syncTime === false;
    handleSaveSettings({ ...settings, syncTime: next });
  };

  const handleSaveIndicatorParams = (params: IndicatorParameters) => {
    handleSaveSettings({ ...settings, indicatorParams: params });
  };

  const isLight = settings.theme === "light";

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
      type: "chart",
      symbol: "XAUUSD",
      timeframe: "15m",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, vwap: false, bollinger: false, rsi: false, macd: false, atr: false },
    },
    {
      id: "pane-2",
      type: "chart",
      symbol: "BTCUSDT",
      timeframe: "1h",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, vwap: false, bollinger: false, rsi: false, macd: false, atr: false },
    },
    {
      id: "pane-3",
      type: "chart",
      symbol: "SPX",
      timeframe: "1D",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, vwap: false, bollinger: false, rsi: false, macd: false, atr: false },
    },
    {
      id: "pane-4",
      type: "chart",
      symbol: "DXY",
      timeframe: "1h",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, vwap: false, bollinger: false, rsi: false, macd: false, atr: false },
    },
    {
      id: "pane-5",
      type: "chart",
      symbol: "EURUSD",
      timeframe: "15m",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, vwap: false, bollinger: false, rsi: false, macd: false, atr: false },
    },
    {
      id: "pane-6",
      type: "chart",
      symbol: "NVDA",
      timeframe: "1D",
      chartType: "candlestick",
      indicators: { sma20: false, ema50: false, vwap: false, bollinger: false, rsi: false, macd: false, atr: false },
    },
  ]);

  // Tabbed charts state (Supports Chart, News, Social, OrderBook, Intel, Calendar)
  const [tabs, setTabs] = useState<TabItem[]>([
    { id: "tab-1", type: "chart", symbol: "XAUUSD", timeframe: "15m", name: "Gold Spot / U.S. Dollar" },
    { id: "tab-2", type: "chart", symbol: "BTCUSDT", timeframe: "1h", name: "Bitcoin / TetherUS" },
    { id: "tab-3", type: "chart", symbol: "SPX", timeframe: "1D", name: "S&P 500 Index" },
  ]);
  const [activeTabId, setActiveTabId] = useState<string>("tab-1");

  // Track if local state has been initialized from localStorage
  const [isInitialized, setIsInitialized] = useState(false);

  // Load saved settings & workspace layout state from localStorage on initial mount
  useEffect(() => {
    try {
      const savedSettings = localStorage.getItem("atlsd_terminal_settings");
      if (savedSettings) {
        setSettings((prev) => ({ ...prev, ...JSON.parse(savedSettings) }));
      }

      const savedLayout = localStorage.getItem("atlsd_terminal_layout") as ChartLayout | null;
      if (savedLayout) {
        setLayout(savedLayout);
      }

      const savedPanes = localStorage.getItem("atlsd_terminal_panes");
      if (savedPanes) {
        const parsed = JSON.parse(savedPanes);
        if (Array.isArray(parsed) && parsed.length > 0) {
          setPanes(parsed);
        }
      }

      const savedTabs = localStorage.getItem("atlsd_terminal_tabs");
      if (savedTabs) {
        const parsed = JSON.parse(savedTabs);
        if (Array.isArray(parsed) && parsed.length > 0) {
          setTabs(parsed);
        }
      }

      const savedActivePane = localStorage.getItem("atlsd_terminal_active_pane_id");
      if (savedActivePane) setActivePaneId(savedActivePane);

      const savedActiveTab = localStorage.getItem("atlsd_terminal_active_tab_id");
      if (savedActiveTab) setActiveTabId(savedActiveTab);

      const savedWidth = localStorage.getItem("atlsd_terminal_sidebar_width");
      if (savedWidth) setSidebarWidth(Number(savedWidth) || 330);

      const savedCollapsed = localStorage.getItem("atlsd_terminal_sidebar_collapsed");
      if (savedCollapsed !== null) setIsSidebarCollapsed(savedCollapsed === "true");

      const savedRightTab = localStorage.getItem("atlsd_terminal_right_tab") as SidebarTab | null;
      if (savedRightTab) setRightSidebarTab(savedRightTab);
    } catch {
      // ignore
    } finally {
      setIsInitialized(true);
    }
  }, []);

  // Auto-persist workspace state on every change once initialized
  useEffect(() => {
    if (!isInitialized) return;
    try {
      localStorage.setItem("atlsd_terminal_layout", layout);
      localStorage.setItem("atlsd_terminal_panes", JSON.stringify(panes));
      localStorage.setItem("atlsd_terminal_tabs", JSON.stringify(tabs));
      localStorage.setItem("atlsd_terminal_active_pane_id", activePaneId);
      localStorage.setItem("atlsd_terminal_active_tab_id", activeTabId);
      localStorage.setItem("atlsd_terminal_sidebar_width", String(sidebarWidth));
      localStorage.setItem("atlsd_terminal_sidebar_collapsed", String(isSidebarCollapsed));
      localStorage.setItem("atlsd_terminal_right_tab", rightSidebarTab);
    } catch {
      // ignore
    }
  }, [
    isInitialized,
    layout,
    panes,
    tabs,
    activePaneId,
    activeTabId,
    sidebarWidth,
    isSidebarCollapsed,
    rightSidebarTab,
  ]);

  const handleManualSave = () => {
    try {
      localStorage.setItem("atlsd_terminal_layout", layout);
      localStorage.setItem("atlsd_terminal_panes", JSON.stringify(panes));
      localStorage.setItem("atlsd_terminal_tabs", JSON.stringify(tabs));
      localStorage.setItem("atlsd_terminal_active_pane_id", activePaneId);
      localStorage.setItem("atlsd_terminal_active_tab_id", activeTabId);
      localStorage.setItem("atlsd_terminal_sidebar_width", String(sidebarWidth));
      localStorage.setItem("atlsd_terminal_sidebar_collapsed", String(isSidebarCollapsed));
      localStorage.setItem("atlsd_terminal_right_tab", rightSidebarTab);
      localStorage.setItem("atlsd_terminal_settings", JSON.stringify(settings));
    } catch {
      // ignore
    }
  };

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
    if (["1x3", "3x1", "1L-2R", "2L-1R", "1T-2B", "2T-1B"].includes(layout)) {
      return panes.slice(0, 3);
    }
    if (layout === "2x2") return panes.slice(0, 4);
    return panes.slice(0, 6);
  }, [layout, activePane, panes]);

  const gridClass = useMemo(() => getGridClass(layout, isLight), [layout, isLight]);

  const handleSelectTab = (tabId: string) => {
    setActiveTabId(tabId);
    const t = tabs.find((x) => x.id === tabId);
    if (!t) return;
    setPanes((curr) =>
      curr.map((p) =>
        p.id === activePaneId
          ? {
              ...p,
              type: t.type || "chart",
              symbol: t.symbol,
              timeframe: t.timeframe,
            }
          : p
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

  const handleNewTab = (type: TabContentType = "chart") => {
    const newId = `tab-${Date.now()}`;
    const defaultSymbols = ["BTCUSDT", "ETHUSDT", "XAUUSD", "SPX", "NVDA", "AAPL", "EURUSD", "NDX", "TSLA"];
    const existing = new Set(tabs.map((t) => t.symbol));
    const nextSym = defaultSymbols.find((s) => !existing.has(s)) || selectedItem.symbol;
    const item = findItem(nextSym);
    const newTab: TabItem = {
      id: newId,
      type,
      symbol: item.symbol,
      timeframe: settings.defaultTimeframe,
      name:
        type === "news"
          ? "News Headlines"
          : type === "social"
          ? "Social Pulse"
          : type === "orderbook"
          ? `DOM · ${item.symbol}`
          : type === "intelligence"
          ? "Market Intel"
          : type === "calendar"
          ? "Economic Calendar"
          : item.name,
    };
    setTabs((curr) => [...curr, newTab]);
    setActiveTabId(newId);
    setPanes((curr) =>
      curr.map((p) =>
        p.id === activePaneId
          ? {
              ...p,
              type,
              symbol: item.symbol,
              timeframe: settings.defaultTimeframe,
            }
          : p
      )
    );
  };

  const handleReorderTabs = (newTabs: TabItem[]) => {
    setTabs(newTabs);
  };

  const handleChangePaneType = (paneId: string, newType: PaneContentType) => {
    setPanes((curr) =>
      curr.map((p) => (p.id === paneId ? { ...p, type: newType } : p))
    );
  };

  const handleSplitHorizontal = () => {
    if (layout === "1x1") setLayout("1x2");
    else if (layout === "1x2") setLayout("1x3");
    else if (layout === "2x1") setLayout("2x2");
    else setLayout("2x2");
  };

  const handleSplitVertical = () => {
    if (layout === "1x1") setLayout("2x1");
    else if (layout === "2x1") setLayout("3x1");
    else if (layout === "1x2") setLayout("2x2");
    else setLayout("2x2");
  };

  const handleClosePane = () => {
    if (layout === "3x2") setLayout("2x2");
    else if (layout === "2x2") setLayout("1x3");
    else if (["1x3", "3x1", "1L-2R", "2L-1R", "1T-2B", "2T-1B"].includes(layout)) setLayout("1x2");
    else setLayout("1x1");
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

        setWatchlist((list) => {
          const existingSyms = new Set<string>();
          const updated = list.map((w) => {
            const sym = w.symbol.toUpperCase();
            existingSyms.add(sym);
            const hit = bySym[sym];
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
          });

          const additions: WatchlistItem[] = [];
          for (const it of data.items) {
            const sym = String(it.symbol ?? "").toUpperCase();
            if (!sym || existingSyms.has(sym) || typeof it.price !== "number") continue;
            existingSyms.add(sym);
            const meta = resolveInstrument(sym, it.asset_type);
            additions.push({
              symbol: sym,
              name: meta.name,
              price: it.price,
              change: 0,
              changePercent: 0,
              category: meta.category,
              provider: meta.provider,
              digits: meta.digits,
            });
          }

          return additions.length > 0 ? [...updated, ...additions] : updated;
        });
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
    <div
      data-theme={settings.theme}
      className={`flex flex-col h-full w-full ${
        isLight ? "bg-[#ffffff] text-[#131722]" : "bg-[#131722] text-[#d1d4dc]"
      } overflow-hidden transition-colors select-none`}
    >
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
        theme={settings.theme}
        onToggleTheme={handleToggleTheme}
        onOpenSettings={() => setIsSettingsOpen(true)}
        onOpenIndicatorSettings={() => setIsIndicatorSettingsOpen(true)}
        indicatorParams={settings.indicatorParams || DEFAULT_INDICATOR_PARAMS}
        syncCrosshair={settings.syncCrosshair !== false}
        onToggleSyncCrosshair={handleToggleSyncCrosshair}
        syncTime={settings.syncTime !== false}
        onToggleSyncTime={handleToggleSyncTime}
        onSave={handleManualSave}
      />

      <ChartTabs
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewTab={handleNewTab}
        onReorderTabs={handleReorderTabs}
        theme={settings.theme}
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
          theme={settings.theme}
        />

        {/* Main Workspace View */}
        <main className="min-w-0 flex-1 h-full overflow-hidden relative">
          <div className={`${gridClass} min-w-0`}>
            {visiblePanes.map((pane, idx) => {
              const meta = findItem(pane.symbol);
              const spanClass = getPaneSpanClass(layout, idx);
              return (
                <div key={pane.id} className={spanClass}>
                  <ChartPaneWrapper
                    pane={pane}
                    isActive={pane.id === activePaneId}
                    onActivate={() => setActivePaneId(pane.id)}
                    activeTool={activeTool}
                    digits={meta.digits}
                    provider={meta.provider}
                    clearDrawingsTrigger={clearDrawingsTrigger}
                    snapshotTrigger={pane.id === activePaneId ? snapshotTrigger : 0}
                    onSnapshotDone={() => setSnapshotTrigger(0)}
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
                    theme={settings.theme}
                    settings={settings}
                    onChangePaneType={(newType) => handleChangePaneType(pane.id, newType)}
                    onSplitHorizontal={handleSplitHorizontal}
                    onSplitVertical={handleSplitVertical}
                    onClosePane={handleClosePane}
                    canClosePane={layout !== "1x1"}
                    showPaneHeader={layout !== "1x1"}
                  />
                </div>
              );
            })}
          </div>
        </main>

        {/* Desktop Splitter & Collapse Button */}
        <div
          onMouseDown={handleSplitterMouseDown}
          className={`hidden lg:flex relative w-1 hover:w-1.5 cursor-col-resize ${
            isLight ? "bg-[#e0e3eb]" : "bg-[#2a2e39]"
          } hover:bg-[#2962ff] transition-all items-center justify-center select-none z-20 group shrink-0 ${
            isDraggingSplitter ? "bg-[#2962ff] w-1.5" : ""
          }`}
          title="Drag to resize width"
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsSidebarCollapsed((c) => !c);
            }}
            className={`absolute -left-2.5 top-1/2 -translate-y-1/2 w-5 h-7 rounded border flex items-center justify-center shadow-md z-30 transition-all cursor-pointer ${
              isLight
                ? "bg-[#ffffff] border-[#e0e3eb] text-[#5d606b] hover:text-[#131722]"
                : "bg-[#1e222d] border-[#2a2e39] text-[#787b86] hover:text-white"
            }`}
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
            fixed inset-y-0 right-0 z-50 w-[calc(100vw-1rem)] max-w-[360px] shadow-2xl transition-transform duration-300 ease-in-out
            lg:static lg:z-auto lg:shadow-none lg:translate-x-0 lg:transition-none
            ${isMobileDrawerOpen ? "translate-x-0" : "translate-x-full lg:translate-x-0"}
            ${isSidebarCollapsed ? "lg:hidden" : "lg:flex"}
            ${isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"}
            flex flex-col h-full overflow-hidden border-l shrink-0 transition-colors
          `}
        >
          {/* Mobile Drawer Close Header */}
          <div
            className={`flex items-center justify-between px-3 py-2.5 border-b lg:hidden shrink-0 ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <span className="font-bold text-xs uppercase tracking-wider">
              {rightSidebarTab}
            </span>
            <button
              onClick={() => setIsMobileDrawerOpen(false)}
              className="p-1 rounded text-[#787b86] hover:text-white hover:bg-black/10 dark:hover:bg-[#2a2e39] transition-colors cursor-pointer"
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
                theme={settings.theme}
              />
            )}
            {rightSidebarTab === "orderbook" && (
              <OrderBookPanel
                symbol={selectedItem.symbol}
                livePrice={selectedItem.price}
                digits={selectedItem.digits}
                theme={settings.theme}
              />
            )}
            {rightSidebarTab === "news" && (
              <NewsPanel symbol={selectedItem.symbol} theme={settings.theme} />
            )}
            {rightSidebarTab === "intelligence" && (
              <MarketIntelligencePanel symbol={selectedItem.symbol} theme={settings.theme} />
            )}
            {rightSidebarTab === "social" && <SocialPanel theme={settings.theme} />}
            {rightSidebarTab === "alerts" && (
              <AlertsPanel
                symbol={selectedItem.symbol}
                livePrice={selectedItem.price}
                digits={selectedItem.digits}
                theme={settings.theme}
              />
            )}
            {rightSidebarTab === "calendar" && <CalendarPanel theme={settings.theme} />}
          </div>
        </aside>

        <RightDock
          activeTab={rightSidebarTab}
          setActiveTab={handleTabChangeFromDock}
          theme={settings.theme}
        />
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
        theme={settings.theme}
      />

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        settings={settings}
        onSaveSettings={handleSaveSettings}
      />

      <IndicatorSettingsModal
        isOpen={isIndicatorSettingsOpen}
        onClose={() => setIsIndicatorSettingsOpen(false)}
        params={settings.indicatorParams || DEFAULT_INDICATOR_PARAMS}
        onSave={handleSaveIndicatorParams}
        theme={settings.theme}
      />
    </div>
  );
}
