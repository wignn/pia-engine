"use client";

import React from "react";
import {
  X,
  BarChart3,
  Layers,
  Newspaper,
  Radio,
  Brain,
  Calendar,
  Grid3X3,
  Sliders,
  Settings,
  Moon,
  Sun,
  Keyboard,
  ShieldCheck,
  Search,
  ExternalLink
} from "lucide-react";
import { PaneContentType } from "@/types";

interface TradingViewMainMenuProps {
  isOpen: boolean;
  onClose: () => void;
  currentPaneType?: PaneContentType;
  onSelectPaneType: (type: PaneContentType) => void;
  theme?: "dark" | "light";
  onToggleTheme?: () => void;
  onOpenSettings?: () => void;
}

export const TradingViewMainMenu: React.FC<TradingViewMainMenuProps> = ({
  isOpen,
  onClose,
  currentPaneType = "chart",
  onSelectPaneType,
  theme = "dark",
  onToggleTheme,
  onOpenSettings
}) => {
  const isLight = theme === "light";

  if (!isOpen) return null;

  const productItems: {
    id: PaneContentType;
    label: string;
    description: string;
    icon: React.ReactNode;
    badge?: string;
  }[] = [
    {
      id: "chart",
      label: "Supercharts",
      description: "Financial charts with multi-pane analysis & indicators",
      icon: <BarChart3 className="w-4 h-4 text-[#2962ff]" />
    },
    {
      id: "orderbook",
      label: "DOM & Order Book",
      description: "Level 2 market depth and live transaction flow",
      icon: <Layers className="w-4 h-4 text-[#089981]" />
    },
    {
      id: "news",
      label: "News & Market Wire",
      description: "Breaking global macro headlines & corporate filings",
      icon: <Newspaper className="w-4 h-4 text-[#f5b942]" />
    },
    {
      id: "social",
      label: "Social Pulse",
      description: "Real-time 𝕏/Twitter verified sentiment streams",
      icon: <Radio className="w-4 h-4 text-[#e040fb]" />,
      badge: "LIVE"
    },
    {
      id: "intelligence",
      label: "Market Intelligence",
      description: "AI institutional signals, COT reports, & liquidity",
      icon: <Brain className="w-4 h-4 text-[#00e5ff]" />
    },
    {
      id: "calendar",
      label: "Economic Calendar",
      description: "Central bank announcements, CPI, rate decisions",
      icon: <Calendar className="w-4 h-4 text-[#ff5252]" />
    }
  ];

  return (
    <div className="fixed inset-0 z-50 flex" onClick={onClose}>
      {/* Backdrop */}
      <div className="fixed inset-0 bg-black/50 backdrop-blur-xs transition-opacity" />

      {/* Slide-out Drawer (TradingView Desktop Style) */}
      <aside
        className={`relative w-[320px] max-w-[85vw] h-full flex flex-col shadow-2xl z-10 select-none overflow-hidden transition-all ${
          isLight
            ? "bg-[#ffffff] border-r border-[#e0e3eb] text-[#131722]"
            : "bg-[#1e222d] border-r border-[#2a2e39] text-[#d1d4dc]"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Drawer Header */}
        <div
          className={`h-12 px-4 border-b flex items-center justify-between shrink-0 ${
            isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
          }`}
        >
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 rounded flex items-center justify-center bg-[#2962ff] text-white font-black text-xs">
              P
            </div>
            <div>
              <span className={`font-black tracking-wider text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>
                PIA TERMINAL
              </span>
              <span className="text-[10px] text-[#787b86] block -mt-1 font-mono">
                TradingView Standard
              </span>
            </div>
          </div>
          <button
            onClick={onClose}
            className={`p-1.5 rounded-lg cursor-pointer transition-colors ${
              isLight ? "hover:bg-[#e0e3eb] text-[#5d606b]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
            }`}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Scrollable Navigation Body */}
        <div className="flex-1 overflow-y-auto p-3 space-y-4 text-xs">
          {/* Section: Products */}
          <div>
            <div className="text-[10px] uppercase font-bold tracking-wider px-2 pb-1.5 text-[#787b86]">
              Products & Workspace
            </div>
            <div className="space-y-0.5">
              {productItems.map((item) => {
                const isCurrent = currentPaneType === item.id;
                return (
                  <button
                    key={item.id}
                    onClick={() => {
                      onSelectPaneType(item.id);
                      onClose();
                    }}
                    className={`w-full flex items-start gap-3 rounded-lg px-3 py-2 text-left cursor-pointer transition-all ${
                      isCurrent
                        ? "bg-[#2962ff]/15 text-[#2962ff] font-bold"
                        : isLight
                        ? "hover:bg-[#f0f3fa] text-[#131722]"
                        : "hover:bg-[#2a2e39] text-[#d1d4dc]"
                    }`}
                  >
                    <div className="mt-0.5 shrink-0">{item.icon}</div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between">
                        <span className="text-xs font-semibold">{item.label}</span>
                        {item.badge && (
                          <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-[#e040fb]/20 text-[#e040fb]">
                            {item.badge}
                          </span>
                        )}
                      </div>
                      <p className="text-[10px] text-[#787b86] font-normal leading-tight mt-0.5 truncate">
                        {item.description}
                      </p>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          <div className={`border-t ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`} />

          {/* Section: Community & Feeds */}
          <div>
            <div className="text-[10px] uppercase font-bold tracking-wider px-2 pb-1.5 text-[#787b86]">
              Community & Feeds
            </div>
            <div className="space-y-0.5">
              <button
                onClick={() => {
                  onSelectPaneType("social");
                  onClose();
                }}
                className={`w-full flex items-center gap-3 rounded-lg px-3 py-2 text-left cursor-pointer transition-colors ${
                  isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
                }`}
              >
                <Radio className="w-4 h-4 text-[#e040fb]" />
                <span className="text-xs font-medium">Trade Ideas & Live Pulse</span>
              </button>
              <button
                onClick={() => {
                  onSelectPaneType("news");
                  onClose();
                }}
                className={`w-full flex items-center gap-3 rounded-lg px-3 py-2 text-left cursor-pointer transition-colors ${
                  isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
                }`}
              >
                <Newspaper className="w-4 h-4 text-[#f5b942]" />
                <span className="text-xs font-medium">Market Wire Releases</span>
              </button>
            </div>
          </div>

          <div className={`border-t ${isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"}`} />

          {/* Section: Preferences & System */}
          <div>
            <div className="text-[10px] uppercase font-bold tracking-wider px-2 pb-1.5 text-[#787b86]">
              System & Preferences
            </div>
            <div className="space-y-0.5">
              {/* Theme Toggle */}
              <button
                onClick={onToggleTheme}
                className={`w-full flex items-center justify-between rounded-lg px-3 py-2 text-left cursor-pointer transition-colors ${
                  isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
                }`}
              >
                <div className="flex items-center gap-3">
                  {isLight ? <Moon className="w-4 h-4 text-[#787b86]" /> : <Sun className="w-4 h-4 text-[#f5b942]" />}
                  <span className="text-xs font-medium">Color Theme</span>
                </div>
                <span className="text-[11px] font-semibold text-[#2962ff]">
                  {isLight ? "Light Mode" : "Dark Mode"}
                </span>
              </button>

              {/* Settings Dialog */}
              <button
                onClick={() => {
                  onClose();
                  onOpenSettings?.();
                }}
                className={`w-full flex items-center gap-3 rounded-lg px-3 py-2 text-left cursor-pointer transition-colors ${
                  isLight ? "hover:bg-[#f0f3fa] text-[#131722]" : "hover:bg-[#2a2e39] text-[#d1d4dc]"
                }`}
              >
                <Settings className="w-4 h-4 text-[#787b86]" />
                <span className="text-xs font-medium">Chart & Visual Settings...</span>
              </button>
            </div>
          </div>
        </div>

        {/* Drawer Footer Status */}
        <div
          className={`h-12 px-4 border-t flex items-center justify-between shrink-0 text-[11px] font-mono ${
            isLight ? "border-[#e0e3eb] bg-[#f8f9fc] text-[#5d606b]" : "border-[#2a2e39] bg-[#141722] text-[#787b86]"
          }`}
        >
          <div className="flex items-center gap-1.5">
            <ShieldCheck className="w-3.5 h-3.5 text-[#089981]" />
            <span>ClickHouse + NATS Core</span>
          </div>
          <span className="text-[10px] text-[#2962ff] font-bold">v1.0.1</span>
        </div>
      </aside>
    </div>
  );
};
