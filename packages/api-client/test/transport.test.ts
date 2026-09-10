import { describe, expect, it } from "bun:test";
import { resolveConfig } from "../src/config";
import {
  ApiError,
  AuthenticationError,
  NetworkError,
  PermissionError,
  RateLimitError,
  TimeoutError,
} from "../src/errors";
import { HttpTransport } from "../src/http/transport";

describe("HttpTransport - Resiliency, Retries, and Telemetry", () => {
  it("injects x-api-key, User-Agent, and custom headers", async () => {
    let capturedHeaders: Record<string, string> = {};

    const mockFetch = async (url: string, init?: RequestInit) => {
      capturedHeaders = (init?.headers as Record<string, string>) || {};
      return new Response(JSON.stringify({ status: "ok" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    };

    const config = resolveConfig({
      apiKey: "wi_live_custom_key",
      fetch: mockFetch as any,
      headers: { "X-Custom-Env": "staging" },
    });

    const transport = new HttpTransport(config);
    const result = await transport.request<{ status: string }>("/api/v1/test");

    expect(result.status).toBe("ok");
    expect(capturedHeaders["x-api-key"]).toBe("wi_live_custom_key");
    expect(capturedHeaders["User-Agent"]).toContain("pia-sdk-ts");
    expect(capturedHeaders["X-Custom-Env"]).toBe("staging");
  });

  it("parses RFC 6585 and daily quota telemetry headers", async () => {
    const mockFetch = async () => {
      return new Response(JSON.stringify({ data: 123 }), {
        status: 200,
        headers: {
          "x-ratelimit-limit": "60",
          "x-ratelimit-remaining": "59",
          "x-ratelimit-reset": "1",
          "x-dailyquota-limit": "1000",
          "x-dailyquota-remaining": "997",
        },
      });
    };

    const config = resolveConfig({
      apiKey: "test",
      fetch: mockFetch as any,
    });

    const transport = new HttpTransport(config);
    await transport.request("/api/v1/test");

    const telemetry = transport.getRateLimitInfo();
    expect(telemetry.limit).toBe(60);
    expect(telemetry.remaining).toBe(59);
    expect(telemetry.resetSeconds).toBe(1);
    expect(telemetry.dailyLimit).toBe(1000);
    expect(telemetry.dailyRemaining).toBe(997);
  });

  it("maps HTTP 401 to AuthenticationError without retrying", async () => {
    let callCount = 0;
    const mockFetch = async () => {
      callCount++;
      return new Response(JSON.stringify({ error: "Invalid API key" }), {
        status: 401,
      });
    };

    const config = resolveConfig({
      apiKey: "bad_key",
      fetch: mockFetch as any,
      maxRetries: 3,
    });

    const transport = new HttpTransport(config);
    await expect(transport.request("/api/v1/test")).rejects.toThrow(AuthenticationError);
    expect(callCount).toBe(1); // 401 must not be retried
  });

  it("maps HTTP 403 to PermissionError without retrying", async () => {
    let callCount = 0;
    const mockFetch = async () => {
      callCount++;
      return new Response(JSON.stringify({ error: "Missing scope" }), {
        status: 403,
      });
    };

    const config = resolveConfig({
      apiKey: "key",
      fetch: mockFetch as any,
      maxRetries: 3,
    });

    const transport = new HttpTransport(config);
    await expect(transport.request("/api/v1/test")).rejects.toThrow(PermissionError);
    expect(callCount).toBe(1);
  });

  it("retries transient HTTP 500 errors and succeeds when backend recovers", async () => {
    let callCount = 0;
    const mockFetch = async () => {
      callCount++;
      if (callCount < 3) {
        return new Response(JSON.stringify({ error: "Internal Gateway Hiccup" }), {
          status: 502,
        });
      }
      return new Response(JSON.stringify({ recovered: true }), {
        status: 200,
      });
    };

    const config = resolveConfig({
      apiKey: "key",
      fetch: mockFetch as any,
      maxRetries: 3,
      retryDelayMs: 10, // Fast for tests
    });

    const transport = new HttpTransport(config);
    const res = await transport.request<{ recovered: boolean }>("/api/v1/test");

    expect(res.recovered).toBe(true);
    expect(callCount).toBe(3);
  });

  it("maps HTTP 429 to RateLimitError with retry-after header", async () => {
    const mockFetch = async () => {
      return new Response(JSON.stringify({ error: "Daily quota exhausted" }), {
        status: 429,
        headers: {
          "retry-after": "45",
          "x-dailyquota-remaining": "0",
        },
      });
    };

    const config = resolveConfig({
      apiKey: "key",
      fetch: mockFetch as any,
      maxRetries: 0, // Do not wait in test
    });

    const transport = new HttpTransport(config);
    try {
      await transport.request("/api/v1/test");
      expect(true).toBe(false); // Unreachable
    } catch (err: any) {
      expect(err instanceof RateLimitError).toBe(true);
      expect(err.retryAfterSeconds).toBe(45);
    }
  });

  it("throws TimeoutError when request duration exceeds timeoutMs", async () => {
    const mockFetch = async (url: string, init?: RequestInit) => {
      return new Promise<Response>((_, reject) => {
        init?.signal?.addEventListener("abort", () => {
          reject(new Error("aborted"));
        });
      });
    };

    const config = resolveConfig({
      apiKey: "key",
      fetch: mockFetch as any,
      timeoutMs: 100,
      maxRetries: 0,
    });

    const transport = new HttpTransport(config);
    await expect(transport.request("/api/v1/slow")).rejects.toThrow(TimeoutError);
  });
});
