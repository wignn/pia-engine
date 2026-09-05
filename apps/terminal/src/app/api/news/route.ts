import { NextResponse } from "next/server";

export async function GET() {
  try {
    const res = await fetch("http://127.0.0.1:8000/api/v1/forex/news/latest", {
      headers: {
        "x-api-key": process.env.ADMIN_API_KEY || "silvia",
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
