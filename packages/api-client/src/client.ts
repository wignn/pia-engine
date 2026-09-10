/**
 * Official PIA SDK - Main Client Facade
 */

import { type PiaClientOptions, type ResolvedPiaConfig, resolveConfig } from "./config";
import { HttpTransport } from "./http/transport";
import { RealtimeClient } from "./realtime/socket";
import { MarketResource } from "./resources/market";
import { NewsResource } from "./resources/news";
import { SocialResource } from "./resources/social";
import { WsResource } from "./resources/ws";
import type { RateLimitInfo } from "./types";

export class PiaClient {
  public readonly config: Readonly<ResolvedPiaConfig>;
  private readonly transport: HttpTransport;

  /**
   * Market data & prices API resource.
   */
  public readonly market: MarketResource;

  /**
   * Social sentiment and discussions resource.
   */
  public readonly social: SocialResource;

  /**
   * Financial news resource.
   */
  public readonly news: NewsResource;

  /**
   * WebSocket ticket issuance resource.
   */
  public readonly ws: WsResource;

  /**
   * Resilient realtime WebSocket streaming client with In-Band Auth.
   */
  public readonly realtime: RealtimeClient;

  /**
   * Initializes a new PIA API Client.
   *
   * @example
   * ```typescript
   * import { PiaClient } from "@piaa/sdk";
   *
   * const client = new PiaClient({ apiKey: "wi_live_..." });
   * const prices = await client.market.getPrices();
   * ```
   */
  constructor(options: PiaClientOptions = {}) {
    this.config = Object.freeze(resolveConfig(options));
    this.transport = new HttpTransport(this.config as ResolvedPiaConfig);

    this.market = new MarketResource(this.transport);
    this.social = new SocialResource(this.transport);
    this.news = new NewsResource(this.transport);
    this.ws = new WsResource(this.transport);
    this.realtime = new RealtimeClient(this.config as ResolvedPiaConfig);
  }

  /**
   * Returns telemetry on rate limit and daily quota usage from the latest request.
   */
  public getRateLimitInfo(): RateLimitInfo {
    return this.transport.getRateLimitInfo();
  }
}
