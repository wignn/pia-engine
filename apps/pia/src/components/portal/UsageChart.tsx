"use client";

import type { DailyUsage } from "@/src/lib/api/account";

interface UsageChartProps {
  history: DailyUsage[];
  days?: number;
}

export function UsageChart({ history, days = 14 }: UsageChartProps) {
  // Ensure we display up to `days` entries
  const displayItems = history && history.length > 0 ? history.slice(-days) : [];

  const maxCount = Math.max(10, ...displayItems.map((d) => d.count));

  return (
    <article className="account-card account-wide">
      <div className="account-card-head" style={{ marginBottom: 16 }}>
        <div>
          <span className="account-card-label">HISTORICAL VOLUME</span>
          <h2>Daily Request Traffic</h2>
          <p className="account-muted">Trailing {days}-day API request volume breakdown.</p>
        </div>
        <span style={{ font: "10px var(--font-geist-mono), monospace", color: "#686d9d" }}>
          PEAK: {maxCount.toLocaleString()} REQ/DAY
        </span>
      </div>

      {displayItems.length === 0 ? (
        <div
          style={{
            height: 140,
            display: "grid",
            placeItems: "center",
            background: "rgba(9, 9, 238, 0.02)",
            border: "1px dotted rgba(9, 9, 238, 0.2)",
            color: "#686d9d",
            font: "11px var(--font-geist-mono), monospace",
          }}
        >
          No request activity recorded in this period yet.
        </div>
      ) : (
        <div style={{ width: "100%", overflowX: "auto" }}>
          <div style={{ minWidth: 400, height: 160, display: "flex", alignItems: "flex-end", gap: 8, paddingTop: 20 }}>
            {displayItems.map((item) => {
              const heightPct = Math.max(4, Math.round((item.count / maxCount) * 100));
              const shortDate = item.day ? item.day.slice(5) : ""; // MM-DD
              return (
                <div
                  key={item.day}
                  style={{
                    flex: 1,
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    gap: 6,
                    height: "100%",
                    justifyContent: "flex-end",
                  }}
                  title={`${item.day}: ${item.count} requests`}
                >
                  <span style={{ font: "9px var(--font-geist-mono), monospace", color: "#686d9d" }}>
                    {item.count > 0 ? item.count : ""}
                  </span>
                  <div
                    style={{
                      width: "100%",
                      maxWidth: 32,
                      height: `${heightPct}%`,
                      background: item.count > 0 ? "var(--blue)" : "rgba(9, 9, 238, 0.1)",
                      border: "1px solid rgba(9, 9, 238, 0.2)",
                      transition: "height 0.3s ease",
                    }}
                  />
                  <span
                    style={{
                      font: "9px var(--font-geist-mono), monospace",
                      color: "#7075a4",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {shortDate}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </article>
  );
}
