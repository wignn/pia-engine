/**
 * Official PIA SDK - Configuration & Validation
 */

import { ConfigurationError } from "./errors";
import { DefaultLogger, type LogLevel, type PiaLogger } from "./logger";

export interface PiaClientOptions {
  /**
   * Secret or live API Key (e.g. "wi_live_...").
   * If omitted, the SDK attempts to read from process.env.PIA_API_KEY or ATSLD_API_KEY.
   */
  apiKey?: string;

  /**
   * Base URL for the unified REST gateway.
   * Default: "https://api-engine.wign.dev"
   */
  baseUrl?: string;

  /**
   * WebSocket URL for realtime streaming.
   * Default: "wss://api-engine.wign.dev/api/v1/ws"
   */
  wsUrl?: string;

  /**
   * Global HTTP request timeout in milliseconds.
   * Default: 15_000 (15s)
   */
  timeoutMs?: number;

  /**
   * Maximum number of retry attempts for transient failures (408, 429, 5xx).
   * Default: 3
   */
  maxRetries?: number;

  /**
   * Base initial delay for exponential backoff in milliseconds.
   * Default: 500
   */
  retryDelayMs?: number;

  /**
   * Maximum cap on retry backoff delay in milliseconds.
   * Default: 10_000 (10s)
   */
  maxRetryDelayMs?: number;

  /**
   * Custom HTTP headers injected into all requests.
   */
  headers?: Record<string, string>;

  /**
   * Custom fetch implementation (useful for tests or runtime environments).
   */
  fetch?: typeof fetch;

  /**
   * Custom WebSocket implementation (e.g. 'ws' in Node.js).
   */
  WebSocket?: unknown;

  /**
   * Custom logger instance.
   */
  logger?: PiaLogger;

  /**
   * Logging level when using the default logger.
   * Default: "warn" (or "debug" if debug: true)
   */
  logLevel?: LogLevel;

  /**
   * Enable verbose debug logs.
   * Default: false
   */
  debug?: boolean;
}

export interface ResolvedPiaConfig {
  apiKey: string;
  baseUrl: string;
  wsUrl: string;
  timeoutMs: number;
  maxRetries: number;
  retryDelayMs: number;
  maxRetryDelayMs: number;
  headers: Record<string, string>;
  fetch: typeof fetch;
  WebSocket?: unknown;
  logger: PiaLogger;
  debug: boolean;
}

export const DEFAULT_BASE_URL = "https://api-engine.wign.dev";
export const DEFAULT_WS_URL = "wss://api-engine.wign.dev/api/v1/ws";
export const DEFAULT_TIMEOUT_MS = 15_000;
export const DEFAULT_MAX_RETRIES = 3;
export const DEFAULT_RETRY_DELAY_MS = 500;
export const DEFAULT_MAX_RETRY_DELAY_MS = 10_000;

function resolveEnvApiKey(): string | undefined {
  if (typeof process !== "undefined" && process?.env) {
    return (
      process.env.PIA_API_KEY ||
      process.env.ATSLD_API_KEY ||
      process.env.CORE_API_KEY ||
      process.env.NEXT_PUBLIC_API_KEY
    );
  }
  return undefined;
}

/**
 * Validates and resolves full client configuration with sensible defaults.
 */
export function resolveConfig(options: PiaClientOptions = {}): ResolvedPiaConfig {
  const apiKey = options.apiKey?.trim() || resolveEnvApiKey()?.trim();

  if (!apiKey) {
    throw new ConfigurationError(
      "API key is required. Pass { apiKey: '...' } to PiaClient or set the PIA_API_KEY environment variable."
    );
  }

  const rawBaseUrl = options.baseUrl?.trim() || DEFAULT_BASE_URL;
  const baseUrl = rawBaseUrl.replace(/\/+$/, "");

  try {
    const parsed = new URL(baseUrl);
    if (!["http:", "https:"].includes(parsed.protocol)) {
      throw new Error("Protocol must be http or https");
    }
  } catch (err) {
    throw new ConfigurationError(
      `Invalid baseUrl '${baseUrl}': ${err instanceof Error ? err.message : String(err)}`
    );
  }

  const rawWsUrl = options.wsUrl?.trim() || DEFAULT_WS_URL;
  const wsUrl = rawWsUrl.replace(/\/+$/, "");

  try {
    const parsedWs = new URL(wsUrl);
    if (!["ws:", "wss:"].includes(parsedWs.protocol)) {
      throw new Error("Protocol must be ws or wss");
    }
  } catch (err) {
    throw new ConfigurationError(
      `Invalid wsUrl '${wsUrl}': ${err instanceof Error ? err.message : String(err)}`
    );
  }

  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  if (timeoutMs < 100) {
    throw new ConfigurationError("timeoutMs must be at least 100 milliseconds.");
  }

  const maxRetries = options.maxRetries ?? DEFAULT_MAX_RETRIES;
  if (maxRetries < 0) {
    throw new ConfigurationError("maxRetries cannot be negative.");
  }

  const retryDelayMs = options.retryDelayMs ?? DEFAULT_RETRY_DELAY_MS;
  const maxRetryDelayMs = options.maxRetryDelayMs ?? DEFAULT_MAX_RETRY_DELAY_MS;

  const debug = options.debug ?? false;
  const logLevel: LogLevel = options.logLevel ?? (debug ? "debug" : "warn");
  const logger = options.logger ?? new DefaultLogger(logLevel);

  const resolvedFetch =
    options.fetch ||
    (typeof globalThis !== "undefined" && typeof globalThis.fetch === "function"
      ? globalThis.fetch.bind(globalThis)
      : undefined);

  if (!resolvedFetch) {
    throw new ConfigurationError(
      "A valid fetch implementation was not found in the runtime. Pass a custom fetch to PiaClientOptions."
    );
  }

  // Resolve native or custom WebSocket
  const resolvedWebSocket =
    options.WebSocket ||
    (typeof globalThis !== "undefined" && (globalThis as any).WebSocket
      ? (globalThis as any).WebSocket
      : undefined);

  return {
    apiKey,
    baseUrl,
    wsUrl,
    timeoutMs,
    maxRetries,
    retryDelayMs,
    maxRetryDelayMs,
    headers: { ...(options.headers || {}) },
    fetch: resolvedFetch,
    WebSocket: resolvedWebSocket,
    logger,
    debug,
  };
}
