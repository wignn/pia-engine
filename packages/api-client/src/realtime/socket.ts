/**
 * Official PIA SDK - Resilient Realtime WebSocket Client with In-Band Auth & Auto-Reconnect
 */

import type { ResolvedPiaConfig } from "../config";
import { AuthenticationError, ConfigurationError, NetworkError } from "../errors";
import { TypedEventEmitter } from "./events";

export type SocketState =
  | "DISCONNECTED"
  | "CONNECTING"
  | "AUTHENTICATING"
  | "AUTHENTICATED"
  | "CLOSING";

export class RealtimeClient extends TypedEventEmitter {
  private ws: any = null;
  private state: SocketState = "DISCONNECTED";
  private readonly subscribedSymbols: Set<string> = new Set();
  private reconnectAttempt = 0;
  private reconnectTimer: any = null;
  private isExplicitlyClosed = false;
  private pingIntervalTimer: any = null;

  constructor(private readonly config: ResolvedPiaConfig) {
    super();
  }

  /**
   * Returns current socket state.
   */
  public getState(): SocketState {
    return this.state;
  }

  /**
   * Connects to the Realtime WebSocket stream and performs In-Band Authentication.
   */
  public connect(): this {
    if (this.state === "CONNECTING" || this.state === "AUTHENTICATING" || this.state === "AUTHENTICATED") {
      this.config.logger.debug("Socket already active or connecting.");
      return this;
    }

    this.isExplicitlyClosed = false;
    this.initiateConnection();
    return this;
  }

  private initiateConnection(): void {
    const WsConstructor: any =
      this.config.WebSocket ||
      (typeof globalThis !== "undefined" && (globalThis as any).WebSocket);

    if (!WsConstructor) {
      throw new ConfigurationError(
        "WebSocket implementation not available. In Node.js environments, pass { WebSocket: require('ws') } in options."
      );
    }

    this.state = "CONNECTING";
    this.config.logger.info(`Connecting to Realtime WebSocket: ${this.config.wsUrl}`);

    try {
      this.ws = new WsConstructor(this.config.wsUrl);
    } catch (err) {
      this.handleSocketFailure(err);
      return;
    }

    this.ws.onopen = () => {
      this.config.logger.info("WebSocket TCP connection opened. Initiating In-Band Auth...");
      this.state = "AUTHENTICATING";
      this.reconnectAttempt = 0;
      this.emit("connect");

      // Send In-Band Auth frame immediately
      this.sendRaw({
        action: "auth",
        api_key: this.config.apiKey,
      });

      this.startHeartbeat();
    };

    this.ws.onmessage = (event: any) => {
      this.handleIncomingMessage(event.data);
    };

    this.ws.onerror = (event: any) => {
      const err = new NetworkError(
        "WebSocket connection error",
        event?.message || event?.error || event
      );
      this.config.logger.error("WebSocket error:", err.message);
      this.emit("error", err);
    };

    this.ws.onclose = (event: any) => {
      const code = event?.code ?? 1006;
      const reason = event?.reason ?? "";
      const wasClean = event?.wasClean ?? false;

      this.stopHeartbeat();
      this.state = "DISCONNECTED";
      this.ws = null;

      this.config.logger.warn(`WebSocket closed: code=${code}, reason=${reason}`);
      this.emit("disconnect", { code, reason, wasClean });

      // Code 4001: Unauthorized (Bad API key or rejected) - do not reconnect loop
      if (code === 4001 || reason.toLowerCase().includes("unauthorized")) {
        this.emit(
          "error",
          new AuthenticationError("WebSocket authentication rejected by server.")
        );
        return;
      }

      if (!this.isExplicitlyClosed) {
        this.scheduleReconnect();
      }
    };
  }

