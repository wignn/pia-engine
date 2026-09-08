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
}

export const RightWatchlist: React.FC<RightWatchlistProps> = ({
  items,
  selectedSymbol,
  onSelectSymbol,
}) => {
  const [activeTab, setActiveTab] = useState<"all" | "crypto" | "forex" | "indices" | "commodities">("all");
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
    <aside className="w-full bg-[#1e222d] border-l border-[#2a2e39] flex flex-col h-full select-none text-xs overflow-hidden">
      {/* Watchlist Header */}
      <div className="h-[44px] border-b border-[#2a2e39] flex items-center justify-between px-3 bg-[#1e222d] shrink-0">
        <div className="flex items-center gap-1.5 font-bold text-sm text-white">
          <span>Watchlist</span>
          <span className="text-[10px] font-mono text-[#787b86] font-normal">
            ({processedItems.length})
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowSearch((v) => !v)}
            className={`p-1.5 rounded transition-colors ${
              showSearch ? "bg-[#2962ff]/20 text-[#2962ff]" : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
            }`}
            title="Search Watchlist"
          >
            <Search className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Search Bar (Collapsible) */}
      {showSearch && (
        <div className="px-3 py-1.5 border-b border-[#2a2e39] bg-[#141722] flex items-center gap-2 shrink-0">
          <Search className="w-3.5 h-3.5 text-[#787b86]" />
          <input
            type="text"
            placeholder="Search symbols..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            autoFocus
            className="flex-1 bg-transparent text-xs text-white placeholder-[#787b86] focus:outline-none"
          />
          {searchQuery && (
            <button onClick={() => setSearchQuery("")} className="text-[#787b86] hover:text-white">
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      )}

      {/* Category Tabs */}
      <div className="flex items-center gap-1 px-3 py-1.5 border-b border-[#2a2e39] bg-[#181b27] overflow-x-auto text-[11px] shrink-0">
        {(["all", "commodities", "indices", "forex", "crypto"] as const).map((cat) => (
          <button
            key={cat}
            onClick={() => setActiveTab(cat)}
            className={`px-2 py-0.5 rounded capitalize font-medium whitespace-nowrap transition-colors ${
              activeTab === cat
                ? "bg-[#2a2e39] text-white font-bold shadow-xs"
                : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* Table Column Headers with Interactive Sorting */}
      <div className="grid grid-cols-12 px-3 py-1.5 text-[10px] font-bold text-[#787b86] uppercase border-b border-[#2a2e39] bg-[#1e222d] shrink-0">
        <button
          onClick={() => handleSort("symbol")}
          className="col-span-6 flex items-center gap-1 text-left hover:text-white transition-colors"
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
          className="col-span-3 flex items-center justify-end gap-1 text-right hover:text-white transition-colors"
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
          className="col-span-3 flex items-center justify-end gap-1 text-right hover:text-white transition-colors"
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
      <div className="flex-1 overflow-y-auto divide-y divide-[#2a2e39]/40">
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
                    ? "bg-[#2962ff]/15 border-l-2 border-[#2962ff]"
                    : "hover:bg-[#262b37]"
                }`}
              >
                {/* Symbol & Name */}
                <div className="col-span-6 flex flex-col pr-1">
                  <div className="flex items-center gap-1.5">
                    <span className="font-bold text-white text-[12px] tracking-wide">
                      {item.symbol}
                    </span>
                    <span className="text-[9px] font-mono uppercase bg-[#141722] text-[#787b86] px-1 rounded">
                      {item.category.slice(0, 3)}
                    </span>
                  </div>
                  <span className="text-[10px] text-[#787b86] truncate mt-0.5">{item.name}</span>
                </div>

                {/* Price */}
                <div className="col-span-3 text-right font-mono font-semibold text-white text-[12px]">
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
        <div className="border-t border-[#2a2e39] bg-[#141722] p-3 flex flex-col gap-2 shrink-0">
          <div className="flex items-start justify-between">
            <div>
              <div className="flex items-center gap-1.5">
                <span className="font-black text-white text-base">{selectedItem.symbol}</span>
                <span className="text-[10px] font-mono uppercase px-1.5 py-0.2 bg-[#1e222d] border border-[#2a2e39] text-[#787b86] rounded">
                  {selectedItem.provider}
                </span>
              </div>
              <div className="text-[11px] text-[#787b86]">{selectedItem.name}</div>
            </div>
            <div className="text-right">
              <div className="font-mono text-base font-bold text-white">
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
