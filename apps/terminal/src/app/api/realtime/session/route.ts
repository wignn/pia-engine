import { NextResponse } from "next/server";
import { CORE_WS_HTTP_TICKET_URL, CORE_API_KEY, PUBLIC_CORE_WS_URL } from "@/lib/config";

// POST /api/realtime/session
// Issues a short-lived realtime WS ticket from the core realtime-gateway,
// so the browser can open wss://.../ws/v1?ticket=... without exposing the API key.
export async function POST() {
  try {
    const res = await fetch(CORE_WS_HTTP_TICKET_URL, {
      method: "POST",
      headers: { "x-api-key": CORE_API_KEY },
      cache: "no-store",
    });
    if (!res.ok) {
      return NextResponse.json({ ticket: null }, { status: res.status });
    }
    const data = await res.json();
    return NextResponse.json({ ...data, wsUrl: PUBLIC_CORE_WS_URL });
  } catch {
    return NextResponse.json({ ticket: null }, { status: 500 });
  }
}
