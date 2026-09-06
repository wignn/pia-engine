import { NextResponse } from "next/server";

// Server-side only: reach the API gateway via the internal traffic-router
// (container-to-container on the private network), never 127.0.0.1.
const CORE_REST_URL = process.env.CORE_REST_URL || "http://traffic-router";
const CORE_API_KEY = process.env.CORE_API_KEY || "silvia";

export async function GET() {
  try {
    const res = await fetch(`${CORE_REST_URL}/api/v1/forex/news/latest`, {
      headers: {
        "x-api-key": CORE_API_KEY,
      },
      next: { revalidate: 30 },
    });

    if (!res.ok) {
      return NextResponse.json({ items: [] }, { status: res.status });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (err) {
    return NextResponse.json({ items: [] }, { status: 500 });
  }
}
