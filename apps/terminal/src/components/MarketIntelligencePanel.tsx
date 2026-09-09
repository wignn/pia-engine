"use client";

import React, { useEffect, useState } from "react";
import { Activity, BarChart3, Gauge, RefreshCw } from "lucide-react";

interface Props {
  symbol: string;
  theme?: "dark" | "light";
}

interface Intelligence {
  options: any;
  yields: any;
  fear_greed: any;
  news: any;
}

function Empty({ label, isLight }: { label: string; isLight: boolean }) {
  return (
    <div
      className={`rounded border px-3 py-3 text-[11px] ${
        isLight ? "border-[#e0e3eb] bg-[#f8f9fc] text-[#5d606b]" : "border-[#2a2e39] bg-[#181b27] text-[#787b86]"
      }`}
    >
      {label} unavailable
    </div>
  );
}

export const MarketIntelligencePanel: React.FC<Props> = ({ symbol, theme = "dark" }) => {
  const isLight = theme === "light";
  const [data, setData] = useState<Intelligence | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    fetch(`/api/market-intelligence?symbol=${encodeURIComponent(symbol)}`, { cache: "no-store" })
      .then((response) => response.json())
      .then((payload) => { if (!cancelled) setData(payload); })
      .catch(() => { if (!cancelled) setData(null); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [symbol]);

  if (loading) {
    return (
      <div
        className={`h-full p-3 text-xs ${
          isLight ? "bg-[#ffffff] text-[#5d606b]" : "bg-[#1e222d] text-[#787b86]"
        }`}
      >
        Loading market intelligence…
      </div>
    );
  }

  const score = data?.fear_greed?.score;
  const options = data?.options;
  const points = data?.yields?.points ?? [];
  const newsItems = data?.news?.items ?? data?.news?.data ?? [];

  return (
    <div
      className={`h-full space-y-3 overflow-y-auto p-3 text-xs transition-colors ${
        isLight ? "bg-[#ffffff] text-[#131722]" : "bg-[#1e222d] text-[#d1d4dc]"
      }`}
    >
      {/* Fear & Greed */}
      <div
        className={`rounded border p-3 ${
          isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
        }`}
      >
        <div className="flex items-center gap-2 text-[11px] font-semibold text-[#787b86]">
          <Gauge className="h-3.5 w-3.5 text-[#2962ff]" />
          <span>Fear & Greed Index</span>
        </div>
        {typeof score === "number" ? (
          <div className="mt-2 flex items-baseline gap-2">
            <span className={`text-2xl font-black ${isLight ? "text-[#131722]" : "text-white"}`}>{score}</span>
            <span
              className={`text-xs font-semibold ${
                score > 60 ? "text-[#089981]" : score < 40 ? "text-[#f23645]" : "text-[#f5b942]"
              }`}
            >
              {data?.fear_greed?.rating ?? "Neutral"}
            </span>
          </div>
        ) : (
          <Empty label="Fear & Greed" isLight={isLight} />
        )}
      </div>

      {/* Yield Curve */}
      <div
        className={`rounded border p-3 ${
          isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
        }`}
      >
        <div className="flex items-center gap-2 text-[11px] font-semibold text-[#787b86]">
          <BarChart3 className="h-3.5 w-3.5 text-[#089981]" />
          <span>US Treasury Yield Curve</span>
        </div>
        {points.length > 0 ? (
          <div className="mt-2 grid grid-cols-3 gap-2">
            {points.slice(0, 6).map((point: any, idx: number) => (
              <div
                key={idx}
                className={`rounded border p-1.5 text-center font-mono ${
                  isLight ? "border-[#e0e3eb] bg-[#ffffff]" : "border-[#2a2e39] bg-[#181b27]"
                }`}
              >
                <div className="text-[10px] text-[#787b86]">{point.maturity ?? point.tenor}</div>
                <div className={`text-xs font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
                  {Number(point.yield ?? point.value).toFixed(2)}%
                </div>
              </div>
            ))}
          </div>
        ) : (
          <Empty label="Yield Curve" isLight={isLight} />
        )}
      </div>

      {/* Options Sentiment */}
      <div
        className={`rounded border p-3 ${
          isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
        }`}
      >
        <div className="flex items-center gap-2 text-[11px] font-semibold text-[#787b86]">
          <Activity className="h-3.5 w-3.5 text-[#f5b942]" />
          <span>Options Flow / PCR</span>
        </div>
        {options?.put_call_ratio ? (
          <div className="mt-2 flex items-center justify-between">
            <span className="text-[11px] text-[#787b86]">Put/Call Ratio</span>
            <span className={`font-mono text-sm font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
              {Number(options.put_call_ratio).toFixed(2)}
            </span>
          </div>
        ) : (
          <Empty label="Options Flow" isLight={isLight} />
        )}
      </div>
    </div>
  );
};
