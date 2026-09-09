"use client";

import React from "react";
import { ChartLayout } from "@/types";

interface VisualLayoutPickerProps {
  currentLayout: ChartLayout;
  onSelectLayout: (layout: ChartLayout) => void;
  onClose?: () => void;
  theme?: "dark" | "light";
}

interface LayoutItem {
  id: ChartLayout;
  name: string;
  panesCount: number;
  diagram: React.ReactNode;
}

export const VisualLayoutPicker: React.FC<VisualLayoutPickerProps> = ({
  currentLayout,
  onSelectLayout,
  onClose,
  theme = "dark",
}) => {
  const isLight = theme === "light";

  const boxClass = isLight
    ? "border-[#b2b5be] bg-[#e0e3eb]"
    : "border-[#434651] bg-[#2a2e39]";
  const activeBoxClass = "border-[#2962ff] bg-[#2962ff]/30";

  const layouts: { group: string; items: LayoutItem[] }[] = [
    {
      group: "1 & 2 Screens",
      items: [
        {
          id: "1x1",
          name: "Single Screen",
          panesCount: 1,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex">
              <div className={`w-full h-full rounded-2xs border ${currentLayout === "1x1" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
        {
          id: "1x2",
          name: "2 Vertical Columns",
          panesCount: 2,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex gap-0.5">
              <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "1x2" ? activeBoxClass : boxClass}`} />
              <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "1x2" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
        {
          id: "2x1",
          name: "2 Horizontal Rows",
          panesCount: 2,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex flex-col gap-0.5">
              <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "2x1" ? activeBoxClass : boxClass}`} />
              <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "2x1" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
      ],
    },
    {
      group: "3 Screens (Triple Layout)",
      items: [
        {
          id: "1x3",
          name: "3 Vertical Columns",
          panesCount: 3,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex gap-0.5">
              <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "1x3" ? activeBoxClass : boxClass}`} />
              <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "1x3" ? activeBoxClass : boxClass}`} />
              <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "1x3" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
        {
          id: "3x1",
          name: "3 Horizontal Rows",
          panesCount: 3,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex flex-col gap-0.5">
              <div className={`w-full h-1/3 rounded-2xs border ${currentLayout === "3x1" ? activeBoxClass : boxClass}`} />
              <div className={`w-full h-1/3 rounded-2xs border ${currentLayout === "3x1" ? activeBoxClass : boxClass}`} />
              <div className={`w-full h-1/3 rounded-2xs border ${currentLayout === "3x1" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
        {
          id: "1L-2R",
          name: "1 Large Left, 2 Right",
          panesCount: 3,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex gap-0.5">
              <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "1L-2R" ? activeBoxClass : boxClass}`} />
              <div className="w-1/2 h-full flex flex-col gap-0.5">
                <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "1L-2R" ? activeBoxClass : boxClass}`} />
                <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "1L-2R" ? activeBoxClass : boxClass}`} />
              </div>
            </div>
          ),
        },
        {
          id: "2L-1R",
          name: "2 Left, 1 Large Right",
          panesCount: 3,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex gap-0.5">
              <div className="w-1/2 h-full flex flex-col gap-0.5">
                <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "2L-1R" ? activeBoxClass : boxClass}`} />
                <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "2L-1R" ? activeBoxClass : boxClass}`} />
              </div>
              <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2L-1R" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
        {
          id: "1T-2B",
          name: "1 Large Top, 2 Bottom",
          panesCount: 3,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex flex-col gap-0.5">
              <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "1T-2B" ? activeBoxClass : boxClass}`} />
              <div className="w-full h-1/2 flex gap-0.5">
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "1T-2B" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "1T-2B" ? activeBoxClass : boxClass}`} />
              </div>
            </div>
          ),
        },
        {
          id: "2T-1B",
          name: "2 Top, 1 Large Bottom",
          panesCount: 3,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex flex-col gap-0.5">
              <div className="w-full h-1/2 flex gap-0.5">
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2T-1B" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2T-1B" ? activeBoxClass : boxClass}`} />
              </div>
              <div className={`w-full h-1/2 rounded-2xs border ${currentLayout === "2T-1B" ? activeBoxClass : boxClass}`} />
            </div>
          ),
        },
      ],
    },
    {
      group: "4 & 6 Screens (Multi-Grid)",
      items: [
        {
          id: "2x2",
          name: "4 Quad Grid (2x2)",
          panesCount: 4,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex flex-col gap-0.5">
              <div className="w-full h-1/2 flex gap-0.5">
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2x2" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2x2" ? activeBoxClass : boxClass}`} />
              </div>
              <div className="w-full h-1/2 flex gap-0.5">
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2x2" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/2 h-full rounded-2xs border ${currentLayout === "2x2" ? activeBoxClass : boxClass}`} />
              </div>
            </div>
          ),
        },
        {
          id: "3x2",
          name: "6 Screens Grid (3x2)",
          panesCount: 6,
          diagram: (
            <div className="w-10 h-7 rounded-xs border p-0.5 flex flex-col gap-0.5">
              <div className="w-full h-1/2 flex gap-0.5">
                <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "3x2" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "3x2" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "3x2" ? activeBoxClass : boxClass}`} />
              </div>
              <div className="w-full h-1/2 flex gap-0.5">
                <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "3x2" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "3x2" ? activeBoxClass : boxClass}`} />
                <div className={`w-1/3 h-full rounded-2xs border ${currentLayout === "3x2" ? activeBoxClass : boxClass}`} />
              </div>
            </div>
          ),
        },
      ],
    },
  ];

  return (
    <div
      className={`w-72 rounded-xl border shadow-2xl p-3 select-none flex flex-col gap-3 transition-colors ${
        isLight
          ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
          : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="flex items-center justify-between pb-1 border-b border-border/50">
        <span className="font-bold text-xs">Select Chart Layout</span>
        <span className="text-[10px] opacity-60">TradingView Style</span>
      </div>

      {layouts.map((grp) => (
        <div key={grp.group} className="space-y-1.5">
          <div className="text-[10px] font-bold uppercase tracking-wider opacity-60">
            {grp.group}
          </div>
          <div className="grid grid-cols-3 gap-2">
            {grp.items.map((item) => {
              const isSelected = currentLayout === item.id;
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => {
                    onSelectLayout(item.id);
                    onClose?.();
                  }}
                  className={`flex flex-col items-center justify-center p-1.5 rounded-lg border transition-all cursor-pointer ${
                    isSelected
                      ? "border-[#2962ff] bg-[#2962ff]/10 ring-1 ring-[#2962ff]"
                      : isLight
                      ? "border-[#e0e3eb] bg-[#f8f9fc] hover:border-[#2962ff]/60 hover:bg-[#f0f3fa]"
                      : "border-[#2a2e39] bg-[#141722] hover:border-[#2962ff]/60 hover:bg-[#2a2e39]"
                  }`}
                  title={item.name}
                >
                  <div className="mb-1 pointer-events-none">{item.diagram}</div>
                  <span className="text-[10px] font-mono leading-none truncate max-w-full">
                    {item.id}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
};
