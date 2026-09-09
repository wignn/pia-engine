"use client";

import React from "react";
import {
  Crosshair,
  TrendingUp,
  Percent,
  Ruler,
  Trash2,
  Minus,
  Lock,
  Unlock,
  Eye,
  EyeOff
} from "lucide-react";
import { DrawingTool } from "@/types";

interface LeftToolbarProps {
  activeTool?: DrawingTool;
  setActiveTool?: (tool: DrawingTool) => void;
  onClearDrawings?: () => void;
  drawingsCount?: number;
  isDrawingModeLocked?: boolean;
  onToggleDrawingModeLock?: () => void;
  isDrawingsHidden?: boolean;
  onToggleHideDrawings?: () => void;
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
}) => {
  const tools: { id: DrawingTool; label: string; icon: React.ReactNode }[] = [
    { id: "cursor", label: "Crosshair (V / Esc)", icon: <Crosshair className="w-4 h-4" /> },
    { id: "trendline", label: "Trend Line (T)", icon: <TrendingUp className="w-4 h-4" /> },
    { id: "horizontal", label: "Horizontal Ray / S&R (H)", icon: <Minus className="w-4 h-4" /> },
    { id: "fibonacci", label: "Fibonacci Retracement (F)", icon: <Percent className="w-4 h-4" /> },
    { id: "measure", label: "Measure / Pips Ruler (M)", icon: <Ruler className="w-4 h-4" /> },
  ];

  return (
    <aside className="hidden md:flex w-[48px] bg-[#1e222d] border-r border-[#2a2e39] flex-col items-center py-2 justify-between select-none z-10 shrink-0">
      {/* Drawing Tools */}
      <div className="flex flex-col items-center gap-1.5 w-full">
        {tools.map((t) => {
          const isActive = activeTool === t.id;
          return (
            <button
              key={t.id}
              onClick={() => setActiveTool?.(t.id)}
              className={`group relative p-2 rounded transition-all cursor-pointer ${
                isActive
                  ? "bg-[#2962ff]/20 text-[#2962ff] shadow-sm"
                  : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
              }`}
              title={t.label}
            >
              {t.icon}
              {isActive && (
                <span className="absolute bottom-1 right-1 w-1.5 h-1.5 bg-[#2962ff] rounded-full" />
              )}
            </button>
          );
        })}
      </div>

      {/* Utility Actions (TradingView-style Lock, Hide, Trash) */}
      <div className="flex flex-col items-center gap-1.5 w-full pt-2 border-t border-[#2a2e39]">
        {/* Stay in Drawing Mode Lock */}
        <button
          onClick={onToggleDrawingModeLock}
          className={`p-2 rounded transition-colors cursor-pointer ${
            isDrawingModeLocked
              ? "bg-[#2962ff]/20 text-[#2962ff]"
              : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
          }`}
          title={isDrawingModeLocked ? "Stay in Drawing Mode: ON" : "Stay in Drawing Mode: OFF"}
        >
          {isDrawingModeLocked ? <Lock className="w-4 h-4" /> : <Unlock className="w-4 h-4" />}
        </button>

        {/* Hide/Show All Drawings */}
        <button
          onClick={onToggleHideDrawings}
          className={`p-2 rounded transition-colors cursor-pointer ${
            isDrawingsHidden
              ? "bg-[#f5b942]/20 text-[#f5b942]"
              : "text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]"
          }`}
          title={isDrawingsHidden ? "Show All Drawings" : "Hide All Drawings"}
        >
          {isDrawingsHidden ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
        </button>

        {/* Clear All Drawings */}
        <button
          onClick={onClearDrawings}
          disabled={drawingsCount === 0}
          className="relative p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#f23645] transition-colors disabled:opacity-40 cursor-pointer"
          title={`Clear Drawings (${drawingsCount})`}
        >
          <Trash2 className="w-4 h-4" />
          {drawingsCount > 0 && (
            <span className="absolute -top-0.5 -right-0.5 bg-[#f23645] text-white text-[8px] font-bold rounded-full w-3.5 h-3.5 flex items-center justify-center">
              {drawingsCount > 9 ? "9+" : drawingsCount}
            </span>
          )}
        </button>
      </div>
    </aside>
  );
};
