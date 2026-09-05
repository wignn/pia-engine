interface UnauthorizedAccessProps {
  email?: string | null;
}

export function UnauthorizedAccess({ email: _email }: UnauthorizedAccessProps) {
  return (
    <main className="admin-unauthorized">
      <div className="admin-unauthorized-card">
        <span className="admin-eyebrow">SLV / ATLSD</span>
        <div className="admin-lock-mark" aria-hidden="true">×</div>
        <h1>Access not authorized.</h1>
        <p>This area is restricted. Sign in with an approved account or return to the public platform.</p>
        <a className="admin-outline-button" href="/portal">Return to SLV</a>
      </div>
    </main>
  );
}
