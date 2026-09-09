"use client";

import React, { useState, useEffect } from "react";
import { Search, X, Check } from "lucide-react";
import { WatchlistItem } from "@/types";

interface SymbolSearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  items: WatchlistItem[];
  onSelect: (item: WatchlistItem) => void;
  initialQuery?: string;
  theme?: "dark" | "light";
}

export const SymbolSearchModal: React.FC<SymbolSearchModalProps> = ({
  isOpen,
  onClose,
  items,
  onSelect,
  initialQuery = "",
  theme = "dark",
}) => {
  const isLight = theme === "light";
  const [query, setQuery] = useState(initialQuery);
  const [tab, setTab] = useState<"all" | "commodities" | "indices" | "forex" | "crypto" | "stocks">("all");
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    if (isOpen) {
      setQuery(initialQuery);
      setSelectedIndex(0);
    }
  }, [isOpen, initialQuery]);

  if (!isOpen) return null;

  const filtered = items.filter((item) => {
    const matchesQuery =
      item.symbol.toLowerCase().includes(query.toLowerCase()) ||
      item.name.toLowerCase().includes(query.toLowerCase());
    const matchesTab = tab === "all" || item.category === tab;
    return matchesQuery && matchesTab;
  });

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, Math.max(0, filtered.length - 1)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === "Enter" && filtered.length > 0) {
      e.preventDefault();
      const target = filtered[selectedIndex] || filtered[0];
      if (target) {
        onSelect(target);
        onClose();
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className={`w-[580px] max-h-[520px] rounded-xl shadow-2xl flex flex-col overflow-hidden text-xs transition-colors ${
          isLight
            ? "bg-[#ffffff] border border-[#e0e3eb] text-[#131722]"
            : "bg-[#1e222d] border border-[#2a2e39] text-[#d1d4dc]"
        }`}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* Header Search Bar */}
        <div
          className={`flex items-center px-4 py-3 border-b gap-3 ${
            isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
          }`}
        >
          <Search className="w-4 h-4 text-[#787b86]" />
          <input
            type="text"
            placeholder="Search symbol, currency, commodity, or stock..."
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            autoFocus
            className={`flex-1 bg-transparent border-none outline-hidden text-sm font-semibold ${
              isLight ? "text-[#131722] placeholder-[#8e929d]" : "text-white placeholder-[#787b86]"
            }`}
          />
          <button
            onClick={onClose}
            className={`p-1 rounded cursor-pointer transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
            }`}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Filter Categories */}
        <div
          className={`flex items-center gap-1.5 px-4 py-2 border-b overflow-x-auto ${
            isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
          }`}
        >
          {(["all", "commodities", "indices", "forex", "crypto", "stocks"] as const).map((c) => (
            <button
              key={c}
              onClick={() => {
                setTab(c);
                setSelectedIndex(0);
              }}
              className={`px-2.5 py-1 rounded capitalize font-medium cursor-pointer transition-colors ${
                tab === c
                  ? "bg-[#2962ff] text-white font-bold"
                  : isLight
                  ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#e0e3eb]"
                  : "text-[#787b86] hover:text-white hover:bg-[#2a2e39]"
              }`}
            >
              {c}
            </button>
          ))}
        </div>

        {/* Results List */}
        <div className={`flex-1 overflow-y-auto divide-y ${isLight ? "divide-[#e0e3eb]" : "divide-[#2a2e39]/50"}`}>
          {filtered.length === 0 ? (
            <div className="p-8 text-center text-[#787b86]">No instruments found for "{query}"</div>
          ) : (
            filtered.map((item, idx) => {
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={item.symbol}
                  onClick={() => {
                    onSelect(item);
                    onClose();
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`flex items-center justify-between px-4 py-3 cursor-pointer transition-colors ${
                    isSelected
                      ? isLight
                        ? "bg-[#2962ff]/10"
                        : "bg-[#2a2e39]"
                      : isLight
                      ? "hover:bg-[#f0f3fa]"
                      : "hover:bg-[#2a2e39]/60"
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className="flex flex-col">
                      <div className="flex items-center gap-2">
                        <span className={`font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>
                          {item.symbol}
                        </span>
                        <span
                          className={`text-[10px] font-mono px-1.5 py-0.5 rounded ${
                            isLight ? "bg-[#f0f3fa] text-[#5d606b]" : "bg-[#131722] text-[#787b86]"
                          }`}
                        >
                          {item.provider}
                        </span>
                        <span className="text-[10px] font-mono uppercase text-[#2962ff] bg-[#2962ff]/10 px-1.5 py-0.5 rounded">
                          {item.category}
                        </span>
                      </div>
                      <span className="text-[11px] text-[#787b86] mt-0.5">{item.name}</span>
                    </div>
                  </div>

                  <div className="text-right font-mono">
                    <div className={`font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>
                      {item.price.toFixed(item.digits)}
                    </div>
                    <div
                      className={`text-[11px] font-semibold ${
                        item.change >= 0 ? "text-[#089981]" : "text-[#f23645]"
                      }`}
                    >
                      {item.change >= 0 ? "+" : ""}
                      {item.change.toFixed(item.digits)} ({item.change >= 0 ? "+" : ""}
                      {item.changePercent.toFixed(2)}%)
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
