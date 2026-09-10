"use client";

import { useEffect, useState, useCallback } from "react";
import {
  accountApi,
  type KeyInfo,
  type Plan,
  type User,
  type UsageSummary,
  type DailyUsage,
} from "@/src/lib/api/account";
import { AuthPanel } from "@/src/components/portal/AuthPanel";
import { CreateKeyModal } from "@/src/components/portal/CreateKeyModal";
import { RevealKeyModal } from "@/src/components/portal/RevealKeyModal";
import { UsageMeter } from "@/src/components/portal/UsageMeter";
import { UsageChart } from "@/src/components/portal/UsageChart";
import { AccountSettings } from "@/src/components/portal/AccountSettings";

const fallbackPlans: Plan[] = [
  { id: "free", name: "Free", price_idr: 0, requests_per_day: 100, ws_connections: 1, news_history_days: 1, rate_limit_per_min: 10 },
  { id: "starter", name: "Starter", price_idr: 149000, requests_per_day: 5000, ws_connections: 3, news_history_days: 7, rate_limit_per_min: 60 },
  { id: "pro", name: "Pro", price_idr: 499000, requests_per_day: 50000, ws_connections: 10, news_history_days: 30, rate_limit_per_min: 300 },
  { id: "enterprise", name: "Enterprise", price_idr: 0, requests_per_day: 999999, ws_connections: 100, news_history_days: 365, rate_limit_per_min: 1000 },
];

const formatDate = (value: string | null | undefined) =>
  value
    ? new Date(value).toLocaleDateString("en-GB", { day: "2-digit", month: "short", year: "numeric" }).toUpperCase()
    : "Never";

const formatPrice = (value: number, id: string) =>
  id === "enterprise"
    ? "Custom Contact"
    : value === 0
    ? "IDR 0 / MO"
    : `IDR ${(value / 1000).toLocaleString("en-US")}K / MO`;

