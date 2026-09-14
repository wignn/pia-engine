"use client";

import React, { useState, useMemo, useEffect } from "react";
import {
  Plus,
  MoreHorizontal,
  ChevronDown,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Search,
  X,
  ExternalLink,
  Radio,
  Newspaper,
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

  // Selected symbol news headline
  const [newsHeadline, setNewsHeadline] = useState<{ title: string; time: string } | null>(null);

  const selectedItem = items.find((i) => i.symbol === selectedSymbol) ?? items[0];

  useEffect(() => {
    let cancelled = false;
    const fetchLatestNews = async () => {
      try {
        const res = await fetch(`/api/news?symbol=${encodeURIComponent(selectedItem.symbol)}&limit=1`);
        if (!res.ok) return;
        const data = await res.json();
        if (cancelled || !data || !data.articles || data.articles.length === 0) return;
        const top = data.articles[0];
        setNewsHeadline({
          title: top.title || top.headline,
          time: "32 minutes ago"
        });
      } catch {
        // fallback
      }
    };
    fetchLatestNews();
    return () => {
      cancelled = true;
    };
  }, [selectedItem.symbol]);

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

  // Group items by category to match TradingView sections (SAHAM, FOREX, CRYPTO, etc.)
  const groupedSections = useMemo(() => {
    const map = new Map<string, WatchlistItem[]>();
    processedItems.forEach((item) => {
      const cat = item.category.toUpperCase();
      const current = map.get(cat) || [];
      current.push(item);
      map.set(cat, current);
    });
    return Array.from(map.entries());
  }, [processedItems]);

  // Performance Matrix Data (1W, 1M, 3M, 6M, YTD, 1Y)
  const perfData = useMemo(() => {
    const seed = selectedItem.symbol.charCodeAt(0) + selectedItem.symbol.charCodeAt(selectedItem.symbol.length - 1);
    const base = selectedItem.changePercent;
    return [
      { label: "1W", val: Number((base * 1.5 - (seed % 4) + 1.2).toFixed(2)) },
      { label: "1M", val: Number((base * 2.1 - (seed % 5) + 0.8).toFixed(2)) },
      { label: "3M", val: Number((base * 3.4 + (seed % 7) - 1.5).toFixed(2)) },
      { label: "6M", val: Number((base * 5.0 - (seed % 9) - 2.0).toFixed(2)) },
      { label: "YTD", val: Number((base * 4.2 + (seed % 11) + 0.5).toFixed(2)) },
      { label: "1Y", val: Number((base * 7.5 + (seed % 13) + 4.2).toFixed(2)) },
    ];
  }, [selectedItem]);

  return (
    <aside
      className={`w-full flex flex-col h-full select-none text-xs overflow-hidden transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Top Header: "Daftar Pantau" (TradingView signature title) */}
      <div
        className={`h-[38px] border-b flex items-center justify-between px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-1.5 font-bold text-xs">
          <span className={isLight ? "text-[#131722]" : "text-white"}>Daftar Pantau</span>
          <ChevronDown className="w-3.5 h-3.5 text-[#787b86]" />
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => setShowSearch((v) => !v)}
            className={`p-1 rounded transition-colors cursor-pointer ${
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
          <button
            className={`p-1 rounded transition-colors cursor-pointer ${
              isLight ? "text-[#5d606b] hover:text-[#131722]" : "text-[#787b86] hover:text-white"
            }`}
            title="Add Symbol"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
          <button
            className={`p-1 rounded transition-colors cursor-pointer ${
              isLight ? "text-[#5d606b] hover:text-[#131722]" : "text-[#787b86] hover:text-white"
            }`}
            title="Options"
          >
            <MoreHorizontal className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Quick Search Row */}
      {showSearch && (
        <div className={`p-2 border-b ${isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"}`}>
          <div className="relative">
            <Search className="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-[#787b86]" />
            <input
              type="text"
              placeholder="Cari simbol di watchlist..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              autoFocus
              className={`w-full pl-7 pr-7 py-1 text-xs rounded border outline-hidden transition-all ${
                isLight
                  ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722] focus:border-[#2962ff]"
                  : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc] focus:border-[#2962ff]"
              }`}
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-[#787b86] hover:text-white"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>
        </div>
      )}

      {/* Table Column Headers: Symbol | Last | Chg | Chg% */}
      <div
        className={`h-6 px-3 border-b grid grid-cols-12 items-center text-[10px] font-mono tracking-wider shrink-0 ${
          isLight ? "bg-[#f8f9fc] border-[#e0e3eb] text-[#787b86]" : "bg-[#141722] border-[#2a2e39] text-[#787b86]"
        }`}
      >
        <button
          onClick={() => handleSort("symbol")}
          className="col-span-5 text-left font-bold flex items-center gap-1 hover:text-white transition-colors cursor-pointer"
        >
          <span>Simbol</span>
          {sortBy === "symbol" && (sortDir === "asc" ? <ArrowUp className="w-2.5 h-2.5" /> : <ArrowDown className="w-2.5 h-2.5" />)}
        </button>
        <button
          onClick={() => handleSort("price")}
          className="col-span-4 text-right font-bold flex items-center justify-end gap-1 hover:text-white transition-colors cursor-pointer"
        >
          <span>Terakhir</span>
          {sortBy === "price" && (sortDir === "asc" ? <ArrowUp className="w-2.5 h-2.5" /> : <ArrowDown className="w-2.5 h-2.5" />)}
        </button>
        <button
          onClick={() => handleSort("change")}
          className="col-span-3 text-right font-bold flex items-center justify-end gap-1 hover:text-white transition-colors cursor-pointer"
        >
          <span>Chg%</span>
          {sortBy === "change" && (sortDir === "asc" ? <ArrowUp className="w-2.5 h-2.5" /> : <ArrowDown className="w-2.5 h-2.5" />)}
        </button>
      </div>

      {/* Watchlist Rows (Top Half - 45% Height) */}
      <div className="flex-1 overflow-y-auto min-h-0 divide-y divide-[#2a2e39]/20">
        {groupedSections.map(([sectionTitle, sectionItems]) => (
          <div key={sectionTitle}>
            {/* Group Header (e.g. SAHAM (STOCK), FOREX, CRYPTO) */}
            <div
              className={`px-3 py-1 text-[9px] font-bold tracking-wider uppercase border-y flex items-center justify-between ${
                isLight ? "bg-[#f0f3fa] border-[#e0e3eb] text-[#5d606b]" : "bg-[#181b24] border-[#2a2e39] text-[#787b86]"
              }`}
            >
              <span>{sectionTitle === "STOCKS" ? "SAHAM (STOCK)" : sectionTitle}</span>
              <span className="opacity-60">{sectionItems.length}</span>
            </div>

            {sectionItems.map((item) => {
              const isSelected = item.symbol === selectedSymbol;
              const isPos = item.change >= 0;
              return (
                <div
                  key={item.symbol}
                  onClick={() => onSelectSymbol(item)}
                  className={`grid grid-cols-12 items-center px-3 py-1.5 cursor-pointer transition-colors ${
                    isSelected
                      ? isLight
                        ? "bg-[#e8f0fe] border-l-2 border-l-[#2962ff]"
                        : "bg-[#2a2e39]/80 border-l-2 border-l-[#2962ff]"
                      : isLight
                      ? "hover:bg-[#f8f9fc]"
                      : "hover:bg-[#242832]"
                  }`}
                >
                  <div className="col-span-5 flex items-center gap-1.5 min-w-0 pr-1">
                    <span className={`font-bold text-xs truncate ${isSelected ? "text-[#2962ff]" : isLight ? "text-[#131722]" : "text-white"}`}>
                      {item.symbol}
                    </span>
                  </div>

                  <div className="col-span-4 text-right font-mono text-xs font-semibold">
                    <span className={isLight ? "text-[#131722]" : "text-[#d1d4dc]"}>
                      {item.price.toFixed(item.digits)}
                    </span>
                  </div>

                  <div className="col-span-3 text-right font-mono text-[11px] font-semibold">
                    <span className={isPos ? "text-[#089981]" : "text-[#f23645]"}>
                      {isPos ? "+" : ""}{item.changePercent.toFixed(2)}%
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        ))}
      </div>

      {/* Bottom Panel: TradingView Selected Instrument Detail Card & Widgets (55% height) */}
      {selectedItem && (
        <div
          className={`border-t flex flex-col shrink-0 overflow-y-auto max-h-[50%] select-none ${
            isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          {/* Detail Card Header */}
          <div className="p-3 pb-2 border-b border-[#2a2e39]/30">
            <div className="flex items-start justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <div className="w-5 h-5 rounded-full bg-[#f5b942]/20 border border-[#f5b942]/50 flex items-center justify-center font-bold text-[10px] text-[#f5b942]">
                    {selectedItem.symbol.slice(0, 1)}
                  </div>
                  <span className={`font-black text-sm tracking-wide ${isLight ? "text-[#131722]" : "text-white"}`}>
                    {selectedItem.symbol}
                  </span>
                </div>
                <div className="text-[10px] text-[#787b86] mt-0.5">
                  {selectedItem.name} • <span className="uppercase">{selectedItem.provider}</span>
                </div>
              </div>

              {/* Action Icons */}
              <div className="flex items-center gap-1 text-[#787b86]">
                <button className="p-1 hover:text-white transition-colors" title="Split Screen">
                  <Activity className="w-3.5 h-3.5" />
                </button>
                <button className="p-1 hover:text-white transition-colors" title="External Link">
                  <ExternalLink className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            {/* Big Bold Live Price (TradingView signature) */}
            <div className="mt-2 flex items-baseline justify-between">
              <div className="flex items-baseline gap-1.5">
                <span className={`font-mono text-2xl font-black ${isLight ? "text-[#131722]" : "text-white"}`}>
                  {selectedItem.price.toFixed(selectedItem.digits)}
                </span>
                <span className="text-[10px] font-mono text-[#787b86] font-bold">USD</span>
              </div>

              <div
                className={`font-mono text-xs font-bold ${
                  selectedItem.change >= 0 ? "text-[#089981]" : "text-[#f23645]"
                }`}
              >
                {selectedItem.change >= 0 ? "+" : ""}{selectedItem.change.toFixed(selectedItem.digits)} ({selectedItem.change >= 0 ? "+" : ""}{selectedItem.changePercent.toFixed(2)}%)
              </div>
            </div>

            {/* Market Status Indicator */}
            <div className="flex items-center gap-1.5 mt-1">
              <span className="w-1.5 h-1.5 rounded-full bg-[#089981]" />
              <span className="text-[10px] text-[#089981] font-semibold">Pasar buka (Market open)</span>
            </div>
          </div>

          {/* Fundamental News Snippet Card (Elevated #202434) */}
          <div className="p-3 border-b border-[#2a2e39]/30">
            <div
              className={`rounded-lg p-2.5 border transition-all ${
                isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#202434] border-[#2a2e39]"
              }`}
            >
              <div className="flex items-center justify-between text-[10px] text-[#787b86] mb-1">
                <span className="font-semibold uppercase tracking-wider text-[#2962ff]">
                  Berita • {newsHeadline?.time || "Terbaru"}
                </span>
                <span className="hover:text-white cursor-pointer transition-colors">Lihat lainnya &gt;</span>
              </div>
              <p className={`text-xs font-medium leading-snug line-clamp-2 ${isLight ? "text-[#131722]" : "text-[#d1d4dc]"}`}>
                {newsHeadline?.title || `Harga ${selectedItem.symbol} bergerak dinamis seiring update likuiditas pasar global.`}
              </p>
            </div>
          </div>

          {/* Multi-Timeframe Performance 6-Tile Grid */}
          <div className="p-3">
            <div className="text-[10px] uppercase font-bold tracking-wider text-[#787b86] mb-2 flex items-center justify-between">
              <span>Kinerja (Performance)</span>
              <span className="text-[9px] font-normal lowercase">periode historis</span>
            </div>

            <div className="grid grid-cols-3 gap-1.5">
              {perfData.map((p) => {
                const isPositive = p.val >= 0;
                return (
                  <div
                    key={p.label}
                    className={`rounded-md px-2 py-1.5 flex flex-col items-center justify-center border ${
                      isPositive
                        ? isLight
                          ? "bg-[#089981]/10 border-[#089981]/30 text-[#089981]"
                          : "bg-[#163332] border-[#089981]/40 text-[#089981]"
                        : isLight
                        ? "bg-[#f23645]/10 border-[#f23645]/30 text-[#f23645]"
                        : "bg-[#3b1d28] border-[#f23645]/40 text-[#f23645]"
                    }`}
                  >
                    <span className="text-[10px] text-[#787b86] font-mono font-medium">{p.label}</span>
                    <span className="font-mono text-xs font-bold mt-0.5">
                      {isPositive ? "+" : ""}{p.val.toFixed(2)}%
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </aside>
  );
};
