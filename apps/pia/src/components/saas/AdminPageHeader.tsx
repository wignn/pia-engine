import type { ReactNode } from "react";

interface AdminPageHeaderProps {
  eyebrow: string;
  title: string;
  description: string;
  action?: ReactNode;
}

export function AdminPageHeader({ eyebrow, title, description, action }: AdminPageHeaderProps) {
  return (
    <header className="admin-page-header">
      <div>
        <span className="admin-eyebrow">{eyebrow}</span>
        <h1>{title}</h1>
        <p>{description}</p>
      </div>
      {action ? <div className="admin-page-action">{action}</div> : null}
    </header>
  );
}