export default function AccountPage() {
  const [user, setUser] = useState<User | null>(null);
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [plans, setPlans] = useState<Plan[]>(fallbackPlans);
  const [usageSummary, setUsageSummary] = useState<UsageSummary | null>(null);
  const [usageHistory, setUsageHistory] = useState<DailyUsage[]>([]);
  const [activeTab, setActiveTab] = useState<"overview" | "keys" | "plans" | "settings">("overview");
  const [notice, setNotice] = useState("LOADING PIA ACCOUNT...");
  const [loading, setLoading] = useState(true);

  // Modals state
  const [isCreateKeyOpen, setIsCreateKeyOpen] = useState(false);
  const [revealKeyData, setRevealKeyData] = useState<{ key: string; label?: string } | null>(null);
  const [editingKeyId, setEditingKeyId] = useState<string | null>(null);
  const [editLabelInput, setEditLabelInput] = useState("");
  const [upgradingPlanId, setUpgradingPlanId] = useState<string | null>(null);

  const [refreshIndex, setRefreshIndex] = useState(0);

  useEffect(() => {
    let ignore = false;

    async function fetchData() {
      try {
        const [meRes, keyList, planList] = await Promise.allSettled([
          accountApi.me(),
          accountApi.keys(),
          accountApi.plans(),
        ]);

        if (ignore) return;

        if (meRes.status === "fulfilled") {
          const u = meRes.value.user;
          setUser(u);
          setNotice(`SESSION ACTIVE · PLAN: ${u.plan.toUpperCase()}`);

          const [uRes, hRes] = await Promise.allSettled([accountApi.usage(), accountApi.usageHistory(14)]);
          if (ignore) return;
          if (uRes.status === "fulfilled") setUsageSummary(uRes.value);
          if (hRes.status === "fulfilled") setUsageHistory(hRes.value);
        } else {
          setUser(null);
          setNotice("SIGN IN TO MANAGE YOUR PIA DEVELOPER ACCOUNT");
        }

        if (keyList.status === "fulfilled") setKeys(keyList.value);
        if (planList.status === "fulfilled" && planList.value.length) setPlans(planList.value);
      } catch {
        if (!ignore) setNotice("ERROR CONNECTING TO CONTROL PLANE");
      } finally {
        if (!ignore) setLoading(false);
      }
    }

    fetchData();
    return () => {
      ignore = true;
    };
  }, [refreshIndex]);

  const refreshDashboard = useCallback(() => {
    setRefreshIndex((i) => i + 1);
  }, []);

  const handleAuthSuccess = (authenticatedUser: User, rawApiKey?: string) => {
    setUser(authenticatedUser);
    setNotice(`WELCOME BACK · ${authenticatedUser.name.toUpperCase()}`);
    refreshDashboard();

    if (rawApiKey) {
      setRevealKeyData({ key: rawApiKey, label: "Default Key" });
    }
  };

  const loginOAuth = async (provider: "google" | "github") => {
    try {
      setNotice(`CONNECTING TO ${provider.toUpperCase()}...`);
      const result = await accountApi.oauthUrl(provider);
      if (!result.url) throw new Error(result.error || "OAuth provider is not configured");
      window.location.assign(result.url);
    } catch (error) {
      setNotice(error instanceof Error ? error.message.toUpperCase() : "OAUTH LOGIN FAILED");
    }
  };

  const handleLogout = async () => {
    try {
      await accountApi.logout();
      setUser(null);
      setKeys([]);
      setUsageSummary(null);
      setUsageHistory([]);
      setNotice("LOGGED OUT OF PIA");
      setActiveTab("overview");
    } catch {
      setUser(null);
    }
  };

  const handleCreateKey = async (label: string, permissions: string[]) => {
    const result = await accountApi.createKey(label, permissions);
    setKeys((current) => [result.key_info, ...current]);
    setRevealKeyData({ key: result.api_key, label: result.key_info.label });
    setNotice(`KEY PROVISIONED · ${result.key_info.label.toUpperCase()}`);
    setActiveTab("keys");
  };

  const handleStartEditKey = (key: KeyInfo) => {
    setEditingKeyId(key.id);
    setEditLabelInput(key.label);
  };

  const handleSaveEditKey = async (id: string) => {
    if (!editLabelInput.trim()) return;
    try {
      await accountApi.updateKey(id, editLabelInput.trim());
      setKeys((current) =>
        current.map((k) => (k.id === id ? { ...k, label: editLabelInput.trim() } : k))
      );
      setEditingKeyId(null);
      setNotice("KEY LABEL UPDATED");
    } catch {
      setNotice("FAILED TO UPDATE KEY");
    }
  };

  const revokeKey = async (id: string) => {
    if (!window.confirm("Revoke this API key? Applications using this key will immediately lose access.")) return;
    try {
      await accountApi.revokeKey(id);
      setKeys((current) => current.filter((key) => key.id !== id));
      setNotice("API KEY REVOKED");
    } catch {
      setNotice("KEY REVOCATION FAILED");
    }
  };

  const handleChoosePlan = async (planId: string) => {
    if (planId === activePlan.id) return;
    const target = plans.find((p) => p.id === planId);
    if (!target) return;

    if (!window.confirm(`Switch to the ${target.name} plan? This will update your rate limits and quotas.`)) {
      return;
    }

    setUpgradingPlanId(planId);
    try {
      const res = await accountApi.upgradePlan(planId);
      if (res.error) throw new Error(res.error);
      setNotice(`PLAN UPGRADED TO ${target.name.toUpperCase()} · LIMITS UPDATED`);
      refreshDashboard();
    } catch (err) {
      setNotice(err instanceof Error ? err.message.toUpperCase() : "FAILED TO UPGRADE PLAN");
    } finally {
      setUpgradingPlanId(null);
    }
  };

  const activePlan =
    plans.find((plan) => plan.id === user?.plan) ||
    plans.find((plan) => plan.id === "free") ||
    fallbackPlans[0];

  return (
    <main className="account-page">
      <header className="account-header">
        <div>
          <span className="account-kicker">PIA / DEVELOPER ACCESS</span>
          <h1>
            YOUR MARKET<br />
            <em>WORKSPACE.</em>
          </h1>
          <p>Manage your real-time API keys, subscriptions, and consumption telemetry.</p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <a
            className="account-button account-button-primary"
            href="/portal/docs"
            style={{ padding: "0 14px", height: 34, textDecoration: "none", display: "inline-flex", alignItems: "center" }}
          >
            API REFERENCE →
          </a>
          {user && (
            <button
              onClick={handleLogout}
              className="account-button account-danger"
              style={{ padding: "0 14px", height: 34 }}
            >
              LOGOUT
            </button>
          )}
          <a className="account-back" href="/portal">
            ← PLATFORM
          </a>
        </div>
      </header>

      <div className="account-notice">{loading ? "LOADING ACCOUNT..." : notice}</div>

      {/* When not authenticated, present the Unified Auth Panel */}
      {!user && !loading ? (
        <AuthPanel onSuccess={handleAuthSuccess} onOAuth={loginOAuth} />
      ) : (
        <>
          <nav className="account-tabs" aria-label="Account sections">
            {(["overview", "keys", "plans", "settings"] as const).map((tab) => (
              <button
                className={activeTab === tab ? "is-active" : ""}
                onClick={() => setActiveTab(tab)}
                key={tab}
              >
                {tab.toUpperCase()}
              </button>
            ))}
          </nav>

          {/* TAB 1: OVERVIEW */}
          {activeTab === "overview" && (
            <section className="account-grid">
              {/* Profile Card */}
              <article className="account-card account-profile">
                <span className="account-card-label">DEVELOPER IDENTITY</span>
                <h2>{user?.name || "Developer"}</h2>
                <p className="account-muted">Your private developer workspace is active.</p>
                <div className="account-identity">
                  <span className="account-avatar">
                    {(user?.name ? user.name.slice(0, 2).toUpperCase() : "PI")}
                  </span>
                  <span>
                    <strong>{user?.name || "Developer Account"}</strong>
                    <small>{user?.email || ""}</small>
                  </span>
                </div>
              </article>

              {/* Live Quota Telemetry */}
              <UsageMeter summary={usageSummary} loading={loading} />

              {/* Active Plan Card */}
              <article className="account-card account-plan-card account-wide">
                <div className="account-card-head">
                  <div>
                    <span className="account-card-label">SUBSCRIPTION ENVELOPE</span>
                    <h2>{activePlan.name} Tier</h2>
                    <p className="account-muted">
                      Cost: <strong>{formatPrice(activePlan.price_idr, activePlan.id)}</strong>
                    </p>
                  </div>
                  <button className="account-button" onClick={() => setActiveTab("plans")}>
                    UPGRADE PLAN →
                  </button>
                </div>

                <dl>
                  <div>
                    <dt>DAILY REQUEST LIMIT</dt>
                    <dd>
                      {activePlan.requests_per_day
                        ? activePlan.requests_per_day.toLocaleString()
                        : "Custom Unlimited"}
                    </dd>
                  </div>
                  <div>
                    <dt>WS CONCURRENCY</dt>
                    <dd>{activePlan.ws_connections} Connection(s)</dd>
                  </div>
                  <div>
                    <dt>BURST RATE LIMIT</dt>
                    <dd>{activePlan.rate_limit_per_min} Req / Min</dd>
                  </div>
                  <div>
                    <dt>NEWS HISTORY</dt>
                    <dd>{activePlan.news_history_days} Days</dd>
                  </div>
                </dl>
              </article>

              {/* 14-Day Usage Volume Chart */}
              <UsageChart history={usageHistory} days={14} />

              {/* Key Strip Summary */}
              <article className="account-card account-wide">
                <div className="account-card-head">
                  <div>
                    <span className="account-card-label">AUTHENTICATION KEYS</span>
                    <h2>{keys.length} Active Key(s)</h2>
                  </div>
                  <button className="account-button" onClick={() => setActiveTab("keys")}>
                    MANAGE KEYS →
                  </button>
                </div>
                <p className="account-muted">
                  Use your credentials with header <code>x-api-key</code> or query <code>?api_key=</code>.
                </p>
                <div className="account-key-strip">
                  {keys.slice(0, 3).map((key) => (
                    <span key={key.id}>
                      <b>{key.key_prefix}...</b>
                      <small>{key.label}</small>
                    </span>
                  ))}
                  {keys.length === 0 && (
                    <span style={{ gridColumn: "1 / -1", textAlign: "center" }}>
                      No keys provisioned yet. Click &quot;Manage Keys&quot; to create one.
                    </span>
                  )}
                </div>
              </article>
            </section>
          )}

          {/* TAB 2: KEYS */}
          {activeTab === "keys" && (
            <section className="account-section">
              <div className="account-section-head">
                <div>
                  <span className="account-card-label">DEVELOPER CREDENTIALS</span>
                  <h2>API KEYS</h2>
                  <p className="account-muted">
                    Provision, label, and revoke access keys for backend systems, trading algorithms, and scripts.
                  </p>
                </div>
                <button
                  className="account-button account-button-primary"
                  onClick={() => setIsCreateKeyOpen(true)}
                >
                  CREATE NEW KEY +
                </button>
              </div>

              <div className="account-key-list">
                {keys.map((key) => (
                  <article className="account-card account-key-card" key={key.id}>
                    <div>
                      <span className="account-status">
                        {key.is_active ? "● ACTIVE" : "○ REVOKED"}
                      </span>

                      {editingKeyId === key.id ? (
                        <div style={{ display: "flex", gap: 6, marginTop: 6, marginBottom: 6 }}>
                          <input
                            type="text"
                            value={editLabelInput}
                            onChange={(e) => setEditLabelInput(e.target.value)}
                            style={{
                              padding: "4px 8px",
                              font: "12px var(--font-geist-mono), monospace",
                              border: "1px solid var(--blue)",
                              background: "#fff",
                              color: "var(--blue)",
                            }}
                          />
                          <button
                            className="account-button account-button-primary"
                            onClick={() => handleSaveEditKey(key.id)}
                            style={{ minHeight: 28, padding: "0 8px" }}
                          >
                            SAVE
                          </button>
                          <button
                            className="account-button"
                            onClick={() => setEditingKeyId(null)}
                            style={{ minHeight: 28, padding: "0 8px" }}
                          >
                            CANCEL
                          </button>
                        </div>
                      ) : (
                        <div style={{ display: "flex", alignItems: "baseline", gap: 10 }}>
                          <h3 style={{ margin: "10px 0 6px" }}>{key.label}</h3>
                          <button
                            onClick={() => handleStartEditKey(key)}
                            style={{
                              background: "none",
                              border: "none",
                              color: "#7075a4",
                              fontSize: 10,
                              fontFamily: "var(--font-geist-mono), monospace",
                              cursor: "pointer",
                              textDecoration: "underline",
                            }}
                          >
                            Rename
                          </button>
                        </div>
                      )}

                      <code>{key.key_prefix}••••••••••••••••••••••••</code>

                      {Array.isArray(key.permissions) && key.permissions.length > 0 && (
                        <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginTop: 10 }}>
                          {key.permissions.map((perm) => (
                            <span
                              key={perm}
                              style={{
                                padding: "2px 6px",
                                background: "rgba(9, 9, 238, 0.06)",
                                border: "1px solid rgba(9, 9, 238, 0.15)",
                                fontSize: 9,
                                fontFamily: "var(--font-geist-mono), monospace",
                                color: "var(--blue)",
                              }}
                            >
                              {perm}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>

                    <dl>
                      <div>
                        <dt>PROVISIONED</dt>
                        <dd>{formatDate(key.created_at)}</dd>
                      </div>
                      <div>
                        <dt>LAST ACCESSED</dt>
                        <dd>{key.last_used_at ? formatDate(key.last_used_at) : "Never"}</dd>
                      </div>
                      <div>
                        <dt>WS LIMIT</dt>
                        <dd>{key.max_ws_connections ? `${key.max_ws_connections} WS` : "Plan Default"}</dd>
                      </div>
                    </dl>

                    <div>
                      <button className="account-danger" onClick={() => revokeKey(key.id)}>
                        REVOKE KEY
                      </button>
                    </div>
                  </article>
                ))}

                {keys.length === 0 && (
                  <div
                    style={{
                      padding: 36,
                      textAlign: "center",
                      background: "rgba(255,255,255,0.4)",
                      border: "1px dashed rgba(9,9,238,0.2)",
                    }}
                  >
                    <p className="account-muted">You have no active API keys.</p>
                    <button
                      className="account-button account-button-primary"
                      onClick={() => setIsCreateKeyOpen(true)}
                      style={{ marginTop: 14 }}
                    >
                      PROVISION FIRST KEY →
                    </button>
                  </div>
                )}
              </div>
            </section>
          )}

          {/* TAB 3: PLANS */}
          {activeTab === "plans" && (
            <section className="account-section">
              <div className="account-section-head">
                <div>
                  <span className="account-card-label">TIERS &amp; CAPABILITIES</span>
                  <h2>SUBSCRIPTION PLANS</h2>
                  <p className="account-muted">
                    Choose the data access envelope and throughput ceiling tailored to your workflow.
                  </p>
                </div>
              </div>

              <div className="account-plans">
                {plans.map((plan) => {
                  const isCurrent = plan.id === activePlan.id;
                  const isProcessing = upgradingPlanId === plan.id;

                  return (
                    <article
                      className={`account-card account-plan-option${isCurrent ? " is-current" : ""}`}
                      key={plan.id}
                    >
                      <span className="account-card-label">
                        {isCurrent ? "ACTIVE PLAN" : plan.id.toUpperCase()}
                      </span>
                      <h3>{plan.name}</h3>
                      <strong>{formatPrice(plan.price_idr, plan.id)}</strong>

                      <p className="account-muted">
                        {plan.requests_per_day
                          ? `${plan.requests_per_day.toLocaleString()} req/day · ${plan.ws_connections} WS · ${plan.rate_limit_per_min} req/min`
                          : "High-volume institutional throughput"}
                      </p>

                      <dl style={{ margin: "16px 0", width: "100%", fontSize: 11 }}>
                        <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 0" }}>
                          <dt style={{ color: isCurrent ? "rgba(255,255,255,0.7)" : "#7075a4" }}>Daily Requests</dt>
                          <dd style={{ margin: 0, fontWeight: 600 }}>{plan.requests_per_day ? plan.requests_per_day.toLocaleString() : "Custom"}</dd>
                        </div>
                        <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 0" }}>
                          <dt style={{ color: isCurrent ? "rgba(255,255,255,0.7)" : "#7075a4" }}>WS Connections</dt>
                          <dd style={{ margin: 0, fontWeight: 600 }}>{plan.ws_connections}</dd>
                        </div>
                        <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 0" }}>
                          <dt style={{ color: isCurrent ? "rgba(255,255,255,0.7)" : "#7075a4" }}>News History</dt>
                          <dd style={{ margin: 0, fontWeight: 600 }}>{plan.news_history_days} Days</dd>
                        </div>
                        <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 0" }}>
                          <dt style={{ color: isCurrent ? "rgba(255,255,255,0.7)" : "#7075a4" }}>Rate Limit</dt>
                          <dd style={{ margin: 0, fontWeight: 600 }}>{plan.rate_limit_per_min} / min</dd>
                        </div>
                      </dl>

                      <button
                        className="account-button"
                        disabled={isCurrent || isProcessing}
                        onClick={() => handleChoosePlan(plan.id)}
                        style={{ width: "100%" }}
                      >
                        {isCurrent ? "CURRENT PLAN" : isProcessing ? "SWITCHING..." : "CHOOSE PLAN →"}
                      </button>
                    </article>
                  );
                })}
              </div>
            </section>
          )}

          {/* TAB 4: SETTINGS */}
          {activeTab === "settings" && user && (
            <section className="account-section">
              <AccountSettings
                user={user}
                onUserUpdate={(updated) => {
                  setUser(updated);
                  setNotice(`PROFILE UPDATED · ${updated.name.toUpperCase()}`);
                }}
                onLogout={handleLogout}
              />
            </section>
          )}
        </>
      )}

      {/* Modals */}
      <CreateKeyModal
        isOpen={isCreateKeyOpen}
        onClose={() => setIsCreateKeyOpen(false)}
        onSubmit={handleCreateKey}
      />

      <RevealKeyModal
        apiKey={revealKeyData?.key ?? null}
        label={revealKeyData?.label}
        onClose={() => setRevealKeyData(null)}
      />

      <footer className="account-footer">
        <span>PIA / MARKET PLATFORM</span>
        <span>HTTPS ENCRYPTED SESSION · ZERO LOG RETENTION</span>
      </footer>
    </main>
  );
}
