"use client";

import React, { useEffect, useState, useMemo } from "react";
import {
  Activity,
  BarChart3,
  Gauge,
  RefreshCw,
  TrendingUp,
  TrendingDown,
  ExternalLink,
  Layers,
  Percent,
  Clock,
  ArrowUpRight,
  ArrowDownRight,
  ShieldCheck,
  AlertTriangle,
  Globe
} from "lucide-react";

interface Props {
  symbol: string;
  theme?: "dark" | "light";
}

interface OptionsSnapshot {
  id: string;
  symbol: string;
  underlying_price: number;
  put_call_ratio: number;
  max_pain_strike: number;
  total_open_interest: number;
  total_volume: number;
  total_gex: number;
  iv_atm: number;
  updated_at?: string;
}

interface YieldPoint {
  tenor: string;
  value: number;
  country?: string;
  raw_series_id?: string;
}

interface YieldSpread {
  spread: string;
  value: number;
}

interface FearGreedData {
  score: number;
  label: string;
  date?: string;
  components?: {
    momentum?: number;
    volatility?: number;
    safe_haven?: number;
    news_risk?: number;
  };
}

interface MacroNewsItem {
  id: string;
  title: string;
  source_name: string;
  url: string;
  published_at?: string;
  impact_level?: "high" | "medium" | "low";
  sentiment?: "positive" | "negative" | "neutral" | "bullish" | "bearish";
  summary?: string;
}

interface IntelligenceData {
  options_list?: OptionsSnapshot[];
  options?: { data?: OptionsSnapshot[] };
  yields?: {
    country: string;
    points: YieldPoint[];
    spreads: YieldSpread[];
    date?: string;
  };
  fear_greed?: FearGreedData;
  news?: {
    items: MacroNewsItem[];
  };
}

function mapSymbolToOptionsAsset(symbol: string): string {
  const s = symbol.toUpperCase();
  if (s === "XAUUSD" || s.startsWith("XAU") || s === "GOLD") return "GLD";
  if (s.includes("BTC")) return "BTC";
  if (s.includes("ETH")) return "ETH";
  if (s === "SPX") return "SPY";
  if (s === "NDX") return "QQQ";
  if (s === "AAPL") return "AAPL";
  if (s === "NVDA") return "NVDA";
  if (s === "MSFT") return "MSFT";
  if (s === "TSLA") return "TSLA";
  return "SPY";
}

function formatGex(val: number): string {
  if (Math.abs(val) >= 1e9) {
    return `${val > 0 ? "+" : ""}${(val / 1e9).toFixed(2)}B`;
  }
  if (Math.abs(val) >= 1e6) {
    return `${val > 0 ? "+" : ""}${(val / 1e6).toFixed(2)}M`;
  }
  return `${val > 0 ? "+" : ""}${val.toFixed(0)}`;
}

