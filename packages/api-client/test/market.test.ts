import { describe, expect, it } from "bun:test";
import { PiaClient, ValidationError } from "../src";

describe("Market Resource", () => {
  it("fetches multi-asset prices snapshot", async () => {
    const mockFetch = async (url: string) => {
      expect(url).toContain("/api/v1/market/prices");
      return new Response(
        JSON.stringify({
          total: 2,
          timestamp: 1726000000,
          items: [
            { symbol: "XAUUSD", price: 2500.5, bid: 2500.4, ask: 2500.6 },
            { symbol: "BTCUSDT", price: 60000.0, bid: 59999.0, ask: 60001.0 },
          ],
        }),
        { status: 200 }
      );
    };

    const client = new PiaClient({ apiKey: "test", fetch: mockFetch as any });
    const res = await client.market.getPrices();

    expect(res.total).toBe(2);
    expect(res.items[0].symbol).toBe("XAUUSD");
    expect(res.items[1].price).toBe(60000.0);
  });

  it("validates symbol when fetching candlestick bars", async () => {
    const client = new PiaClient({ apiKey: "test", fetch: (async () => {}) as any });

    await expect(client.market.getCandles("")).rejects.toThrow(ValidationError);
    await expect((client.market.getCandles as any)()).rejects.toThrow(ValidationError);
  });

  it("builds correct URL query params for candlestick request", async () => {
    let requestedUrl = "";
    const mockFetch = async (url: string) => {
      requestedUrl = url;
      return new Response(
        JSON.stringify({
          symbol: "BTCUSDT",
          timeframe: "1h",
          count: 1,
          candles: [{ time: 1000, open: 1, high: 2, low: 0.5, close: 1.5, volume: 100 }],
        }),
        { status: 200 }
      );
    };

    const client = new PiaClient({ apiKey: "test", fetch: mockFetch as any });
    const res = await client.market.getCandles("btcusdt", {
      timeframe: "1h",
      limit: 100,
      since: 1720000000,
    });

    expect(requestedUrl).toContain("/api/v1/market/history/BTCUSDT");
    expect(requestedUrl).toContain("resolution=1h");
    expect(requestedUrl).toContain("limit=100");
    expect(requestedUrl).toContain("since=1720000000");
    expect(res.symbol).toBe("BTCUSDT");
  });
});
