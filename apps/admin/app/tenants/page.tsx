"use client";

import { useEffect, useState } from "react";
import {
  adminClient,
  type AdminUser,
  type AdminApiKey,
  type UserUsage,
} from "@/lib/admin-client";
import { StatusBadge } from "@/components/StatusBadge";
import { formatDate, formatNumber } from "@/lib/formatters";

export default function TenantsPage() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [planFilter, setPlanFilter] = useState("all");
  const [actionNotice, setActionNotice] = useState<string | null>(null);

  // Modal: Keys Inspector
  const [selectedUserForKeys, setSelectedUserForKeys] = useState<AdminUser | null>(null);
  const [userKeys, setUserKeys] = useState<AdminApiKey[]>([]);
  const [loadingKeys, setLoadingKeys] = useState(false);

  // Modal: Usage & Quota Inspector
  const [selectedUserForUsage, setSelectedUserForUsage] = useState<AdminUser | null>(null);
  const [usageData, setUsageData] = useState<UserUsage | null>(null);
  const [loadingUsage, setLoadingUsage] = useState(false);
  const [resettingQuota, setResettingQuota] = useState(false);

  const loadUsers = async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await adminClient.getUsers();
      setUsers(list);
    } catch (err: any) {
      setError(err?.message || "Failed to load tenants");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadUsers();
  }, []);

  const handlePlanChange = async (userId: string, newPlan: string) => {
    try {
      await adminClient.setUserPlan(userId, newPlan);
      setActionNotice(`✓ USER PLAN UPDATED TO ${newPlan.toUpperCase()}`);
      setTimeout(() => setActionNotice(null), 3000);
      setUsers((prev) =>
        prev.map((u) => (u.id === userId ? { ...u, plan: newPlan } : u))
      );
    } catch (err: any) {
      setActionNotice(`✕ FAILED TO UPDATE PLAN: ${err?.message || "Error"}`);
    }
  };

  const handleToggleUser = async (userId: string) => {
    try {
      const res = await adminClient.toggleUser(userId);
      setActionNotice(`✓ USER ${res.is_active ? "ACTIVATED" : "DEACTIVATED"}`);
      setTimeout(() => setActionNotice(null), 3000);
      setUsers((prev) =>
        prev.map((u) => (u.id === userId ? { ...u, is_active: res.is_active } : u))
      );
    } catch (err: any) {
      setActionNotice(`✕ ACTION FAILED: ${err?.message || "Error"}`);
    }
  };

  const openKeysModal = async (u: AdminUser) => {
    setSelectedUserForKeys(u);
    setLoadingKeys(true);
    try {
      const keys = await adminClient.getUserKeys(u.id);
      setUserKeys(keys);
    } catch {
      setUserKeys([]);
    } finally {
      setLoadingKeys(false);
    }
  };

  const openUsageModal = async (u: AdminUser) => {
    setSelectedUserForUsage(u);
    setLoadingUsage(true);
    try {
      const usage = await adminClient.getUserUsage(u.id);
      setUsageData(usage);
    } catch {
      setUsageData(null);
    } finally {
      setLoadingUsage(false);
    }
  };

  const handleResetQuota = async () => {
    if (!selectedUserForUsage) return;
    setResettingQuota(true);
    try {
      await adminClient.resetUserQuota(selectedUserForUsage.id);
      setActionNotice(`✓ DAILY QUOTA RESET TO 0 FOR ${selectedUserForUsage.email.toUpperCase()}`);
      setTimeout(() => setActionNotice(null), 4000);
      // Reload usage
      const refreshed = await adminClient.getUserUsage(selectedUserForUsage.id);
      setUsageData(refreshed);
    } catch (err: any) {
      setActionNotice(`✕ RESET FAILED: ${err?.message || "Error"}`);
    } finally {
      setResettingQuota(false);
    }
  };

  const filteredUsers = users.filter((u) => {
    const matchesSearch =
      u.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
      u.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      u.id.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesPlan = planFilter === "all" || u.plan === planFilter;
    return matchesSearch && matchesPlan;
  });

  return (
    <main className="admin-container">
      {/* Header */}
      <div className="admin-header">
        <div>
          <span className="admin-kicker">TENANT DIRECTORY</span>
          <h1>
            USER ACCOUNTS &amp; <br />
            <em>API KEY ACCESS.</em>
          </h1>
          <p>
            Oversee tenant accounts, manage entitlement tiers, inspect cryptographic keys, and enforce usage quotas.
          </p>
        </div>

        <div style={{ display: "flex", gap: 12 }}>
          <button
            type="button"
            onClick={loadUsers}
            disabled={loading}
            className="admin-button"
          >
            {loading ? "FETCHING..." : "RELOAD DIRECTORY"}
          </button>
        </div>
      </div>

      {actionNotice && (
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
          {actionNotice}
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
          ERROR: {error}
        </div>
      )}

      {/* Toolbar / Search */}
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
            placeholder="Search email, name, user ID..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{ width: 280 }}
          />

          <select
            className="admin-select"
            value={planFilter}
            onChange={(e) => setPlanFilter(e.target.value)}
          >
            <option value="all">ALL PLANS</option>
            <option value="enterprise">ENTERPRISE</option>
            <option value="pro">PRO</option>
            <option value="starter">STARTER</option>
            <option value="free">FREE</option>
          </select>
        </div>

        <div style={{ fontSize: 10, fontFamily: "var(--font-geist-mono), monospace", color: "#6a6f9f" }}>
          SHOWING {filteredUsers.length} OF {users.length} TENANTS
        </div>
      </div>

      {/* Table */}
      <div className="admin-card" style={{ padding: 0 }}>
        <div className="admin-table-wrapper">
          <table className="admin-table">
            <thead>
              <tr>
                <th>TENANT IDENTITY</th>
                <th>PLAN TIER</th>
                <th>KEYS</th>
                <th>QUOTA / USAGE</th>
                <th>STATUS</th>
                <th>JOINED</th>
                <th style={{ textAlign: "right" }}>ACTIONS</th>
              </tr>
            </thead>
            <tbody>
              {filteredUsers.length === 0 ? (
                <tr>
                  <td colSpan={7} style={{ textAlign: "center", padding: 32, color: "#6a6f9f" }}>
                    {loading ? "Loading tenants..." : "No matching tenants found."}
                  </td>
                </tr>
              ) : (
                filteredUsers.map((u) => (
                  <tr key={u.id}>
                    <td>
                      <div style={{ fontWeight: 600, color: "var(--blue)" }}>{u.name || "Nameless User"}</div>
                      <code style={{ fontSize: 10, color: "#5d6090" }}>{u.email}</code>
                      <div style={{ fontSize: 9, color: "#8b90bd" }}>ID: {u.id.substring(0, 13)}...</div>
                    </td>

                    <td>
                      <select
                        className="admin-select"
                        value={u.plan}
                        onChange={(e) => handlePlanChange(u.id, e.target.value)}
                        style={{ height: 28, fontSize: 10, fontWeight: 700 }}
                      >
                        <option value="free">FREE</option>
                        <option value="starter">STARTER</option>
                        <option value="pro">PRO</option>
                        <option value="enterprise">ENTERPRISE</option>
                      </select>
                    </td>

                    <td>
                      <button
                        type="button"
                        onClick={() => openKeysModal(u)}
                        className="admin-button"
                        style={{ height: 26, fontSize: 9, padding: "0 8px" }}
                      >
                        🔑 {u.active_keys} ACTIVE
                      </button>
                    </td>

                    <td>
                      <button
                        type="button"
                        onClick={() => openUsageModal(u)}
                        className="admin-button"
                        style={{ height: 26, fontSize: 9, padding: "0 8px" }}
                      >
                        📊 VIEW QUOTA
                      </button>
                    </td>

                    <td>
                      <StatusBadge status={u.is_active ? "active" : "suspended"} />
                    </td>

                    <td style={{ fontSize: 10, color: "#6a6f9f" }}>
                      {formatDate(u.created_at)}
                    </td>

                    <td style={{ textAlign: "right" }}>
                      <button
                        type="button"
                        onClick={() => handleToggleUser(u.id)}
                        className={`admin-button ${u.is_active ? "admin-button-danger" : "admin-button-primary"}`}
                        style={{ height: 26, fontSize: 9, padding: "0 10px" }}
                      >
                        {u.is_active ? "SUSPEND" : "ACTIVATE"}
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* MODAL 1: KEYS INSPECTOR */}
      {selectedUserForKeys && (
        <div className="admin-modal-backdrop" onClick={() => setSelectedUserForKeys(null)}>
          <div className="admin-modal" style={{ width: "min(100%, 720px)" }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
              <div>
                <span className="admin-kicker">KEY ENCLAVE INSPECTION</span>
                <h3 style={{ margin: "4px 0 0", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
                  {selectedUserForKeys.email}
                </h3>
              </div>
              <button
                type="button"
                onClick={() => setSelectedUserForKeys(null)}
                style={{ background: "none", border: 0, fontSize: 18, cursor: "pointer", color: "var(--blue)" }}
              >
                ✕
              </button>
            </div>

            {loadingKeys ? (
              <div style={{ padding: 24, textAlign: "center", color: "#6a6f9f" }}>Loading keys...</div>
            ) : userKeys.length === 0 ? (
              <div style={{ padding: 24, textAlign: "center", color: "#6a6f9f" }}>No API keys created yet.</div>
            ) : (
              <div className="admin-table-wrapper" style={{ maxHeight: 360, overflowY: "auto" }}>
                <table className="admin-table">
                  <thead>
                    <tr>
                      <th>KEY PREFIX</th>
                      <th>LABEL</th>
                      <th>STATUS</th>
                      <th>PERMISSIONS</th>
                      <th>LAST USED</th>
                    </tr>
                  </thead>
                  <tbody>
                    {userKeys.map((k) => (
                      <tr key={k.id}>
                        <td>
                          <code>{k.key_prefix}</code>
                        </td>
                        <td>{k.label || "default"}</td>
                        <td>
                          <StatusBadge status={k.is_active ? "active" : "revoked"} />
                        </td>
                        <td>
                          <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
                            {k.permissions?.map((p) => (
                              <span
                                key={p}
                                style={{
                                  padding: "2px 4px",
                                  background: "rgba(9, 9, 238, 0.05)",
                                  border: "1px solid rgba(9, 9, 238, 0.15)",
                                  fontSize: 8,
                                }}
                              >
                                {p}
                              </span>
                            )) || <span style={{ color: "#999" }}>None</span>}
                          </div>
                        </td>
                        <td style={{ fontSize: 10, color: "#6a6f9f" }}>
                          {formatDate(k.last_used_at)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            <div style={{ marginTop: 20, textAlign: "right" }}>
              <button
                type="button"
                onClick={() => setSelectedUserForKeys(null)}
                className="admin-button"
              >
                CLOSE
              </button>
            </div>
          </div>
        </div>
      )}

      {/* MODAL 2: USAGE & QUOTA INSPECTOR */}
      {selectedUserForUsage && (
        <div className="admin-modal-backdrop" onClick={() => setSelectedUserForUsage(null)}>
          <div className="admin-modal" style={{ width: "min(100%, 540px)" }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
              <div>
                <span className="admin-kicker">USAGE TELEMETRY &amp; QUOTA</span>
                <h3 style={{ margin: "4px 0 0", font: "400 24px var(--font-display)", color: "var(--blue)" }}>
                  {selectedUserForUsage.email}
                </h3>
              </div>
              <button
                type="button"
                onClick={() => setSelectedUserForUsage(null)}
                style={{ background: "none", border: 0, fontSize: 18, cursor: "pointer", color: "var(--blue)" }}
              >
                ✕
              </button>
            </div>

            {loadingUsage ? (
              <div style={{ padding: 24, textAlign: "center", color: "#6a6f9f" }}>Fetching usage metrics...</div>
            ) : usageData ? (
              <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
                <div style={{ padding: 14, background: "rgba(9, 9, 238, 0.04)", border: "1px solid rgba(9, 9, 238, 0.15)" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8, fontSize: 11, fontFamily: "var(--font-geist-mono), monospace" }}>
                    <span>PLAN ENVELOPE:</span>
                    <strong>{usageData.plan.toUpperCase()}</strong>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8, fontSize: 11, fontFamily: "var(--font-geist-mono), monospace" }}>
                    <span>TODAY REQUESTS (REDIS):</span>
                    <strong>{formatNumber(usageData.today)} / {formatNumber(usageData.daily_limit)}</strong>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8, fontSize: 11, fontFamily: "var(--font-geist-mono), monospace" }}>
                    <span>REMAINING TODAY:</span>
                    <strong style={{ color: "#2b7a4b" }}>{formatNumber(usageData.remaining_today)}</strong>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8, fontSize: 11, fontFamily: "var(--font-geist-mono), monospace" }}>
                    <span>7-DAY ROLLING VOLUME:</span>
                    <strong>{formatNumber(usageData.this_week)}</strong>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, fontFamily: "var(--font-geist-mono), monospace" }}>
                    <span>30-DAY ROLLING VOLUME:</span>
                    <strong>{formatNumber(usageData.this_month)}</strong>
                  </div>
                </div>

                <div style={{ padding: 12, background: "rgba(168, 59, 59, 0.05)", border: "1px solid rgba(168, 59, 59, 0.2)" }}>
                  <div style={{ fontSize: 11, fontWeight: 700, color: "#a83b3b", marginBottom: 4 }}>
                    EMERGENCY QUOTA OVERRIDE
                  </div>
                  <div style={{ fontSize: 10, color: "#6a6f9f", lineHeight: 1.5, marginBottom: 12 }}>
                    If this user hits their daily limit and is blocked with HTTP 429, you can instantly clear their Redis counter.
                  </div>
                  <button
                    type="button"
                    onClick={handleResetQuota}
                    disabled={resettingQuota}
                    className="admin-button admin-button-danger"
                    style={{ width: "100%" }}
                  >
                    {resettingQuota ? "RESETTING IN REDIS..." : "⚡ RESET TODAY'S QUOTA TO ZERO"}
                  </button>
                </div>
              </div>
            ) : (
              <div style={{ padding: 24, textAlign: "center", color: "#6a6f9f" }}>Failed to retrieve usage info.</div>
            )}

            <div style={{ marginTop: 20, textAlign: "right" }}>
              <button
                type="button"
                onClick={() => setSelectedUserForUsage(null)}
                className="admin-button"
              >
                CLOSE
              </button>
            </div>
          </div>
        </div>
      )}
    </main>
  );
}
