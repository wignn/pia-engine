"use client";

import React, { useState, useRef } from "react";
import {
  Plus,
  X,
  BarChart3,
  Newspaper,
  Radio,
  Layers,
  Brain,
  Calendar,
  Tv,
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
  const [menuPos, setMenuPos] = useState<{ top: number; left: number } | null>(null);
  const menuButtonRef = useRef<HTMLButtonElement | null>(null);

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

  const toggleAddMenu = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (menuPos) {
      setMenuPos(null);
    } else {
      const rect = e.currentTarget.getBoundingClientRect();
      const left = Math.min(rect.left, window.innerWidth - 200);
      setMenuPos({ top: rect.bottom + 4, left: Math.max(8, left) });
    }
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
      case "live":
        return <Tv className="w-3.5 h-3.5 text-[#f23645]" />;
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
        case "live":
          return "Live TV";
      }
    }
    return tab.symbol;
  };

  return (
    <>
      <div
        className={`h-[34px] border-b flex items-center px-2 select-none overflow-x-auto gap-1 relative z-20 shrink-0 ${
          isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
        }`}
      >
        {/* Tab Items */}
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
              className={`group h-[27px] px-2.5 rounded-t flex items-center gap-2 cursor-pointer text-xs font-semibold transition-all border border-b-0 shrink-0 ${
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

        {/* New Tab Action Buttons (Immediate Add + Dropdown Menu) */}
        <div className="flex items-center ml-1 border rounded shrink-0 overflow-hidden border-inherit bg-inherit">
          {/* Main '+' Button -> Directly Creates New Tab */}
          <button
            onClick={() => onNewTab("chart")}
            className={`p-1.5 transition-colors cursor-pointer flex items-center justify-center ${
              isLight ? "hover:bg-[#e0e3eb] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
            }`}
            title="Add New Chart Tab"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>

          {/* Secondary Chevron Button -> Opens Custom Tab Type Menu */}
          <button
            ref={menuButtonRef}
            onClick={toggleAddMenu}
            className={`px-1 py-1.5 border-l transition-colors cursor-pointer flex items-center justify-center border-inherit ${
              isLight ? "hover:bg-[#e0e3eb] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
            }`}
            title="Add Custom Tab (News, Social, DOM, Intel, Calendar)"
          >
            <ChevronDown className="w-2.5 h-2.5 opacity-70" />
          </button>
        </div>
      </div>

      {/* Floating Dropdown Menu (Fixed positioned so it is NEVER clipped by overflow-x-auto) */}
      {menuPos && (
        <>
          <div
            className="fixed inset-0 z-50 bg-transparent"
            onClick={() => setMenuPos(null)}
          />
          <div
            style={{ top: `${menuPos.top}px`, left: `${menuPos.left}px` }}
            className={`fixed z-50 w-48 rounded-xl border shadow-2xl p-1 flex flex-col gap-0.5 text-xs select-none animate-in fade-in zoom-in-95 duration-100 ${
              isLight
                ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
                : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
            }`}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="text-[9px] uppercase tracking-wider font-bold px-2 py-1 opacity-50 border-b border-inherit mb-0.5">
              Add New Tab
            </div>
            <button
              onClick={() => {
                onNewTab("chart");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <BarChart3 className="w-3.5 h-3.5 text-[#2962ff]" />
              <div className="flex flex-col">
                <span className="font-semibold">Chart Tab</span>
                <span className="text-[10px] text-[#787b86]">Candlestick &amp; tools</span>
              </div>
            </button>
            <button
              onClick={() => {
                onNewTab("news");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <Newspaper className="w-3.5 h-3.5 text-[#f5b942]" />
              <div className="flex flex-col">
                <span className="font-semibold">News Stream</span>
                <span className="text-[10px] text-[#787b86]">Live market wire</span>
              </div>
            </button>
            <button
              onClick={() => {
                onNewTab("social");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <Radio className="w-3.5 h-3.5 text-[#00b4d8]" />
              <div className="flex flex-col">
                <span className="font-semibold">Social Pulse</span>
                <span className="text-[10px] text-[#787b86]">Real-time X tweets</span>
              </div>
            </button>
            <button
              onClick={() => {
                onNewTab("orderbook");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <Layers className="w-3.5 h-3.5 text-[#089981]" />
              <div className="flex flex-col">
                <span className="font-semibold">Order Book &amp; DOM</span>
                <span className="text-[10px] text-[#787b86]">Depth &amp; trade tape</span>
              </div>
            </button>
            <button
              onClick={() => {
                onNewTab("intelligence");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <Brain className="w-3.5 h-3.5 text-[#a855f7]" />
              <div className="flex flex-col">
                <span className="font-semibold">Market Intel</span>
                <span className="text-[10px] text-[#787b86]">Macro, yield, options</span>
              </div>
            </button>
            <button
              onClick={() => {
                onNewTab("calendar");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <Calendar className="w-3.5 h-3.5 text-[#f23645]" />
              <div className="flex flex-col">
                <span className="font-semibold">Economic Calendar</span>
                <span className="text-[10px] text-[#787b86]">CPI, NFP, interest rates</span>
              </div>
            </button>
            <button
              onClick={() => {
                onNewTab("live");
                setMenuPos(null);
              }}
              className={`flex items-center gap-2 px-2.5 py-1.5 rounded cursor-pointer text-left transition-colors ${
                isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <Tv className="w-3.5 h-3.5 text-[#f23645]" />
              <div className="flex flex-col">
                <span className="font-semibold">Live Broadcast</span>
                <span className="text-[10px] text-[#787b86]">Bloomberg, CNBC, FOMC</span>
              </div>
            </button>
          </div>
        </>
      )}
    </>
  );
};
