interface AdminStatCardProps {
  label: string;
  value: string | number;
  note?: string;
  tone?: "blue" | "green" | "amber" | "red";
}

export function AdminStatCard({ label, value, note, tone = "blue" }: AdminStatCardProps) {
  return (
    <article className={`admin-stat-card admin-tone-${tone}`}>
      <span className="admin-stat-label">{label}</span>
      <strong>{value}</strong>
      {note ? <small>{note}</small> : null}
    </article>
  );
}
