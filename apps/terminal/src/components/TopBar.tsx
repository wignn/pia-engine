"use client";

import { useEffect, useRef, useState } from "react";
import {
  ChevronDown,
  LayoutGrid,
  Menu,
  Moon,
  PanelRight,
  Search,
  Settings,
  Sun,
} from "lucide-react";
import { ChartLayout, PaneContentType } from "@/types";
import { VisualLayoutPicker } from "./VisualLayoutPicker";

interface TopBarProps {
  onSearchClick: () => void;
  onOpenMainMenu: () => void;
  paneType: PaneContentType;
  layout: ChartLayout;
  onLayoutChange: (layout: ChartLayout) => void;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  onOpenSettings: () => void;
  onToggleMarketPanel: () => void;
}

export const WORKSPACE_LABELS: Record<PaneContentType, string> = {
  chart: "Charts",
  controlpanel: "Market Dashboard",
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

const iconButton =
  "flex size-8 shrink-0 items-center justify-center rounded border border-transparent text-[#9baabd] transition-colors hover:border-[#425166] hover:bg-[#202d3c] hover:text-white";

export function TopBar({
  onSearchClick,
  onOpenMainMenu,
  paneType,
  layout,
  onLayoutChange,
  theme,
  onToggleTheme,
  onOpenSettings,
  onToggleMarketPanel,
}: TopBarProps) {
  const [isLayoutPickerOpen, setIsLayoutPickerOpen] = useState(false);
  const layoutRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isLayoutPickerOpen) return;
    const closeOutside = (event: MouseEvent) => {
      if (!layoutRef.current?.contains(event.target as Node)) setIsLayoutPickerOpen(false);
    };
    document.addEventListener("mousedown", closeOutside);
    return () => document.removeEventListener("mousedown", closeOutside);
  }, [isLayoutPickerOpen]);

  return (
    <header className="relative z-40 flex h-[52px] min-h-[52px] items-center gap-3 border-b border-[#263546] bg-[#101923] px-3 text-[#e8eef5] sm:px-4">
      <div className="flex min-w-0 items-center gap-3">
        <div className="flex shrink-0 items-center gap-2.5 pr-1">
          <span className="flex h-7 items-center gap-1.5 border-l-[3px] border-[#c6a16b] pl-2.5 text-[15px] font-bold tracking-[-0.07em] text-white">PIA</span>
          <span className="hidden border-l border-[#344357] pl-2.5 text-[9px] font-semibold uppercase tracking-[0.16em] text-[#95a5b9] sm:block">Trading terminal</span>
        </div>
        <span className="hidden h-5 w-px bg-[#344357] md:block" />
        <button
          type="button"
          onClick={onOpenMainMenu}
          className="flex h-8 min-w-0 items-center gap-2 rounded px-2 text-left text-xs font-medium text-[#c7d3e0] transition-colors hover:bg-[#202d3c] hover:text-white"
          aria-label="Choose workspace"
          title="Choose workspace"
        >
          <Menu className="size-4 shrink-0 sm:hidden" />
          <span className="hidden max-w-44 truncate sm:inline">{WORKSPACE_LABELS[paneType]}</span>
          <ChevronDown className="hidden size-3.5 shrink-0 text-[#778ba2] sm:block" />
        </button>
      </div>

      <button
        type="button"
        onClick={onSearchClick}
        className="ml-auto flex h-8 min-w-0 items-center gap-2 rounded border border-[#334255] bg-[#1a2735] px-2.5 text-left text-xs text-[#9dafc2] transition-colors hover:border-[#657a92] hover:text-white sm:w-56 lg:mx-auto lg:w-80"
        title="Search instruments (Ctrl+K)"
      >
        <Search className="size-4 shrink-0" />
        <span className="hidden flex-1 truncate sm:block">Search symbol or company</span>
        <kbd className="hidden rounded border border-[#3d4c60] px-1.5 py-0.5 font-mono text-[10px] text-[#7f92a9] lg:block">Ctrl K</kbd>
      </button>

      <div className="flex shrink-0 items-center gap-0.5 border-l border-[#344357] pl-2 sm:gap-1 sm:pl-3">
        <div ref={layoutRef} className="relative">
          <button type="button" onClick={() => setIsLayoutPickerOpen((open) => !open)} className={iconButton} title="Chart layout" aria-label="Chart layout">
            <LayoutGrid className="size-4" />
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
        <button type="button" onClick={onToggleMarketPanel} className={iconButton} title="Toggle market panel" aria-label="Toggle market panel">
          <PanelRight className="size-4" />
        </button>
        <button type="button" onClick={onToggleTheme} className={iconButton} title={theme === "dark" ? "Light theme" : "Dark theme"} aria-label="Toggle theme">
          {theme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
        </button>
        <button type="button" onClick={onOpenSettings} className={iconButton} title="Settings" aria-label="Settings">
          <Settings className="size-4" />
        </button>
      </div>
    </header>
  );
}
