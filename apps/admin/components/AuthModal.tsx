"use client";

import { useState } from "react";

interface AuthModalProps {
  currentKey: string;
  onClose: () => void;
  onSave: (key: string) => void;
}

export function AuthModal({ currentKey, onClose, onSave }: AuthModalProps) {
  const [keyInput, setKeyInput] = useState(currentKey);

  return (
    <div className="admin-modal-backdrop">
      <div className="admin-modal">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
          <span className="admin-kicker">AUTHENTICATION SECURITY</span>
          <button
            type="button"
            onClick={onClose}
            style={{ background: "none", border: 0, fontSize: 16, cursor: "pointer", color: "var(--blue)" }}
          >
            ✕
          </button>
        </div>

        <h2 style={{ margin: "0 0 8px", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
          Admin Master API Key
        </h2>
        <p style={{ margin: "0 0 20px", fontSize: 12, color: "#5d6090", lineHeight: 1.5 }}>
          This key is injected into requests to access the control plane backend. Default is <code>silvia</code>.
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 8, marginBottom: 20 }}>
          <label style={{ fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "#6a6f9f" }}>
            MASTER KEY (x-admin-key):
          </label>
          <input
            type="password"
            className="admin-input"
            value={keyInput}
            onChange={(e) => setKeyInput(e.target.value)}
            placeholder="Enter ADMIN_API_KEY..."
            style={{ width: "100%" }}
          />
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 10 }}>
          <button type="button" onClick={onClose} className="admin-button">
            CANCEL
          </button>
          <button
            type="button"
            onClick={() => onSave(keyInput.trim())}
            className="admin-button admin-button-primary"
          >
            SAVE &amp; APPLY
          </button>
        </div>
      </div>
    </div>
  );
}
