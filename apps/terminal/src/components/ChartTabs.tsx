"use client";

import React from "react";
import { Plus, X, BarChart3 } from "lucide-react";
import { TabItem } from "@/types";

interface ChartTabsProps {
  tabs: TabItem[];
  activeTabId: string;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onNewTab: () => void;
}

export const ChartTabs: React.FC<ChartTabsProps> = ({
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
  onNewTab,
}) => {
  return (
    <div className="h-[34px] bg-[#181b27] border-b border-[#2a2e39] flex items-center px-2 select-none overflow-x-auto gap-1">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            onClick={() => onSelectTab(tab.id)}
            className={`group h-[28px] px-3 rounded flex items-center gap-2 cursor-pointer text-xs font-semibold transition-all border ${
              isActive
                ? "bg-[#1e222d] text-white border-[#2a2e39] shadow-xs"
                : "bg-transparent text-[#787b86] border-transparent hover:bg-[#1e222d]/60 hover:text-[#d1d4dc]"
            }`}
          >
            <BarChart3 className={`w-3.5 h-3.5 ${isActive ? "text-[#2962ff]" : "text-[#787b86]"}`} />
            <span className="tracking-wide">{tab.symbol}</span>
            <span className="text-[10px] font-mono px-1 py-0.2 bg-[#131722] rounded text-[#787b86]">
              {tab.timeframe}
            </span>

            {tabs.length > 1 && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onCloseTab(tab.id);
                }}
                className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-white transition-opacity"
                title="Close Tab"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>
        );
      })}

      {/* New Tab Button */}
      <button
        onClick={onNewTab}
        className="p-1 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-white transition-colors ml-1"
        title="Open New Tab (Select Symbol)"
      >
        <Plus className="w-3.5 h-3.5" />
      </button>
    </div>
  );
};
