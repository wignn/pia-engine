"use client";

import React from "react";
import { 
  Bookmark, 
  Newspaper, 
  Bell, 
  Layers, 
  Calendar,
  Brain,
  Radio,
  Tv,
  Settings2 
} from "lucide-react";

export type SidebarTab = "watchlist" | "news" | "alerts" | "calendar" | "intelligence" | "social" | "orderbook" | "live";

interface RightDockProps {
  activeTab: SidebarTab;
  setActiveTab: (tab: SidebarTab) => void;
  theme?: "dark" | "light";
}

export const RightDock: React.FC<RightDockProps> = ({ activeTab, setActiveTab, theme = "dark" }) => {
  const isLight = theme === "light";

  const tabs: { id: SidebarTab; label: string; icon: React.ReactNode }[] = [
    { id: "watchlist", label: "Watchlist & Details", icon: <Bookmark className="w-4 h-4" /> },
    { id: "orderbook", label: "Order Book & Trades", icon: <Layers className="w-4 h-4" /> },
    { id: "news", label: "News Headlines", icon: <Newspaper className="w-4 h-4" /> },
    { id: "alerts", label: "Price Alerts", icon: <Bell className="w-4 h-4" /> },
    { id: "calendar", label: "Economic Calendar", icon: <Calendar className="w-4 h-4" /> },
    { id: "intelligence", label: "Market Intelligence", icon: <Brain className="w-4 h-4" /> },
    { id: "social", label: "Social Pulse", icon: <Radio className="w-4 h-4" /> },
    { id: "live", label: "Live Broadcast & TV", icon: <Tv className="w-4 h-4 text-[#f23645]" /> }
  ];

  return (
    <div
      className={`hidden lg:flex w-[45px] border-l flex-col items-center py-2 justify-between select-none z-10 shrink-0 transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
      }`}
    >
      <div className="flex flex-col items-center gap-1 w-full">
        {tabs.map((tab) => {
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`p-2.5 rounded transition-all relative group cursor-pointer ${
                isActive
                  ? "text-[#2962ff] bg-[#2962ff]/10"
                  : isLight
                  ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
                  : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
              }`}
              title={tab.label}
            >
              {tab.icon}
              {isActive && (
                <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-[#2962ff] rounded-r" />
              )}
            </button>
          );
        })}
      </div>

      <div className={`flex flex-col items-center gap-1 w-full pt-2 border-t ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
        <button
          className={`p-2.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
          }`}
          title="Dock Bar"
        >
          <Settings2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
