"use client";

import React from "react";
import {
  Crosshair,
  TrendingUp,
  Percent,
  Type,
  Smile,
  Ruler,
  ZoomIn,
  Magnet,
  Lock,
  Eye,
  Trash2,
  ChevronRight
} from "lucide-react";

export const LeftToolbar: React.FC = () => {
  return (
    <aside className="w-[48px] bg-[#1e222d] border-r border-[#2a2e39] flex flex-col items-center py-2 justify-between select-none z-10">
      {/* Top Drawing Tools */}
      <div className="flex flex-col items-center gap-1 w-full">
        <button className="group relative p-2 rounded hover:bg-[#2a2e39] text-[#2962ff] transition-colors" title="Crosshair">
          <Crosshair className="w-4 h-4" />
          <span className="absolute bottom-1 right-1 w-1 h-1 bg-[#787b86] rounded-full group-hover:bg-white" />
        </button>

        <button className="group relative p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors" title="Trend Line">
          <TrendingUp className="w-4 h-4" />
          <span className="absolute bottom-1 right-1 w-1 h-1 bg-[#787b86] rounded-full group-hover:bg-white" />
        </button>

        <button className="group relative p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors" title="Fibonacci Retracement">
          <Percent className="w-4 h-4" />
          <span className="absolute bottom-1 right-1 w-1 h-1 bg-[#787b86] rounded-full group-hover:bg-white" />
        </button>

        <button className="group relative p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors" title="Text Note">
          <Type className="w-4 h-4" />
          <span className="absolute bottom-1 right-1 w-1 h-1 bg-[#787b86] rounded-full group-hover:bg-white" />
        </button>

        <button className="group relative p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors" title="Icons & Stickers">
          <Smile className="w-4 h-4" />
        </button>

        <button className="p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors" title="Measure">
          <Ruler className="w-4 h-4" />
        </button>

        <button className="p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors" title="Zoom In">
          <ZoomIn className="w-4 h-4" />
        </button>
      </div>

      {/* Utility Tools */}
      <div className="flex flex-col items-center gap-1 w-full pt-2 border-t border-[#2a2e39]">
        <button className="p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Magnet Mode">
          <Magnet className="w-4 h-4" />
        </button>
        <button className="p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Lock All Drawing Tools">
          <Lock className="w-4 h-4" />
        </button>
        <button className="p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]" title="Hide All Drawings">
          <Eye className="w-4 h-4" />
        </button>
        <button className="p-2 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#f23645]" title="Remove Objects">
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
    </aside>
  );
};
