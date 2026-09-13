"use client";

import React from "react";
import {
  TrendingUp,
  Bookmark,
  Layers,
  Bell,
  Newspaper,
  Calendar,
  PenTool,
} from "lucide-react";
import { SidebarTab } from "./RightDock";
import { DrawingTool } from "@/types";

interface MobileBottomNavProps {
  activeTab: SidebarTab;
  setActiveTab: (tab: SidebarTab) => void;
  isDrawerOpen: boolean;
  setIsDrawerOpen: (open: boolean) => void;
  activeTool: DrawingTool;
  setActiveTool: (tool: DrawingTool) => void;
}

export const MobileBottomNav: React.FC<MobileBottomNavProps> = ({
  activeTab,
  setActiveTab,
  isDrawerOpen,
  setIsDrawerOpen,
  activeTool,
  setActiveTool,
}) => {
  const navItems: {
    id: "chart" | SidebarTab | "drawing";
    label: string;
    icon: React.ReactNode;
  }[] = [
    {
      id: "chart",
      label: "Chart",
      icon: <TrendingUp className="w-4 h-4" />,
    },
    {
      id: "watchlist",
      label: "Watchlist",
      icon: <Bookmark className="w-4 h-4" />,
    },
    {
      id: "orderbook",
      label: "DOM Tape",
      icon: <Layers className="w-4 h-4" />,
    },
    {
      id: "alerts",
      label: "Alerts",
      icon: <Bell className="w-4 h-4" />,
    },
    {
      id: "news",
      label: "News",
      icon: <Newspaper className="w-4 h-4" />,
    },
    {
      id: "calendar",
      label: "Calendar",
      icon: <Calendar className="w-4 h-4" />,
    },
    {
      id: "drawing",
      label: activeTool === "cursor" ? "Tools" : activeTool,
      icon: <PenTool className="w-4 h-4" />,
    },
  ];

  const handleItemClick = (id: "chart" | SidebarTab | "drawing") => {
    if (id === "chart") {
      setIsDrawerOpen(false);
      return;
    }
    if (id === "drawing") {
      setIsDrawerOpen(false);
      // Toggle drawing tools
      if (activeTool === "cursor") setActiveTool("trendline");
      else if (activeTool === "trendline") setActiveTool("horizontal");
      else if (activeTool === "horizontal") setActiveTool("fibonacci");
      else if (activeTool === "fibonacci") setActiveTool("measure");
      else setActiveTool("cursor");
      return;
    }

    setActiveTab(id);
    setIsDrawerOpen(true);
  };

  return (
    <nav className="flex md:hidden h-[56px] min-h-[56px] w-full items-center justify-around border-t border-[#2a2e39] bg-[#1e222d] px-0.5 pb-[env(safe-area-inset-bottom)] z-30 select-none">
      {navItems.map((item) => {
        const isCurrentActive =
          item.id === "chart"
            ? !isDrawerOpen && activeTool === "cursor"
            : item.id === "drawing"
            ? activeTool !== "cursor"
            : isDrawerOpen && activeTab === item.id;

        return (
          <button
            key={item.id}
            type="button"
            onClick={() => handleItemClick(item.id)}
            className={`flex min-w-0 flex-1 max-w-[84px] flex-col items-center justify-center gap-0.5 px-0.5 py-1 transition-colors ${
              isCurrentActive
                ? "text-[#2962ff] font-bold"
                : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {item.icon}
            <span className="max-w-full truncate text-[9px] capitalize tracking-tight">{item.label}</span>
          </button>
        );
      })}
    </nav>
  );
};
