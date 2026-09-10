/**
 * Official PIA SDK - Market Intelligence API Resource
 */

import { ValidationError } from "../errors";
import type { HttpTransport } from "../http/transport";
import type {
  CandleResponse,
  GetCandlesOptions,
  MarketPricesResponse,
  OrderBook,
  RequestOptions,
} from "../types";

export class MarketResource {
  constructor(private readonly transport: HttpTransport) {}

  /**
   * Retrieves current multi-asset price snapshot for all tracked global symbols.
   */
  public async getPrices(options?: RequestOptions): Promise<MarketPricesResponse> {
    return this.transport.request<MarketPricesResponse>(
      "/api/v1/market/prices",
      "GET",
      undefined,
      options
    );
  }

  /**
   * Fetches historical OHLCV candlestick bars for a given symbol and timeframe.
   *
   * @param symbol Alphanumeric instrument symbol (e.g. "XAUUSD", "BTCUSDT", "BBCA")
   * @param options Timeframe ("1m", "5m", "15m", "1h", "4h", "1d"), limit, timestamps
   */
  public async getCandles(
    symbol: string,
    options?: GetCandlesOptions
  ): Promise<CandleResponse> {
    if (!symbol || typeof symbol !== "string" || symbol.trim() === "") {
      throw new ValidationError("Symbol must be a non-empty string.", "symbol");
    }

    const cleanSymbol = symbol.trim().toUpperCase();
    const params = new URLSearchParams();
    if (options?.timeframe) params.set("resolution", options.timeframe);
    if (options?.limit) params.set("limit", String(options.limit));
    if (options?.since) params.set("since", String(options.since));
    if (options?.until) params.set("before", String(options.until));

    const query = params.toString() ? `?${params.toString()}` : "";
    const raw = await this.transport.request<any>(
      `/api/v1/market/history/${encodeURIComponent(cleanSymbol)}${query}`,
      "GET",
      undefined,
      options
    );

    const candleList = raw?.candles || raw?.items || [];
    return {
      symbol: cleanSymbol,
      timeframe: options?.timeframe || "1m",
      count: candleList.length,
      candles: candleList,
    };
  }

  /**
   * Retrieves Level 2 DOM (Depth of Market) order book for a symbol.
   */
  public async getOrderBook(symbol: string, options?: RequestOptions): Promise<OrderBook> {
    if (!symbol || typeof symbol !== "string" || symbol.trim() === "") {
      throw new ValidationError("Symbol must be a non-empty string.", "symbol");
    }

    const cleanSymbol = symbol.trim().toUpperCase();
    return this.transport.request<OrderBook>(
      `/api/v1/market/orderbook/${encodeURIComponent(cleanSymbol)}`,
      "GET",
      undefined,
      options
    );
  }
}
