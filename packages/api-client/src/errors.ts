/**
 * Official PIA SDK - Strongly Typed Error Hierarchy
 */

export interface PiaErrorDetails {
  statusCode?: number;
  endpoint?: string;
  method?: string;
  requestId?: string | null;
  headers?: Record<string, string>;
  rawResponse?: unknown;
}

/**
 * Base class for all errors thrown by the PIA SDK.
 */
export class PiaError extends Error {
  public readonly isPiaError = true;
  public readonly statusCode?: number;
  public readonly endpoint?: string;
  public readonly method?: string;
  public readonly requestId?: string | null;

  constructor(message: string, details?: PiaErrorDetails) {
    super(message);
    this.name = "PiaError";
    this.statusCode = details?.statusCode;
    this.endpoint = details?.endpoint;
    this.method = details?.method;
    this.requestId = details?.requestId;

    // Restore prototype chain for instanceof checks
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/**
 * Thrown when client configuration is invalid (e.g. missing API key, invalid URL).
 */
export class ConfigurationError extends PiaError {
  constructor(message: string) {
    super(message);
    this.name = "ConfigurationError";
  }
}

/**
 * Thrown when caller passes invalid arguments (e.g. empty symbol, negative limit).
 */
export class ValidationError extends PiaError {
  public readonly paramName?: string;

  constructor(message: string, paramName?: string) {
    super(message);
    this.name = "ValidationError";
    this.paramName = paramName;
  }
}

/**
 * Thrown on HTTP 401 Unauthorized (invalid or revoked API key).
 * Redacts any sensitive data.
 */
export class AuthenticationError extends PiaError {
  constructor(message = "Authentication failed. Verify that your API key is valid and active.", details?: PiaErrorDetails) {
    super(message, details);
    this.name = "AuthenticationError";
  }
}

/**
 * Thrown on HTTP 403 Forbidden (missing required scope/permissions, e.g. 'realtime:ws').
 */
export class PermissionError extends PiaError {
  public readonly requiredScope?: string;

  constructor(message: string, requiredScope?: string, details?: PiaErrorDetails) {
    super(message, details);
    this.name = "PermissionError";
    this.requiredScope = requiredScope;
  }
}

/**
 * Thrown on HTTP 429 Too Many Requests (rate limit or daily quota exceeded).
 */
export class RateLimitError extends PiaError {
  public readonly retryAfterSeconds?: number;
  public readonly dailyLimit?: number;
  public readonly dailyRemaining?: number;
  public readonly minuteLimit?: number;
  public readonly minuteRemaining?: number;

  constructor(
    message: string,
    options?: {
      retryAfterSeconds?: number;
      dailyLimit?: number;
      dailyRemaining?: number;
      minuteLimit?: number;
      minuteRemaining?: number;
      details?: PiaErrorDetails;
    }
  ) {
    super(message, options?.details);
    this.name = "RateLimitError";
    this.retryAfterSeconds = options?.retryAfterSeconds;
    this.dailyLimit = options?.dailyLimit;
    this.dailyRemaining = options?.dailyRemaining;
    this.minuteLimit = options?.minuteLimit;
    this.minuteRemaining = options?.minuteRemaining;
  }
}

/**
 * Thrown when a request or operation exceeds the configured timeout threshold.
 */
export class TimeoutError extends PiaError {
  public readonly timeoutMs: number;

  constructor(message: string, timeoutMs: number, details?: PiaErrorDetails) {
    super(message, details);
    this.name = "TimeoutError";
    this.timeoutMs = timeoutMs;
  }
}

/**
 * Thrown when network transport fails (DNS resolution, connection refused, connection reset).
 */
export class NetworkError extends PiaError {
  public readonly causeError?: unknown;

  constructor(message: string, cause?: unknown, details?: PiaErrorDetails) {
    super(message, details);
    this.name = "NetworkError";
    this.causeError = cause;
  }
}

/**
 * Thrown when response parsing fails (malformed JSON or unexpected response body).
 */
export class ParseError extends PiaError {
  public readonly rawText?: string;

  constructor(message: string, rawText?: string, details?: PiaErrorDetails) {
    super(message, details);
    this.name = "ParseError";
    this.rawText = rawText;
  }
}

/**
 * Thrown on generic unexpected API responses (4xx, 5xx).
 */
export class ApiError extends PiaError {
  public readonly rawBody?: unknown;

  constructor(message: string, statusCode: number, rawBody?: unknown, details?: PiaErrorDetails) {
    super(message, { ...details, statusCode });
    this.name = "ApiError";
    this.rawBody = rawBody;
  }
}
