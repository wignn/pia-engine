"use client";

import { useEffect, useRef, useState } from "react";
import {
  Bell,
  BookOpen,
  CalendarDays,
  ChevronDown,
  FileText,
  Globe2,
  Layers3,
  Newspaper,
  Radio,
  ShieldAlert,
  TrendingUp,
  Tv,
  Wallet,
  Zap,
  BarChart3,
  Brain,
} from "lucide-react";

export type SidebarTab =
  | "watchlist" | "orderbook" | "news" | "alerts" | "calendar"
  | "intelligence" | "social" | "live" | "options" | "macro"
  | "yields" | "geosignals" | "energy" | "sec" | "paper";

interface RightDockProps {
  activeTab: SidebarTab;
  setActiveTab: (tab: SidebarTab) => void;
  theme?: "dark" | "light";
}

const mainTabs = [
  { id: "watchlist", label: "Watchlist", icon: BarChart3 },
  { id: "news", label: "News", icon: Newspaper },
  { id: "orderbook", label: "Order book", icon: Layers3 },
] as const;

const moreTabs = [
  { id: "alerts", label: "Price alerts", icon: Bell },
  { id: "intelligence", label: "Market intelligence", icon: Brain },
  { id: "calendar", label: "Economic calendar", icon: CalendarDays },
  { id: "options", label: "Options & GEX", icon: BookOpen },
  { id: "macro", label: "Macro & central banks", icon: Globe2 },
  { id: "yields", label: "Treasury yields", icon: TrendingUp },
  { id: "geosignals", label: "Geopolitical signals", icon: ShieldAlert },
  { id: "energy", label: "Energy markets", icon: Zap },
  { id: "sec", label: "SEC filings", icon: FileText },
  { id: "paper", label: "Paper trading", icon: Wallet },
  { id: "social", label: "Social pulse", icon: Radio },
  { id: "live", label: "Live broadcast", icon: Tv },
] as const;

export function RightDock({ activeTab, setActiveTab }: RightDockProps) {
  const [isMoreOpen, setIsMoreOpen] = useState(false);
  const moreRef = useRef<HTMLDivElement>(null);
  const moreActive = moreTabs.some((tab) => tab.id === activeTab);

  useEffect(() => {
    const closeOutside = (event: MouseEvent) => {
      if (!moreRef.current?.contains(event.target as Node)) setIsMoreOpen(false);
    };
    document.addEventListener("mousedown", closeOutside);
    return () => document.removeEventListener("mousedown", closeOutside);
  }, []);

  return (
    <nav className="relative z-20 flex h-10 min-h-10 items-center border-b border-border bg-muted/50 px-2" aria-label="Market panels">
      {mainTabs.map(({ id, label, icon: Icon }) => (
        <button
          key={id}
          type="button"
          onClick={() => { setActiveTab(id); setIsMoreOpen(false); }}
          className={`flex h-10 flex-1 items-center justify-center gap-1.5 border-b-2 px-1 text-[11px] font-medium transition-colors ${activeTab === id ? "border-primary bg-card text-foreground" : "border-transparent text-muted-foreground hover:text-foreground"}`}
          aria-current={activeTab === id ? "page" : undefined}
          title={label}
        >
          <Icon className="size-3.5 shrink-0" /><span className="truncate">{label}</span>
        </button>
      ))}
      <div ref={moreRef} className="relative shrink-0">
        <button
          type="button"
          onClick={() => setIsMoreOpen((open) => !open)}
          className={`flex h-10 items-center gap-1 border-b-2 px-2 text-[11px] font-medium transition-colors ${moreActive || isMoreOpen ? "border-primary bg-card text-foreground" : "border-transparent text-muted-foreground hover:text-foreground"}`}
          aria-expanded={isMoreOpen}
          aria-label="More market panels"
        >
          More <ChevronDown className="size-3" />
        </button>
        {isMoreOpen && (
          <div className="absolute right-0 top-10 z-50 max-h-[min(420px,70vh)] w-56 overflow-y-auto rounded border border-border bg-card p-1.5 shadow-2xl">
            <div className="px-2.5 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">More panels</div>
            {moreTabs.map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                type="button"
                onClick={() => { setActiveTab(id); setIsMoreOpen(false); }}
                className={`flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-xs transition-colors ${activeTab === id ? "bg-primary/10 text-primary" : "text-foreground hover:bg-muted"}`}
              >
                <Icon className="size-4 shrink-0 text-muted-foreground" /><span>{label}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </nav>
  );
}
