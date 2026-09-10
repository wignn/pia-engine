/**
 * Official PIA SDK - News Intelligence API Resource
 */

import type { HttpTransport } from "../http/transport";
import type { GetNewsOptions, NewsFeedResponse } from "../types";

export class NewsResource {
  constructor(private readonly transport: HttpTransport) {}

  /**
   * Fetches curated financial and macro news articles.
   */
  public async getNews(options?: GetNewsOptions): Promise<NewsFeedResponse> {
    const params = new URLSearchParams();
    if (options?.symbols && options.symbols.length > 0) {
      params.set("symbols", options.symbols.map((s) => s.trim().toUpperCase()).join(","));
    }
    if (options?.limit) params.set("limit", String(options.limit));

    const query = params.toString() ? `?${params.toString()}` : "";
    return this.transport.request<NewsFeedResponse>(
      `/api/v1/news${query}`,
      "GET",
      undefined,
      options
    );
  }
}
