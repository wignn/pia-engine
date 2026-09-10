/**
 * Official PIA SDK - Resilient HTTP Transport with Exponential Backoff & Telemetry
 */

import type { ResolvedPiaConfig } from "../config";
import {
  ApiError,
  AuthenticationError,
  NetworkError,
  ParseError,
  PermissionError,
  RateLimitError,
  TimeoutError,
} from "../errors";
import type { RateLimitInfo, RequestOptions } from "../types";

const RETRYABLE_STATUS_CODES = new Set([408, 429, 500, 502, 503, 504]);
const SDK_VERSION = "1.0.1";

export class HttpTransport {
  private lastRateLimitInfo: RateLimitInfo = {};

  constructor(private readonly config: ResolvedPiaConfig) {}

  /**
   * Returns the most recent rate limit headers telemetry recorded by the SDK.
   */
  public getRateLimitInfo(): RateLimitInfo {
    return { ...this.lastRateLimitInfo };
  }

  /**
   * Executes an HTTP request with automatic retry, jittered backoff, and timeout.
   */
  public async request<T>(
    endpoint: string,
    method: "GET" | "POST" | "PUT" | "DELETE" = "GET",
    body?: unknown,
    options?: RequestOptions
  ): Promise<T> {
    const url = `${this.config.baseUrl}${endpoint.startsWith("/") ? "" : "/"}${endpoint}`;
    const maxRetries = options?.maxRetries ?? this.config.maxRetries;
    const timeoutMs = options?.timeoutMs ?? this.config.timeoutMs;

    let attempt = 0;

    while (true) {
      attempt++;
      this.config.logger.debug(
        `HTTP ${method} ${endpoint} (Attempt ${attempt}/${maxRetries + 1})`
      );

      const controller = new AbortController();
      let isTimedOut = false;

      const timer = setTimeout(() => {
        isTimedOut = true;
        controller.abort();
      }, timeoutMs);

      // Chain caller's abort signal if provided
      if (options?.signal) {
        if (options.signal.aborted) {
          clearTimeout(timer);
          throw options.signal.reason;
        }
        options.signal.addEventListener(
          "abort",
          () => {
            clearTimeout(timer);
            controller.abort(options.signal?.reason);
          },
          { once: true }
        );
      }

      try {
        const headers: Record<string, string> = {
          "x-api-key": this.config.apiKey,
          Accept: "application/json",
          "User-Agent": `pia-sdk-ts/${SDK_VERSION}`,
          ...this.config.headers,
          ...(options?.headers || {}),
        };

        let serializedBody: string | undefined;
        if (body !== undefined) {
          headers["Content-Type"] = "application/json";
          serializedBody = JSON.stringify(body);
        }

        const response = await this.config.fetch(url, {
          method,
          headers,
          body: serializedBody,
          signal: controller.signal,
        });

        clearTimeout(timer);

        // Extract rate limit and quota headers
        this.parseTelemetryHeaders(response.headers);

        if (response.ok) {
          return await this.parseResponseBody<T>(response, endpoint, method);
        }

        // Handle error responses
        const status = response.status;
        const errorText = await response.text().catch(() => "");
        let errorJson: any = null;
        try {
          if (errorText) errorJson = JSON.parse(errorText);
        } catch {
          // Ignored if not JSON
        }

        const errorMessage =
          errorJson?.message ||
          errorJson?.error ||
          (errorText ? errorText.slice(0, 200) : `HTTP ${status}`);

        const isRetryable = RETRYABLE_STATUS_CODES.has(status) && attempt <= maxRetries;

        if (isRetryable) {
          const delay = this.calculateBackoffDelay(attempt, response.headers);
          this.config.logger.warn(
            `Request to ${endpoint} failed with HTTP ${status}. Retrying in ${delay}ms...`
          );
          await this.sleep(delay);
          continue;
        }

        // Map non-retryable or exhausted errors
        this.handleHttpError(status, errorMessage, endpoint, method, response.headers, errorJson);
      } catch (err: unknown) {
        clearTimeout(timer);

        if (isTimedOut) {
          throw new TimeoutError(
            `Request to ${endpoint} exceeded timeout of ${timeoutMs}ms.`,
            timeoutMs,
            { endpoint, method }
          );
        }

        // If caller explicitly aborted
        if (options?.signal?.aborted) {
          throw options.signal.reason;
        }

        // If it's already a typed PiaError, rethrow
        if (err instanceof Error && "isPiaError" in err) {
          throw err;
        }

        // Check if transient network failure is retryable
        if (attempt <= maxRetries) {
          const delay = this.calculateBackoffDelay(attempt);
          this.config.logger.warn(
            `Network error calling ${endpoint}: ${err instanceof Error ? err.message : String(err)}. Retrying in ${delay}ms...`
          );
          await this.sleep(delay);
          continue;
        }

        throw new NetworkError(
          `Network transport failed connecting to ${endpoint}: ${err instanceof Error ? err.message : String(err)}`,
          err,
          { endpoint, method }
        );
      }
    }
  }

