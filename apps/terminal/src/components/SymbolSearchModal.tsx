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
}

export const SymbolSearchModal: React.FC<SymbolSearchModalProps> = ({
  isOpen,
  onClose,
  items,
  onSelect,
  initialQuery = "",
}) => {
  const [query, setQuery] = useState(initialQuery);
  const [tab, setTab] = useState<"all" | "crypto" | "forex" | "indices" | "commodities" | "stocks">("all");
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
        className="w-[580px] max-h-[520px] bg-[#1e222d] border border-[#2a2e39] rounded-xl shadow-2xl flex flex-col overflow-hidden text-xs text-[#d1d4dc]"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* Header Search Bar */}
        <div className="flex items-center px-4 py-3 border-b border-[#2a2e39] gap-3">
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
            className="flex-1 bg-transparent border-none outline-hidden text-white placeholder-[#787b86] text-sm font-semibold"
          />
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-white cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Filter Categories */}
        <div className="flex items-center gap-1.5 px-4 py-2 border-b border-[#2a2e39] bg-[#181b27] overflow-x-auto">
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
                  : "text-[#787b86] hover:text-white hover:bg-[#2a2e39]"
              }`}
            >
              {c}
            </button>
          ))}
        </div>

        {/* Results List */}
        <div className="flex-1 overflow-y-auto divide-y divide-[#2a2e39]/50">
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
                    isSelected ? "bg-[#2a2e39]" : "hover:bg-[#2a2e39]/60"
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className="flex flex-col">
                      <div className="flex items-center gap-2">
                        <span className="font-bold text-white text-sm">{item.symbol}</span>
                        <span className="text-[10px] font-mono text-[#787b86] bg-[#131722] px-1.5 py-0.5 rounded">
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
                    <div className="text-white font-bold text-sm">
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
