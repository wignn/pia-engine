"use client";

import React, { useState, useEffect } from "react";
import { TopBar } from "@/components/TopBar";
import { LeftToolbar } from "@/components/LeftToolbar";
import { ChartArea } from "@/components/ChartArea";
import { RightWatchlist } from "@/components/RightWatchlist";
import { SymbolSearchModal } from "@/components/SymbolSearchModal";
import { INITIAL_WATCHLIST } from "@/lib/constants";
import { generateRealisticHistory } from "@/lib/mockData";
import { WatchlistItem, Timeframe, CandleData } from "@/types";

export default function TerminalPage() {
  const [watchlist, setWatchlist] = useState<WatchlistItem[]>(INITIAL_WATCHLIST);
  const [selectedItem, setSelectedItem] = useState<WatchlistItem>(INITIAL_WATCHLIST[0]); // XAUUSD default
  const [timeframe, setTimeframe] = useState<Timeframe>("15m");
  const [candles, setCandles] = useState<CandleData[]>(() => 
    generateRealisticHistory(INITIAL_WATCHLIST[0].price, 300, 900)
  );
  const [livePrice, setLivePrice] = useState<number>(INITIAL_WATCHLIST[0].price);
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  // When symbol changes, regenerate chart candles matching exact trajectory
  const handleSelectSymbol = (item: WatchlistItem) => {
    setSelectedItem(item);
    setLivePrice(item.price);
    const intervalSec = timeframe === "1m" ? 60 : timeframe === "5m" ? 300 : timeframe === "15m" ? 900 : 3600;
    setCandles(generateRealisticHistory(item.price, 300, intervalSec));
  };

  // Simulate real-time tick updates (interleaved with market movements)
  useEffect(() => {
    const interval = setInterval(() => {
      setLivePrice((prev) => {
        const delta = (Math.random() - 0.495) * (prev * 0.0004);
        const nextPrice = Number((prev + delta).toFixed(selectedItem.digits));
        
        // update watchlist item in real-time
        setWatchlist((list) =>
          list.map((it) =>
            it.symbol === selectedItem.symbol
              ? {
                  ...it,
                  price: nextPrice,
                  change: nextPrice - (it.price - it.change),
                  changePercent: Number((((nextPrice - (it.price - it.change)) / (it.price - it.change)) * 100).toFixed(2)),
                }
              : it
          )
        );

        return nextPrice;
      });
    }, 1500);

    return () => clearInterval(interval);
  }, [selectedItem.symbol, selectedItem.digits]);

  return (
    <div className="flex flex-col h-full w-full bg-[#131722] overflow-hidden">
      {/* Top TradingView App Navigation Bar */}
      <TopBar
        symbol={selectedItem.symbol}
        timeframe={timeframe}
        setTimeframe={(tf) => {
          setTimeframe(tf);
          const intervalSec = tf === "1m" ? 60 : tf === "5m" ? 300 : tf === "15m" ? 900 : 3600;
          setCandles(generateRealisticHistory(selectedItem.price, 300, intervalSec));
        }}
        price={livePrice}
        change={selectedItem.change}
        changePercent={selectedItem.changePercent}
        digits={selectedItem.digits}
        onSearchClick={() => setIsSearchOpen(true)}
      />

      {/* Main Workspace: Left Drawing Tools + Center Interactive Chart + Right Sidebar Watchlist */}
      <div className="flex-1 flex w-full overflow-hidden">
        {/* Left Vertical Tools */}
        <LeftToolbar />

        {/* Center Interactive Candlestick Chart */}
        <main className="flex-1 h-full overflow-hidden relative">
          <ChartArea
            symbol={selectedItem.symbol}
            provider={selectedItem.provider}
            timeframe={timeframe}
            digits={selectedItem.digits}
            initialCandles={candles}
            livePrice={livePrice}
          />
        </main>

        {/* Right Financial Watchlist & Horizon Details */}
        <RightWatchlist
          items={watchlist}
          selectedSymbol={selectedItem.symbol}
          onSelectSymbol={handleSelectSymbol}
        />
      </div>

      {/* Symbol Search Dialog Modal */}
      <SymbolSearchModal
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
        items={watchlist}
        onSelect={handleSelectSymbol}
      />
    </div>
  );
}
