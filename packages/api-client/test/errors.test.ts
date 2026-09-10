import { describe, expect, it } from "bun:test";
import {
  ApiError,
  AuthenticationError,
  ConfigurationError,
  NetworkError,
  ParseError,
  PermissionError,
  PiaError,
  RateLimitError,
  redactSensitive,
  TimeoutError,
  ValidationError,
} from "../src";

describe("SDK Error Hierarchy & Security", () => {
  it("all typed errors correctly inherit from PiaError and Error", () => {
    const errors = [
      new ConfigurationError("bad config"),
      new ValidationError("bad input", "symbol"),
      new AuthenticationError("bad key"),
      new PermissionError("no scope", "realtime:ws"),
      new RateLimitError("too many reqs"),
      new TimeoutError("slow", 1000),
      new NetworkError("dns failed"),
      new ParseError("bad json", "{broken"),
      new ApiError("server error", 500),
    ];

    for (const err of errors) {
      expect(err instanceof Error).toBe(true);
      expect(err instanceof PiaError).toBe(true);
      expect(err.isPiaError).toBe(true);
    }
  });

  it("RateLimitError retains quota telemetry properties", () => {
    const err = new RateLimitError("Quota reached", {
      retryAfterSeconds: 60,
      dailyLimit: 1000,
      dailyRemaining: 0,
      minuteLimit: 60,
      minuteRemaining: 0,
    });

    expect(err.name).toBe("RateLimitError");
    expect(err.retryAfterSeconds).toBe(60);
    expect(err.dailyRemaining).toBe(0);
    expect(err.minuteRemaining).toBe(0);
  });

  it("redacts sensitive keys from error logs and message strings", () => {
    const rawString =
      "Request failed with key wi_live_abcdef1234567890abcdef and Bearer secret_token_xyz";
    const clean = redactSensitive(rawString);

    expect(clean).not.toContain("wi_live_abcdef1234567890abcdef");
    expect(clean).not.toContain("secret_token_xyz");
    expect(clean).toContain("wi_live_***cdef");
    expect(clean).toContain("Bearer [REDACTED]");
  });
});
