import { NextRequest, NextResponse } from "next/server";
import { CORE_API_KEY, CORE_REST_URL } from "@/lib/config";

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  const targetPath = path.join("/");
  const search = request.nextUrl.search;

  try {
    const res = await fetch(`${CORE_REST_URL}/api/v1/${targetPath}${search}`, {
      headers: {
        "x-api-key": CORE_API_KEY,
      },
      cache: "no-store",
    });

    const data = await res.text();
    return new NextResponse(data, {
      status: res.status,
      headers: {
        "content-type": res.headers.get("content-type") || "application/json",
      },
    });
  } catch (err: any) {
    return NextResponse.json(
      { error: "gateway_proxy_error", message: err?.message || String(err) },
      { status: 502 }
    );
  }
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  const targetPath = path.join("/");
  const search = request.nextUrl.search;

  try {
    const body = await request.text();
    const res = await fetch(`${CORE_REST_URL}/api/v1/${targetPath}${search}`, {
      method: "POST",
      headers: {
        "x-api-key": CORE_API_KEY,
        "content-type": request.headers.get("content-type") || "application/json",
      },
      body: body || undefined,
      cache: "no-store",
    });

    const data = await res.text();
    return new NextResponse(data, {
      status: res.status,
      headers: {
        "content-type": res.headers.get("content-type") || "application/json",
      },
    });
  } catch (err: any) {
    return NextResponse.json(
      { error: "gateway_proxy_error", message: err?.message || String(err) },
      { status: 502 }
    );
  }
}
