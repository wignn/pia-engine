"use client";

import React from "react";
import { 
  Bookmark, 
  Newspaper, 
  Bell, 
  Layers, 
  Calendar, 
  Flame, 
  TrendingUp, 
  Settings2 
} from "lucide-react";

export type SidebarTab = "watchlist" | "news" | "alerts" | "calendar";

interface RightDockProps {
  activeTab: SidebarTab;
  setActiveTab: (tab: SidebarTab) => void;
}

export const RightDock: React.FC<RightDockProps> = ({ activeTab, setActiveTab }) => {
  const tabs: { id: SidebarTab; label: string; icon: React.ReactNode }[] = [
    { id: "watchlist", label: "Watchlist & Details", icon: <Bookmark className="w-4 h-4" /> },
    { id: "news", label: "News Headlines", icon: <Newspaper className="w-4 h-4" /> },
    { id: "alerts", label: "Price Alerts", icon: <Bell className="w-4 h-4" /> },
    { id: "calendar", label: "Economic Calendar", icon: <Calendar className="w-4 h-4" /> },
  ];

  return (
    <div className="w-[45px] bg-[#1e222d] border-l border-[#2a2e39] flex flex-col items-center py-2 justify-between select-none z-10">
      <div className="flex flex-col items-center gap-1 w-full">
        {tabs.map((tab) => {
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`p-2.5 rounded transition-all relative group ${
                isActive
                  ? "text-[#2962ff] bg-[#2962ff]/10"
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

      <div className="flex flex-col items-center gap-1 w-full pt-2 border-t border-[#2a2e39]">
        <button className="p-2.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Layout Settings">
          <Settings2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