export const MarketIntelligencePanel: React.FC<Props> = ({ symbol, theme = "dark" }) => {
  const isLight = theme === "light";
  const [data, setData] = useState<IntelligenceData | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeOptionsSymbol, setActiveOptionsSymbol] = useState<string>(() => mapSymbolToOptionsAsset(symbol));

  const fetchData = async () => {
    setLoading(true);
    try {
      const res = await fetch(`/api/market-intelligence?symbol=${encodeURIComponent(symbol)}`, { cache: "no-store" });
      if (res.ok) {
        const payload = await res.json();
        setData(payload);
      }
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    setActiveOptionsSymbol(mapSymbolToOptionsAsset(symbol));
  }, [symbol]);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 45000);
    return () => clearInterval(interval);
  }, [symbol]);

  // Options snapshot parsing
  const optionsSnapshots: OptionsSnapshot[] = useMemo(() => {
    if (Array.isArray(data?.options_list) && data!.options_list.length > 0) {
      return data!.options_list;
    }
    if (Array.isArray(data?.options?.data) && data!.options!.data.length > 0) {
      return data!.options!.data;
    }
    return [];
  }, [data]);

  const currentOption = useMemo(() => {
    if (optionsSnapshots.length === 0) return null;
    return (
      optionsSnapshots.find((o) => o.symbol.toUpperCase() === activeOptionsSymbol.toUpperCase()) ||
      optionsSnapshots[0]
    );
  }, [optionsSnapshots, activeOptionsSymbol]);

  // US Treasury Yield points filtering
  const { tenorPoints, breakevenPoint, realPoint, spreads } = useMemo(() => {
    const rawPoints = data?.yields?.points ?? [];
    const rawSpreads = data?.yields?.spreads ?? [];

    const tenorOrder = ["3M", "2Y", "5Y", "10Y", "30Y"];
    const filteredTenors: YieldPoint[] = [];

    for (const t of tenorOrder) {
      const hit = rawPoints.find((p) => p.tenor.toUpperCase() === t);
      if (hit) filteredTenors.push(hit);
    }

    const bei = rawPoints.find((p) => p.tenor.toUpperCase().includes("BREAKEVEN"));
    const real = rawPoints.find((p) => p.tenor.toUpperCase().includes("REAL"));

    return {
      tenorPoints: filteredTenors,
      breakevenPoint: bei,
      realPoint: real,
      spreads: rawSpreads,
    };
  }, [data?.yields]);

  // Fear & Greed parsing
  const fg = data?.fear_greed;
  const fgScore = typeof fg?.score === "number" ? Math.round(fg.score) : 50;
  const fgLabel = fg?.label ? fg.label.toUpperCase() : fgScore > 60 ? "GREED" : fgScore < 40 ? "FEAR" : "NEUTRAL";

  // Macro News
  const newsItems = data?.news?.items ?? [];

  return (
    <div
      className={`h-full flex flex-col select-none text-xs transition-colors overflow-hidden ${
        isLight ? "bg-[#ffffff] text-[#131722]" : "bg-[#1e222d] text-[#d1d4dc]"
      }`}
    >
      {/* Panel Header */}
      <div
        className={`h-[44px] border-b flex items-center justify-between px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-1.5 font-bold text-sm">
          <Globe className="w-4 h-4 text-[#2962ff]" />
          <span className={isLight ? "text-[#131722]" : "text-white"}>Market Intelligence</span>
        </div>
        <button
          onClick={fetchData}
          disabled={loading}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
          }`}
          title="Refresh Intelligence Data"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {/* Main Content Scroll Area */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3.5">
        {/* ================= 1. Fear & Greed Index Section ================= */}
        <div
          className={`rounded-xl border p-3.5 shadow-xs transition-colors ${
            isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-center justify-between mb-2.5">
            <div className="flex items-center gap-2">
              <Gauge className="w-4 h-4 text-[#2962ff]" />
              <span className="font-bold text-xs">Fear & Greed Index</span>
            </div>
            <span
              className={`text-[10px] font-mono px-2 py-0.5 rounded font-bold uppercase ${
                fgScore >= 65
                  ? "bg-[#089981]/15 text-[#089981]"
                  : fgScore <= 35
                  ? "bg-[#f23645]/15 text-[#f23645]"
                  : "bg-[#f5b942]/15 text-[#f5b942]"
              }`}
            >
              {fgLabel}
            </span>
          </div>

          {/* Big Score Display & Gauge Bar */}
          <div className="flex items-baseline gap-2 mb-2">
            <span className={`font-mono text-3xl font-black ${isLight ? "text-[#131722]" : "text-white"}`}>
              {fgScore}
            </span>
            <span className="text-[11px] text-[#787b86] font-medium">/ 100 Global Market Sentiment</span>
          </div>

          {/* Multi-segment Sentiment Bar */}
          <div className="relative w-full mb-3">
            <div className="h-2 w-full rounded-full overflow-hidden flex">
              <div className="w-1/4 h-full bg-[#ef4444]" title="Extreme Fear (0-25)" />
              <div className="w-1/4 h-full bg-[#f59e0b]" title="Fear (25-50)" />
              <div className="w-1/4 h-full bg-[#3b82f6]" title="Neutral (50-75)" />
              <div className="w-1/4 h-full bg-[#10b981]" title="Extreme Greed (75-100)" />
            </div>
            {/* Pointer Marker */}
            <div
              style={{ left: `${Math.max(2, Math.min(98, fgScore))}%` }}
              className="absolute -top-1 w-2.5 h-4 -translate-x-1/2 bg-white dark:bg-[#ffffff] rounded-xs shadow-md border border-black/40 pointer-events-none"
            />
            <div className="flex justify-between text-[9px] font-mono text-[#787b86] mt-1.5">
              <span>0 Extreme Fear</span>
              <span>50 Neutral</span>
              <span>100 Extreme Greed</span>
            </div>
          </div>

          {/* Sub-components breakdown */}
          {fg?.components && (
            <div className="grid grid-cols-2 gap-2 pt-2.5 border-t border-inherit">
              <div className={`p-2 rounded-lg border ${isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"}`}>
                <div className="flex justify-between text-[10px] text-[#787b86] mb-1">
                  <span>Momentum</span>
                  <span className="font-mono font-bold text-inherit">{Math.round(fg.components.momentum ?? 50)}%</span>
                </div>
                <div className="h-1 rounded bg-[#e0e3eb] dark:bg-[#141722] overflow-hidden">
                  <div style={{ width: `${fg.components.momentum ?? 50}%` }} className="h-full bg-[#2962ff]" />
                </div>
              </div>

              <div className={`p-2 rounded-lg border ${isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"}`}>
                <div className="flex justify-between text-[10px] text-[#787b86] mb-1">
                  <span>Volatility</span>
                  <span className="font-mono font-bold text-inherit">{Math.round(fg.components.volatility ?? 50)}%</span>
                </div>
                <div className="h-1 rounded bg-[#e0e3eb] dark:bg-[#141722] overflow-hidden">
                  <div style={{ width: `${fg.components.volatility ?? 50}%` }} className="h-full bg-[#f23645]" />
                </div>
              </div>

              <div className={`p-2 rounded-lg border ${isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"}`}>
                <div className="flex justify-between text-[10px] text-[#787b86] mb-1">
                  <span>Safe Haven</span>
                  <span className="font-mono font-bold text-inherit">{Math.round(fg.components.safe_haven ?? 50)}%</span>
                </div>
                <div className="h-1 rounded bg-[#e0e3eb] dark:bg-[#141722] overflow-hidden">
                  <div style={{ width: `${fg.components.safe_haven ?? 50}%` }} className="h-full bg-[#089981]" />
                </div>
              </div>

              <div className={`p-2 rounded-lg border ${isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"}`}>
                <div className="flex justify-between text-[10px] text-[#787b86] mb-1">
                  <span>News Risk</span>
                  <span className="font-mono font-bold text-inherit">{Math.round(fg.components.news_risk ?? 50)}%</span>
                </div>
                <div className="h-1 rounded bg-[#e0e3eb] dark:bg-[#141722] overflow-hidden">
                  <div style={{ width: `${fg.components.news_risk ?? 50}%` }} className="h-full bg-[#f5b942]" />
                </div>
              </div>
            </div>
          )}
        </div>

        {/* ================= 2. US Treasury Yield Curve Section ================= */}
        <div
          className={`rounded-xl border p-3.5 shadow-xs transition-colors ${
            isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-center justify-between mb-2.5">
            <div className="flex items-center gap-2">
              <BarChart3 className="w-4 h-4 text-[#089981]" />
              <span className="font-bold text-xs">US Sovereign Yield Curve</span>
            </div>
            {spreads.length > 0 && (() => {
              const s2s10s = spreads.find((s) => s.spread === "2s10s")?.value;
              if (typeof s2s10s !== "number") return null;
              return (
                <span
                  className={`text-[10px] font-mono px-2 py-0.5 rounded font-bold ${
                    s2s10s >= 0 ? "bg-[#089981]/15 text-[#089981]" : "bg-[#f23645]/15 text-[#f23645]"
                  }`}
                >
                  2s10s: {s2s10s > 0 ? "+" : ""}{s2s10s}%
                </span>
              );
            })()}
          </div>

          {/* SVG Yield Curve Graph */}
          {tenorPoints.length >= 3 && (
            <div className="relative w-full h-24 mb-3">
              <svg className="w-full h-full" viewBox="0 0 500 100" preserveAspectRatio="none">
                <defs>
                  <linearGradient id="yieldGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#089981" stopOpacity="0.25" />
                    <stop offset="100%" stopColor="#089981" stopOpacity="0.0" />
                  </linearGradient>
                </defs>

                {/* Calculate points for SVG */}
                {(() => {
                  const minVal = Math.min(...tenorPoints.map((p) => p.value)) * 0.9;
                  const maxVal = Math.max(...tenorPoints.map((p) => p.value)) * 1.08;
                  const range = maxVal - minVal || 1;

                  const coords = tenorPoints.map((p, idx) => {
                    const x = (idx / (tenorPoints.length - 1)) * 460 + 20;
                    const y = 90 - ((p.value - minVal) / range) * 75;
                    return { x, y, tenor: p.tenor, val: p.value };
                  });

                  const pathD =
                    coords.length > 0
                      ? `M ${coords[0].x} ${coords[0].y} ` +
                        coords
                          .slice(1)
                          .map((c) => `L ${c.x} ${c.y}`)
                          .join(" ")
                      : "";

                  const areaD =
                    coords.length > 0
                      ? `${pathD} L ${coords[coords.length - 1].x} 95 L ${coords[0].x} 95 Z`
                      : "";

                  return (
                    <>
                      {/* Area */}
                      <path d={areaD} fill="url(#yieldGrad)" />
                      {/* Curve */}
                      <path d={pathD} fill="none" stroke="#089981" strokeWidth="2.5" strokeLinecap="round" />
                      {/* Dots & Labels */}
                      {coords.map((c, i) => (
                        <g key={i}>
                          <circle cx={c.x} cy={c.y} r="3.5" fill="#089981" stroke="#ffffff" strokeWidth="1.5" />
                          <text
                            x={c.x}
                            y={c.y - 7}
                            textAnchor="middle"
                            fill={isLight ? "#131722" : "#ffffff"}
                            fontSize="9"
                            fontFamily="monospace"
                            fontWeight="bold"
                          >
                            {c.val.toFixed(2)}%
                          </text>
                        </g>
                      ))}
                    </>
                  );
                })()}
              </svg>
            </div>
          )}

          {/* Tenor Tiles */}
          <div className="grid grid-cols-5 gap-1.5 mb-2.5">
            {tenorPoints.map((pt) => (
              <div
                key={pt.tenor}
                className={`p-1.5 rounded-lg border text-center ${
                  isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
                }`}
              >
                <div className="text-[10px] text-[#787b86] font-bold">{pt.tenor}</div>
                <div className={`font-mono text-xs font-black ${isLight ? "text-[#131722]" : "text-white"}`}>
                  {pt.value.toFixed(2)}%
                </div>
              </div>
            ))}
          </div>

          {/* Inflation Expectations & Real Yield */}
          {(breakevenPoint || realPoint) && (
            <div className="grid grid-cols-2 gap-2 pt-2.5 border-t border-inherit">
              {breakevenPoint && (
                <div className="flex items-center justify-between text-[11px]">
                  <span className="text-[#787b86]">10Y Breakeven Inf.</span>
                  <span className="font-mono font-bold text-amber-500">{breakevenPoint.value.toFixed(2)}%</span>
                </div>
              )}
              {realPoint && (
                <div className="flex items-center justify-between text-[11px]">
                  <span className="text-[#787b86]">10Y Real TIPS Yield</span>
                  <span className={`font-mono font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
                    {realPoint.value.toFixed(2)}%
                  </span>
                </div>
              )}
            </div>
          )}
        </div>

        {/* ================= 3. Options Flow, Sentiment & GEX ================= */}
        <div
          className={`rounded-xl border p-3.5 shadow-xs transition-colors ${
            isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <Activity className="w-4 h-4 text-[#f5b942]" />
              <span className="font-bold text-xs">Options Flow & Gamma Exposure</span>
            </div>
            {currentOption && (
              <span className="font-mono text-[10px] text-[#2962ff] font-bold uppercase">
                Spot: ${currentOption.underlying_price.toLocaleString()}
              </span>
            )}
          </div>

          {/* Asset Selector Tabs */}
          {optionsSnapshots.length > 0 && (
            <div className="flex items-center gap-1 mb-3 overflow-x-auto pb-1">
              {optionsSnapshots.map((o) => {
                const isSelected = o.symbol.toUpperCase() === (currentOption?.symbol.toUpperCase() || "");
                return (
                  <button
                    key={o.symbol}
                    onClick={() => setActiveOptionsSymbol(o.symbol)}
                    className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold transition-all cursor-pointer whitespace-nowrap ${
                      isSelected
                        ? "bg-[#2962ff] text-white shadow-xs"
                        : isLight
                        ? "bg-white border border-[#e0e3eb] text-[#5d606b] hover:text-[#131722]"
                        : "bg-[#1e222d] border border-[#2a2e39] text-[#787b86] hover:text-white"
                    }`}
                  >
                    {o.symbol}
                  </button>
                );
              })}
            </div>
          )}

          {currentOption ? (
            <div className="space-y-2.5">
              {/* Put/Call Ratio & Sentiment Gauge */}
              <div
                className={`p-2.5 rounded-lg border ${
                  isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
                }`}
              >
                <div className="flex items-center justify-between mb-1.5">
                  <span className="text-[11px] text-[#787b86]">Put / Call Ratio (PCR)</span>
                  <div className="flex items-center gap-1.5">
                    <span className={`font-mono text-sm font-black ${isLight ? "text-[#131722]" : "text-white"}`}>
                      {currentOption.put_call_ratio.toFixed(2)}
                    </span>
                    <span
                      className={`text-[9px] font-bold px-1.5 py-0.2 rounded uppercase ${
                        currentOption.put_call_ratio <= 0.7
                          ? "bg-[#089981]/15 text-[#089981]"
                          : currentOption.put_call_ratio >= 1.0
                          ? "bg-[#f23645]/15 text-[#f23645]"
                          : "bg-[#f5b942]/15 text-[#f5b942]"
                      }`}
                    >
                      {currentOption.put_call_ratio <= 0.7 ? "Calls Heavy" : currentOption.put_call_ratio >= 1.0 ? "Puts Heavy" : "Balanced"}
                    </span>
                  </div>
                </div>

                {/* Sentiment distribution bar */}
                <div className="w-full h-1.5 rounded-full overflow-hidden flex bg-[#141722]">
                  <div
                    style={{ width: `${Math.max(15, Math.min(85, (1 - (currentOption.put_call_ratio / 2)) * 100))}%` }}
                    className="h-full bg-[#089981]"
                    title="Call Volume Demand"
                  />
                  <div className="flex-1 h-full bg-[#f23645]" title="Put Volume Demand" />
                </div>
              </div>

              {/* Gamma Exposure (GEX) */}
              <div
                className={`p-2.5 rounded-lg border ${
                  isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-[11px] text-[#787b86]">Total Gamma Exposure (GEX)</span>
                  <span
                    className={`font-mono text-sm font-black ${
                      currentOption.total_gex >= 0 ? "text-[#089981]" : "text-[#f23645]"
                    }`}
                  >
                    {formatGex(currentOption.total_gex)}
                  </span>
                </div>
                <div className="text-[10px] text-[#787b86]">
                  {currentOption.total_gex >= 0 ? (
                    <span className="text-[#089981]">Positive GEX · Volatility Suppressor (Mean-reverting)</span>
                  ) : (
                    <span className="text-[#f23645]">Negative GEX · Volatility Accelerator (Breakout prone)</span>
                  )}
                </div>
              </div>

              {/* Max Pain & ATM Implied Volatility */}
              <div className="grid grid-cols-2 gap-2">
                <div
                  className={`p-2.5 rounded-lg border ${
                    isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
                  }`}
                >
                  <span className="text-[10px] text-[#787b86] block">Max Pain Strike</span>
                  <span className={`font-mono text-sm font-black ${isLight ? "text-[#131722]" : "text-white"}`}>
                    ${currentOption.max_pain_strike.toLocaleString()}
                  </span>
                </div>

                <div
                  className={`p-2.5 rounded-lg border ${
                    isLight ? "bg-white border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
                  }`}
                >
                  <span className="text-[10px] text-[#787b86] block">ATM Implied Vol (IV)</span>
                  <span className={`font-mono text-sm font-black ${isLight ? "text-[#131722]" : "text-white"}`}>
                    {(currentOption.iv_atm * 100).toFixed(1)}%
                  </span>
                </div>
              </div>

              {/* Open Interest & Volume */}
              <div className="grid grid-cols-2 gap-2">
                <div className="text-[11px] flex justify-between px-1">
                  <span className="text-[#787b86]">Total Open Interest:</span>
                  <span className="font-mono font-bold text-inherit">{currentOption.total_open_interest.toLocaleString()}</span>
                </div>
                <div className="text-[11px] flex justify-between px-1">
                  <span className="text-[#787b86]">24h Options Vol:</span>
                  <span className="font-mono font-bold text-inherit">{currentOption.total_volume.toLocaleString()}</span>
                </div>
              </div>
            </div>
          ) : (
            <div className="p-4 text-center text-[#787b86] text-xs">
              No options contracts available for {symbol}
            </div>
          )}
        </div>

        {/* ================= 4. Central Bank & Macro Intelligence Feed ================= */}
        {newsItems.length > 0 && (
          <div
            className={`rounded-xl border p-3.5 shadow-xs transition-colors ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="flex items-center gap-2 mb-2.5">
              <Clock className="w-4 h-4 text-[#2962ff]" />
              <span className="font-bold text-xs">Macro News & Central Bank Wire</span>
            </div>

            <div className="space-y-2">
              {newsItems.slice(0, 5).map((item) => (
                <a
                  key={item.id}
                  href={item.url}
                  target="_blank"
                  rel="noreferrer"
                  className={`block p-2 rounded-lg border transition-colors cursor-pointer ${
                    isLight
                      ? "bg-white border-[#e0e3eb] hover:bg-[#f0f3fa]"
                      : "bg-[#1e222d] border-[#2a2e39] hover:bg-[#262b37]"
                  }`}
                >
                  <div className="flex items-center justify-between text-[10px] text-[#787b86] mb-1">
                    <span className="font-bold uppercase text-[#2962ff]">{item.source_name}</span>
                    {item.impact_level === "high" && (
                      <span className="bg-[#f23645]/15 text-[#f23645] px-1 rounded font-bold">HIGH</span>
                    )}
                  </div>
                  <div className={`font-semibold text-xs leading-snug line-clamp-2 ${isLight ? "text-[#131722]" : "text-white"}`}>
                    {item.title}
                  </div>
                </a>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
