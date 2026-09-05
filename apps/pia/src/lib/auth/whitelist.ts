export function isEmailAdminWhitelisted(email?: string | null): boolean {
  if (!email) return false;
  const whitelistEnv = process.env.ADMIN_WHITELIST_EMAILS || '';
  const allowed = whitelistEnv
    .split(',')
    .map((e) => e.trim().toLowerCase())
    .filter(Boolean);

  return allowed.includes(email.toLowerCase());
}
