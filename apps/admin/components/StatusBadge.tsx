import React from "react";

interface StatusBadgeProps {
  status: "active" | "healthy" | "open" | "suspended" | "closed" | "warn" | "error" | "neutral" | string;
  label?: string;
}

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const norm = status.toLowerCase();

  let badgeClass = "admin-badge-neutral";
  if (["active", "healthy", "open", "true", "live"].includes(norm)) {
    badgeClass = "admin-badge-active";
  } else if (["warn", "degraded", "expiring"].includes(norm)) {
    badgeClass = "admin-badge-warn";
  } else if (["suspended", "closed", "error", "false", "banned", "dead"].includes(norm)) {
    badgeClass = "admin-badge-danger";
  }

  return (
    <span className={`admin-badge ${badgeClass}`}>
      <span
        style={{
          width: 5,
          height: 5,
          borderRadius: "50%",
          backgroundColor: "currentColor",
          display: "inline-block",
        }}
      />
      {label || status.toUpperCase()}
    </span>
  );
}
