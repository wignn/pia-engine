"use client";

import { useEffect, useRef, useState, type ReactNode } from "react";
import {
  ArrowUpRight, Check, Circle, Columns, Crosshair, Eye, EyeOff,
  FolderOpen, Lock, Minus, MoveRight, Percent, Ruler, Save,
  SeparatorVertical, Square, SquareDashed, Tag, Trash2, TrendingUp,
  Type, Unlock,
} from "lucide-react";
import { DrawingTool } from "@/types";

interface ToolItem {
  id: DrawingTool;
  label: string;
  shortcut?: string;
  icon: ReactNode;
}

interface ToolGroup {
  label: string;
  tools: ToolItem[];
}

const TOOL_GROUPS: ToolGroup[] = [
  { label: "Cursor", tools: [
    { id: "cursor", label: "Crosshair", shortcut: "V", icon: <Crosshair className="size-4" /> },
  ] },
  { label: "Lines & channels", tools: [
    { id: "trendline", label: "Trend line", shortcut: "T", icon: <TrendingUp className="size-4" /> },
    { id: "ray", label: "Ray line", icon: <ArrowUpRight className="size-4" /> },
    { id: "horizontal", label: "Horizontal line", shortcut: "H", icon: <Minus className="size-4" /> },
    { id: "horizontal_ray", label: "Horizontal ray", icon: <MoveRight className="size-4" /> },
    { id: "vertical", label: "Vertical line", icon: <SeparatorVertical className="size-4" /> },
    { id: "parallel_channel", label: "Parallel channel", shortcut: "P", icon: <Columns className="size-4" /> },
    { id: "trend_channel", label: "Trend channel", icon: <SquareDashed className="size-4" /> },
  ] },
  { label: "Fibonacci", tools: [
    { id: "fibonacci", label: "Retracement", shortcut: "F", icon: <Percent className="size-4" /> },
    { id: "fib_extension", label: "Extension", shortcut: "Shift F", icon: <TrendingUp className="size-4" /> },
  ] },
  { label: "Shapes", tools: [
    { id: "rectangle", label: "Rectangle", shortcut: "R", icon: <Square className="size-4" /> },
    { id: "circle", label: "Circle", icon: <Circle className="size-4" /> },
  ] },
  { label: "Annotations", tools: [
    { id: "price_line", label: "Price marker", icon: <Tag className="size-4" /> },
    { id: "text", label: "Text", icon: <Type className="size-4" /> },
  ] },
  { label: "Measure", tools: [
    { id: "measure", label: "Measure / Pips", shortcut: "M", icon: <Ruler className="size-4" /> },
  ] },
];

interface LeftToolbarProps {
  activeTool?: DrawingTool;
  setActiveTool?: (tool: DrawingTool) => void;
  onClearDrawings?: () => void;
  drawingsCount?: number;
  isDrawingModeLocked?: boolean;
  onToggleDrawingModeLock?: () => void;
  isDrawingsHidden?: boolean;
  onToggleHideDrawings?: () => void;
  canUndo?: boolean;
  canRedo?: boolean;
  onUndo?: () => void;
  onRedo?: () => void;
  onSaveDrawings?: () => void;
  onLoadDrawings?: () => void;
  theme?: "dark" | "light";
}

const railButton = "flex size-9 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-30";

export function LeftToolbar({
  activeTool = "cursor",
  setActiveTool,
  onClearDrawings,
  drawingsCount = 0,
  isDrawingModeLocked = false,
  onToggleDrawingModeLock,
  isDrawingsHidden = false,
  onToggleHideDrawings,
  onSaveDrawings,
  onLoadDrawings,
}: LeftToolbarProps) {
  const [openGroup, setOpenGroup] = useState<string | null>(null);
  const railRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const closeOutside = (event: MouseEvent) => {
      if (!railRef.current?.contains(event.target as Node)) setOpenGroup(null);
    };
    document.addEventListener("mousedown", closeOutside);
    return () => document.removeEventListener("mousedown", closeOutside);
  }, []);

  return (
    <aside ref={railRef} className="absolute inset-y-0 left-0 z-30 flex w-12 shrink-0 flex-col items-center justify-between border-r border-border bg-card py-2 shadow-lg md:static md:shadow-none" aria-label="Drawing tools">
      <div className="flex w-full flex-col items-center gap-1">
        {TOOL_GROUPS.map((group) => {
          const selected = group.tools.find((item) => item.id === activeTool);
          const mainTool = selected || group.tools[0];
          const expanded = openGroup === group.label;
          return (
            <div key={group.label} className="relative flex w-full justify-center">
              <button
                type="button"
                onClick={() => {
                  if (selected && group.tools.length > 1) setOpenGroup(expanded ? null : group.label);
                  else { setActiveTool?.(mainTool.id); setOpenGroup(null); }
                }}
                onContextMenu={(event) => { event.preventDefault(); if (group.tools.length > 1) setOpenGroup(group.label); }}
                className={`${railButton} relative ${selected ? "bg-primary/15 text-primary" : ""}`}
                title={mainTool.label}
                aria-label={mainTool.label}
                aria-expanded={group.tools.length > 1 ? expanded : undefined}
              >
                {mainTool.icon}
                {group.tools.length > 1 && <span className="absolute bottom-1 right-1 size-1 rounded-full bg-current opacity-60" />}
              </button>
              {expanded && group.tools.length > 1 && (
                <div className="absolute left-11 top-0 z-50 w-56 rounded-lg border border-border bg-card p-1.5 shadow-2xl">
                  <div className="px-2 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">{group.label}</div>
                  {group.tools.map((item) => (
                    <button key={item.id} type="button" onClick={() => { setActiveTool?.(item.id); setOpenGroup(null); }} className={`flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-xs ${activeTool === item.id ? "bg-primary/10 text-primary" : "text-foreground hover:bg-muted"}`}>
                      {item.icon}<span className="min-w-0 flex-1 truncate">{item.label}</span>
                      {item.shortcut && <kbd className="font-mono text-[10px] text-muted-foreground">{item.shortcut}</kbd>}
                      {activeTool === item.id && <Check className="size-3" />}
                    </button>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="flex w-full flex-col items-center gap-0.5 border-t border-border pt-2">
        <button type="button" onClick={onToggleDrawingModeLock} className={`${railButton} ${isDrawingModeLocked ? "bg-primary/15 text-primary" : ""}`} title="Keep drawing tool active" aria-label="Keep drawing tool active" aria-pressed={isDrawingModeLocked}>
          {isDrawingModeLocked ? <Lock className="size-4" /> : <Unlock className="size-4" />}
        </button>
        <button type="button" onClick={onSaveDrawings} disabled={drawingsCount === 0} className={railButton} title="Save drawings" aria-label="Save drawings"><Save className="size-4" /></button>
        <button type="button" onClick={onLoadDrawings} className={railButton} title="Load drawings" aria-label="Load drawings"><FolderOpen className="size-4" /></button>
        <button type="button" onClick={onToggleHideDrawings} className={`${railButton} ${isDrawingsHidden ? "bg-primary/15 text-primary" : ""}`} title={isDrawingsHidden ? "Show drawings" : "Hide drawings"} aria-label={isDrawingsHidden ? "Show drawings" : "Hide drawings"}>
          {isDrawingsHidden ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
        </button>
        <button type="button" onClick={onClearDrawings} disabled={drawingsCount === 0} className={`${railButton} hover:text-down`} title={`Clear drawings (${drawingsCount})`} aria-label="Clear drawings"><Trash2 className="size-4" /></button>
      </div>
    </aside>
  );
}
