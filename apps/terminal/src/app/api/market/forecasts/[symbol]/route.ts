import { NextRequest, NextResponse } from "next/server";
import { CORE_API_KEY, CORE_REST_URL } from "@/lib/config";

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ symbol: string }> },
) {
  const { symbol } = await params;
  const normalized = symbol.toUpperCase();
  const timeframe = request.nextUrl.searchParams.get("timeframe") || "15m";
  if (normalized !== "XAUUSD" || timeframe !== "15m") {
    return NextResponse.json({ status: "unavailable", error: "unsupported_forecast_scope" }, { status: 400 });
  }

  try {
    const response = await fetch(
      `${CORE_REST_URL}/api/v1/market/forecasts/XAUUSD?timeframe=15m`,
      { headers: { "x-api-key": CORE_API_KEY }, cache: "no-store" },
    );
    const payload = await response.json();
    return NextResponse.json(payload, { status: response.status });
  } catch {
    return NextResponse.json({ status: "unavailable", error: "forecast_unavailable" }, { status: 503 });
  }
}
