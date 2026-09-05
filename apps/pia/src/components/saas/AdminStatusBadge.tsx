interface AdminStatusBadgeProps {
  label: string;
  tone?: "active" | "healthy" | "blocked" | "error" | "stale" | "neutral";
}

export function AdminStatusBadge({ label, tone = "neutral" }: AdminStatusBadgeProps) {
  return <span className={`admin-status-badge admin-status-${tone}`}>{label}</span>;
}
