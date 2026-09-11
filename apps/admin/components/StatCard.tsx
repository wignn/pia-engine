import React from "react";

interface StatCardProps {
  label: string;
  value: string | number;
  subtext?: string;
  tone?: "green" | "amber" | "red" | "neutral";
}

export function StatCard({ label, value, subtext, tone = "green" }: StatCardProps) {
  let subColor = "#4a9c68";
  if (tone === "amber") subColor = "#bd8535";
  if (tone === "red") subColor = "#b24a4a";
  if (tone === "neutral") subColor = "#6a6f9f";

  return (
    <div className="admin-stat-card">
      <span className="admin-stat-label">{label.toUpperCase()}</span>
      <span className="admin-stat-value">{value}</span>
      {subtext && (
        <span className="admin-stat-sub" style={{ color: subColor }}>
          {subtext}
        </span>
      )}
    </div>
  );
}
