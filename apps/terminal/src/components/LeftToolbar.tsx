"use client";

import React, { useState, useRef, useEffect } from "react";
import {
  Crosshair,
  TrendingUp,
  ArrowUpRight,
  Minus,
  MoveRight,
  SquareDashed,
  SeparatorVertical,
  Columns,
  Percent,
  Square,
  Circle,
  Type,
  Tag,
  Ruler,
  Trash2,
  Lock,
  Unlock,
  Eye,
  EyeOff,
  Undo2,
  Save,
  FolderOpen,
  Redo2,
  ChevronRight,
  Pencil,
  Check,
} from "lucide-react";
import { DrawingTool } from "@/types";

interface ToolItem {
  id: DrawingTool;
  label: string;
  shortcut?: string;
  icon: React.ReactNode;
}

interface ToolGroup {
  id: string;
  label: string;
  defaultTool: DrawingTool;
  tools: ToolItem[];
}

const TOOL_GROUPS: ToolGroup[] = [
  {
    id: "cursor",
    label: "Cursor Tools",
    defaultTool: "cursor",
    tools: [
      { id: "cursor", label: "Crosshair", shortcut: "V / Esc", icon: <Crosshair className="w-4 h-4" /> },
    ],
  },
  {
    id: "lines",
    label: "Lines & Trend Channels",
    defaultTool: "trendline",
    tools: [
      { id: "trendline", label: "Trend Line", shortcut: "T", icon: <TrendingUp className="w-4 h-4" /> },
      { id: "ray", label: "Ray Line", shortcut: "Alt+T", icon: <ArrowUpRight className="w-4 h-4" /> },
      { id: "horizontal", label: "Horizontal Line (S&R)", shortcut: "H", icon: <Minus className="w-4 h-4" /> },
      { id: "horizontal_ray", label: "Horizontal Ray", shortcut: "Alt+H", icon: <MoveRight className="w-4 h-4" /> },
      { id: "vertical", label: "Vertical Time Line", shortcut: "Alt+V", icon: <SeparatorVertical className="w-4 h-4" /> },
      { id: "parallel_channel", label: "Parallel Channel", shortcut: "P", icon: <Columns className="w-4 h-4" /> },
      { id: "trend_channel", label: "Trend Channel", shortcut: "Alt+P", icon: <SquareDashed className="w-4 h-4" /> },
    ],
  },
  {
    id: "fibonacci",
    label: "Fibonacci & Gann",
    defaultTool: "fibonacci",
    tools: [
      { id: "fibonacci", label: "Fibonacci Retracement", shortcut: "F", icon: <Percent className="w-4 h-4" /> },
      { id: "fib_extension", label: "Fibonacci Extension", shortcut: "Shift+F", icon: <TrendingUp className="w-4 h-4" /> },
    ],
  },
  {
    id: "shapes",
    label: "Geometric Shapes & Order Blocks",
    defaultTool: "rectangle",
    tools: [
      { id: "rectangle", label: "Rectangle (Order Block / Zone)", shortcut: "R", icon: <Square className="w-4 h-4" /> },
      { id: "circle", label: "Circle Highlight", shortcut: "C", icon: <Circle className="w-4 h-4" /> },
    ],
  },
  {
    id: "annotations",
    label: "Text & Price Markers",
    defaultTool: "price_line",
    tools: [
      { id: "price_line", label: "Price Level Marker", shortcut: "L", icon: <Tag className="w-4 h-4" /> },
      { id: "text", label: "Text Annotation", shortcut: "Alt+N", icon: <Type className="w-4 h-4" /> },
    ],
  },
  {
    id: "measure",
    label: "Prediction & Measurement",
    defaultTool: "measure",
    tools: [
      { id: "measure", label: "Measure / Pips Ruler", shortcut: "M", icon: <Ruler className="w-4 h-4" /> },
    ],
  },
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

export const LeftToolbar: React.FC<LeftToolbarProps> = ({
  activeTool = "cursor",
  setActiveTool,
  onClearDrawings,
  drawingsCount = 0,
  isDrawingModeLocked = false,
  onToggleDrawingModeLock,
  isDrawingsHidden = false,
  onToggleHideDrawings,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
  onSaveDrawings,
  onLoadDrawings,
  theme = "dark",
}) => {
  const isLight = theme === "light";
  const [openGroupId, setOpenGroupId] = useState<string | null>(null);
  const flyoutRef = useRef<HTMLDivElement>(null);

  // Close flyout when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (flyoutRef.current && !flyoutRef.current.contains(e.target as Node)) {
        setOpenGroupId(null);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Find active tool item across all groups
  const currentActiveToolItem = TOOL_GROUPS.flatMap((g) => g.tools).find(
    (t) => t.id === activeTool
  );

  return (
    <aside
      className={`hidden md:flex w-[48px] border-r flex-col items-center py-2.5 justify-between select-none z-30 shrink-0 transition-colors relative ${
        isLight
          ? "bg-[#ffffff] border-[#e0e3eb]"
          : "bg-[#181b24] border-[#2a2e39]"
      }`}
    >
      {/* Top Drawing Tools Groups */}
      <div className="flex flex-col items-center gap-1.5 w-full" ref={flyoutRef}>
        {TOOL_GROUPS.map((group) => {
          const isGroupActive = group.tools.some((t) => t.id === activeTool);
          const activeToolInGroup =
            group.tools.find((t) => t.id === activeTool) || group.tools[0];
          const hasMultiple = group.tools.length > 1;
          const isFlyoutOpen = openGroupId === group.id;

          return (
            <div key={group.id} className="relative group/btn w-full flex justify-center">
              <button
                onClick={() => {
                  if (hasMultiple && isGroupActive) {
                    setOpenGroupId(isFlyoutOpen ? null : group.id);
                  } else {
                    setActiveTool?.(activeToolInGroup.id);
                    setOpenGroupId(null);
                  }
                }}
                onContextMenu={(e) => {
                  e.preventDefault();
                  if (hasMultiple) setOpenGroupId(group.id);
                }}
                className={`relative w-8 h-8 rounded-lg flex items-center justify-center transition-all cursor-pointer ${
                  isGroupActive
                    ? "bg-[#2962ff]/20 text-[#2962ff] shadow-xs"
                    : isLight
                    ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
                    : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#252a36]"
                }`}
                title={`${activeToolInGroup.label} ${
                  activeToolInGroup.shortcut ? `(${activeToolInGroup.shortcut})` : ""
                }`}
              >
                {activeToolInGroup.icon}

                {/* Blue active indicator pip */}
                {isGroupActive && (
                  <span className="absolute bottom-1 right-1 w-1.5 h-1.5 bg-[#2962ff] rounded-full" />
                )}

                {/* Sub-menu tiny indicator corner triangle if group has multiple options */}
                {hasMultiple && (
                  <span
                    onClick={(e) => {
                      e.stopPropagation();
                      setOpenGroupId(isFlyoutOpen ? null : group.id);
                    }}
                    className="absolute -bottom-0.5 right-0 text-[8px] opacity-40 hover:opacity-100 hover:text-[#2962ff] p-0.5"
                  >
                    ▸
                  </span>
                )}
              </button>

              {/* Flyout Sub-menu (TradingView style popover) */}
              {isFlyoutOpen && hasMultiple && (
                <div
                  className={`absolute left-[44px] top-0 z-50 w-60 rounded-xl shadow-2xl border py-1.5 px-1 animate-in fade-in slide-in-from-left-2 duration-150 backdrop-blur-md ${
                    isLight
                      ? "bg-[#ffffff]/98 border-[#e0e3eb] text-[#131722]"
                      : "bg-[#1e222d]/98 border-[#2a2e39] text-[#d1d4dc]"
                  }`}
                >
                  <div
                    className={`px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider border-b mb-1 ${
                      isLight ? "text-[#787b86] border-[#e0e3eb]" : "text-[#787b86] border-[#2a2e39]"
                    }`}
                  >
                    {group.label}
                  </div>
                  {group.tools.map((item) => {
                    const isSelected = activeTool === item.id;
                    return (
                      <button
                        key={item.id}
                        onClick={() => {
                          setActiveTool?.(item.id);
                          setOpenGroupId(null);
                        }}
                        className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
                          isSelected
                            ? "bg-[#2962ff] text-white"
                            : isLight
                            ? "hover:bg-[#f0f3fa] text-[#131722]"
                            : "hover:bg-[#2a2e39] text-[#d1d4dc] hover:text-white"
                        }`}
                      >
                        <div className="flex items-center gap-2">
                          <span className={isSelected ? "text-white" : "text-[#2962ff]"}>
                            {item.icon}
                          </span>
                          <span>{item.label}</span>
                        </div>
                        {item.shortcut && (
                          <span
                            className={`text-[10px] font-mono px-1 rounded ${
                              isSelected
                                ? "text-white/80"
                                : isLight
                                ? "text-[#787b86]"
                                : "text-[#787b86]"
                            }`}
                          >
                            {item.shortcut}
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Bottom Utility Controls (Undo, Redo, Lock Mode, Hide, Trash) */}
      <div
        className={`flex flex-col items-center gap-1.5 w-full pt-2.5 border-t ${
          isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"
        }`}
      >
        {/* Undo Drawing */}
        <button
          onClick={onUndo}
          disabled={!canUndo}
          className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors disabled:opacity-30 cursor-pointer ${
            isLight
              ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
              : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#252a36]"
          }`}
          title="Undo Drawing (Ctrl+Z)"
        >
          <Undo2 className="w-4 h-4" />
        </button>

        {/* Redo Drawing */}
        <button
          onClick={onRedo}
          disabled={!canRedo}
          className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors disabled:opacity-30 cursor-pointer ${
            isLight
              ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
              : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#252a36]"
          }`}
          title="Redo Drawing (Ctrl+Y)"
        >
          <Redo2 className="w-4 h-4" />
        </button>

        {/* Stay in Drawing Mode Lock */}
        <button
          onClick={onToggleDrawingModeLock}
          className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer ${
            isDrawingModeLocked
              ? "bg-[#2962ff]/20 text-[#2962ff]"
              : isLight
              ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
              : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#252a36]"
          }`}
          title={isDrawingModeLocked ? "Stay in Drawing Mode: ON" : "Stay in Drawing Mode: OFF"}
        >
          {isDrawingModeLocked ? <Lock className="w-4 h-4" /> : <Unlock className="w-4 h-4" />}
        </button>

        {/* Save / Load drawing templates */}
        <div className="flex items-center gap-0.5">
          <button
            onClick={onSaveDrawings}
            disabled={drawingsCount === 0}
            aria-label="Save drawing set"
            className={`w-7 h-7 rounded-lg flex items-center justify-center disabled:opacity-30 ${isLight ? "text-[#5d606b] hover:bg-[#f0f3fa]" : "text-[#787b86] hover:bg-[#252a36] hover:text-[#d1d4dc]"}`}
            title="Save drawings for this symbol and timeframe"
          ><Save className="w-3.5 h-3.5" /></button>
          <button
            onClick={onLoadDrawings}
            aria-label="Load saved drawings"
            className={`w-7 h-7 rounded-lg flex items-center justify-center ${isLight ? "text-[#5d606b] hover:bg-[#f0f3fa]" : "text-[#787b86] hover:bg-[#252a36] hover:text-[#d1d4dc]"}`}
            title="Load saved drawings"
          ><FolderOpen className="w-3.5 h-3.5" /></button>
        </div>
        {/* Hide / Show All Drawings */}
        <button
          onClick={onToggleHideDrawings}
          className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer ${
            isDrawingsHidden
              ? "bg-[#f5b942]/20 text-[#f5b942]"
              : isLight
              ? "text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa]"
              : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#252a36]"
          }`}
          title={isDrawingsHidden ? "Show All Drawings" : "Hide All Drawings"}
        >
          {isDrawingsHidden ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
        </button>

        {/* Clear All Drawings */}
        <button
          onClick={onClearDrawings}
          disabled={drawingsCount === 0}
          className={`relative w-8 h-8 rounded-lg flex items-center justify-center transition-colors disabled:opacity-30 cursor-pointer ${
            isLight
              ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#f23645]"
              : "hover:bg-[#252a36] text-[#787b86] hover:text-[#f23645]"
          }`}
          title={`Hapus Semua Gambar (${drawingsCount})`}
        >
          <Trash2 className="w-4 h-4" />
          {drawingsCount > 0 && (
            <span className="absolute -top-1 -right-1 px-1 py-0.2 min-w-[14px] text-[9px] font-bold bg-[#f23645] text-white rounded-full flex items-center justify-center shadow-xs">
              {drawingsCount > 9 ? "9+" : drawingsCount}
            </span>
          )}
        </button>
      </div>
    </aside>
  );
};
