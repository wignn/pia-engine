/**
 * Official PIA SDK
 * Enterprise market intelligence & realtime streaming SDK.
 */

export { PiaClient } from "./client";
export {
  type PiaClientOptions,
  type ResolvedPiaConfig,
  resolveConfig,
  DEFAULT_BASE_URL,
  DEFAULT_WS_URL,
  DEFAULT_TIMEOUT_MS,
  DEFAULT_MAX_RETRIES,
} from "./config";

export {
  PiaError,
  type PiaErrorDetails,
  ConfigurationError,
  ValidationError,
  AuthenticationError,
  PermissionError,
  RateLimitError,
  TimeoutError,
  NetworkError,
  ParseError,
  ApiError,
} from "./errors";

export {
  type PiaLogger,
  type LogLevel,
  DefaultLogger,
  redactSensitive,
} from "./logger";

export {
  type Timeframe,
  type RateLimitInfo,
  type RequestOptions,
  type MarketPrice,
  type MarketPricesResponse,
  type Candle,
  type CandleResponse,
  type GetCandlesOptions,
  type OrderBook,
  type OrderBookLevel,
  type SocialPost,
  type SocialFeedResponse,
  type GetSocialOptions,
  type NewsArticle,
  type NewsFeedResponse,
  type GetNewsOptions,
  type WsTicketResponse,
} from "./types";

export { RealtimeClient, type SocketState } from "./realtime/socket";
export { type RealtimeEvents } from "./realtime/events";
