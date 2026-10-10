"use client";

import { useState } from "react";
import { BarChart3, CalendarDays, Layers3, Newspaper, Plus, Radio, Tv, X, Brain } from "lucide-react";
import { TabContentType, TabItem } from "@/types";
import { WORKSPACE_LABELS } from "./TopBar";

interface ChartTabsProps {
  tabs: TabItem[];
  activeTabId: string;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onNewTab: (type?: TabContentType) => void;
  onReorderTabs: (tabs: TabItem[]) => void;
  theme?: "dark" | "light";
}

const TAB_TYPES: { id: TabContentType; label: string; icon: typeof BarChart3 }[] = [
  { id: "chart", label: "Chart", icon: BarChart3 },
  { id: "news", label: "News", icon: Newspaper },
  { id: "orderbook", label: "Order book", icon: Layers3 },
  { id: "intelligence", label: "Market intelligence", icon: Brain },
  { id: "calendar", label: "Calendar", icon: CalendarDays },
  { id: "social", label: "Social pulse", icon: Radio },
  { id: "live", label: "Live broadcast", icon: Tv },
];

export function ChartTabs({ tabs, activeTabId, onSelectTab, onCloseTab, onNewTab, onReorderTabs }: ChartTabsProps) {
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [menuPosition, setMenuPosition] = useState<{ top: number; left: number } | null>(null);

  const labelFor = (tab: TabItem) => tab.type && tab.type !== "chart"
    ? WORKSPACE_LABELS[tab.type]
    : tab.symbol;

  return (
    <>
      <div className="flex h-9 min-h-9 items-stretch overflow-x-auto border-b border-border bg-muted/60 px-3 no-scrollbar" role="tablist" aria-label="Open instruments">
        {tabs.map((tab, index) => {
          const active = tab.id === activeTabId;
          const Icon = TAB_TYPES.find((type) => type.id === (tab.type || "chart"))?.icon || BarChart3;
          return (
            <div
              key={tab.id}
              draggable
              onDragStart={(event) => { setDraggedIndex(index); event.dataTransfer.effectAllowed = "move"; }}
              onDragOver={(event) => event.preventDefault()}
              onDrop={(event) => {
                event.preventDefault();
                if (draggedIndex === null || draggedIndex === index) return;
                const reordered = [...tabs];
                const [moved] = reordered.splice(draggedIndex, 1);
                reordered.splice(index, 0, moved);
                onReorderTabs(reordered);
                setDraggedIndex(null);
              }}
              onDragEnd={() => setDraggedIndex(null)}
              className={`group flex max-w-44 shrink-0 items-center gap-1.5 border-x border-t-2 px-3 text-[11px] transition-colors ${active ? "border-x-border border-t-primary bg-card font-semibold text-foreground" : "border-x-transparent border-t-transparent text-muted-foreground hover:bg-card/60 hover:text-foreground"} ${draggedIndex === index ? "opacity-50" : ""}`}
            >
              <button type="button" role="tab" aria-selected={active} onClick={() => onSelectTab(tab.id)} className="flex min-w-0 items-center gap-1.5 text-left">
                <Icon className={`size-3.5 shrink-0 ${active ? "text-primary" : ""}`} />
                <span className="truncate font-mono">{labelFor(tab)}</span>
              </button>
              {tabs.length > 1 && (
                <button type="button" onClick={() => onCloseTab(tab.id)} className="rounded p-0.5 text-muted-foreground opacity-0 hover:bg-muted hover:text-foreground group-hover:opacity-100 focus:opacity-100" title={`Close ${labelFor(tab)}`} aria-label={`Close ${labelFor(tab)}`}>
                  <X className="size-3" />
                </button>
              )}
            </div>
          );
        })}
        <button
          type="button"
          onClick={(event) => {
            const rect = event.currentTarget.getBoundingClientRect();
            setMenuPosition((current) => current ? null : { top: rect.bottom + 4, left: Math.min(rect.left, window.innerWidth - 208) });
          }}
          className="my-auto ml-1 flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground"
          title="Open new tab"
          aria-label="Open new tab"
        >
          <Plus className="size-4" />
        </button>
      </div>
      {menuPosition && (
        <>
          <button type="button" className="fixed inset-0 z-40 cursor-default" onClick={() => setMenuPosition(null)} aria-label="Close new tab menu" />
          <div className="fixed z-50 w-48 rounded-lg border border-border bg-card p-1.5 shadow-2xl" style={{ top: menuPosition.top, left: Math.max(8, menuPosition.left) }}>
            <div className="px-2 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">New tab</div>
            {TAB_TYPES.map(({ id, label, icon: Icon }) => (
              <button key={id} type="button" onClick={() => { onNewTab(id); setMenuPosition(null); }} className="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-foreground hover:bg-muted">
                <Icon className="size-3.5 text-muted-foreground" />{label}
              </button>
            ))}
          </div>
        </>
      )}
    </>
  );
}