  private parseTelemetryHeaders(headers: Headers): void {
    const parseNum = (name: string): number | undefined => {
      const val = headers.get(name);
      if (!val) return undefined;
      const num = parseInt(val, 10);
      return Number.isFinite(num) ? num : undefined;
    };

    this.lastRateLimitInfo = {
      limit: parseNum("x-ratelimit-limit"),
      remaining: parseNum("x-ratelimit-remaining"),
      resetSeconds: parseNum("x-ratelimit-reset"),
      dailyLimit: parseNum("x-dailyquota-limit"),
      dailyRemaining: parseNum("x-dailyquota-remaining"),
    };
  }

  private calculateBackoffDelay(attempt: number, headers?: Headers): number {
    // Respect Retry-After header if provided
    const retryAfter = headers?.get("retry-after");
    if (retryAfter) {
      const parsedSeconds = parseInt(retryAfter, 10);
      if (Number.isFinite(parsedSeconds) && parsedSeconds > 0) {
        return Math.min(parsedSeconds * 1000, this.config.maxRetryDelayMs);
      }
    }

    // Exponential backoff with Full Jitter
    const base = this.config.retryDelayMs * Math.pow(2, attempt - 1);
    const capped = Math.min(base, this.config.maxRetryDelayMs);
    const jitter = 0.5 + Math.random() * 0.5;
    return Math.round(capped * jitter);
  }

  private async parseResponseBody<T>(
    response: Response,
    endpoint: string,
    method: string
  ): Promise<T> {
    const text = await response.text();
    if (!text || text.trim() === "") {
      return {} as T;
    }

    try {
      return JSON.parse(text) as T;
    } catch (err) {
      throw new ParseError(
        `Failed to parse JSON response from ${endpoint}: ${err instanceof Error ? err.message : String(err)}`,
        text.slice(0, 500),
        { statusCode: response.status, endpoint, method }
      );
    }
  }

  private handleHttpError(
    status: number,
    message: string,
    endpoint: string,
    method: string,
    headers: Headers,
    body: unknown
  ): never {
    const details = {
      statusCode: status,
      endpoint,
      method,
      requestId: headers.get("x-request-id") || headers.get("cf-ray"),
      rawResponse: body,
    };

    if (status === 401) {
      throw new AuthenticationError(message, details);
    }

    if (status === 403) {
      throw new PermissionError(message, undefined, details);
    }

    if (status === 429) {
      const retryAfter = headers.get("retry-after");
      const retryAfterSeconds = retryAfter ? parseInt(retryAfter, 10) : undefined;
      throw new RateLimitError(message, {
        retryAfterSeconds: Number.isFinite(retryAfterSeconds) ? retryAfterSeconds : undefined,
        dailyLimit: this.lastRateLimitInfo.dailyLimit,
        dailyRemaining: this.lastRateLimitInfo.dailyRemaining,
        minuteLimit: this.lastRateLimitInfo.limit,
        minuteRemaining: this.lastRateLimitInfo.remaining,
        details,
      });
    }

    throw new ApiError(message, status, body, details);
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
