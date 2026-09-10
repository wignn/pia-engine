"use client";

import { useState } from "react";
import { accountApi, type User } from "@/src/lib/api/account";

interface AccountSettingsProps {
  user: User;
  onUserUpdate: (updated: User) => void;
  onLogout: () => void;
}

export function AccountSettings({ user, onUserUpdate, onLogout }: AccountSettingsProps) {
  // Name update state
  const [name, setName] = useState(user.name || "");
  const [nameLoading, setNameLoading] = useState(false);
  const [nameNotice, setNameNotice] = useState<{ text: string; type: "success" | "error" } | null>(null);

  // Password update state
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordLoading, setPasswordLoading] = useState(false);
  const [passwordNotice, setPasswordNotice] = useState<{ text: string; type: "success" | "error" } | null>(null);

  const handleUpdateName = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setNameLoading(true);
    setNameNotice(null);

    try {
      const res = await accountApi.updateProfile(name.trim());
      onUserUpdate(res.user);
      setNameNotice({ text: "PROFILE NAME UPDATED SUCCESSFULLY", type: "success" });
    } catch (err) {
      setNameNotice({
        text: err instanceof Error ? err.message.toUpperCase() : "FAILED TO UPDATE PROFILE",
        type: "error",
      });
    } finally {
      setNameLoading(false);
    }
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newPassword.length < 6) {
      setPasswordNotice({ text: "NEW PASSWORD MUST BE AT LEAST 6 CHARACTERS", type: "error" });
      return;
    }
    if (newPassword !== confirmPassword) {
      setPasswordNotice({ text: "NEW PASSWORDS DO NOT MATCH", type: "error" });
      return;
    }

    setPasswordLoading(true);
    setPasswordNotice(null);

    try {
      await accountApi.changePassword(newPassword, currentPassword || undefined);
      setPasswordNotice({ text: "PASSWORD CHANGED SUCCESSFULLY", type: "success" });
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (err) {
      setPasswordNotice({
        text: err instanceof Error ? err.message.toUpperCase() : "FAILED TO CHANGE PASSWORD",
        type: "error",
      });
    } finally {
      setPasswordLoading(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
      {/* 1. Developer Profile Card */}
      <article className="account-card account-wide">
        <div className="account-card-head" style={{ marginBottom: 16 }}>
          <div>
            <span className="account-card-label">DEVELOPER PROFILE</span>
            <h2>Account Details</h2>
            <p className="account-muted">Update your display name and view account metadata.</p>
          </div>
        </div>

        {nameNotice && (
          <div
            style={{
              padding: "8px 12px",
              marginBottom: 16,
              color: nameNotice.type === "success" ? "var(--blue)" : "#a34d4d",
              background: nameNotice.type === "success" ? "rgba(9,9,238,0.06)" : "rgba(163,77,77,0.08)",
              border: `1px solid ${nameNotice.type === "success" ? "rgba(9,9,238,0.2)" : "rgba(163,77,77,0.25)"}`,
              font: "11px var(--font-geist-mono), monospace",
            }}
          >
            {nameNotice.type === "success" ? "✓ " : "⚠ "}
            {nameNotice.text}
          </div>
        )}

        <form onSubmit={handleUpdateName} style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 480 }}>
          <div>
            <label style={{ display: "block", marginBottom: 6, font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
              EMAIL ADDRESS (READ-ONLY)
            </label>
            <input
              type="text"
              disabled
              value={user.email}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "#7075a4",
                background: "rgba(0,0,0,0.03)",
                border: "1px solid rgba(9,9,238,0.15)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
                cursor: "not-allowed",
              }}
            />
          </div>

          <div>
            <label style={{ display: "block", marginBottom: 6, font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
              DISPLAY NAME
            </label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "var(--blue)",
                background: "rgba(255,255,255,0.7)",
                border: "1px solid rgba(9,9,238,0.25)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
              }}
            />
          </div>

          <button
            type="submit"
            disabled={nameLoading || name.trim() === user.name}
            className="account-button account-button-primary"
            style={{ width: "fit-content", padding: "0 20px", height: 36, marginTop: 4 }}
          >
            {nameLoading ? "SAVING..." : "SAVE PROFILE →"}
          </button>
        </form>
      </article>

      {/* 2. Security & Password Card */}
      <article className="account-card account-wide">
        <div className="account-card-head" style={{ marginBottom: 16 }}>
          <div>
            <span className="account-card-label">AUTHENTICATION &amp; CREDENTIALS</span>
            <h2>Change Password</h2>
            <p className="account-muted">
              Update the password used to sign in to your PIA developer account.
            </p>
          </div>
        </div>

        {passwordNotice && (
          <div
            style={{
              padding: "8px 12px",
              marginBottom: 16,
              color: passwordNotice.type === "success" ? "var(--blue)" : "#a34d4d",
              background: passwordNotice.type === "success" ? "rgba(9,9,238,0.06)" : "rgba(163,77,77,0.08)",
              border: `1px solid ${passwordNotice.type === "success" ? "rgba(9,9,238,0.2)" : "rgba(163,77,77,0.25)"}`,
              font: "11px var(--font-geist-mono), monospace",
            }}
          >
            {passwordNotice.type === "success" ? "✓ " : "⚠ "}
            {passwordNotice.text}
          </div>
        )}

        <form onSubmit={handleChangePassword} style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 480 }}>
          <div>
            <label style={{ display: "block", marginBottom: 6, font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
              CURRENT PASSWORD
            </label>
            <input
              type="password"
              placeholder="••••••••••••"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "var(--blue)",
                background: "rgba(255,255,255,0.7)",
                border: "1px solid rgba(9,9,238,0.25)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
              }}
            />
          </div>

          <div>
            <label style={{ display: "block", marginBottom: 6, font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
              NEW PASSWORD (MIN 6 CHARACTERS)
            </label>
            <input
              type="password"
              required
              placeholder="••••••••••••"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "var(--blue)",
                background: "rgba(255,255,255,0.7)",
                border: "1px solid rgba(9,9,238,0.25)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
              }}
            />
          </div>

          <div>
            <label style={{ display: "block", marginBottom: 6, font: "10px var(--font-geist-mono), monospace", color: "#7075a4" }}>
              CONFIRM NEW PASSWORD
            </label>
            <input
              type="password"
              required
              placeholder="••••••••••••"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              style={{
                width: "100%",
                minHeight: 38,
                padding: "0 12px",
                color: "var(--blue)",
                background: "rgba(255,255,255,0.7)",
                border: "1px solid rgba(9,9,238,0.25)",
                font: "12px var(--font-geist-mono), monospace",
                outline: "none",
              }}
            />
          </div>

          <button
            type="submit"
            disabled={passwordLoading || !newPassword}
            className="account-button account-button-primary"
            style={{ width: "fit-content", padding: "0 20px", height: 36, marginTop: 4 }}
          >
            {passwordLoading ? "UPDATING..." : "UPDATE PASSWORD →"}
          </button>
        </form>
      </article>

      {/* 3. Session Management */}
      <article className="account-card account-wide">
        <div className="account-card-head">
          <div>
            <span className="account-card-label">SESSION ENVELOPE</span>
            <h2>Active Session</h2>
            <p className="account-muted">
              Terminate your active cookie session on this browser.
            </p>
          </div>
          <button type="button" className="account-button account-danger" onClick={onLogout} style={{ height: 36, padding: "0 16px" }}>
            SIGN OUT OF THIS DEVICE
          </button>
        </div>
      </article>
    </div>
  );
}
