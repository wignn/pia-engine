"use client";

import React, { useState, useMemo } from "react";
import {
  Plus,
  MoreHorizontal,
  ChevronDown,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Search,
  X,
  TrendingUp,
  TrendingDown,
  Activity,
  Globe
} from "lucide-react";
import { WatchlistItem } from "@/types";

interface RightWatchlistProps {
  items: WatchlistItem[];
  selectedSymbol: string;
  onSelectSymbol: (item: WatchlistItem) => void;
  theme?: "dark" | "light";
}

export const RightWatchlist: React.FC<RightWatchlistProps> = ({
  items,
  selectedSymbol,
  onSelectSymbol,
  theme = "dark",
}) => {
  const isLight = theme === "light";
  const [activeTab, setActiveTab] = useState<"all" | "commodities" | "indices" | "forex" | "crypto" | "stocks">("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [showSearch, setShowSearch] = useState(false);
  const [sortBy, setSortBy] = useState<"symbol" | "price" | "change" | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

  const selectedItem = items.find((i) => i.symbol === selectedSymbol) ?? items[0];

  const handleSort = (field: "symbol" | "price" | "change") => {
    if (sortBy === field) {
      if (sortDir === "desc") setSortDir("asc");
      else {
        setSortBy(null);
        setSortDir("desc");
      }
    } else {
      setSortBy(field);
      setSortDir("desc");
    }
  };

  const processedItems = useMemo(() => {
    let result = activeTab === "all" ? items : items.filter((i) => i.category === activeTab);

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (i) => i.symbol.toLowerCase().includes(q) || i.name.toLowerCase().includes(q)
      );
    }

    if (sortBy) {
      result = [...result].sort((a, b) => {
        let cmp = 0;
        if (sortBy === "symbol") cmp = a.symbol.localeCompare(b.symbol);
        else if (sortBy === "price") cmp = a.price - b.price;
        else if (sortBy === "change") cmp = a.changePercent - b.changePercent;
        return sortDir === "asc" ? cmp : -cmp;
      });
    }

    return result;
  }, [items, activeTab, searchQuery, sortBy, sortDir]);

  return (
    <aside
      className={`w-full flex flex-col h-full select-none text-xs overflow-hidden transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Watchlist Header */}
      <div
        className={`h-[44px] border-b flex items-center justify-between px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-1.5 font-bold text-sm">
          <span className={isLight ? "text-[#131722]" : "text-white"}>Watchlist</span>
          <span className="text-[10px] font-mono text-[#787b86] font-normal">
            ({processedItems.length})
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowSearch((v) => !v)}
            className={`p-1.5 rounded transition-colors cursor-pointer ${
              showSearch
                ? "bg-[#2962ff]/20 text-[#2962ff]"
                : isLight
                ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
                : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
            }`}
            title="Search Watchlist"
          >
            <Search className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Search Bar (Collapsible) */}
      {showSearch && (
        <div
          className={`px-3 py-1.5 border-b flex items-center gap-2 shrink-0 ${
            isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          <Search className="w-3.5 h-3.5 text-[#787b86]" />
          <input
            type="text"
            placeholder="Search symbols..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            autoFocus
            className={`flex-1 bg-transparent text-xs focus:outline-none ${
              isLight ? "text-[#131722] placeholder-[#8e929d]" : "text-white placeholder-[#787b86]"
            }`}
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className={`cursor-pointer ${isLight ? "text-[#5d606b] hover:text-[#131722]" : "text-[#787b86] hover:text-white"}`}
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      )}

      {/* Category Tabs */}
      <div
        className={`flex items-center gap-1 px-3 py-1.5 border-b overflow-x-auto text-[11px] shrink-0 ${
          isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
        }`}
      >
        {(["all", "commodities", "indices", "forex", "crypto", "stocks"] as const).map((cat) => (
          <button
            key={cat}
            onClick={() => setActiveTab(cat)}
            className={`px-2 py-0.5 rounded capitalize font-medium whitespace-nowrap transition-colors cursor-pointer ${
              activeTab === cat
                ? isLight
                  ? "bg-[#ffffff] text-[#131722] font-bold shadow-xs"
                  : "bg-[#2a2e39] text-white font-bold shadow-xs"
                : isLight
                ? "text-[#5d606b] hover:text-[#131722]"
                : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* Table Column Headers with Interactive Sorting */}
      <div
        className={`grid grid-cols-12 px-3 py-1.5 text-[10px] font-bold uppercase border-b shrink-0 ${
          isLight ? "bg-[#f8f9fc] border-[#e0e3eb] text-[#5d606b]" : "bg-[#1e222d] border-[#2a2e39] text-[#787b86]"
        }`}
      >
        <button
          onClick={() => handleSort("symbol")}
          className={`col-span-6 flex items-center gap-1 text-left transition-colors cursor-pointer ${
            isLight ? "hover:text-[#131722]" : "hover:text-white"
          }`}
        >
          <span>Symbol</span>
          {sortBy === "symbol" ? (
            sortDir === "asc" ? <ArrowUp className="w-3 h-3 text-[#2962ff]" /> : <ArrowDown className="w-3 h-3 text-[#2962ff]" />
          ) : (
            <ArrowUpDown className="w-2.5 h-2.5 opacity-40" />
          )}
        </button>

        <button
          onClick={() => handleSort("price")}
          className={`col-span-3 flex items-center justify-end gap-1 text-right transition-colors cursor-pointer ${
            isLight ? "hover:text-[#131722]" : "hover:text-white"
          }`}
        >
          <span>Last</span>
          {sortBy === "price" ? (
            sortDir === "asc" ? <ArrowUp className="w-3 h-3 text-[#2962ff]" /> : <ArrowDown className="w-3 h-3 text-[#2962ff]" />
          ) : (
            <ArrowUpDown className="w-2.5 h-2.5 opacity-40" />
          )}
        </button>

        <button
          onClick={() => handleSort("change")}
          className={`col-span-3 flex items-center justify-end gap-1 text-right transition-colors cursor-pointer ${
            isLight ? "hover:text-[#131722]" : "hover:text-white"
          }`}
        >
          <span>Chg %</span>
          {sortBy === "change" ? (
            sortDir === "asc" ? <ArrowUp className="w-3 h-3 text-[#2962ff]" /> : <ArrowDown className="w-3 h-3 text-[#2962ff]" />
          ) : (
            <ArrowUpDown className="w-2.5 h-2.5 opacity-40" />
          )}
        </button>
      </div>

      {/* Watchlist Items Scroll Area */}
      <div className={`flex-1 overflow-y-auto divide-y ${isLight ? "divide-[#e0e3eb]" : "divide-[#2a2e39]/40"}`}>
        {processedItems.length === 0 ? (
          <div className="p-4 text-center text-[#787b86] text-xs">
            No matching symbols found
          </div>
        ) : (
          processedItems.map((item) => {
            const isSelected = item.symbol === selectedSymbol;
            const isPos = item.change >= 0;

            return (
              <div
                key={item.symbol}
                onClick={() => onSelectSymbol(item)}
                className={`grid grid-cols-12 items-center px-3 py-2 cursor-pointer transition-all ${
                  isSelected
                    ? isLight
                      ? "bg-[#2962ff]/10 border-l-2 border-[#2962ff]"
                      : "bg-[#2962ff]/15 border-l-2 border-[#2962ff]"
                    : isLight
                    ? "hover:bg-[#f0f3fa]"
                    : "hover:bg-[#262b37]"
                }`}
              >
                {/* Symbol & Name */}
                <div className="col-span-6 flex flex-col pr-1">
                  <div className="flex items-center gap-1.5">
                    <span className={`font-bold text-[12px] tracking-wide ${isLight ? "text-[#131722]" : "text-white"}`}>
                      {item.symbol}
                    </span>
                    <span
                      className={`text-[9px] font-mono uppercase px-1 rounded ${
                        isLight ? "bg-[#f0f3fa] text-[#5d606b]" : "bg-[#141722] text-[#787b86]"
                      }`}
                    >
                      {item.category.slice(0, 3)}
                    </span>
                  </div>
                  <span className="text-[10px] text-[#787b86] truncate mt-0.5">{item.name}</span>
                </div>

                {/* Price */}
                <div className={`col-span-3 text-right font-mono font-semibold text-[12px] ${isLight ? "text-[#131722]" : "text-white"}`}>
                  {item.price.toFixed(item.digits)}
                </div>

                {/* Change % Badge */}
                <div className="col-span-3 flex justify-end">
                  <span
                    className={`font-mono text-[11px] font-bold px-1.5 py-0.5 rounded ${
                      isPos ? "text-[#089981] bg-[#089981]/15" : "text-[#f23645] bg-[#f23645]/15"
                    }`}
                  >
                    {isPos ? "+" : ""}{item.changePercent.toFixed(2)}%
                  </span>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Bottom Panel: Selected Instrument Details */}
      {selectedItem && (
        <div
          className={`border-t p-3 flex flex-col gap-2 shrink-0 ${
            isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-start justify-between">
            <div>
              <div className="flex items-center gap-1.5">
                <span className={`font-black text-base ${isLight ? "text-[#131722]" : "text-white"}`}>
                  {selectedItem.symbol}
                </span>
                <span
                  className={`text-[10px] font-mono uppercase px-1.5 py-0.5 border rounded ${
                    isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#5d606b]" : "bg-[#1e222d] border-[#2a2e39] text-[#787b86]"
                  }`}
                >
                  {selectedItem.provider}
                </span>
              </div>
              <div className="text-[11px] text-[#787b86]">{selectedItem.name}</div>
            </div>
            <div className="text-right">
              <div className={`font-mono text-base font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
                {selectedItem.price.toFixed(selectedItem.digits)}
              </div>
              <div
                className={`font-mono text-[11px] font-semibold ${
                  selectedItem.change >= 0 ? "text-[#089981]" : "text-[#f23645]"
                }`}
              >
                {selectedItem.change >= 0 ? "+" : ""}
                {selectedItem.change.toFixed(selectedItem.digits)} ({selectedItem.change >= 0 ? "+" : ""}
                {selectedItem.changePercent.toFixed(2)}%)
              </div>
            </div>
          </div>
        </div>
      )}
    </aside>
  );
};
