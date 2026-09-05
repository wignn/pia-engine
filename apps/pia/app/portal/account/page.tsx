"use client";

import { useEffect, useState } from "react";
import { accountApi, type KeyInfo, type Plan, type User } from "@/src/lib/api/account";

const fallbackPlans: Plan[] = [
  { id: "free", name: "Free", price_idr: 0, requests_per_day: 1000, ws_connections: 1, news_history_days: 1, rate_limit_per_min: 30 },
  { id: "starter", name: "Starter", price_idr: 99000, requests_per_day: 25000, ws_connections: 3, news_history_days: 7, rate_limit_per_min: 120 },
  { id: "pro", name: "Pro", price_idr: 399000, requests_per_day: 250000, ws_connections: 10, news_history_days: 30, rate_limit_per_min: 600 },
  { id: "enterprise", name: "Enterprise", price_idr: 0, requests_per_day: 0, ws_connections: 0, news_history_days: 0, rate_limit_per_min: 0 },
];

const formatDate = (value: string | null | undefined) => value ? new Date(value).toLocaleDateString("en-GB", { day: "2-digit", month: "short", year: "numeric" }).toUpperCase() : "Never";
const formatPrice = (value: number, id: string) => id === "enterprise" ? "Custom" : value === 0 ? "IDR 0" : `IDR ${(value / 1000).toLocaleString("en-US")}K`;

