"use client";

import type { UsageSummary } from "@/src/lib/api/account";

interface UsageMeterProps {
  summary: UsageSummary | null;
  loading?: boolean;
}

export function UsageMeter({ summary, loading }: UsageMeterProps) {
  if (loading || !summary) {
    return (
      <article className="account-card account-profile">
        <span className="account-card-label">USAGE &amp; QUOTA</span>
        <h2>Telemetry</h2>
        <p className="account-muted">Loading live consumption telemetry...</p>
      </article>
    );
  }

  const { today, this_week, this_month, daily_limit, remaining_today } = summary;
  const pct = daily_limit > 0 ? Math.min(100, Math.round((today / daily_limit) * 100)) : 0;

  const barColor = pct >= 90 ? "#a34d4d" : pct >= 70 ? "#d97706" : "var(--blue)";

  return (
    <article className="account-card account-profile">
      <span className="account-card-label">USAGE &amp; QUOTA</span>
      <h2>Telemetry</h2>
      <p className="account-muted">Live consumption recorded across API Gateway &amp; WebSocket sessions.</p>

      <div style={{ marginTop: 20, marginBottom: 20 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 8 }}>
          <span style={{ font: "11px var(--font-geist-mono), monospace", color: "var(--blue)", fontWeight: 600 }}>
            DAILY API REQUESTS
          </span>
          <span style={{ font: "11px var(--font-geist-mono), monospace", color: "#565b94" }}>
            <strong style={{ color: "var(--blue)", fontSize: 13 }}>{today.toLocaleString()}</strong> / {daily_limit.toLocaleString()} ({pct}%)
          </span>
        </div>

        {/* Progress Bar */}
        <div
          style={{
            width: "100%",
            height: 8,
            background: "rgba(9, 9, 238, 0.08)",
            border: "1px solid rgba(9, 9, 238, 0.15)",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              width: `${Math.max(2, pct)}%`,
              height: "100%",
              background: barColor,
              transition: "width 0.5s ease",
            }}
          />
        </div>
      </div>

      {pct >= 100 ? (
        <div
          style={{
            padding: "6px 10px",
            marginBottom: 14,
            background: "rgba(163, 77, 77, 0.1)",
            border: "1px solid rgba(163, 77, 77, 0.3)",
            color: "#a34d4d",
            fontSize: 10,
            fontFamily: "var(--font-geist-mono), monospace",
          }}
        >
          ⚠ DAILY QUOTA EXHAUSTED (100%) — Requests are throttled with HTTP 429. Upgrade plan to continue.
        </div>
      ) : pct >= 80 ? (
        <div
          style={{
            padding: "6px 10px",
            marginBottom: 14,
            background: "rgba(217, 119, 6, 0.1)",
            border: "1px solid rgba(217, 119, 6, 0.3)",
            color: "#b45309",
            fontSize: 10,
            fontFamily: "var(--font-geist-mono), monospace",
          }}
        >
          ⚠ HIGH USAGE WARNING ({pct}%) — Approaching daily quota ceiling.
        </div>
      ) : null}

      <dl
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: 12,
          borderTop: "1px dotted rgba(9, 9, 238, 0.2)",
          paddingTop: 16,
          margin: 0,
        }}
      >
        <div>
          <dt style={{ color: "#7075a4", font: "8px var(--font-geist-mono), monospace" }}>REMAINING TODAY</dt>
          <dd style={{ margin: "6px 0 0", color: "var(--blue)", font: "13px var(--font-geist-mono), monospace", fontWeight: 600 }}>
            {remaining_today.toLocaleString()}
          </dd>
        </div>
        <div>
          <dt style={{ color: "#7075a4", font: "8px var(--font-geist-mono), monospace" }}>THIS WEEK</dt>
          <dd style={{ margin: "6px 0 0", color: "var(--blue)", font: "13px var(--font-geist-mono), monospace", fontWeight: 600 }}>
            {this_week.toLocaleString()}
          </dd>
        </div>
        <div>
          <dt style={{ color: "#7075a4", font: "8px var(--font-geist-mono), monospace" }}>THIS MONTH</dt>
          <dd style={{ margin: "6px 0 0", color: "var(--blue)", font: "13px var(--font-geist-mono), monospace", fontWeight: 600 }}>
            {this_month.toLocaleString()}
          </dd>
        </div>
      </dl>
    </article>
  );
}
