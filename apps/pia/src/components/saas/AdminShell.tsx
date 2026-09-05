"use client";

import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState, type ReactNode } from "react";
import { UnauthorizedAccess } from "./UnauthorizedAccess";

interface AdminShellProps {
  children: ReactNode;
  userEmail?: string | null;
  isAuthorized?: boolean;
}

const groups = [
  {
    label: "OPERATIONS",
    items: [
      ["Overview", "/admin", "01"],
      ["Feed sources", "/admin/feeds", "02"],
      ["Macro signals", "/admin/macro", "03"],
      ["Why move", "/admin/why-move", "04"],
    ],
  },
  {
    label: "ACCOUNTS",
    items: [
      ["Users & keys", "/admin/users", "05"],
      ["Plans", "/admin/plans", "06"],
    ],
  },
  {
    label: "SYSTEM",
    items: [["Service status", "/admin/system", "07"]],
  },
];

function isCurrentPath(pathname: string, href: string) {
  return href === "/admin" ? pathname === href : pathname.startsWith(href);
}

export function AdminShell({ children, userEmail = null, isAuthorized = false }: AdminShellProps) {
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);
  const [prevPathname, setPrevPathname] = useState(pathname);

  // Close the mobile menu on navigation: reset state during render instead
  // of inside an effect (cascading renders).
  if (prevPathname !== pathname) {
    setPrevPathname(pathname);
    setMobileOpen(false);
  }

  useEffect(() => {
    if (!mobileOpen) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setMobileOpen(false);
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [mobileOpen]);

  if (!isAuthorized) return <UnauthorizedAccess email={userEmail} />;

  return (
    <div className="admin-app">
      <button className="admin-mobile-toggle" onClick={() => setMobileOpen(true)} aria-label="Open admin navigation">
        <span />
        <span />
      </button>
      {mobileOpen ? <button className="admin-backdrop" onClick={() => setMobileOpen(false)} aria-label="Close admin navigation" /> : null}

      <aside className={`admin-sidebar${mobileOpen ? " is-open" : ""}`}>
        <div className="admin-brand-row">
          <Link href="/admin" className="admin-brand" aria-label="SLV admin overview">
            <span className="admin-brand-mark"><Image src="/logo.png" alt="" width={34} height={34} priority /></span>
            <span><strong>SLV</strong><small>ATLSD</small></span>
          </Link>
          <button className="admin-mobile-close" onClick={() => setMobileOpen(false)} aria-label="Close admin navigation">×</button>
        </div>

        <div className="admin-sidebar-meta"><span>ATLSD</span><b>INTERNAL</b></div>
        <nav className="admin-nav" aria-label="Admin navigation">
          {groups.map((group) => (
            <div className="admin-nav-group" key={group.label}>
              <span className="admin-nav-label">{group.label}</span>
              {group.items.map(([label, href, number]) => (
                <Link className={isCurrentPath(pathname, href) ? "is-active" : ""} href={href} key={href}>
                  <i aria-hidden="true">{number}</i><span>{label}</span>
                </Link>
              ))}
            </div>
          ))}
        </nav>

        <div className="admin-sidebar-footer">
          <span className="admin-live-dot" />
          <span><small>SESSION</small><b>{userEmail || "AUTHORIZED USER"}</b></span>
        </div>
      </aside>

      <div className="admin-content">
        <header className="admin-topbar">
          <div className="admin-breadcrumb"><span>SLV</span><b>/</b><strong>ATLSD</strong><b>/</b><span>{pathname === "/admin" ? "OVERVIEW" : pathname.split("/").at(-1)?.replaceAll("-", " ").toUpperCase()}</span></div>
          <div className="admin-topbar-actions"><span className="admin-environment"><i />INTERNAL</span><a href="/portal">PUBLIC SITE ↗</a></div>
        </header>
        <main className="admin-main">{children}</main>
      </div>
    </div>
  );
}
