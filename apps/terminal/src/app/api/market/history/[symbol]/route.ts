import { NextRequest, NextResponse } from "next/server";
import { CORE_REST_URL, CORE_API_KEY } from "@/lib/config";

// GET /api/market/history/[symbol]?resolution=15m
// Proxies to core market-data via the internal traffic-router.
export async function GET(
  req: NextRequest,
  { params }: { params: Promise<{ symbol: string }> }
) {
  const { symbol } = await params;
  const resolution = req.nextUrl.searchParams.get("resolution") || "15m";
  const limit = req.nextUrl.searchParams.get("limit");
  const before = req.nextUrl.searchParams.get("before");

  const qs = new URLSearchParams({ resolution });
  if (before) qs.set("before", before); /* pagination cursor */
  qs.set("limit", limit || "240");


  try {
    const res = await fetch(
      `${CORE_REST_URL}/api/v1/market/history/${encodeURIComponent(symbol)}?${qs}`,
      {
        headers: { "x-api-key": CORE_API_KEY },
        cache: "no-store",
      }
    );

    if (!res.ok) {
      return NextResponse.json([], { status: res.status });
    }
    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json([], { status: 500 });
  }
}
