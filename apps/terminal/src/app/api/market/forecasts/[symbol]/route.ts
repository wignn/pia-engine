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
    if (response.status === 404) {
      return NextResponse.json(
        { status: "unavailable", symbol: "XAUUSD", timeframe: "15m", forecast: null },
        { status: 200 },
      );
    }
    const body = await response.text();
    let payload: unknown;
    try {
      payload = JSON.parse(body);
    } catch {
      console.error("XAUUSD forecast upstream returned non-JSON", {
        status: response.status,
        contentType: response.headers.get("content-type"),
      });
      return NextResponse.json(
        { status: "unavailable", error: "forecast_upstream_non_json", upstream_status: response.status },
        { status: 502 },
      );
    }
    return NextResponse.json(payload, { status: response.status });
  } catch (error) {
    console.error("XAUUSD forecast upstream request failed", error);
    const causeCode =
      error && typeof error === "object" && "cause" in error
        ? (error.cause as { code?: unknown } | undefined)?.code
        : undefined;
    const reason =
      typeof causeCode === "string" && /^[A-Z0-9_]+$/.test(causeCode)
        ? causeCode
        : "UNKNOWN";
    return NextResponse.json(
      { status: "unavailable", error: "forecast_unavailable", reason },
      { status: 503 },
    );
  }
}
