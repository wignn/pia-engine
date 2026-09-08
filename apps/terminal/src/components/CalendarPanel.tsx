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

export const CalendarPanel: React.FC = () => {
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

  const currencies = ["all", "USD", "EUR", "GBP", "JPY"];

  const filteredEvents = events.filter((e) => {
    if (impactFilter === "high" && e.impact !== "high") return false;
    if (impactFilter === "medium" && e.impact !== "medium") return false;
    if (currencyFilter !== "all" && e.currency !== currencyFilter) return false;
    return true;
  });

  return (
    <div className="flex h-full flex-col bg-[#1e222d] border-l border-[#2a2e39] text-xs text-[#d1d4dc] overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[#2a2e39] px-4 py-3 shrink-0">
        <div className="flex items-center gap-2">
          <CalendarIcon className="w-4 h-4 text-[#2962ff]" />
          <span className="font-bold text-white text-sm">Economic Calendar</span>
        </div>
        <button
          onClick={loadCalendar}
          disabled={loading}
          className="p-1 rounded text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-colors disabled:opacity-50"
          title="Refresh Calendar"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin text-[#2962ff]" : ""}`} />
        </button>
      </div>

      {/* Filter Tabs */}
      <div className="flex flex-col gap-2 border-b border-[#2a2e39] bg-[#141722] p-2.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1">
            {(["all", "high", "medium"] as const).map((imp) => (
              <button
                key={imp}
                onClick={() => setImpactFilter(imp)}
                className={`rounded px-2 py-0.5 text-[10px] font-bold uppercase transition-colors ${
                  impactFilter === imp
                    ? imp === "high"
                      ? "bg-[#f23645] text-white"
                      : imp === "medium"
                      ? "bg-[#f5b942] text-black"
                      : "bg-[#2962ff] text-white"
                    : "text-[#787b86] hover:bg-[#1e222d] hover:text-[#d1d4dc]"
                }`}
              >
                {imp}
              </button>
            ))}
          </div>

          <div className="flex items-center gap-1 font-mono text-[10px]">
            {currencies.map((curr) => (
              <button
                key={curr}
                onClick={() => setCurrencyFilter(curr)}
                className={`rounded px-1.5 py-0.5 uppercase transition-colors ${
                  currencyFilter === curr
                    ? "bg-[#2a2e39] text-white font-bold"
                    : "text-[#787b86] hover:text-[#d1d4dc]"
                }`}
              >
                {curr}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Events List */}
      <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
        {loading && events.length === 0 ? (
          <div className="py-8 text-center text-[#787b86]">Loading calendar events…</div>
        ) : filteredEvents.length === 0 ? (
          <div className="rounded border border-[#2a2e39] bg-[#181b27] p-4 text-center text-[#787b86]">
            No economic events match current filters.
          </div>
        ) : (
          filteredEvents.map((evt) => {
            const isHigh = evt.impact === "high";
            const isMedium = evt.impact === "medium";

            return (
              <div
                key={evt.id}
                className="rounded border border-[#2a2e39] bg-[#181b27] p-2.5 transition-colors hover:border-[#363a45]"
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="flex items-center gap-1.5">
                    <span
                      className={`font-mono text-[9px] font-bold px-1.5 py-0.5 rounded border ${
                        evt.currency === "USD"
                          ? "bg-[#2962ff]/15 border-[#2962ff]/30 text-[#2962ff]"
                          : evt.currency === "EUR"
                          ? "bg-[#089981]/15 border-[#089981]/30 text-[#089981]"
                          : evt.currency === "GBP"
                          ? "bg-[#ab47bc]/15 border-[#ab47bc]/30 text-[#ab47bc]"
                          : "bg-[#f5b942]/15 border-[#f5b942]/30 text-[#f5b942]"
                      }`}
                    >
                      {evt.currency}
                    </span>
                    <span
                      className={`h-2 w-2 rounded-full ${
                        isHigh ? "bg-[#f23645]" : isMedium ? "bg-[#f5b942]" : "bg-[#787b86]"
                      }`}
                      title={`${evt.impact.toUpperCase()} Impact`}
                    />
                    <span className="font-semibold text-white leading-snug">{evt.title}</span>
                  </div>

                  <div className="shrink-0 text-right font-mono text-[10px] text-[#787b86]">
                    <div>{evt.time}</div>
                    <div className="text-[9px] text-[#787b86]/80">{evt.date}</div>
                  </div>
                </div>

                <div className="mt-2.5 flex items-center justify-between border-t border-[#2a2e39]/60 pt-2 font-mono text-[10px]">
                  <div>
                    <span className="text-[#787b86]">Actual: </span>
                    <span
                      className={`font-bold ${
                        evt.actual ? "text-white" : "text-[#787b86]"
                      }`}
                    >
                      {evt.actual ?? "—"}
                    </span>
                  </div>
                  <div>
                    <span className="text-[#787b86]">Forecast: </span>
                    <span className="text-[#d1d4dc]">{evt.forecast ?? "—"}</span>
                  </div>
                  <div>
                    <span className="text-[#787b86]">Prev: </span>
                    <span className="text-[#787b86]">{evt.previous ?? "—"}</span>
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