export default function AccountPage() {
  const [user, setUser] = useState<User | null>(null);
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [plans, setPlans] = useState<Plan[]>(fallbackPlans);
  const [activeTab, setActiveTab] = useState<"overview" | "keys" | "plans">("overview");
  const [notice, setNotice] = useState("LOADING ACCOUNT...");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.allSettled([accountApi.me(), accountApi.keys(), accountApi.plans()]).then(([me, keyList, planList]) => {
      if (me.status === "fulfilled") {
        setUser(me.value.user);
        setNotice("ACCOUNT CONNECTED · SESSION ACTIVE");
      } else {
        setNotice("SIGN IN TO MANAGE YOUR SLV ACCOUNT");
      }
      if (keyList.status === "fulfilled") setKeys(keyList.value);
      if (planList.status === "fulfilled" && planList.value.length) setPlans(planList.value);
      setLoading(false);
    });
  }, []);

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

  const createKey = async () => {
    const label = window.prompt("API key label", "Development");
    if (!label?.trim()) return;
    try {
      const result = await accountApi.createKey(label.trim());
      setKeys((current) => [result.key_info, ...current]);
      setNotice(`KEY CREATED · SAVE THIS SECRET NOW: ${result.api_key}`);
      setActiveTab("keys");
    } catch {
      setNotice("KEY CREATION FAILED");
    }
  };

  const revokeKey = async (id: string) => {
    if (!window.confirm("Revoke this API key? This cannot be undone.")) return;
    try {
      await accountApi.revokeKey(id);
      setKeys((current) => current.filter((key) => key.id !== id));
      setNotice("KEY REVOKED");
    } catch {
      setNotice("KEY REVOCATION FAILED");
    }
  };

  const activePlan = plans.find((plan) => plan.id === user?.plan) || plans.find((plan) => plan.id === "free") || fallbackPlans[0];

  return (
    <main className="account-page">
      <header className="account-header">
        <div><span className="account-kicker">SLV / ACCOUNT</span><h1>YOUR MARKET<br /><em>ACCESS.</em></h1><p>Manage your ATLSD subscription and API access from one private workspace.</p></div>
        <a className="account-back" href="/portal">← BACK TO PLATFORM</a>
      </header>

      <div className="account-notice">{loading ? "LOADING ACCOUNT..." : notice}</div>
      <nav className="account-tabs" aria-label="Account sections">
        {(["overview", "keys", "plans"] as const).map((tab) => <button className={activeTab === tab ? "is-active" : ""} onClick={() => setActiveTab(tab)} key={tab}>{tab.toUpperCase()}</button>)}
      </nav>

      {activeTab === "overview" ? <section className="account-grid">
        <article className="account-card account-profile"><span className="account-card-label">CURRENT SESSION</span><h2>{user ? "Welcome back." : "Sign in first."}</h2><p className="account-muted">{user ? "Your private ATLSD workspace is ready." : "Use Google or GitHub to manage your account and API access."}</p>{user ? <div className="account-identity"><span className="account-avatar">{user.name.slice(0, 2).toUpperCase()}</span><span><strong>{user.name}</strong><small>{user.email}</small></span></div> : <div className="account-identity"><button className="account-button account-button-primary" onClick={() => loginOAuth("google")}>GOOGLE LOGIN</button><button className="account-button" onClick={() => loginOAuth("github")}>GITHUB LOGIN</button></div>}</article>
        <article className="account-card account-plan-card"><span className="account-card-label">ACTIVE PLAN</span><h2>{activePlan.name}</h2><p className="account-muted">Your current access envelope.</p><dl><div><dt>REQUESTS / DAY</dt><dd>{activePlan.requests_per_day ? activePlan.requests_per_day.toLocaleString() : "Custom"}</dd></div><div><dt>WS CONNECTIONS</dt><dd>{activePlan.ws_connections || "Custom"}</dd></div><div><dt>NEWS HISTORY</dt><dd>{activePlan.news_history_days ? `${activePlan.news_history_days} DAYS` : "Custom"}</dd></div></dl><button className="account-button" onClick={() => setActiveTab("plans")}>VIEW PLANS →</button></article>
        <article className="account-card account-wide"><div className="account-card-head"><span><span className="account-card-label">API ACCESS</span><h2>{keys.length} active keys</h2></span><button className="account-button" onClick={() => setActiveTab("keys")}>MANAGE KEYS →</button></div><p className="account-muted">Keys are only shown as safe prefixes after creation. Never share a secret.</p><div className="account-key-strip">{keys.slice(0, 3).map((key) => <span key={key.id}><b>{key.key_prefix}</b><small>{key.label}</small></span>)}</div></article>
      </section> : null}

      {activeTab === "keys" ? <section className="account-section"><div className="account-section-head"><div><span className="account-card-label">DEVELOPER ACCESS</span><h2>API KEYS</h2><p className="account-muted">Create, label, and revoke keys for your own account.</p></div><button className="account-button account-button-primary" onClick={createKey} disabled={!user}>CREATE KEY +</button></div><div className="account-key-list">{keys.map((key) => <article className="account-card account-key-card" key={key.id}><div><span className="account-status">{key.is_active ? "ACTIVE" : "REVOKED"}</span><h3>{key.label}</h3><code>{key.key_prefix}</code></div><dl><div><dt>CREATED</dt><dd>{formatDate(key.created_at)}</dd></div><div><dt>LAST USED</dt><dd>{key.last_used_at ? formatDate(key.last_used_at) : "Never"}</dd></div><div><dt>WS LIMIT</dt><dd>{key.max_ws_connections || "Plan default"}</dd></div></dl><button className="account-danger" onClick={() => revokeKey(key.id)}>REVOKE</button></article>)}</div></section> : null}

      {activeTab === "plans" ? <section className="account-section"><div className="account-section-head"><div><span className="account-card-label">SUBSCRIPTION</span><h2>PLANS & LIMITS</h2><p className="account-muted">Choose the access envelope that fits your workflow.</p></div></div><div className="account-plans">{plans.map((plan) => <article className={`account-card account-plan-option${plan.id === activePlan.id ? " is-current" : ""}`} key={plan.id}><span className="account-card-label">{plan.id === activePlan.id ? "CURRENT" : plan.id.toUpperCase()}</span><h3>{plan.name}</h3><strong>{formatPrice(plan.price_idr, plan.id)}</strong><p className="account-muted">{plan.requests_per_day ? `${plan.requests_per_day.toLocaleString()} requests/day · ${plan.ws_connections} WS · ${plan.news_history_days} days` : "Custom limits and access"}</p><button className="account-button" disabled={plan.id === activePlan.id}>{plan.id === activePlan.id ? "ACTIVE PLAN" : "CHOOSE PLAN →"}</button></article>)}</div></section> : null}

      <footer className="account-footer"><span>SLV / ATLSD ACCOUNT</span><span>PRIVATE SESSION · HTTPS ONLY</span></footer>
    </main>
  );
}