  private handleIncomingMessage(rawPayload: any): void {
    let messageText: string;
    if (typeof rawPayload === "string") {
      messageText = rawPayload;
    } else if (rawPayload instanceof ArrayBuffer || (typeof Buffer !== "undefined" && Buffer && Buffer.isBuffer(rawPayload))) {
      messageText = new TextDecoder().decode(rawPayload);
    } else {
      messageText = String(rawPayload);
    }

    let payload: any;
    try {
      payload = JSON.parse(messageText);
    } catch {
      this.emit("raw", messageText);
      return;
    }

    this.emit("raw", payload);

    // 1. Auth confirmation
    if (payload.event === "authenticated") {
      this.state = "AUTHENTICATED";
      this.config.logger.info("WebSocket authenticated successfully.");
      this.emit("authenticated", payload.data || {});

      // Resubscribe to tracked symbols
      if (this.subscribedSymbols.size > 0) {
        this.flushSubscriptions();
      }
      return;
    }

    // 2. Market price tick
    if (payload.event === "market.trade" || payload.event === "market.tick") {
      const tick = payload.data?.tick || payload.data;
      if (tick) {
        this.emit("tick", tick);
      }
      return;
    }

    // 3. Initial market snapshot
    if (payload.event === "market.snapshot" && payload.data) {
      this.emit("snapshot", payload.data);
      return;
    }

    // 4. Server error frame
    if (payload.error) {
      if (payload.error === "unauthorized" || payload.error === 401) {
        this.emit(
          "error",
          new AuthenticationError(payload.message || "Unauthorized WebSocket message.")
        );
      } else {
        this.emit(
          "error",
          new NetworkError(payload.message || `Server reported error: ${payload.error}`)
        );
      }
    }
  }

  /**
   * Subscribes to realtime updates for one or more symbols.
   */
  public subscribe(symbols: string | string[]): this {
    const list = Array.isArray(symbols) ? symbols : [symbols];
    const newSymbols: string[] = [];

    for (const sym of list) {
      const clean = sym.trim().toUpperCase();
      if (clean && !this.subscribedSymbols.has(clean)) {
        this.subscribedSymbols.add(clean);
        newSymbols.push(clean);
      }
    }

    if (newSymbols.length > 0 && this.state === "AUTHENTICATED") {
      this.sendRaw({
        action: "subscribe",
        symbols: newSymbols,
      });
    }

    return this;
  }

  /**
   * Unsubscribes from realtime updates for one or more symbols.
   */
  public unsubscribe(symbols: string | string[]): this {
    const list = Array.isArray(symbols) ? symbols : [symbols];
    const removed: string[] = [];

    for (const sym of list) {
      const clean = sym.trim().toUpperCase();
      if (this.subscribedSymbols.delete(clean)) {
        removed.push(clean);
      }
    }

    if (removed.length > 0 && this.state === "AUTHENTICATED") {
      this.sendRaw({
        action: "unsubscribe",
        symbols: removed,
      });
    }

    return this;
  }

  private flushSubscriptions(): void {
    if (this.subscribedSymbols.size === 0) return;
    this.sendRaw({
      action: "subscribe",
      symbols: Array.from(this.subscribedSymbols),
    });
  }

  private sendRaw(data: unknown): boolean {
    if (!this.ws || this.ws.readyState !== 1 /* OPEN */) {
      return false;
    }
    try {
      this.ws.send(JSON.stringify(data));
      return true;
    } catch (err) {
      this.config.logger.error("Failed to send WebSocket message:", err);
      return false;
    }
  }

  private scheduleReconnect(): void {
    this.reconnectAttempt++;
    // Exponential backoff: 1s, 2s, 4s, 8s, capped at 15s with jitter
    const delay = Math.min(1000 * Math.pow(1.5, this.reconnectAttempt - 1), 15000);
    const jitteredDelay = Math.round(delay * (0.8 + Math.random() * 0.4));

    this.config.logger.info(
      `Reconnecting in ${jitteredDelay}ms (Attempt ${this.reconnectAttempt})...`
    );

    clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(() => {
      if (!this.isExplicitlyClosed) {
        this.initiateConnection();
      }
    }, jitteredDelay);
  }

  private handleSocketFailure(err: unknown): void {
    this.state = "DISCONNECTED";
    this.ws = null;
    const netErr = new NetworkError("Failed to initiate WebSocket connection", err);
    this.emit("error", netErr);
    if (!this.isExplicitlyClosed) {
      this.scheduleReconnect();
    }
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    // Ping every 30 seconds
    this.pingIntervalTimer = setInterval(() => {
      if (this.state === "AUTHENTICATED") {
        this.sendRaw({ action: "ping" });
      }
    }, 30_000);
  }

  private stopHeartbeat(): void {
    if (this.pingIntervalTimer) {
      clearInterval(this.pingIntervalTimer);
      this.pingIntervalTimer = null;
    }
  }

  /**
   * Explicitly closes the WebSocket connection and prevents auto-reconnection.
   */
  public disconnect(): void {
    this.isExplicitlyClosed = true;
    this.stopHeartbeat();
    clearTimeout(this.reconnectTimer);

    if (this.ws) {
      this.state = "CLOSING";
      try {
        this.ws.close(1000, "Client disconnect");
      } catch {
        // Ignored
      }
      this.ws = null;
    }

    this.state = "DISCONNECTED";
  }
}
