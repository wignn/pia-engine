import { describe, expect, it } from "bun:test";
import { ConfigurationError, resolveConfig } from "../src";

describe("SDK Config Resolution", () => {
  it("resolves default configuration when apiKey is provided", () => {
    const config = resolveConfig({ apiKey: "wi_live_test123" });
    expect(config.apiKey).toBe("wi_live_test123");
    expect(config.baseUrl).toBe("https://api-engine.wign.dev");
    expect(config.wsUrl).toBe("wss://api-engine.wign.dev/api/v1/ws");
    expect(config.timeoutMs).toBe(15000);
    expect(config.maxRetries).toBe(3);
    expect(config.retryDelayMs).toBe(500);
    expect(config.debug).toBe(false);
  });

  it("throws ConfigurationError when apiKey is missing and env is unset", () => {
    const originalEnv = process.env.PIA_API_KEY;
    delete process.env.PIA_API_KEY;
    delete process.env.ATSLD_API_KEY;
    delete process.env.CORE_API_KEY;

    expect(() => resolveConfig({})).toThrow(ConfigurationError);

    if (originalEnv) process.env.PIA_API_KEY = originalEnv;
  });

  it("reads API key from process.env.PIA_API_KEY fallback", () => {
    process.env.PIA_API_KEY = "wi_live_from_env";
    const config = resolveConfig({});
    expect(config.apiKey).toBe("wi_live_from_env");
    delete process.env.PIA_API_KEY;
  });

  it("strips trailing slashes from baseUrl and wsUrl", () => {
    const config = resolveConfig({
      apiKey: "test",
      baseUrl: "https://custom.wign.dev///",
      wsUrl: "wss://custom.wign.dev/ws///",
    });
    expect(config.baseUrl).toBe("https://custom.wign.dev");
    expect(config.wsUrl).toBe("wss://custom.wign.dev/ws");
  });

  it("throws on invalid URL protocol", () => {
    expect(() =>
      resolveConfig({ apiKey: "test", baseUrl: "ftp://invalid-url" })
    ).toThrow(ConfigurationError);

    expect(() =>
      resolveConfig({ apiKey: "test", wsUrl: "http://invalid-ws" })
    ).toThrow(ConfigurationError);
  });

  it("throws on negative maxRetries or too small timeoutMs", () => {
    expect(() =>
      resolveConfig({ apiKey: "test", maxRetries: -1 })
    ).toThrow(ConfigurationError);

    expect(() =>
      resolveConfig({ apiKey: "test", timeoutMs: 50 })
    ).toThrow(ConfigurationError);
  });
});
