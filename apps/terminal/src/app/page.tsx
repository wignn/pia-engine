"use client";

import React, { useState, useEffect, useCallback } from "react";
import { TopBar } from "@/components/TopBar";
import { LeftToolbar } from "@/components/LeftToolbar";
import { ChartArea } from "@/components/ChartArea";
import { RightWatchlist } from "@/components/RightWatchlist";
import { RightDock, SidebarTab } from "@/components/RightDock";
import { NewsPanel } from "@/components/NewsPanel";
import { MarketIntelligencePanel } from "@/components/MarketIntelligencePanel";
import { ChartTabs } from "@/components/ChartTabs";
import { SymbolSearchModal } from "@/components/SymbolSearchModal";
import { INITIAL_WATCHLIST } from "@/lib/constants";
import { useMarketFeed } from "@/lib/useMarketFeed";
import { WatchlistItem, Timeframe, TabItem, ChartType } from "@/types";

export default function TerminalPage() {
  const [watchlist, setWatchlist] = useState<WatchlistItem[]>(INITIAL_WATCHLIST);
  const [selectedItem, setSelectedItem] = useState<WatchlistItem>(INITIAL_WATCHLIST[0]);
  const [timeframe, setTimeframe] = useState<Timeframe>("15m");
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [rightSidebarTab, setRightSidebarTab] = useState<SidebarTab>("watchlist");
  const [indicators, setIndicators] = useState({ sma20: false, ema50: false });
  const [chartType, setChartType] = useState<ChartType>("candlestick");

  // Real market feed for the active symbol + timeframe.
  const { candles, livePrice, connected, loading, loadingOlder, hasMoreHistory, loadOlder } = useMarketFeed(selectedItem.symbol, timeframe); // live + paginated history

  // Effective live price for header (real if present, else the seed price).
  const headerPrice = livePrice ?? selectedItem.price;

  // Tabbed charts state
  const [tabs, setTabs] = useState<TabItem[]>([
    { id: "tab-1", symbol: "XAUUSD", timeframe: "15m", name: "Gold Spot / U.S. Dollar" },
    { id: "tab-2", symbol: "BTCUSDT", timeframe: "1h", name: "Bitcoin / TetherUS" },
    { id: "tab-3", symbol: "SPX", timeframe: "1D", name: "S&P 500 Index" },
  ]);
  const [activeTabId, setActiveTabId] = useState<string>("tab-1");

  const findItem = useCallback(
    (sym: string) => watchlist.find((w) => w.symbol === sym) ?? INITIAL_WATCHLIST.find((w) => w.symbol === sym) ?? watchlist[0],
    [watchlist]
  );

  const handleSelectTab = (tabId: string) => {
    setActiveTabId(tabId);
    const t = tabs.find((x) => x.id === tabId);
    if (!t) return;
    setSelectedItem(findItem(t.symbol));
    setTimeframe(t.timeframe);
  };

  const handleCloseTab = (tabId: string) => {
    if (tabs.length <= 1) return;
    const remaining = tabs.filter((t) => t.id !== tabId);
    setTabs(remaining);
    if (activeTabId === tabId) handleSelectTab(remaining[remaining.length - 1].id);
  };

  const handleSelectSymbol = (item: WatchlistItem) => {
    setSelectedItem(item);
    setTabs((curr) => curr.map((t) => (t.id === activeTabId ? { ...t, symbol: item.symbol, name: item.name } : t)));
  };

  const handleCreateNewTab = () => {
    const newId = `tab-${Date.now()}`;
    const item = findItem("ETHUSDT");
    setTabs((curr) => [...curr, { id: newId, symbol: item.symbol, timeframe: "15m", name: item.name }]);
    setActiveTabId(newId);
    setSelectedItem(item);
    setTimeframe("15m");
  };

  const handleTimeframe = (tf: Timeframe) => {
    setTimeframe(tf);
    setTabs((curr) => curr.map((t) => (t.id === activeTabId ? { ...t, timeframe: tf } : t)));
  };

  const toggleFullscreen = useCallback(async () => {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  }, []);

  // Poll real prices for the whole watchlist every 5s.
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
              changePercent: w.price ? Number(((change / (hit.price - change)) * 100).toFixed(2)) : 0,
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

  // Keep the header's selected item change % synced to live price.
  const liveChange = livePrice != null ? livePrice - (selectedItem.price - selectedItem.change) : selectedItem.change;
  const liveChangePct =
    livePrice != null && selectedItem.price - selectedItem.change !== 0
      ? Number(((liveChange / (selectedItem.price - selectedItem.change)) * 100).toFixed(2))
      : selectedItem.changePercent;

  return (
    <div className="flex flex-col h-full w-full bg-[#131722] overflow-hidden">
      <TopBar
        symbol={selectedItem.symbol}
        timeframe={timeframe}
        setTimeframe={handleTimeframe}
        price={headerPrice}
        change={liveChange}
        changePercent={liveChangePct}
        digits={selectedItem.digits}
        onSearchClick={() => setIsSearchOpen(true)}
        indicators={indicators}
        chartType={chartType}
        onChartTypeChange={setChartType}
        onToggleIndicator={(indicator) => setIndicators((current) => ({ ...current, [indicator]: !current[indicator] }))}
        onFullscreen={toggleFullscreen}
      />

      <ChartTabs
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewTab={handleCreateNewTab}
      />

      <div className="flex-1 flex w-full overflow-hidden">
        <LeftToolbar />

        <main className="flex-1 h-full overflow-hidden relative">
          <ChartArea
            symbol={selectedItem.symbol}
            provider={selectedItem.provider}
            timeframe={timeframe}
            chartType={chartType}
            indicators={indicators}
            digits={selectedItem.digits}
            candles={candles}
            livePrice={livePrice}
            connected={connected}
            loading={loading}
            loadingOlder={loadingOlder}
            hasMoreHistory={hasMoreHistory}
            onLoadOlder={loadOlder}
          />
        </main>

        <div className="w-[330px] h-full flex flex-col overflow-hidden">
          {rightSidebarTab === "watchlist" && (
            <RightWatchlist
              items={watchlist}
              selectedSymbol={selectedItem.symbol}
              onSelectSymbol={handleSelectSymbol}
            />
          )}
          {rightSidebarTab === "news" && <NewsPanel symbol={selectedItem.symbol} />}
          {rightSidebarTab === "intelligence" && <MarketIntelligencePanel symbol={selectedItem.symbol} />}
          {rightSidebarTab === "alerts" && (
            <div className="h-full bg-[#1e222d] border-l border-[#2a2e39] p-4 flex flex-col items-center justify-center text-center text-[#787b86]">
              <span className="font-bold text-white mb-1">No Active Price Alerts</span>
              <p className="text-xs">Set triggers on {selectedItem.symbol} to receive Telegram / Discord notifications.</p>
            </div>
          )}
          {rightSidebarTab === "calendar" && (
            <div className="h-full bg-[#1e222d] border-l border-[#2a2e39] p-4 flex flex-col gap-3">
              <span className="font-bold text-white text-sm">Upcoming Macro Events</span>
              <div className="flex flex-col gap-2 text-xs">
                <div className="p-2 rounded bg-[#181b27] border border-[#2a2e39]">
                  <div className="flex justify-between font-bold text-[#f23645]">
                    <span>USD Non-Farm Payrolls</span><span>19:30 UTC</span>
                  </div>
                  <div className="text-[11px] text-[#787b86] mt-1">Forecast: 165K | Previous: 142K</div>
                </div>
                <div className="p-2 rounded bg-[#181b27] border border-[#2a2e39]">
                  <div className="flex justify-between font-bold text-[#2962ff]">
                    <span>USD CPI (MoM)</span><span>Tomorrow</span>
                  </div>
                  <div className="text-[11px] text-[#787b86] mt-1">Forecast: 0.2% | Previous: 0.2%</div>
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
