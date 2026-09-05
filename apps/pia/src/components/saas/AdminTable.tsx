import type { ReactNode } from "react";

interface AdminTableProps {
  children: ReactNode;
  empty?: boolean;
  loading?: boolean;
  emptyMessage?: string;
}

export function AdminTable({ children, empty = false, loading = false, emptyMessage = "No records found." }: AdminTableProps) {
  return (
    <div className="admin-table-wrap">
      {loading ? <div className="admin-table-state">Loading records…</div> : empty ? <div className="admin-table-state">{emptyMessage}</div> : <table className="admin-table">{children}</table>}
    </div>
  );
}
