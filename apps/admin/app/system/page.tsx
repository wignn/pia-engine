"use client";

import { useState } from "react";
import { adminClient } from "@/lib/admin-client";
import { StatusBadge } from "@/components/StatusBadge";

export default function SystemPage() {
  const [flushing, setFlushing] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const handleFlushCache = async () => {
    setFlushing(true);
    setNotice("BROADCASTING CONFIG SYNC TO REDIS & NATS...");
    try {
      const res = await adminClient.flushCache();
      setNotice(`✓ ${res.message.toUpperCase()}`);
      setTimeout(() => setNotice(null), 4000);
    } catch (err: any) {
      setNotice(`✕ ERROR: ${err?.message || "Failed"}`);
    } finally {
      setFlushing(false);
    }
  };

  return (
    <main className="admin-container">
      {/* Header */}
      <div className="admin-header">
        <div>
          <span className="admin-kicker">INFRASTRUCTURE &amp; OPERATIONS</span>
          <h1>
            SYSTEM HUBS &amp; <br />
            <em>DEPLOYMENT STATE.</em>
          </h1>
          <p>
            Audit blue-green deployment topology, trigger cache purges, inspect data retention, and execute emergency controls.
          </p>
        </div>

        <div style={{ display: "flex", gap: 12 }}>
          <button
            type="button"
            onClick={handleFlushCache}
            disabled={flushing}
            className="admin-button admin-button-primary"
          >
            {flushing ? "BROADCASTING..." : "⚡ BROADCAST CONFIG SYNC"}
          </button>
        </div>
      </div>

      {notice && (
        <div
          style={{
            padding: "10px 16px",
            marginBottom: 20,
            background: "rgba(9, 9, 238, 0.05)",
            border: "1px solid rgba(9, 9, 238, 0.2)",
            font: "10px var(--font-geist-mono), monospace",
            color: "var(--blue)",
          }}
        >
          {notice}
        </div>
      )}

      {/* Grid: Blue-Green Architecture & Emergency Controls */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(340px, 1fr))", gap: 24, marginBottom: 28 }}>
        
        {/* Card 1: Blue-Green Production Topology */}
        <div className="admin-card">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 16 }}>
            <div>
              <span className="admin-kicker">DEPLOYMENT TOPOLOGY</span>
              <h3 style={{ margin: "4px 0 0", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
                Zero-Downtime Blue-Green Stack
              </h3>
            </div>
            <StatusBadge status="active" label="GREEN ACTIVE" />
          </div>

          <div style={{ fontSize: 11, color: "#5d6090", lineHeight: 1.6, marginBottom: 16 }}>
            The production router routes incoming traffic from Cloudflare Tunnels to the active singleton stack.
          </div>

          <div className="admin-table-wrapper">
            <table className="admin-table">
              <thead>
                <tr>
                  <th>LAYER</th>
                  <th>PUBLIC ROUTER</th>
                  <th>GREEN (+1)</th>
                  <th>BLUE (+2)</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>REST API Gateway</td>
                  <td><code>8000</code></td>
                  <td><strong style={{ color: "var(--blue)" }}>8001 (ACTIVE)</strong></td>
                  <td><span style={{ color: "#8b90bd" }}>8002 (IDLE)</span></td>
                </tr>
                <tr>
                  <td>WebSocket Gateway</td>
                  <td><code>8020</code></td>
                  <td><strong style={{ color: "var(--blue)" }}>8021 (ACTIVE)</strong></td>
                  <td><span style={{ color: "#8b90bd" }}>8022 (IDLE)</span></td>
                </tr>
                <tr>
                  <td>Public Web (Svelte)</td>
                  <td><code>5173</code></td>
                  <td><strong style={{ color: "var(--blue)" }}>5274 (ACTIVE)</strong></td>
                  <td><span style={{ color: "#8b90bd" }}>5172 (IDLE)</span></td>
                </tr>
                <tr>
                  <td>Terminal App (Next)</td>
                  <td><code>5175</code></td>
                  <td><strong style={{ color: "var(--blue)" }}>5171 (ACTIVE)</strong></td>
                  <td><span style={{ color: "#8b90bd" }}>5170 (IDLE)</span></td>
                </tr>
                <tr>
                  <td>PIA Client Portal</td>
                  <td><code>5176</code></td>
                  <td><strong style={{ color: "var(--blue)" }}>5176 (SINGLETON)</strong></td>
                  <td>-</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        {/* Card 2: Disaster Recovery & Cloudflare R2 Backups */}
        <div className="admin-card">
          <span className="admin-kicker">DATA REPLICATION &amp; BACKUP</span>
          <h3 style={{ margin: "4px 0 12px", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
            Cloudflare R2 Snapshot Engine
          </h3>
          <p style={{ margin: "0 0 16px", fontSize: 11, color: "#5d6090", lineHeight: 1.6 }}>
            Automated daily snapshot backups protecting all multi-tenant PostgreSQL state and ClickHouse 1-minute candlestick rollup series.
          </p>

          <div style={{ display: "flex", flexDirection: "column", gap: 10, padding: 14, background: "rgba(9, 9, 238, 0.04)", border: "1px solid rgba(9, 9, 238, 0.15)", fontSize: 11, fontFamily: "var(--font-geist-mono), monospace" }}>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span style={{ color: "#6a6f9f" }}>R2 DESTINATION BUCKET:</span>
              <strong>atlsd-backup</strong>
            </div>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span style={{ color: "#6a6f9f" }}>POSTGRESQL TABLES:</span>
              <strong>users, api_keys, tenant_config, usage_logs</strong>
            </div>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span style={{ color: "#6a6f9f" }}>CLICKHOUSE ROLLUPS:</span>
              <strong>candles_1m_v2, price_ticks</strong>
            </div>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span style={{ color: "#6a6f9f" }}>SCHEDULE FREQUENCY:</span>
              <strong style={{ color: "#2b7a4b" }}>Daily 02:00 UTC (Cron)</strong>
            </div>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span style={{ color: "#6a6f9f" }}>BACKUP SCRIPT:</span>
              <code>scripts/backup-to-r2.sh</code>
            </div>
          </div>
        </div>

      </div>

      {/* RFC 6585 Gateway Telemetry Policy */}
      <div className="admin-card">
        <span className="admin-kicker">GATEWAY HARDENING</span>
        <h3 style={{ margin: "4px 0 12px", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
          RFC 6585 Rate Limiting &amp; Telemetry Enforcement
        </h3>
        <p style={{ margin: "0 0 16px", fontSize: 11, color: "#5d6090", lineHeight: 1.6 }}>
          All requests passing through the API Gateway are checked against Redis atomic buckets before hitting upstream microservices.
        </p>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))", gap: 14 }}>
          <div style={{ padding: 12, background: "rgba(255, 255, 255, 0.6)", border: "1px solid rgba(9, 9, 238, 0.15)" }}>
            <strong style={{ display: "block", fontSize: 11, color: "var(--blue)", marginBottom: 4 }}>
              1. BURST RATE LIMITER
            </strong>
            <span style={{ fontSize: 10, color: "#6a6f9f", lineHeight: 1.5 }}>
              Redis sliding window: <code>rate:min:&lt;user_id&gt;:&lt;minute&gt;</code>. Emits <code>X-RateLimit-Limit</code> &amp; <code>X-RateLimit-Remaining</code>.
            </span>
          </div>

          <div style={{ padding: 12, background: "rgba(255, 255, 255, 0.6)", border: "1px solid rgba(9, 9, 238, 0.15)" }}>
            <strong style={{ display: "block", fontSize: 11, color: "var(--blue)", marginBottom: 4 }}>
              2. DAILY ENVELOPE COUNTER
            </strong>
            <span style={{ fontSize: 10, color: "#6a6f9f", lineHeight: 1.5 }}>
              Atomic counter: <code>usage:daily:&lt;user_id&gt;:&lt;date&gt;</code>. Emits <code>X-DailyQuota-Limit</code> &amp; <code>X-DailyQuota-Remaining</code>.
            </span>
          </div>

          <div style={{ padding: 12, background: "rgba(255, 255, 255, 0.6)", border: "1px solid rgba(9, 9, 238, 0.15)" }}>
            <strong style={{ display: "block", fontSize: 11, color: "var(--blue)", marginBottom: 4 }}>
              3. IN-BAND WEBSOCKET AUTH
            </strong>
            <span style={{ fontSize: 10, color: "#6a6f9f", lineHeight: 1.5 }}>
              5-second handshake window at <code>/ws</code>. Prevents token leakage in query parameters and browser logs.
            </span>
          </div>
        </div>
      </div>
    </main>
  );
}
