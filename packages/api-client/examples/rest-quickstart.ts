import { PiaClient, RateLimitError } from "../src";

async function main() {
  const client = new PiaClient({
    apiKey: process.env.PIA_API_KEY || "wi_live_sample_key",
    debug: true,
  });

  try {
    // 1. Fetch Multi-Asset Price Snapshot
    console.log("Fetching live market prices...");
    const prices = await client.market.getPrices();
    console.log(`Total instruments: ${prices.total}`);

    for (const item of prices.items.slice(0, 5)) {
      console.log(`[${item.symbol}] $${item.price} (bid: ${item.bid}, ask: ${item.ask})`);
    }

    // 2. Fetch Candlesticks
    console.log("\nFetching historical 1-hour candles for XAUUSD...");
    const candles = await client.market.getCandles("XAUUSD", {
      timeframe: "1h",
      limit: 10,
    });
    console.log(`Received ${candles.count} candles.`);

    // 3. Inspect telemetry
    const telemetry = client.getRateLimitInfo();
    console.log(`\nRemaining Minute Quota: ${telemetry.remaining}/${telemetry.limit}`);
    console.log(`Remaining Daily Quota: ${telemetry.dailyRemaining}/${telemetry.dailyLimit}`);
  } catch (err) {
    if (err instanceof RateLimitError) {
      console.error(`Rate limit exceeded! Retry after ${err.retryAfterSeconds}s`);
    } else {
      console.error("API error:", err);
    }
  }
}

main().catch(console.error);
