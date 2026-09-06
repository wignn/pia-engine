"use client";

import React, { useEffect, useState } from "react";
import { Activity, BarChart3, Gauge, RefreshCw } from "lucide-react";

interface Props { symbol: string; }
interface Intelligence {
  options: any;
  yields: any;
  fear_greed: any;
  news: any;
}

function Empty({ label }: { label: string }) {
  return <div className="rounded border border-[#2a2e39] bg-[#181b27] px-3 py-3 text-[11px] text-[#787b86]">{label} unavailable</div>;
}

export const MarketIntelligencePanel: React.FC<Props> = ({ symbol }) => {
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

  if (loading) return <div className="h-full bg-[#1e222d] p-3 text-xs text-[#787b86]">Loading market intelligence…</div>;

  const score = data?.fear_greed?.score;
  const options = data?.options;
  const points = data?.yields?.points ?? [];
  const newsItems = data?.news?.items ?? data?.news?.data ?? [];

  return (
    <aside className="h-full overflow-y-auto bg-[#1e222d] border-l border-[#2a2e39] p-3 text-xs text-[#d1d4dc]">
      <div className="mb-3 flex items-center justify-between">
        <div><div className="font-bold text-white">Market intelligence</div><div className="text-[10px] text-[#787b86]">Cross-asset context for {symbol}</div></div>
        <RefreshCw className="h-3.5 w-3.5 text-[#787b86]" />
      </div>

      <section className="mb-3">
        <div className="mb-1.5 flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wide text-[#787b86]"><Gauge className="h-3 w-3" />Regime</div>
        {score == null ? <Empty label="Fear & Greed" /> : <div className="flex items-end justify-between rounded border border-[#2a2e39] bg-[#181b27] p-3"><div><div className="text-2xl font-bold text-white">{Number(score).toFixed(0)}</div><div className="text-[10px] text-[#787b86]">{data?.fear_greed?.label ?? "Unknown"}</div></div><Activity className="h-5 w-5 text-[#f5b942]" /></div>}
      </section>

      <section className="mb-3">
        <div className="mb-1.5 flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wide text-[#787b86]"><BarChart3 className="h-3 w-3" />Options</div>
        {!options ? <Empty label="Options snapshot" /> : <div className="grid grid-cols-2 gap-1.5 rounded border border-[#2a2e39] bg-[#181b27] p-2"><div><span className="text-[10px] text-[#787b86]">Put/Call</span><div className="font-mono font-bold text-white">{Number(options.put_call_ratio ?? 0).toFixed(2)}</div></div><div><span className="text-[10px] text-[#787b86]">Max pain</span><div className="font-mono font-bold text-white">{Number(options.max_pain_strike ?? 0).toFixed(2)}</div></div><div><span className="text-[10px] text-[#787b86]">Open interest</span><div className="font-mono font-bold text-white">{Number(options.total_open_interest ?? 0).toLocaleString()}</div></div><div><span className="text-[10px] text-[#787b86]">GEX</span><div className="font-mono font-bold text-white">{Number(options.total_gex ?? 0).toFixed(0)}</div></div></div>}
      </section>

      <section className="mb-3">
        <div className="mb-1.5 text-[10px] font-bold uppercase tracking-wide text-[#787b86]">US yield curve</div>
        {!points.length ? <Empty label="Yield curve" /> : <div className="rounded border border-[#2a2e39] bg-[#181b27] p-2">{points.slice(0, 6).map((point: any) => <div key={`${point.tenor}-${point.date}`} className="flex justify-between border-b border-[#2a2e39]/60 py-1 last:border-0"><span className="text-[#787b86]">{point.tenor}</span><span className="font-mono text-white">{Number(point.value).toFixed(2)}%</span></div>)}</div>}
      </section>

      <section>
        <div className="mb-1.5 text-[10px] font-bold uppercase tracking-wide text-[#787b86]">Latest news</div>
        {!newsItems.length ? <Empty label="News" /> : <div className="space-y-1.5">{newsItems.slice(0, 5).map((item: any, index: number) => <a key={item.id ?? index} href={item.url} target="_blank" rel="noreferrer" className="block rounded border border-[#2a2e39] bg-[#181b27] p-2 hover:border-[#2962ff]"><div className="line-clamp-2 text-[11px] font-medium text-white">{item.title ?? item.headline}</div><div className="mt-1 text-[10px] text-[#787b86]">{item.source ?? "ATLSD feed"}</div></a>)}</div>}
      </section>
    </aside>
  );
};
