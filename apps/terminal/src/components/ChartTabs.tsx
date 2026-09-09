"use client";

import React, { useState } from "react";
import {
  Plus,
  X,
  BarChart3,
  Newspaper,
  Radio,
  Layers,
  Brain,
  Calendar,
  GripVertical,
  ChevronDown
} from "lucide-react";
import { TabItem, TabContentType } from "@/types";

interface ChartTabsProps {
  tabs: TabItem[];
  activeTabId: string;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onNewTab: (type?: TabContentType) => void;
  onReorderTabs: (newTabs: TabItem[]) => void;
  theme?: "dark" | "light";
}

export const ChartTabs: React.FC<ChartTabsProps> = ({
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
  onNewTab,
  onReorderTabs,
  theme = "dark",
}) => {
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [isAddMenuOpen, setIsAddMenuOpen] = useState(false);

  const isLight = theme === "light";

  const handleDragStart = (e: React.DragEvent, index: number) => {
    setDraggedIndex(index);
    e.dataTransfer.effectAllowed = "move";
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) return;
    const newTabs = [...tabs];
    const item = newTabs.splice(draggedIndex, 1)[0];
    newTabs.splice(index, 0, item);
    setDraggedIndex(index);
    onReorderTabs(newTabs);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
  };

  const getTabIcon = (type?: TabContentType) => {
    switch (type) {
      case "news":
        return <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" />;
      case "social":
        return <Radio className="w-3.5 h-3.5 text-[#00b4d8]" />;
      case "orderbook":
        return <Layers className="w-3.5 h-3.5 text-[#089981]" />;
      case "intelligence":
        return <Brain className="w-3.5 h-3.5 text-[#a855f7]" />;
      case "calendar":
        return <Calendar className="w-3.5 h-3.5 text-[#f23645]" />;
      default:
        return <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" />;
    }
  };

  const getTabLabel = (tab: TabItem) => {
    if (tab.type && tab.type !== "chart") {
      switch (tab.type) {
        case "news":
          return "News Stream";
        case "social":
          return "Social Pulse";
        case "orderbook":
          return `DOM · ${tab.symbol}`;
        case "intelligence":
          return "Market Intel";
        case "calendar":
          return "Economic Calendar";
      }
    }
    return tab.symbol;
  };

  return (
    <div
      className={`h-[34px] border-b flex items-center px-2 select-none overflow-x-auto gap-1 relative z-20 shrink-0 ${
        isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
      }`}
    >
      {tabs.map((tab, idx) => {
        const isActive = tab.id === activeTabId;
        const isDragging = draggedIndex === idx;

        return (
          <div
            key={tab.id}
            draggable={true}
            onDragStart={(e) => handleDragStart(e, idx)}
            onDragOver={(e) => handleDragOver(e, idx)}
            onDragEnd={handleDragEnd}
            onClick={() => onSelectTab(tab.id)}
            className={`group h-[27px] px-2.5 rounded-t flex items-center gap-2 cursor-pointer text-xs font-semibold transition-all border border-b-0 ${
              isDragging ? "opacity-40 scale-95" : "opacity-100"
            } ${
              isActive
                ? isLight
                  ? "bg-[#ffffff] text-[#131722] border-[#e0e3eb] shadow-xs"
                  : "bg-[#1e222d] text-white border-[#2a2e39] shadow-xs"
                : isLight
                ? "bg-transparent text-[#5d606b] border-transparent hover:bg-[#ffffff]/60 hover:text-black"
                : "bg-transparent text-[#787b86] border-transparent hover:bg-[#1e222d]/60 hover:text-[#d1d4dc]"
            }`}
          >
            <GripVertical className="w-3 h-3 text-[#787b86]/40 group-hover:text-[#787b86] -ml-0.5 cursor-grab" />
            {getTabIcon(tab.type)}
            <span className="tracking-wide">{getTabLabel(tab)}</span>

            {(!tab.type || tab.type === "chart") && (
              <span
                className={`text-[10px] font-mono px-1 py-0.2 rounded ${
                  isLight ? "bg-[#e0e3eb] text-[#5d606b]" : "bg-[#131722] text-[#787b86]"
                }`}
              >
                {tab.timeframe}
              </span>
            )}

            {tabs.length > 1 && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onCloseTab(tab.id);
                }}
                className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-black/10 dark:hover:bg-white/10 text-[#787b86] hover:text-white transition-opacity cursor-pointer"
                title="Close Tab"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>
        );
      })}

      {/* New Tab Menu Button */}
      <div className="relative">
        <button
          onClick={() => setIsAddMenuOpen((v) => !v)}
          className={`flex items-center gap-0.5 p-1 rounded transition-colors ml-1 cursor-pointer ${
            isLight ? "hover:bg-[#e0e3eb] text-[#5d606b]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
          }`}
          title="Add Custom Tab"
        >
          <Plus className="w-3.5 h-3.5" />
          <ChevronDown className="w-2.5 h-2.5 opacity-60" />
        </button>

        {isAddMenuOpen && (
          <>
            <div
              className="fixed inset-0 z-30"
              onClick={() => setIsAddMenuOpen(false)}
            />
            <div
              className={`absolute top-full left-0 z-40 mt-1 w-44 rounded-lg border shadow-xl p-1 flex flex-col gap-0.5 text-xs select-none ${
                isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <button
                onClick={() => {
                  onNewTab("chart");
                  setIsAddMenuOpen(false);
                }}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
              >
                <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" />
                <span>New Chart Tab</span>
              </button>
              <button
                onClick={() => {
                  onNewTab("news");
                  setIsAddMenuOpen(false);
                }}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
              >
                <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" />
                <span>News Headlines</span>
              </button>
              <button
                onClick={() => {
                  onNewTab("social");
                  setIsAddMenuOpen(false);
                }}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
              >
                <Radio className="w-3.5 h-3.5 text-[#00b4d8]" />
                <span>Social Pulse</span>
              </button>
              <button
                onClick={() => {
                  onNewTab("orderbook");
                  setIsAddMenuOpen(false);
                }}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
              >
                <Layers className="w-3.5 h-3.5 text-[#089981]" />
                <span>Order Book &amp; DOM</span>
              </button>
              <button
                onClick={() => {
                  onNewTab("intelligence");
                  setIsAddMenuOpen(false);
                }}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
              >
                <Brain className="w-3.5 h-3.5 text-[#a855f7]" />
                <span>Market Intel</span>
              </button>
              <button
                onClick={() => {
                  onNewTab("calendar");
                  setIsAddMenuOpen(false);
                }}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded hover:bg-black/5 dark:hover:bg-[#2a2e39] cursor-pointer text-left"
              >
                <Calendar className="w-3.5 h-3.5 text-[#f23645]" />
                <span>Economic Calendar</span>
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
};
