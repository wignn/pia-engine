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
  const [optionsSummary, yields, fearGreed, news] = await Promise.allSettled([
    read("/api/v1/options/summary"),
    read("/api/v1/fixed-income/yield-curve?country=US"),
    read("/api/v1/fear-greed?scope=global"),
    read("/api/v1/news/latest"),
  ]);

  const allSnapshots =
    optionsSummary.status === "fulfilled" && Array.isArray(optionsSummary.value?.data)
      ? optionsSummary.value.data
      : [];

  return NextResponse.json({
    options: {
      data: allSnapshots,
    },
    options_list: allSnapshots,
    requested_symbol: symbol,
    yields: yields.status === "fulfilled" ? yields.value : null,
    fear_greed: fearGreed.status === "fulfilled" ? fearGreed.value : null,
    news: news.status === "fulfilled" ? news.value : { items: [] },
  });
}
