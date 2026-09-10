import { PiaClient } from "../src";

async function main() {
  const client = new PiaClient({
    apiKey: process.env.PIA_API_KEY || "wi_live_sample_key",
  });

  // 1. Listen for connection events
  client.realtime.on("connect", () => {
    console.log("WebSocket connected. Authenticating via In-Band frame...");
  });

  // 2. Listen for authentication confirmation
  client.realtime.on("authenticated", (info) => {
    console.log("WebSocket authenticated successfully!", info);

    // 3. Subscribe to target market symbols
    console.log("Subscribing to XAUUSD and BTCUSDT...");
    client.realtime.subscribe(["XAUUSD", "BTCUSDT"]);
  });

  // 4. Handle incoming live market ticks
  client.realtime.on("tick", (tick) => {
    console.log(`[TICK] ${tick.symbol}: $${tick.price} (bid: ${tick.bid}, ask: ${tick.ask})`);
  });

  // 5. Handle disconnections and errors
  client.realtime.on("disconnect", (reason) => {
    console.warn("Disconnected:", reason);
  });

  client.realtime.on("error", (err) => {
    console.error("Realtime error:", err.message);
  });

  // Start connection
  client.realtime.connect();
}

main().catch(console.error);
