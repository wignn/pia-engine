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
  Eye
} from "lucide-react";
import { DrawingTool } from "@/types";

interface LeftToolbarProps {
  activeTool?: DrawingTool;
  setActiveTool?: (tool: DrawingTool) => void;
  onClearDrawings?: () => void;
  drawingsCount?: number;
}

export const LeftToolbar: React.FC<LeftToolbarProps> = ({
  activeTool = "cursor",
  setActiveTool,
  onClearDrawings,
  drawingsCount = 0,
}) => {
  const tools: { id: DrawingTool; label: string; icon: React.ReactNode }[] = [
    { id: "cursor", label: "Crosshair (Navigation)", icon: <Crosshair className="w-4 h-4" /> },
    { id: "trendline", label: "Trend Line", icon: <TrendingUp className="w-4 h-4" /> },
    { id: "horizontal", label: "Horizontal Line / S&R", icon: <Minus className="w-4 h-4" /> },
    { id: "fibonacci", label: "Fibonacci Retracement", icon: <Percent className="w-4 h-4" /> },
    { id: "measure", label: "Measure / Pips Ruler", icon: <Ruler className="w-4 h-4" /> },
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
              className={`group relative p-2 rounded transition-all ${
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

      {/* Utility Actions */}
      <div className="flex flex-col items-center gap-1 w-full pt-2 border-t border-[#2a2e39]">
        <button
          onClick={onClearDrawings}
          disabled={drawingsCount === 0}
          className="relative p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#f23645] transition-colors disabled:opacity-40"
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
