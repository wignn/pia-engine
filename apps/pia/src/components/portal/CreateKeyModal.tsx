"use client";

import { useState } from "react";

interface CreateKeyModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (label: string, permissions: string[]) => Promise<void>;
}

const AVAILABLE_SCOPES = [
  { id: "market:read", label: "Market Prices & OHLCV", desc: "Live ticks and historical candlestick queries" },
  { id: "realtime:ws", label: "WebSocket Streaming", desc: "Real-time subscriptions to market data & trades" },
  { id: "news:read", label: "Financial News & Filings", desc: "Aggregated news wire and sentiment signals" },
  { id: "macro:read", label: "Macro & Yield Data", desc: "US Treasury yield curves and economic events" },
  { id: "social:read", label: "Social Sentiment Pulse", desc: "Real-time X/Twitter financial curation" },
];

export function CreateKeyModal({ isOpen, onClose, onSubmit }: CreateKeyModalProps) {
  const [label, setLabel] = useState("");
  const [selectedScopes, setSelectedScopes] = useState<string[]>(["market:read", "realtime:ws"]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const toggleScope = (scopeId: string) => {
    setSelectedScopes((prev) =>
      prev.includes(scopeId) ? prev.filter((s) => s !== scopeId) : [...prev, scopeId]
    );
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!label.trim()) {
      setError("Label is required");
      return;
    }
    setError(null);
    setLoading(true);

    try {
      await onSubmit(label.trim(), selectedScopes);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create key");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 100,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "rgba(9, 9, 238, 0.2)",
        backdropFilter: "blur(4px)",
        padding: 20,
      }}
      onClick={onClose}
    >
      <div
        className="account-card"
        style={{
          width: "100%",
          maxWidth: 520,
          background: "var(--cream)",
          border: "1px solid var(--blue)",
          boxShadow: "0 20px 40px rgba(9, 9, 238, 0.15)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="account-card-head" style={{ marginBottom: 18 }}>
          <div>
            <span className="account-card-label">PROVISION ACCESS</span>
            <h2>Create New API Key</h2>
            <p className="account-muted">
              Configure key label and granted capabilities.
            </p>
          </div>
          <button type="button" className="account-button" onClick={onClose}>
            ✕
          </button>
        </div>

        {error && (
          <div
            style={{
              padding: "8px 12px",
              marginBottom: 14,
              color: "#a34d4d",
              background: "rgba(163, 77, 77, 0.08)",
              border: "1px solid rgba(163, 77, 77, 0.25)",
              font: "10px var(--font-geist-mono), monospace",
            }}
          >
            ⚠ {error.toUpperCase()}
          </div>
        )}

        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <div>
            <label
              style={{
                display: "block",
                marginBottom: 6,
                font: "10px var(--font-geist-mono), monospace",
                color: "#7075a4",
              }}
            >
              KEY LABEL
            </label>
            <input
              type="text"
              autoFocus
              required
              placeholder="e.g. Production Trading Terminal"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "var(--blue)",
                background: "rgba(255,255,255,0.7)",
                border: "1px solid rgba(9,9,238,.25)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
              }}
            />
          </div>

          <div>
            <label
              style={{
                display: "block",
                marginBottom: 6,
                font: "10px var(--font-geist-mono), monospace",
                color: "#7075a4",
              }}
            >
              PERMISSION SCOPES
            </label>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {AVAILABLE_SCOPES.map((sc) => {
                const checked = selectedScopes.includes(sc.id);
                return (
                  <label
                    key={sc.id}
                    style={{
                      display: "flex",
                      alignItems: "flex-start",
                      gap: 10,
                      padding: "8px 10px",
                      background: checked ? "rgba(9,9,238,0.06)" : "rgba(255,255,255,0.4)",
                      border: "1px solid rgba(9,9,238,0.15)",
                      cursor: "pointer",
                    }}
                  >
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={() => toggleScope(sc.id)}
                      style={{ marginTop: 2, accentColor: "var(--blue)" }}
                    />
                    <div style={{ flex: 1 }}>
                      <span style={{ font: "11px var(--font-geist-mono), monospace", color: "var(--blue)", fontWeight: 600 }}>
                        {sc.label}
                      </span>
                      <small style={{ display: "block", color: "#686d9d", fontSize: 10, marginTop: 2 }}>
                        {sc.desc}
                      </small>
                    </div>
                  </label>
                );
              })}
            </div>
          </div>

          <div style={{ display: "flex", justifyContent: "flex-end", gap: 10, marginTop: 10 }}>
            <button type="button" className="account-button" onClick={onClose} disabled={loading}>
              CANCEL
            </button>
            <button
              type="submit"
              className="account-button account-button-primary"
              disabled={loading || !label.trim()}
            >
              {loading ? "PROVISIONING..." : "PROVISION KEY →"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
