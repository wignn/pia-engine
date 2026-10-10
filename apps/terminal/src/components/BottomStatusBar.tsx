"use client";

import { useEffect, useState } from "react";
import { useMarketStore } from "@/stores/useMarketStore";

export function BottomStatusBar({ symbol, timeframe }: { symbol: string; timeframe: string }) {
  const { connectionState, isLoadingCandles } = useMarketStore();
  const [utcTime, setUtcTime] = useState("");

  useEffect(() => {
    const update = () => setUtcTime(new Date().toISOString().slice(11, 19) + " UTC");
    update();
    const timer = window.setInterval(update, 1000);
    return () => window.clearInterval(timer);
  }, []);

  const status = connectionState.status;
  const statusLabel = status === "connected" ? "Live feed" : status === "connecting" ? "Connecting" : status === "error" ? "Connection issue" : "Offline";
  const statusColor = status === "connected" ? "bg-up" : status === "connecting" ? "bg-amber-400" : "bg-down";

  return (
    <footer className="hidden h-7 min-h-7 items-center gap-4 border-t border-[#263546] bg-[#101923] px-4 text-[10px] text-[#8fa0b4] md:flex">
      <div className="flex items-center gap-1.5" title={connectionState.error || statusLabel}>
        <span className={`size-1.5 rounded-full ${statusColor}`} />
        <span className="font-medium text-[#d8e2ec]">{statusLabel}</span>
        {status === "connected" && connectionState.latencyMs !== undefined && <span className="font-mono">{connectionState.latencyMs} ms</span>}
      </div>
      <span className="h-3 w-px bg-[#344357]" />
      <span className="font-mono text-[#d8e2ec]">{symbol || "—"} <span className="text-[#6f8195]">·</span> {timeframe || "—"}</span>
      {isLoadingCandles && <span>Loading history…</span>}
      <span className="ml-auto font-mono tabular-nums">{utcTime}</span>
    </footer>
  );
}
