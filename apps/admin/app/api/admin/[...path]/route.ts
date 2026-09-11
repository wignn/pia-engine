import { NextRequest, NextResponse } from "next/server";

const CONTROL_PLANE_URL = process.env.CONTROL_PLANE_INTERNAL_URL || "http://127.0.0.1:8081";
const DEFAULT_ADMIN_KEY = process.env.ADMIN_API_KEY || "silvia";

export async function GET(
  request: NextRequest,
  context: { params: Promise<{ path: string[] }> }
) {
  return handleProxy(request, context, "GET");
}

export async function POST(
  request: NextRequest,
  context: { params: Promise<{ path: string[] }> }
) {
  return handleProxy(request, context, "POST");
}

export async function PATCH(
  request: NextRequest,
  context: { params: Promise<{ path: string[] }> }
) {
  return handleProxy(request, context, "PATCH");
}

export async function DELETE(
  request: NextRequest,
  context: { params: Promise<{ path: string[] }> }
) {
  return handleProxy(request, context, "DELETE");
}

async function handleProxy(
  request: NextRequest,
  context: { params: Promise<{ path: string[] }> },
  method: string
) {
  const { path } = await context.params;
  const subPath = path.join("/");
  const url = new URL(request.url);
  const targetUrl = `${CONTROL_PLANE_URL}/api/v1/${subPath}${url.search}`;

  const clientAdminKey =
    request.headers.get("x-admin-key") ||
    request.cookies.get("admin_key")?.value ||
    DEFAULT_ADMIN_KEY;

  const headers: Record<string, string> = {
    "x-api-key": clientAdminKey,
    "Content-Type": "application/json",
  };

  let body: string | undefined = undefined;
  if (method !== "GET" && method !== "HEAD") {
    try {
      body = await request.text();
    } catch {
      // no body
    }
  }

  try {
    const res = await fetch(targetUrl, {
      method,
      headers,
      body: body ? body : undefined,
    });

    const data = await res.text();
    return new NextResponse(data, {
      status: res.status,
      headers: {
        "Content-Type": res.headers.get("Content-Type") || "application/json",
      },
    });
  } catch (err: any) {
    return NextResponse.json(
      { error: "Failed to connect to control-plane", detail: String(err) },
      { status: 502 }
    );
  }
}
