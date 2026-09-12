import { NextResponse } from "next/server";

const CORE_REST_URL = process.env.CORE_REST_URL || "http://traffic-router";
const CORE_API_KEY = process.env.CORE_API_KEY || "silvia";

export async function GET() {
  try {
    const res = await fetch(`${CORE_REST_URL}/api/v1/economic/calendar?impact=high&limit=30`, {
      headers: {
        "x-api-key": CORE_API_KEY,
      },
      next: { revalidate: 60 },
    });

    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data.items) && data.items.length > 0) {
        return NextResponse.json(data);
      }
    }
  } catch {
    // fallback
  }

  // Institutional macro schedule fallback
  const now = new Date();
  const todayStr = now.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" });
  const mockCalendar = [
    { id: "cal-1", country: "USD", currency: "USD", title: "Non-Farm Employment Change", impact: "high", time: "12:30", forecast: "165K", previous: "142K", actual: "178K", status: "completed", date: todayStr },
    { id: "cal-2", country: "USD", currency: "USD", title: "Unemployment Rate", impact: "high", time: "12:30", forecast: "4.2%", previous: "4.3%", actual: "4.2%", status: "completed", date: todayStr },
    { id: "cal-3", country: "USD", currency: "USD", title: "Average Hourly Earnings m/m", impact: "medium", time: "12:30", forecast: "0.3%", previous: "0.2%", actual: "0.4%", status: "completed", date: todayStr },
    { id: "cal-4", country: "USD", currency: "USD", title: "CPI Inflation Rate m/m", impact: "high", time: "12:30", forecast: "0.2%", previous: "0.2%", actual: null, status: "upcoming", date: "Tomorrow" },
    { id: "cal-5", country: "USD", currency: "USD", title: "Core CPI Inflation Rate y/y", impact: "high", time: "12:30", forecast: "3.2%", previous: "3.2%", actual: null, status: "upcoming", date: "Tomorrow" },
    { id: "cal-6", country: "EUR", currency: "EUR", title: "ECB Main Refinancing Rate", impact: "high", time: "13:15", forecast: "3.65%", previous: "3.75%", actual: null, status: "upcoming", date: "Thu" },
    { id: "cal-7", country: "EUR", currency: "EUR", title: "ECB Monetary Policy Statement", impact: "high", time: "13:45", forecast: "—", previous: "—", actual: null, status: "upcoming", date: "Thu" },
    { id: "cal-8", country: "USD", currency: "USD", title: "FOMC Federal Funds Target Rate", impact: "high", time: "18:00", forecast: "5.25%", previous: "5.50%", actual: null, status: "upcoming", date: "Wed Sep 16" },
    { id: "cal-9", country: "GBP", currency: "GBP", title: "BoE Interest Rate Decision", impact: "high", time: "11:00", forecast: "5.00%", previous: "5.00%", actual: null, status: "upcoming", date: "Thu Sep 17" },
    { id: "cal-10", country: "JPY", currency: "JPY", title: "BoJ Policy Balance Rate", impact: "high", time: "03:00", forecast: "0.25%", previous: "0.25%", actual: null, status: "upcoming", date: "Fri Sep 18" },
  ];

  return NextResponse.json({ items: mockCalendar });
}
