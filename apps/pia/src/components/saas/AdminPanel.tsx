import type { ReactNode } from "react";

interface AdminPanelProps {
  children: ReactNode;
  className?: string;
}

export function AdminPanel({ children, className = "" }: AdminPanelProps) {
  return <section className={`admin-panel ${className}`}>{children}</section>;
}
