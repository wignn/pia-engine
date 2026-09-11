"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { AuthModal } from "./AuthModal";

export function AdminNav() {
  const pathname = usePathname();
  const [utcTime, setUtcTime] = useState("");
  const [showAuthModal, setShowAuthModal] = useState(false);
  const [adminKey, setAdminKey] = useState("silvia");

  useEffect(() => {
    const updateTime = () => {
      const now = new Date();
      setUtcTime(
        now.toISOString().replace("T", " ").substring(0, 19) + " UTC"
      );
    };
    updateTime();
    const timer = setInterval(updateTime, 1000);

    const stored = localStorage.getItem("pia_admin_key");
    if (stored) {
      setAdminKey(stored);
    } else {
      localStorage.setItem("pia_admin_key", "silvia");
    }

    return () => clearInterval(timer);
  }, []);

  const navItems = [
    { label: "OVERVIEW", href: "/" },
    { label: "TENANTS & KEYS", href: "/tenants" },
    { label: "FEEDS & MARKET", href: "/feeds" },
    { label: "SYSTEM & OPS", href: "/system" },
  ];

  return (
    <>
      <header className="admin-nav">
        <div className="admin-nav-left">
          <Link href="/" className="admin-brand">
            <span className="admin-brand-tag">OWNER</span>
            <span>PIA / MISSION CONTROL</span>
          </Link>

          <nav className="admin-nav-links">
            {navItems.map((item) => {
              const isActive =
                item.href === "/"
                  ? pathname === "/"
                  : pathname.startsWith(item.href);
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`admin-nav-link ${isActive ? "active" : ""}`}
                >
                  {item.label}
                </Link>
              );
            })}
          </nav>
        </div>

        <div className="admin-nav-right">
          <div style={{ display: "flex", alignItems: "center", gap: 6, color: "#2b7a4b" }}>
            <span className="admin-pulse-dot" />
            <span>STACK: GREEN</span>
          </div>

          <div style={{ color: "#6a6f9f" }}>
            {utcTime || "SYNCING CLOCK..."}
          </div>

          <button
            type="button"
            onClick={() => setShowAuthModal(true)}
            className="admin-button"
            style={{ minHeight: 26, padding: "0 8px", fontSize: 9 }}
            title="Configure Admin Key"
          >
            KEY: {adminKey.substring(0, 3)}••••
          </button>

          <a
            href="https://pia.wign.dev/portal"
            target="_blank"
            rel="noopener noreferrer"
            style={{
              fontSize: 10,
              fontFamily: "var(--font-geist-mono), monospace",
              color: "var(--blue)",
              textDecoration: "underline",
            }}
          >
            CLIENT PORTAL ↗
          </a>
        </div>
      </header>

      {showAuthModal && (
        <AuthModal
          currentKey={adminKey}
          onClose={() => setShowAuthModal(false)}
          onSave={(newKey) => {
            setAdminKey(newKey);
            localStorage.setItem("pia_admin_key", newKey);
            setShowAuthModal(false);
            window.location.reload();
          }}
        />
      )}
    </>
  );
}
