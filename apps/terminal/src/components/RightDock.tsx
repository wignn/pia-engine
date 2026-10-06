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
  PieChart,
  Globe2,
  TrendingUp,
  ShieldAlert,
  Zap,
  FileText,
  Briefcase,
  Settings2 
} from "lucide-react";

export type SidebarTab =
  | "watchlist"
  | "orderbook"
  | "news"
  | "alerts"
  | "calendar"
  | "intelligence"
  | "social"
  | "live"
  | "options"
  | "macro"
  | "yields"
  | "geosignals"
  | "energy"
  | "sec"
  | "paper";

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
    { id: "options", label: "Options & GEX Analysis", icon: <PieChart className="w-4 h-4 text-[#00e5ff]" /> },
    { id: "macro", label: "Macro & Central Banks", icon: <Globe2 className="w-4 h-4 text-[#f59e0b]" /> },
    { id: "yields", label: "Treasury Yields & Spreads", icon: <TrendingUp className="w-4 h-4 text-[#10b981]" /> },
    { id: "geosignals", label: "Geopolitical Signals", icon: <ShieldAlert className="w-4 h-4 text-[#f43f5e]" /> },
    { id: "energy", label: "Energy Complex", icon: <Zap className="w-4 h-4 text-[#eab308]" /> },
    { id: "sec", label: "SEC Filings", icon: <FileText className="w-4 h-4 text-[#8b5cf6]" /> },
    { id: "paper", label: "Paper Trading Simulator", icon: <Briefcase className="w-4 h-4 text-[#06b6d4]" /> },
    { id: "live", label: "Live Broadcast & TV", icon: <Tv className="w-4 h-4 text-[#f23645]" /> }
  ];

  return (
    <div
      className={`hidden lg:flex w-[45px] border-l flex-col items-center py-2 justify-between select-none z-10 shrink-0 transition-colors overflow-y-auto ${
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

      <div className={`flex flex-col items-center gap-1 w-full pt-2 border-t shrink-0 ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`}>
        <button
          className={`p-2.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
          }`}
          title="Dock Bar Settings"
        >
          <Settings2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
