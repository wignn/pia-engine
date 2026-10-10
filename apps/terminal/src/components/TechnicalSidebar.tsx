"use client";

import React, { useState } from "react";
import { Check, ChevronDown, ChevronRight, RotateCcw, X } from "lucide-react";
import { IndicatorState } from "@/types";

interface TechnicalSidebarProps {
  indicators: IndicatorState;
  onToggleIndicator: (key: keyof IndicatorState) => void;
  onResetIndicators?: () => void;
  onApplyPreset?: (preset: "Default" | "Minimal" | "Klasik" | "Tren" | "Momentum") => void;
  activePreset?: string;
  onClose?: () => void;
}

interface IndicatorMeta {
  key: keyof IndicatorState;
  name: string;
  category: "ma" | "bands" | "trend" | "momentum" | "volatility";
  description: string;
}

const INDICATORS: IndicatorMeta[] = [
  { key: "sma20", name: "SMA 20 (Simple Moving Average)", category: "ma", description: "20-period simple moving average" },
  { key: "ema50", name: "EMA 50 (Exponential Moving Average)", category: "ma", description: "50-period exponential moving average" },
  { key: "vwap", name: "VWAP (Volume Weighted Average Price)", category: "ma", description: "Intraday volume-weighted average price" },
  { key: "bollinger", name: "Bollinger Bands (20, 2)", category: "bands", description: "Standard-deviation volatility bands" },
  { key: "rsi", name: "RSI (Relative Strength Index 14)", category: "momentum", description: "Overbought and oversold momentum" },
  { key: "macd", name: "MACD (12, 26, 9)", category: "momentum", description: "Moving average convergence divergence" },
  { key: "atr", name: "ATR (Average True Range 14)", category: "volatility", description: "Market volatility range" },
];

const PRESETS = [
  { id: "Default", label: "Default" },
  { id: "Minimal", label: "Minimal" },
  { id: "Klasik", label: "Classic" },
  { id: "Tren", label: "Trend" },
  { id: "Momentum", label: "Momentum" },
] as const;

const CATEGORIES = [
  { id: "ma", title: "Moving Averages" },
  { id: "bands", title: "Bands & Channels" },
  { id: "momentum", title: "Momentum" },
  { id: "volatility", title: "Volatility" },
] as const;

export const TechnicalSidebar: React.FC<TechnicalSidebarProps> = ({
  indicators,
  onToggleIndicator,
  onResetIndicators,
  onApplyPreset,
  activePreset = "Default",
  onClose,
}) => {
  const [collapsedSections, setCollapsedSections] = useState<Record<string, boolean>>({});
  const activeCount = Object.values(indicators).filter(Boolean).length;

  const toggleSection = (id: string) => {
    setCollapsedSections((sections) => ({ ...sections, [id]: !sections[id] }));
  };

  return (
    <aside className="absolute inset-y-0 left-0 z-30 flex h-full w-[min(280px,85vw)] shrink-0 flex-col overflow-hidden border-r border-border bg-card shadow-xl md:static md:w-64 md:shadow-none">
      <div className="border-b border-border px-4 py-3">
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-sm font-semibold text-foreground">Indicators</h2>
          {onClose && <button type="button" onClick={onClose} className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground md:hidden" aria-label="Close indicators"><X className="size-4" /></button>}
        </div>
        <label className="mt-3 block text-[10px] font-semibold uppercase tracking-widest text-muted-foreground" htmlFor="indicator-preset">Preset</label>
        <select id="indicator-preset" value={activePreset} onChange={(event) => onApplyPreset?.(event.target.value as "Default" | "Minimal" | "Klasik" | "Tren" | "Momentum")} className="mt-1 h-9 w-full rounded-md border border-border bg-background px-2 text-xs text-foreground">
          {PRESETS.map(({ id, label }) => <option key={id} value={id}>{label}</option>)}
        </select>
      </div>

      <div className="flex items-center justify-between border-b border-border px-3 py-2.5">
        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">Active ({activeCount})</span>
        {activeCount > 0 && onResetIndicators && (
          <button onClick={onResetIndicators} className="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-primary" title="Reset indicators">
            <RotateCcw className="size-3" /> Reset
          </button>
        )}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="divide-y divide-border">
          {CATEGORIES.map(({ id, title }) => {
            const collapsed = collapsedSections[id];
            const categoryIndicators = INDICATORS.filter((indicator) => indicator.category === id);
            return (
              <section key={id} className="p-3">
                <button onClick={() => toggleSection(id)} className="flex w-full items-center justify-between py-1 text-left text-xs font-semibold text-foreground hover:text-primary" aria-expanded={!collapsed}>
                  {title}
                  {collapsed ? <ChevronRight className="size-3.5 text-muted-foreground" /> : <ChevronDown className="size-3.5 text-muted-foreground" />}
                </button>
                {!collapsed && (
                  <div className="mt-2 space-y-1">
                    {categoryIndicators.map((indicator) => {
                      const enabled = Boolean(indicators[indicator.key]);
                      return (
                        <button
                          key={indicator.key}
                          onClick={() => onToggleIndicator(indicator.key)}
                          className={`flex w-full items-start gap-2.5 rounded-md border p-2 text-left transition-colors ${
                            enabled ? "border-primary/30 bg-primary/10" : "border-transparent hover:bg-muted/60"
                          }`}
                          aria-pressed={enabled}
                        >
                          <span className={`mt-0.5 flex size-4 shrink-0 items-center justify-center rounded border ${enabled ? "border-primary bg-primary text-background" : "border-border bg-card"}`}>
                            {enabled && <Check className="size-3 stroke-[3]" />}
                          </span>
                          <span className="min-w-0">
                            <span className={`block text-xs font-medium leading-tight ${enabled ? "text-primary" : "text-foreground"}`}>{indicator.name}</span>
                            <span className="mt-0.5 block text-[10px] leading-tight text-muted-foreground">{indicator.description}</span>
                          </span>
                        </button>
                      );
                    })}
                  </div>
                )}
              </section>
            );
          })}
        </div>
      </div>
    </aside>
  );
};
