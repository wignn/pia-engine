import { NextResponse } from "next/server";
import { CORE_REST_URL, CORE_API_KEY } from "@/lib/config";

// GET /api/market/prices  -> proxies core /api/v1/market/prices (list)
export async function GET() {
  try {
    const res = await fetch(`${CORE_REST_URL}/api/v1/market/prices`, {
      headers: { "x-api-key": CORE_API_KEY },
      cache: "no-store",
    });
    if (!res.ok) {
      return NextResponse.json({ items: [] }, { status: res.status });
    }
    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ items: [] }, { status: 500 });
  }
}
