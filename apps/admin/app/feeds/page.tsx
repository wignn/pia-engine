"use client";

import { useEffect, useState } from "react";
import { adminClient, type MarketPriceItem } from "@/lib/admin-client";
import { StatusBadge } from "@/components/StatusBadge";
import { formatDate } from "@/lib/formatters";

export default function FeedsPage() {
  const [prices, setPrices] = useState<MarketPriceItem[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState("all");

  const loadFeeds = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await adminClient.getMarketPrices();
      setPrices(data.items || []);
      setTotal(data.total || 0);
    } catch (err: any) {
      setError(err?.message || "Failed to load market feeds");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadFeeds();
    const interval = setInterval(loadFeeds, 5000);
    return () => clearInterval(interval);
  }, []);

  const filteredPrices = prices.filter((p) => {
    const matchesSearch = p.symbol.toLowerCase().includes(search.toLowerCase());
    const matchesType = typeFilter === "all" || p.asset_type === typeFilter;
    return matchesSearch && matchesType;
  });

  const assetTypes = Array.from(new Set(prices.map((p) => p.asset_type)));

  return (
    <main className="admin-container">
      {/* Header */}
      <div className="admin-header">
        <div>
          <span className="admin-kicker">MARKET INGESTION ENGINE</span>
          <h1>
            FEED LIVENESS &amp; <br />
            <em>TICK STREAM AUDIT.</em>
          </h1>
          <p>
            Real-time feed health of 105+ instruments across Foreign Exchange, Cryptocurrencies, and IDX Equities.
          </p>
        </div>

        <div style={{ display: "flex", gap: 12 }}>
          <button
            type="button"
            onClick={loadFeeds}
            disabled={loading}
            className="admin-button"
          >
            {loading ? "POLLING..." : "REFRESH FEEDS"}
          </button>
        </div>
      </div>

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
          ERROR: {error}
        </div>
      )}

      {/* Ingestion Channels Overview */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 16, marginBottom: 24 }}>
        <div className="admin-card">
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
            <span className="admin-kicker">FEED A: FX ENGINE</span>
            <StatusBadge status="active" />
          </div>
          <div style={{ font: "400 20px var(--font-display)", color: "var(--blue)", marginBottom: 4 }}>
            Institutional Forex &amp; Metals
          </div>
          <p style={{ margin: 0, fontSize: 11, color: "#6a6f9f" }}>
            Single-session primary/secondary FX stream feeding Gold (XAUUSD) &amp; major currency pairs.
          </p>
        </div>

        <div className="admin-card">
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
            <span className="admin-kicker">FEED B: CRYPTO CLUSTER</span>
            <StatusBadge status="active" />
          </div>
          <div style={{ font: "400 20px var(--font-display)", color: "var(--blue)", marginBottom: 4 }}>
            24/7 Digital Asset Stream
          </div>
          <p style={{ margin: 0, fontSize: 11, color: "#6a6f9f" }}>
            High-frequency tick feeds for BTC, ETH, SOL directly routed into ClickHouse 1m rollups.
          </p>
        </div>

        <div className="admin-card">
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
            <span className="admin-kicker">FEED C: IDX INDONESIAN MARKET</span>
            <StatusBadge status="open" />
          </div>
          <div style={{ font: "400 20px var(--font-display)", color: "var(--blue)", marginBottom: 4 }}>
            IHSG &amp; Blue-Chip Equities
          </div>
          <p style={{ margin: 0, fontSize: 11, color: "#6a6f9f" }}>
            IDX:COMPOSITE, BBCA, BBRI, BMRI. Ticks persisted to ClickHouse during active market sessions.
          </p>
        </div>
      </div>

      {/* Search & Filters */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: 12,
          marginBottom: 16,
        }}
      >
        <div style={{ display: "flex", gap: 10, flexWrap: "wrap", alignItems: "center" }}>
          <input
            type="text"
            className="admin-input"
            placeholder="Search symbol (e.g. XAUUSD, BTC, BBCA)..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{ width: 300 }}
          />

          <select
            className="admin-select"
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value)}
          >
            <option value="all">ALL ASSET TYPES</option>
            {assetTypes.map((t) => (
              <option key={t} value={t}>
                {t.toUpperCase()}
              </option>
            ))}
          </select>
        </div>

        <div style={{ fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "#6a6f9f" }}>
          STREAMING {filteredPrices.length} OF {total} ASSETS
        </div>
      </div>

      {/* Ticker Table */}
      <div className="admin-card" style={{ padding: 0 }}>
        <div className="admin-table-wrapper">
          <table className="admin-table">
            <thead>
              <tr>
                <th>SYMBOL</th>
                <th>ASSET TYPE</th>
                <th>LATEST PRICE</th>
                <th>MARKET SESSION</th>
                <th>EXCHANGE</th>
                <th>LAST INGESTED TIMESTAMP</th>
                <th style={{ textAlign: "right" }}>HEALTH</th>
              </tr>
            </thead>
            <tbody>
              {filteredPrices.length === 0 ? (
                <tr>
                  <td colSpan={7} style={{ textAlign: "center", padding: 32, color: "#6a6f9f" }}>
                    {loading ? "Loading ticks..." : "No symbols matching filter."}
                  </td>
                </tr>
              ) : (
                filteredPrices.map((item) => (
                  <tr key={item.symbol}>
                    <td>
                      <strong style={{ color: "var(--blue)", fontSize: 12 }}>{item.symbol}</strong>
                    </td>
                    <td>
                      <span
                        style={{
                          padding: "2px 6px",
                          background: "rgba(9, 9, 238, 0.05)",
                          border: "1px solid rgba(9, 9, 238, 0.15)",
                          fontSize: 9,
                          fontWeight: 600,
                        }}
                      >
                        {item.asset_type.toUpperCase()}
                      </span>
                    </td>
                    <td style={{ fontSize: 12, fontWeight: 700, color: "var(--blue)" }}>
                      ${item.price?.toLocaleString("en-US", { minimumFractionDigits: 2 })}
                    </td>
                    <td>
                      <StatusBadge
                        status={item.session?.is_open ? "open" : "closed"}
                        label={item.session?.state?.toUpperCase() || "UNKNOWN"}
                      />
                    </td>
                    <td style={{ fontSize: 10, color: "#6a6f9f" }}>
                      {item.session?.exchange || "-"}
                    </td>
                    <td style={{ fontSize: 10, color: "#6a6f9f" }}>
                      {formatDate(item.received_at)}
                    </td>
                    <td style={{ textAlign: "right" }}>
                      <span style={{ color: "#2b7a4b", fontSize: 10, fontWeight: 600 }}>
                        ● LIVE STREAM
                      </span>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </main>
  );
}
