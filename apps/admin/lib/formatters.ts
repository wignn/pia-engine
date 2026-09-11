export function formatDate(dateStr: string | null | undefined): string {
  if (!dateStr) return "Never";
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString("en-GB", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    }).toUpperCase();
  } catch {
    return dateStr;
  }
}

export function formatNumber(num: number | null | undefined): string {
  if (num === null || num === undefined) return "0";
  return num.toLocaleString("en-US");
}

export function formatPriceIDR(idr: number): string {
  if (idr === 0) return "IDR 0 / MO";
  return `IDR ${(idr / 1000).toLocaleString("en-US")}K / MO`;
}
