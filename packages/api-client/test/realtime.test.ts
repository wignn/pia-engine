import { describe, expect, it } from "bun:test";
import { resolveConfig } from "../src/config";
import { RealtimeClient } from "../src/realtime/socket";

class MockWebSocket {
  public static instances: MockWebSocket[] = [];
  public readyState = 1; // OPEN
  public sentMessages: string[] = [];
  public onopen: (() => void) | null = null;
  public onmessage: ((e: { data: string }) => void) | null = null;
  public onerror: ((e: any) => void) | null = null;
  public onclose: ((e: any) => void) | null = null;

  constructor(public url: string) {
    MockWebSocket.instances.push(this);
    setTimeout(() => {
      if (this.onopen) this.onopen();
    }, 5);
  }

  send(data: string) {
    this.sentMessages.push(data);
  }

  close(code = 1000, reason = "") {
    this.readyState = 3; // CLOSED
    if (this.onclose) this.onclose({ code, reason, wasClean: true });
  }

  simulateMessage(obj: any) {
    if (this.onmessage) {
      this.onmessage({ data: JSON.stringify(obj) });
    }
  }
}

describe("RealtimeClient - In-Band Message Auth & Streaming", () => {
  it("automatically sends In-Band Auth frame upon TCP open", async () => {
    MockWebSocket.instances = [];

    const config = resolveConfig({
      apiKey: "wi_live_secretkey123",
      WebSocket: MockWebSocket as any,
    });

    const client = new RealtimeClient(config);

    let connectFired = false;
    client.on("connect", () => {
      connectFired = true;
    });

    client.connect();

    // Wait for mock onopen
    await new Promise((resolve) => setTimeout(resolve, 20));

    expect(connectFired).toBe(true);
    const mockWs = MockWebSocket.instances[0];
    expect(mockWs).toBeDefined();

    // Verify first sent frame was In-Band Auth
    expect(mockWs.sentMessages.length).toBeGreaterThanOrEqual(1);
    const authFrame = JSON.parse(mockWs.sentMessages[0]);
    expect(authFrame.action).toBe("auth");
    expect(authFrame.api_key).toBe("wi_live_secretkey123");

    client.disconnect();
  });

  it("emits authenticated event and sends subscriptions once authorized", async () => {
    MockWebSocket.instances = [];

    const config = resolveConfig({
      apiKey: "test_key",
      WebSocket: MockWebSocket as any,
    });

    const client = new RealtimeClient(config);
    client.subscribe(["XAUUSD", "BTCUSDT"]);

    let authPayload: any = null;
    client.on("authenticated", (data) => {
      authPayload = data;
    });

    client.connect();
    await new Promise((resolve) => setTimeout(resolve, 20));

    const mockWs = MockWebSocket.instances[0];

    // Simulate backend sending auth confirmation
    mockWs.simulateMessage({
      event: "authenticated",
      data: { plan: "starter", ws_connections_max: 3 },
    });

    expect(authPayload).toBeDefined();
    expect(authPayload.plan).toBe("starter");

    // Check that subscriptions were flushed
    const subMessage = mockWs.sentMessages.find((m) => m.includes('"action":"subscribe"'));
    expect(subMessage).toBeDefined();
    const parsedSub = JSON.parse(subMessage!);
    expect(parsedSub.symbols).toContain("XAUUSD");
    expect(parsedSub.symbols).toContain("BTCUSDT");

    client.disconnect();
  });

  it("dispatches live tick events to listeners", async () => {
    MockWebSocket.instances = [];

    const config = resolveConfig({
      apiKey: "test_key",
      WebSocket: MockWebSocket as any,
    });

    const client = new RealtimeClient(config);
    let capturedTick: any = null;

    client.on("tick", (tick) => {
      capturedTick = tick;
    });

    client.connect();
    await new Promise((resolve) => setTimeout(resolve, 20));

    const mockWs = MockWebSocket.instances[0];
    mockWs.simulateMessage({
      event: "market.trade",
      data: {
        tick: {
          symbol: "XAUUSD",
          price: 2510.25,
          bid: 2510.2,
          ask: 2510.3,
        },
      },
    });

    expect(capturedTick).toBeDefined();
    expect(capturedTick.symbol).toBe("XAUUSD");
    expect(capturedTick.price).toBe(2510.25);

    client.disconnect();
  });
});
