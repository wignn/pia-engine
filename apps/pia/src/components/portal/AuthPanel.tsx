"use client";

import { useState } from "react";
import { accountApi, type User } from "@/src/lib/api/account";

interface AuthPanelProps {
  onSuccess: (user: User, rawApiKey?: string) => void;
  onOAuth: (provider: "google" | "github") => void;
}

export function AuthPanel({ onSuccess, onOAuth }: AuthPanelProps) {
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (mode === "login") {
        const res = await accountApi.login(email.trim(), password);
        onSuccess(res.user);
      } else {
        if (!name.trim()) {
          throw new Error("Name is required");
        }
        if (password.length < 6) {
          throw new Error("Password must be at least 6 characters");
        }
        const res = await accountApi.register(email.trim(), name.trim(), password);
        onSuccess(res.user, res.api_key);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <article className="account-card account-wide" style={{ maxWidth: 640, margin: "0 auto" }}>
      <div className="account-card-head" style={{ marginBottom: 20 }}>
        <div>
          <span className="account-card-label">AUTHENTICATION</span>
          <h2>{mode === "login" ? "Sign in to PIA" : "Create developer account"}</h2>
          <p className="account-muted">
            {mode === "login"
              ? "Access your live market keys, quotas, and subscriptions."
              : "Register to get your instant free API key and 100 requests/day."}
          </p>
        </div>
        <div style={{ display: "flex", gap: 6 }}>
          <button
            type="button"
            className={`account-button ${mode === "login" ? "account-button-primary" : ""}`}
            onClick={() => {
              setMode("login");
              setError(null);
            }}
          >
            SIGN IN
          </button>
          <button
            type="button"
            className={`account-button ${mode === "register" ? "account-button-primary" : ""}`}
            onClick={() => {
              setMode("register");
              setError(null);
            }}
          >
            REGISTER
          </button>
        </div>
      </div>

      {error && (
        <div
          style={{
            padding: "10px 14px",
            marginBottom: 16,
            color: "#a34d4d",
            background: "rgba(163, 77, 77, 0.08)",
            border: "1px solid rgba(163, 77, 77, 0.25)",
            font: "11px var(--font-geist-mono), monospace",
          }}
        >
          ⚠ {error.toUpperCase()}
        </div>
      )}

      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        {mode === "register" && (
          <div>
            <label
              style={{
                display: "block",
                marginBottom: 6,
                font: "10px var(--font-geist-mono), monospace",
                color: "#7075a4",
              }}
            >
              FULL NAME
            </label>
            <input
              type="text"
              required
              placeholder="Satoshi Nakamoto"
              value={name}
              onChange={(e) => setName(e.target.value)}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "var(--blue)",
                background: "var(--cream)",
                border: "1px solid rgba(9,9,238,.25)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
              }}
            />
          </div>
        )}

        <div>
          <label
            style={{
              display: "block",
              marginBottom: 6,
              font: "10px var(--font-geist-mono), monospace",
              color: "#7075a4",
            }}
          >
            EMAIL ADDRESS
          </label>
          <input
            type="email"
            required
            placeholder="developer@atlsd.dev"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            style={{
              width: "100%",
              minHeight: 38,
              padding: "0 12px",
              color: "var(--blue)",
              background: "var(--cream)",
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
            PASSWORD (MIN 6 CHARS)
          </label>
          <input
            type="password"
            required
            placeholder="••••••••••••"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            style={{
              width: "100%",
              minHeight: 38,
              padding: "0 12px",
              color: "var(--blue)",
              background: "var(--cream)",
              border: "1px solid rgba(9,9,238,.25)",
              font: "12px var(--font-geist-mono), monospace",
              outline: "none",
            }}
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          className="account-button account-button-primary"
          style={{ width: "100%", marginTop: 8, height: 42 }}
        >
          {loading ? "PROCESSING..." : mode === "login" ? "SIGN IN TO PORTAL →" : "CREATE ACCOUNT & GET KEY →"}
        </button>
      </form>

      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 14,
          margin: "24px 0 16px",
          color: "#7075a4",
          font: "9px var(--font-geist-mono), monospace",
          letterSpacing: ".1em",
        }}
      >
        <div style={{ flex: 1, height: 1, background: "rgba(9,9,238,.15)" }} />
        <span>OR CONTINUE WITH</span>
        <div style={{ flex: 1, height: 1, background: "rgba(9,9,238,.15)" }} />
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
        <button
          type="button"
          className="account-button"
          onClick={() => onOAuth("google")}
          style={{ width: "100%" }}
        >
          GOOGLE OAUTH
        </button>
        <button
          type="button"
          className="account-button"
          onClick={() => onOAuth("github")}
          style={{ width: "100%" }}
        >
          GITHUB OAUTH
        </button>
      </div>
    </article>
  );
}
