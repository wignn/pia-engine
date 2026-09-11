import { NextResponse } from "next/server";

const API_GATEWAY_URL = process.env.API_GATEWAY_URL || "http://127.0.0.1:8000";
const ADMIN_API_KEY = process.env.ADMIN_API_KEY || "silvia";

export async function GET() {
  try {
    const res = await fetch(`${API_GATEWAY_URL}/api/v1/market/prices`, {
      headers: {
        "x-api-key": ADMIN_API_KEY,
      },
      next: { revalidate: 3 },
    });

    if (!res.ok) {
      return NextResponse.json(
        { error: `Market API returned status ${res.status}` },
        { status: res.status }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (err: any) {
    return NextResponse.json(
      { error: "Failed to connect to market gateway", detail: String(err) },
      { status: 502 }
    );
  }
}
