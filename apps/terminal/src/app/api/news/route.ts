import { NextResponse } from "next/server";

// Server-side only: reach the API gateway via the internal traffic-router
// (container-to-container on the private network), never 127.0.0.1.
const CORE_REST_URL = process.env.CORE_REST_URL || "http://traffic-router";
const CORE_API_KEY = process.env.CORE_API_KEY || "silvia";

export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const id = searchParams.get("id");
    const limit = searchParams.get("limit") || "40";
    const q = searchParams.get("q") || searchParams.get("symbol") || "";

    const targetUrl = id
      ? `${CORE_REST_URL}/api/v1/news/${encodeURIComponent(id)}`
      : `${CORE_REST_URL}/api/v1/news?limit=${encodeURIComponent(limit)}${q ? `&q=${encodeURIComponent(q)}` : ""}`;

    const res = await fetch(targetUrl, {
      headers: {
        "x-api-key": CORE_API_KEY,
      },
      cache: "no-store",
    });

    if (!res.ok) {
      const fallback = await fetch(`${CORE_REST_URL}/api/v1/news/latest?limit=${encodeURIComponent(limit)}`, {
        headers: { "x-api-key": CORE_API_KEY },
        cache: "no-store",
      });
      if (fallback.ok) {
        return NextResponse.json(await fallback.json());
      }
      return NextResponse.json({ items: [] }, { status: res.status });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (err) {
    return NextResponse.json({ items: [] }, { status: 500 });
  }
}
