/**
 * Official PIA SDK - Secure Logger & Sensitive Data Redaction
 */

export type LogLevel = "debug" | "info" | "warn" | "error" | "silent";

export interface PiaLogger {
  debug(message: string, ...args: unknown[]): void;
  info(message: string, ...args: unknown[]): void;
  warn(message: string, ...args: unknown[]): void;
  error(message: string, ...args: unknown[]): void;
}

const SENSITIVE_KEY_PATTERNS = [
  /wi_live_[a-zA-Z0-9_-]{10,}/g,
  /Bearer\s+[a-zA-Z0-9._-]+/gi,
  /api[_-]?key["':\s]+["']?([a-zA-Z0-9_-]+)["']?/gi,
  /password["':\s]+["']?([^"'\s]+)["']?/gi,
];

/**
 * Redacts known sensitive patterns (API keys, bearer tokens) from strings.
 */
export function redactSensitive(input: string): string {
  let redacted = input;
  for (const pattern of SENSITIVE_KEY_PATTERNS) {
    redacted = redacted.replace(pattern, (match) => {
      if (match.startsWith("wi_live_")) {
        return `wi_live_***${match.slice(-4)}`;
      }
      if (match.toLowerCase().startsWith("bearer ")) {
        return "Bearer [REDACTED]";
      }
      return "[REDACTED]";
    });
  }
  return redacted;
}

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 10,
  info: 20,
  warn: 30,
  error: 40,
  silent: 50,
};

export class DefaultLogger implements PiaLogger {
  private readonly levelNum: number;
  private readonly prefix = "[PIA-SDK]";

  constructor(level: LogLevel = "warn") {
    this.levelNum = LOG_LEVELS[level] ?? LOG_LEVELS.warn;
  }

  private safeFormat(msg: string, args: unknown[]): [string, ...unknown[]] {
    const cleanMsg = `${this.prefix} ${redactSensitive(msg)}`;
    const cleanArgs = args.map((arg) => {
      if (typeof arg === "string") return redactSensitive(arg);
      if (typeof arg === "object" && arg !== null) {
        try {
          return JSON.parse(redactSensitive(JSON.stringify(arg)));
        } catch {
          return "[Unserializable Object]";
        }
      }
      return arg;
    });
    return [cleanMsg, ...cleanArgs];
  }

  debug(message: string, ...args: unknown[]): void {
    if (this.levelNum <= LOG_LEVELS.debug) {
      const [msg, ...cleanArgs] = this.safeFormat(message, args);
      console.debug(msg, ...cleanArgs);
    }
  }

  info(message: string, ...args: unknown[]): void {
    if (this.levelNum <= LOG_LEVELS.info) {
      const [msg, ...cleanArgs] = this.safeFormat(message, args);
      console.info(msg, ...cleanArgs);
    }
  }

  warn(message: string, ...args: unknown[]): void {
    if (this.levelNum <= LOG_LEVELS.warn) {
      const [msg, ...cleanArgs] = this.safeFormat(message, args);
      console.warn(msg, ...cleanArgs);
    }
  }

  error(message: string, ...args: unknown[]): void {
    if (this.levelNum <= LOG_LEVELS.error) {
      const [msg, ...cleanArgs] = this.safeFormat(message, args);
      console.error(msg, ...cleanArgs);
    }
  }
}
