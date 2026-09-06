import { NextRequest, NextResponse } from "next/server";
import { CORE_API_KEY, CORE_REST_URL } from "@/lib/config";

async function read(path: string) {
  const response = await fetch(`${CORE_REST_URL}${path}`, {
    headers: { "x-api-key": CORE_API_KEY },
    cache: "no-store",
  });
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return response.json();
}

export async function GET(request: NextRequest) {
  const symbol = request.nextUrl.searchParams.get("symbol") || "BTC";
  const [options, yields, fearGreed, news] = await Promise.allSettled([
    read(`/api/v1/options/summary?symbol=${encodeURIComponent(symbol)}`),
    read("/api/v1/rates/yield-curve?country=US"),
    read("/api/v1/fear-greed?scope=global"),
    read("/api/v1/forex/news/latest"),
  ]);

  return NextResponse.json({
    options: options.status === "fulfilled" ? options.value : null,
    yields: yields.status === "fulfilled" ? yields.value : null,
    fear_greed: fearGreed.status === "fulfilled" ? fearGreed.value : null,
    news: news.status === "fulfilled" ? news.value : { items: [] },
  });
}
