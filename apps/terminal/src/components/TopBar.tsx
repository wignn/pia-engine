"use client";

import React, { useState } from "react";
import Image from "next/image";
import {
  BarChart3,
  ChevronDown,
  LayoutGrid,
  Moon,
  Search,
  Settings,
  Sun,
} from "lucide-react";
import { ChartLayout, PaneContentType } from "@/types";
import { VisualLayoutPicker } from "./VisualLayoutPicker";

interface TopBarProps {
  onSearchClick: () => void;
  onOpenMainMenu?: () => void;
  paneType?: PaneContentType;
  layout?: ChartLayout;
  onLayoutChange?: (layout: ChartLayout) => void;
  theme?: "dark" | "light";
  onToggleTheme?: () => void;
  onOpenSettings?: () => void;
}

const WORKSPACE_LABELS: Record<PaneContentType, string> = {
  chart: "Market Overview",
  controlpanel: "Financial War Room",
  macromaps: "Macro Map",
  hub: "Workspace Gallery",
  orderbook: "Order Book",
  options: "Options & GEX",
  macro: "Macro & Central Banks",
  yields: "Treasury Yields",
  geosignals: "Geopolitical Signals",
  energy: "Energy Markets",
  sec: "SEC Filings",
  paper: "Paper Trading",
  news: "Market News",
  social: "Social Pulse",
  intelligence: "Market Intelligence",
  calendar: "Economic Calendar",
  live: "Live Broadcast",
};

export const TopBar: React.FC<TopBarProps> = ({
  onSearchClick,
  onOpenMainMenu,
  paneType = "chart",
  layout = "1x1",
  onLayoutChange,
  theme = "dark",
  onToggleTheme,
  onOpenSettings,
}) => {
  const isLight = theme === "light";
  const [isLayoutPickerOpen, setIsLayoutPickerOpen] = useState(false);

  return (
    <header className="z-30 flex h-12 min-h-12 shrink-0 items-center justify-between gap-3 border-b border-border bg-card px-3 text-xs sm:px-4">
      <div className="flex min-w-0 items-center gap-3">
        <div className="flex shrink-0 items-center gap-2.5">
          <div className="flex size-7 items-center justify-center overflow-hidden rounded-md border border-border bg-background p-0.5">
            <Image src="/logo.png" alt="PIA Terminal" width={24} height={24} priority className="size-full object-contain" />
          </div>
          <span className="hidden font-semibold tracking-tight text-foreground sm:inline">PIA Terminal</span>
        </div>

        <div className="hidden h-6 w-px bg-border md:block" />

        <button
          onClick={onOpenMainMenu}
          className="flex h-8 shrink-0 items-center gap-2 rounded-md border border-border bg-background px-2.5 font-medium text-foreground transition-colors hover:border-primary/40 hover:bg-muted"
          title="Open workspace navigation"
          aria-label="Open workspace navigation"
        >
          <BarChart3 className="size-3.5 text-primary" />
          <span className="hidden sm:inline">{WORKSPACE_LABELS[paneType]}</span>
          <ChevronDown className="size-3 text-muted-foreground" />
        </button>

        <button
          onClick={onSearchClick}
          type="button"
          title="Search symbols and companies (Ctrl+K)"
          className="group flex h-8 min-w-0 w-40 items-center gap-2 rounded-md border border-border bg-background px-2.5 text-left transition-colors hover:border-primary/40 sm:w-56 lg:w-72"
        >
          <Search className="size-3.5 shrink-0 text-muted-foreground group-hover:text-primary" />
          <span className="truncate text-muted-foreground group-hover:text-foreground">Search symbols or companies</span>
          <kbd className="ml-auto hidden shrink-0 rounded border border-border bg-muted/60 px-1.5 font-mono text-[9px] text-muted-foreground lg:inline-block">Ctrl K</kbd>
        </button>
      </div>

      <div className="flex shrink-0 items-center gap-1.5">
        {onLayoutChange && (
          <div className="relative">
            <button
              onClick={() => setIsLayoutPickerOpen((open) => !open)}
              className="flex size-8 items-center justify-center rounded-md border border-border bg-background text-muted-foreground transition-colors hover:text-foreground"
              title="Chart layout"
              aria-label="Chart layout"
            >
              <LayoutGrid className="size-3.5" />
            </button>
            {isLayoutPickerOpen && (
              <VisualLayoutPicker
                currentLayout={layout}
                onSelectLayout={(nextLayout) => {
                  onLayoutChange(nextLayout);
                  setIsLayoutPickerOpen(false);
                }}
                onClose={() => setIsLayoutPickerOpen(false)}
                theme={theme}
              />
            )}
          </div>
        )}

        {onToggleTheme && (
          <button
            onClick={onToggleTheme}
            className="flex size-8 items-center justify-center rounded-md border border-border bg-background text-muted-foreground transition-colors hover:text-foreground"
            title={isLight ? "Switch to dark theme" : "Switch to light theme"}
            aria-label="Toggle theme"
          >
            {isLight ? <Moon className="size-3.5" /> : <Sun className="size-3.5" />}
          </button>
        )}

        {onOpenSettings && (
          <button
            onClick={onOpenSettings}
            className="flex size-8 items-center justify-center rounded-md border border-border bg-background text-muted-foreground transition-colors hover:text-foreground"
            title="Terminal settings"
            aria-label="Terminal settings"
          >
            <Settings className="size-3.5" />
          </button>
        )}
      </div>
    </header>
  );
};
