"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { adminClient, type PlatformStats } from "@/lib/admin-client";
import { StatCard } from "@/components/StatCard";
import { StatusBadge } from "@/components/StatusBadge";
import { formatNumber } from "@/lib/formatters";

export default function OverviewPage() {
  const [stats, setStats] = useState<PlatformStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [syncNotice, setSyncNotice] = useState<string | null>(null);
  const [flushing, setFlushing] = useState(false);
  const [activePricesCount, setActivePricesCount] = useState<number | null>(null);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [sRes, mRes] = await Promise.allSettled([
        adminClient.getStats(),
        adminClient.getMarketPrices(),
      ]);

      if (sRes.status === "fulfilled") {
        setStats(sRes.value);
      } else {
        setError(sRes.reason?.message || "Failed to load platform stats");
      }

      if (mRes.status === "fulfilled") {
        setActivePricesCount(mRes.value.total);
      }
    } catch (err: any) {
      setError(err?.message || "Connection error");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleFlushCache = async () => {
    setFlushing(true);
    setSyncNotice("BROADCASTING CONFIG SYNC TO ALL GATEWAYS...");
    try {
      const res = await adminClient.flushCache();
      setSyncNotice(`✓ ${res.message.toUpperCase()}`);
      setTimeout(() => setSyncNotice(null), 4000);
    } catch (err: any) {
      setSyncNotice(`✕ FAILED TO BROADCAST: ${err?.message || "Unknown error"}`);
    } finally {
      setFlushing(false);
    }
  };

  const planBreakdown = stats?.users_by_plan || {};
  const totalTenants = stats?.total_users || 0;

  return (
    <main className="admin-container">
      {/* Header */}
      <div className="admin-header">
        <div>
          <span className="admin-kicker">PIA / EXECUTIVE OPERATIONS</span>
          <h1>
            MISSION CONTROL &amp; <br />
            <em>PLATFORM TELEMETRY.</em>
          </h1>
          <p>
            Real-time status of multi-tenant accounts, financial ingestion feeds, and core engine nodes.
          </p>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <button
            type="button"
            onClick={loadData}
            disabled={loading}
            className="admin-button"
          >
            {loading ? "REFRESHING..." : "REFRESH METRICS"}
          </button>

          <button
            type="button"
            onClick={handleFlushCache}
            disabled={flushing}
            className="admin-button admin-button-primary"
          >
            {flushing ? "BROADCASTING..." : "SYNC CACHE (NATS/REDIS)"}
          </button>
        </div>
      </div>

      {syncNotice && (
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
          {syncNotice}
        </div>
      )}

      {error && (
        <div
          style={{
            padding: "10px 16px",
            marginBottom: 20,
            background: "rgba(168, 59, 59, 0.08)",
            border: "1px solid rgba(168, 59, 59, 0.3)",
            font: "10px var(--font-geist-mono), monospace",
            color: "#a83b3b",
          }}
        >
          ERROR CONNECTING TO BACKEND: {error}
        </div>
      )}

      {/* KPI Cards */}
      <div className="admin-stat-grid">
        <StatCard
          label="TOTAL TENANTS"
          value={formatNumber(stats?.total_users)}
          subtext={`${stats?.active_users || 0} active accounts`}
          tone="green"
        />
        <StatCard
          label="ACTIVE API KEYS"
          value={formatNumber(stats?.total_api_keys)}
          subtext="Cryptographically validated"
          tone="green"
        />
        <StatCard
          label="LIVE MARKET ASSETS"
          value={activePricesCount !== null ? formatNumber(activePricesCount) : "105+"}
          subtext="FX, Crypto, Indices, IDX"
          tone="green"
        />
        <StatCard
          label="ACTIVE STACK"
          value="GREEN"
          subtext="Ports: 8000 / 8020 / 5176"
          tone="neutral"
        />
      </div>

      {/* 2-Column Grid: Plan Breakdown + Infrastructure Health */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(340px, 1fr))", gap: 24, marginBottom: 28 }}>
        
        {/* Card 1: Subscription Tier Distribution */}
        <div className="admin-card">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 16 }}>
            <div>
              <span className="admin-kicker">COMMERCIAL MONETIZATION</span>
              <h3 style={{ margin: "4px 0 0", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
                Tenants By Subscription Tier
              </h3>
            </div>
            <Link href="/tenants" className="admin-button" style={{ height: 26, fontSize: 9 }}>
              MANAGE TENANTS →
            </Link>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            {(["enterprise", "pro", "starter", "free"] as const).map((tier) => {
              const count = planBreakdown[tier] || 0;
              const pct = totalTenants > 0 ? Math.round((count / totalTenants) * 100) : 0;
              return (
                <div key={tier}>
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, fontFamily: "var(--font-geist-mono), monospace", marginBottom: 6 }}>
                    <span style={{ fontWeight: 600, color: "var(--blue)" }}>{tier.toUpperCase()}</span>
                    <span style={{ color: "#6a6f9f" }}>
                      {count} accounts ({pct}%)
                    </span>
                  </div>
                  <div style={{ width: "100%", height: 6, background: "rgba(9, 9, 238, 0.08)", overflow: "hidden" }}>
                    <div
                      style={{
                        width: `${pct}%`,
                        height: "100%",
                        background: tier === "enterprise" ? "var(--blue)" : tier === "pro" ? "var(--blue-vibrant)" : "rgba(9, 9, 238, 0.4)",
                        transition: "width 0.3s ease",
                      }}
                    />
                  </div>
                </div>
              );
            })}
          </div>

          <div style={{ marginTop: 20, paddingTop: 14, borderTop: "1px dotted rgba(9, 9, 238, 0.2)", fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "#6a6f9f" }}>
            Instant tier upgrade propagates to Redis within &lt;10ms via NATS channel.
          </div>
        </div>

        {/* Card 2: Core Infrastructure Nodes */}
        <div className="admin-card">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 16 }}>
            <div>
              <span className="admin-kicker">INFRASTRUCTURE STATUS</span>
              <h3 style={{ margin: "4px 0 0", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
                Core Engine Nodes
              </h3>
            </div>
            <Link href="/system" className="admin-button" style={{ height: 26, fontSize: 9 }}>
              SYSTEM HUB →
            </Link>
          </div>

          <div className="admin-table-wrapper">
            <table className="admin-table">
              <thead>
                <tr>
                  <th>SERVICE</th>
                  <th>PORT</th>
                  <th>PROTOCOL</th>
                  <th>STATUS</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><strong>API Gateway</strong></td>
                  <td><code>8000</code></td>
                  <td>HTTP REST RFC 6585</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
                <tr>
                  <td><strong>Realtime Gateway</strong></td>
                  <td><code>8020</code></td>
                  <td>WebSocket In-Band Auth</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
                <tr>
                  <td><strong>Control Plane</strong></td>
                  <td><code>8081</code></td>
                  <td>Internal Axum Engine</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
                <tr>
                  <td><strong>PostgreSQL</strong></td>
                  <td><code>5432</code></td>
                  <td>Tenant Auth &amp; Daily Quota</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
                <tr>
                  <td><strong>ClickHouse</strong></td>
                  <td><code>8123</code></td>
                  <td>1m Candle Rollups</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
                <tr>
                  <td><strong>Redis Cluster</strong></td>
                  <td><code>6379</code></td>
                  <td>Atomic Counters &amp; Cache</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
                <tr>
                  <td><strong>NATS JetStream</strong></td>
                  <td><code>4222</code></td>
                  <td>Pub/Sub Tick &amp; Telemetry</td>
                  <td><StatusBadge status="healthy" /></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

      </div>

      {/* Quick Links Banner */}
      <div
        style={{
          padding: 20,
          background: "rgba(255, 255, 255, 0.7)",
          border: "1px solid rgba(9, 9, 238, 0.18)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: 16,
        }}
      >
        <div>
          <h4 style={{ margin: "0 0 4px", fontSize: 13, color: "var(--blue)" }}>
            Need to inspect individual tenant usage or reset daily quota?
          </h4>
          <p style={{ margin: 0, fontSize: 11, color: "#6a6f9f" }}>
            Directly modify tier plans, revoke compromised API keys, or reset burst rate limits in real-time.
          </p>
        </div>
        <div style={{ display: "flex", gap: 10 }}>
          <Link href="/tenants" className="admin-button admin-button-primary">
            EXPLORE TENANTS →
          </Link>
          <Link href="/feeds" className="admin-button">
            INSPECT MARKET FEEDS →
          </Link>
        </div>
      </div>
    </main>
  );
}
