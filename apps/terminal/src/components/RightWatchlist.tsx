"use client";

import React, { useState } from "react";
import { 
  Plus, 
  MoreHorizontal, 
  ChevronRight, 
  ChevronDown,
  TrendingDown,
  TrendingUp,
  Search,
  Settings2,
  Sliders,
  Sparkles
} from "lucide-react";
import { WatchlistItem } from "@/types";

interface RightWatchlistProps {
  items: WatchlistItem[];
  selectedSymbol: string;
  onSelectSymbol: (item: WatchlistItem) => void;
}

export const RightWatchlist: React.FC<RightWatchlistProps> = ({
  items,
  selectedSymbol,
  onSelectSymbol,
}) => {
  const [activeTab, setActiveTab] = useState<"all" | "crypto" | "forex" | "indices" | "commodities">("all");
  const selectedItem = items.find((i) => i.symbol === selectedSymbol) ?? items[0];

  const filteredItems = activeTab === "all" ? items : items.filter((i) => i.category === activeTab);

  return (
    <aside className="w-[320px] bg-[#1e222d] border-l border-[#2a2e39] flex flex-col h-full select-none text-xs">
      {/* Watchlist Header */}
      <div className="h-[46px] border-b border-[#2a2e39] flex items-center justify-between px-3 bg-[#1e222d]">
        <div className="flex items-center gap-1.5 font-bold text-sm text-white">
          <span>Watchlist</span>
          <ChevronDown className="w-3.5 h-3.5 text-[#787b86]" />
        </div>
        <div className="flex items-center gap-1">
          <button className="p-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Add Symbol">
            <Plus className="w-4 h-4" />
          </button>
          <button className="p-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Manage List">
            <MoreHorizontal className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Category Tabs */}
      <div className="flex items-center gap-1 px-3 py-1.5 border-b border-[#2a2e39] bg-[#181b27] overflow-x-auto text-[11px]">
        {(["all", "commodities", "indices", "forex", "crypto"] as const).map((cat) => (
          <button
            key={cat}
            onClick={() => setActiveTab(cat)}
            className={`px-2 py-0.5 rounded capitalize font-medium whitespace-nowrap transition-colors ${
              activeTab === cat
                ? "bg-[#2a2e39] text-white font-bold"
                : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* Table Column Headers */}
      <div className="grid grid-cols-12 px-3 py-1.5 text-[10px] font-bold text-[#787b86] uppercase border-b border-[#2a2e39] bg-[#1e222d]">
        <span className="col-span-6">Symbol</span>
        <span className="col-span-3 text-right">Last</span>
        <span className="col-span-3 text-right">Chg %</span>
      </div>

      {/* Watchlist Items Scroll Area */}
      <div className="flex-1 overflow-y-auto divide-y divide-[#2a2e39]/50">
        {filteredItems.map((item) => {
          const isSelected = item.symbol === selectedSymbol;
          const isPos = item.change >= 0;

          return (
            <div
              key={item.symbol}
              onClick={() => onSelectSymbol(item)}
              className={`grid grid-cols-12 items-center px-3 py-2 cursor-pointer transition-colors ${
                isSelected
                  ? "bg-[#2a2e39] border-l-2 border-[#2962ff]"
                  : "hover:bg-[#262b37]"
              }`}
            >
              {/* Symbol & Name */}
              <div className="col-span-6 flex flex-col pr-1">
                <span className="font-bold text-white text-[12px] tracking-wide">{item.symbol}</span>
                <span className="text-[10px] text-[#787b86] truncate">{item.name}</span>
              </div>

              {/* Price */}
              <div className="col-span-3 text-right font-mono font-semibold text-white text-[12px]">
                {item.price.toFixed(item.digits)}
              </div>

              {/* Change % Badge */}
              <div className="col-span-3 flex justify-end">
                <span
                  className={`font-mono text-[11px] font-bold px-1.5 py-0.5 rounded ${
                    isPos ? "text-[#089981] bg-[#089981]/10" : "text-[#f23645] bg-[#f23645]/10"
                  }`}
                >
                  {isPos ? "+" : ""}{item.changePercent.toFixed(2)}%
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Bottom Panel: Selected Instrument Details (Matching Image Sidebar) */}
      {selectedItem && (
        <div className="border-t border-[#2a2e39] bg-[#181b27] p-3 flex flex-col gap-2.5">
          <div className="flex items-start justify-between">
            <div>
              <div className="flex items-center gap-1.5">
                <span className="font-bold text-sm text-white">{selectedItem.symbol}</span>
                <span className="text-[10px] font-mono text-[#787b86] bg-[#2a2e39] px-1 rounded">
                  {selectedItem.provider}
                </span>
              </div>
              <p className="text-[11px] text-[#787b86] mt-0.5">{selectedItem.name}</p>
            </div>
            <div className="text-right">
              <span className="font-mono text-base font-bold text-white">
                {selectedItem.price.toFixed(selectedItem.digits)}
              </span>
              <div className={`font-mono text-xs font-semibold ${selectedItem.change >= 0 ? "text-[#089981]" : "text-[#f23645]"}`}>
                {selectedItem.change >= 0 ? "+" : ""}{selectedItem.change.toFixed(selectedItem.digits)} ({selectedItem.changePercent.toFixed(2)}%)
              </div>
            </div>
          </div>

          {/* Performance Horizon Stats */}
          <div className="grid grid-cols-3 gap-2 pt-2 border-t border-[#2a2e39] text-center">
            <div className="flex flex-col bg-[#1e222d] p-1.5 rounded">
              <span className="text-[10px] text-[#787b86]">1W</span>
              <span className="font-mono text-xs font-bold text-[#f23645]">-3.74%</span>
            </div>
            <div className="flex flex-col bg-[#1e222d] p-1.5 rounded">
              <span className="text-[10px] text-[#787b86]">1M</span>
              <span className="font-mono text-xs font-bold text-[#089981]">+4.12%</span>
            </div>
            <div className="flex flex-col bg-[#1e222d] p-1.5 rounded">
              <span className="text-[10px] text-[#787b86]">1Y</span>
              <span className="font-mono text-xs font-bold text-[#089981]">+24.95%</span>
            </div>
          </div>

          {/* Technical Summary Gauge */}
          <div className="flex items-center justify-between text-[11px] px-1">
            <span className="text-[#787b86]">Technical Rating</span>
            <span className="font-bold text-[#089981]">Strong Buy</span>
          </div>
        </div>
      )}
    </aside>
  );
};
