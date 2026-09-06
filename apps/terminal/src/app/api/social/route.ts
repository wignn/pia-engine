import { NextRequest, NextResponse } from "next/server";
import { CORE_API_KEY, CORE_REST_URL } from "@/lib/config";

export async function GET(request: NextRequest) {
  const query = request.nextUrl.searchParams.toString();
  try {
    const response = await fetch(`${CORE_REST_URL}/api/v1/social/posts${query ? `?${query}` : ""}`, {
      headers: { "x-api-key": CORE_API_KEY },
      cache: "no-store",
    });
    const payload = await response.json();
    return NextResponse.json(payload, { status: response.status });
  } catch {
    return NextResponse.json({ items: [], next_before: null, has_more: false, error: "social_unavailable" }, { status: 502 });
  }
}
