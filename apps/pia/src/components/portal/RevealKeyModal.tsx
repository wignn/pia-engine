"use client";

import { useState } from "react";

interface RevealKeyModalProps {
  apiKey: string | null;
  label?: string;
  onClose: () => void;
}

export function RevealKeyModal({ apiKey, label, onClose }: RevealKeyModalProps) {
  const [copied, setCopied] = useState(false);

  if (!apiKey) return null;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(apiKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2500);
    } catch {
      // fallback
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 110,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "rgba(9, 9, 238, 0.25)",
        backdropFilter: "blur(6px)",
        padding: 20,
      }}
    >
      <div
        className="account-card"
        style={{
          width: "100%",
          maxWidth: 560,
          background: "var(--cream)",
          border: "2px solid var(--blue)",
          boxShadow: "0 24px 48px rgba(9, 9, 238, 0.2)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <span className="account-card-label" style={{ color: "#a34d4d" }}>
          ⚠ SENSITIVE CREDENTIAL CREATED
        </span>
        <h2>Save Your Secret Key</h2>
        <p className="account-muted" style={{ marginBottom: 16 }}>
          {label ? `API Key "${label}" has been generated. ` : "Your API Key has been generated. "}
          For security, this secret token will <strong>never be shown again</strong>. Store it in an environment variable or secret manager.
        </p>

        <div
          style={{
            padding: 14,
            background: "rgba(255,255,255,0.85)",
            border: "1px solid rgba(9,9,238,0.2)",
            marginBottom: 16,
          }}
        >
          <label style={{ display: "block", color: "#7075a4", font: "9px var(--font-geist-mono), monospace", marginBottom: 6 }}>
            BEARER / X-API-KEY TOKEN
          </label>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <code
              style={{
                flex: 1,
                padding: "8px 10px",
                background: "rgba(9,9,238,0.06)",
                border: "1px dotted rgba(9,9,238,0.3)",
                color: "var(--blue)",
                font: "12px var(--font-geist-mono), monospace",
                fontWeight: 600,
                wordBreak: "break-all",
                userSelect: "all",
              }}
            >
              {apiKey}
            </code>
            <button
              type="button"
              className={`account-button ${copied ? "account-button-primary" : ""}`}
              onClick={handleCopy}
              style={{ minWidth: 80, height: 38 }}
            >
              {copied ? "COPIED! ✓" : "COPY"}
            </button>
          </div>
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end" }}>
          <button
            type="button"
            className="account-button account-button-primary"
            onClick={onClose}
            style={{ width: "100%", height: 40 }}
          >
            I HAVE SECURED THIS KEY →
          </button>
        </div>
      </div>
    </div>
  );
}
