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
  Tv,
  LayoutDashboard,
  Map,
  LayoutGrid,
  PieChart,
  Globe2,
  TrendingUp,
  ShieldAlert,
  Zap,
  FileText,
  Briefcase
} from "lucide-react";
import { PaneContentType } from "@/types";

interface TradingViewMainMenuProps {
  isOpen: boolean;
  onClose: () => void;
  currentPaneType?: PaneContentType;
  onSelectPaneType: (type: PaneContentType) => void;
  theme?: "dark" | "light";
}

export const TradingViewMainMenu: React.FC<TradingViewMainMenuProps> = ({
  isOpen,
  onClose,
  currentPaneType = "chart",
  onSelectPaneType,
}) => {
  if (!isOpen) return null;

  const productItems: {
    id: PaneContentType;
    group: "Markets" | "Research" | "Trading" | "Workspaces";
    label: string;
    description: string;
    icon: React.ReactNode;
  }[] = [
    {
      id: "chart",
      group: "Markets",
      label: "Supercharts",
      description: "Financial charts with multi-pane analysis & indicators",
      icon: <BarChart3 className="w-4 h-4 text-[#2962ff]" />
    },
    {
      id: "controlpanel",
      group: "Workspaces",
      label: "Financial War Room",
      description: "Customizable 12-col dashboard with TV, mini charts & depth",
      icon: <LayoutDashboard className="w-4 h-4 text-[#3b82f6]" />
    },
    {
      id: "macromaps",
      group: "Markets",
      label: "Macro World Map",
      description: "Global macroeconomic choropleth (Inflation, Rates, GDP)",
      icon: <Map className="w-4 h-4 text-[#10b981]" />
    },
    {
      id: "hub",
      group: "Workspaces",
      label: "Supercharts Hub",
      description: "Layout gallery, rapid asset suites & custom workspaces",
      icon: <LayoutGrid className="w-4 h-4 text-[#a855f7]" />
    },
    {
      id: "options",
      group: "Research",
      label: "Options & GEX Analysis",
      description: "Put/Call ratios, Gamma Exposure profile & Max Pain",
      icon: <PieChart className="w-4 h-4 text-[#00e5ff]" />
    },
    {
      id: "macro",
      group: "Research",
      label: "Macro & Central Banks",
      description: "Fear & Greed index, COT positioning & Fed/ECB stance",
      icon: <Globe2 className="w-4 h-4 text-[#f59e0b]" />
    },
    {
      id: "yields",
      group: "Research",
      label: "Treasury Yields & Spreads",
      description: "US Sovereign yield curve, 2s10s & 3m10y spreads",
      icon: <TrendingUp className="w-4 h-4 text-[#10b981]" />
    },
    {
      id: "geosignals",
      group: "Research",
      label: "Geopolitical Signals",
      description: "Real-time flash event stream & affected asset matrices",
      icon: <ShieldAlert className="w-4 h-4 text-[#f43f5e]" />
    },
    {
      id: "energy",
      group: "Research",
      label: "Energy Complex",
      description: "Crude oil (WTI/Brent), natural gas & storage telemetry",
      icon: <Zap className="w-4 h-4 text-[#eab308]" />
    },
    {
      id: "sec",
      group: "Research",
      label: "SEC Filings",
      description: "Real-time EDGAR corporate disclosures (10-K, 10-Q, 8-K)",
      icon: <FileText className="w-4 h-4 text-[#8b5cf6]" />
    },
    {
      id: "paper",
      group: "Trading",
      label: "Paper Trading Simulator",
      description: "Institutional practice trading with real-time mark-to-market",
      icon: <Briefcase className="w-4 h-4 text-[#06b6d4]" />
    },
    {
      id: "orderbook",
      group: "Markets",
      label: "DOM & Order Book",
      description: "Level 2 market depth and live transaction flow",
      icon: <Layers className="w-4 h-4 text-[#089981]" />
    },
    {
      id: "news",
      group: "Research",
      label: "News & Market Wire",
      description: "Breaking global macro headlines & corporate filings",
      icon: <Newspaper className="w-4 h-4 text-[#f5b942]" />
    },
    {
      id: "social",
      group: "Research",
      label: "Social Pulse",
      description: "Real-time 𝕏/Twitter verified sentiment streams",
      icon: <Radio className="w-4 h-4 text-[#e040fb]" />
    },
    {
      id: "intelligence",
      group: "Research",
      label: "Market Intelligence",
      description: "AI institutional signals, COT reports, & liquidity",
      icon: <Brain className="w-4 h-4 text-[#00e5ff]" />
    },
    {
      id: "calendar",
      group: "Research",
      label: "Economic Calendar",
      description: "Central bank announcements, CPI, rate decisions",
      icon: <Calendar className="w-4 h-4 text-[#ff5252]" />
    },
    {
      id: "live",
      group: "Research",
      label: "Live Broadcast & TV",
      description: "24/7 Financial TV: Bloomberg, CNBC, FOMC stream",
      icon: <Tv className="w-4 h-4 text-[#f23645]" />
    }
  ];

  return (
    <div className="fixed inset-0 z-50 flex" onClick={onClose}>
      {/* Backdrop */}
      <div className="fixed inset-0 bg-black/50 backdrop-blur-xs transition-opacity" />

      {/* Workspace navigation */}
      <aside
        className="relative z-10 flex h-full w-[340px] max-w-[90vw] flex-col overflow-hidden border-r border-border bg-card text-foreground shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Drawer Header */}
        <div className="flex h-14 shrink-0 items-center justify-between border-b border-border px-4">
          <div className="flex items-center gap-2">
            <div className="flex size-7 items-center justify-center rounded-md bg-primary text-xs font-bold text-primary-foreground">
              P
            </div>
            <div>
              <span className="text-sm font-semibold tracking-tight text-foreground">
                PIA Terminal
              </span>
              <span className="block text-[10px] text-muted-foreground">
                Workspaces
              </span>
            </div>
          </div>
          <button
            onClick={onClose}
            className="rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
            aria-label="Close workspace menu"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Scrollable Navigation Body */}
        <div className="flex-1 overflow-y-auto p-3 text-xs">
          {(["Markets", "Research", "Trading", "Workspaces"] as const).map((group) => {
            const items = productItems.filter((item) => item.group === group);
            return (
              <details key={group} open={group === "Markets"} className="border-b border-border/70 py-1 last:border-0">
                <summary className="cursor-pointer list-none px-2 py-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">
                  <span className="flex items-center justify-between">{group}<span className="font-mono">{items.length}</span></span>
                </summary>
                <div className="space-y-0.5 pb-2">
                  {items.map((item) => {
                    const isCurrent = currentPaneType === item.id;
                    return (
                      <button
                        key={item.id}
                        onClick={() => {
                          onSelectPaneType(item.id);
                          onClose();
                        }}
                        className={`flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left transition-colors ${
                          isCurrent
                            ? "bg-primary/10 text-primary"
                            : "text-foreground hover:bg-muted"
                        }`}
                      >
                        <span className="shrink-0">{item.icon}</span>
                        <span className="min-w-0 flex-1">
                          <span className="flex items-center justify-between gap-2">
                            <span className="text-xs font-semibold">{item.label}</span>
                          </span>
                          <span className="mt-0.5 block truncate text-[10px] font-normal leading-tight text-muted-foreground">{item.description}</span>
                        </span>
                      </button>
                    );
                  })}
                </div>
              </details>
            );
          })}
        </div>

      </aside>
    </div>
  );
};
