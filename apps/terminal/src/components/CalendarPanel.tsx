"use client";

import React, { useState, useEffect } from "react";
import { Calendar as CalendarIcon, Filter, RefreshCw, AlertCircle } from "lucide-react";

interface CalendarEvent {
  id: string;
  country: string;
  currency: string;
  title: string;
  impact: "high" | "medium" | "low";
  time: string;
  date: string;
  forecast?: string;
  previous?: string;
  actual?: string | null;
  status?: "completed" | "upcoming";
}

interface CalendarPanelProps {
  theme?: "dark" | "light";
}

export const CalendarPanel: React.FC<CalendarPanelProps> = ({ theme = "dark" }) => {
  const isLight = theme === "light";
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [impactFilter, setImpactFilter] = useState<"all" | "high" | "medium">("all");
  const [currencyFilter, setCurrencyFilter] = useState<string>("all");

  const loadCalendar = async () => {
    setLoading(true);
    try {
      const res = await fetch("/api/calendar", { cache: "no-store" });
      if (res.ok) {
        const data = await res.json();
        setEvents(data.items ?? []);
      }
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadCalendar();
  }, []);

  const filtered = events.filter((e) => {
    if (impactFilter === "high" && e.impact !== "high") return false;
    if (impactFilter === "medium" && e.impact === "low") return false;
    if (currencyFilter !== "all" && e.currency !== currencyFilter) return false;
    return true;
  });

  return (
    <div
      className={`w-full flex flex-col h-full select-none text-xs transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Header */}
      <div
        className={`h-[44px] border-b flex items-center justify-between px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-1.5 font-bold text-sm">
          <CalendarIcon className="w-4 h-4 text-[#2962ff]" />
          <span className={isLight ? "text-[#131722]" : "text-white"}>Economic Calendar</span>
        </div>
        <button
          onClick={loadCalendar}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
          }`}
          title="Refresh Calendar"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {/* Filter Tabs */}
      <div
        className={`flex items-center gap-1 px-3 py-1.5 border-b overflow-x-auto text-[11px] shrink-0 ${
          isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
        }`}
      >
        {(["all", "high", "medium"] as const).map((imp) => (
          <button
            key={imp}
            onClick={() => setImpactFilter(imp)}
            className={`px-2 py-0.5 rounded capitalize font-medium transition-colors cursor-pointer ${
              impactFilter === imp
                ? isLight
                  ? "bg-[#ffffff] text-[#131722] font-bold shadow-xs"
                  : "bg-[#2a2e39] text-white font-bold shadow-xs"
                : isLight
                ? "text-[#5d606b] hover:text-[#131722]"
                : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {imp === "high" ? "🔴 High Impact" : imp}
          </button>
        ))}
      </div>

      {/* Events List */}
      <div className={`flex-1 overflow-y-auto divide-y ${isLight ? "divide-[#e0e3eb]" : "divide-[#2a2e39]/50"}`}>
        {loading && events.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">Loading economic events...</div>
        ) : filtered.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">No economic events found</div>
        ) : (
          filtered.map((item) => {
            const isHigh = item.impact === "high";
            const isMed = item.impact === "medium";

            return (
              <div
                key={item.id}
                className={`p-3 transition-colors ${
                  isLight ? "hover:bg-[#f8f9fc]" : "hover:bg-[#262b37]"
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-1.5">
                    <span
                      className={`text-[10px] font-mono font-bold px-1.5 py-0.2 rounded border ${
                        isLight ? "bg-[#f0f3fa] text-[#131722] border-[#e0e3eb]" : "bg-[#141722] text-white border-[#2a2e39]"
                      }`}
                    >
                      {item.currency}
                    </span>
                    <span
                      className={`w-2 h-2 rounded-full ${
                        isHigh ? "bg-[#f23645]" : isMed ? "bg-[#f5b942]" : "bg-[#787b86]"
                      }`}
                      title={`${item.impact} impact`}
                    />
                  </div>
                  <span className="text-[10px] text-[#787b86] font-mono">{item.time}</span>
                </div>

                <div className={`font-semibold text-xs leading-snug mb-2 ${isLight ? "text-[#131722]" : "text-white"}`}>
                  {item.title}
                </div>

                {/* Macro metrics */}
                <div className="grid grid-cols-3 gap-2 text-[10px] font-mono">
                  <div>
                    <span className="text-[#787b86] block">Actual</span>
                    <span className={`font-bold ${item.actual ? (isLight ? "text-[#131722]" : "text-white") : "text-[#787b86]"}`}>
                      {item.actual || "--"}
                    </span>
                  </div>
                  <div>
                    <span className="text-[#787b86] block">Forecast</span>
                    <span className={isLight ? "text-[#5d606b]" : "text-[#d1d4dc]"}>{item.forecast || "--"}</span>
                  </div>
                  <div>
                    <span className="text-[#787b86] block">Previous</span>
                    <span className={isLight ? "text-[#5d606b]" : "text-[#d1d4dc]"}>{item.previous || "--"}</span>
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
