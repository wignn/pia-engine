/**
 * Official PIA SDK - Typed Event Emitter for Realtime Streaming
 */

import type { MarketPrice } from "../types";

export interface RealtimeEvents {
  connect: () => void;
  authenticated: (data: { user_id?: string; plan?: string; ws_connections_max?: number }) => void;
  disconnect: (reason: { code: number; reason: string; wasClean: boolean }) => void;
  error: (error: Error) => void;
  tick: (tick: MarketPrice) => void;
  snapshot: (snapshot: { total: number; items: MarketPrice[] }) => void;
  raw: (data: unknown) => void;
}

export type EventKey = keyof RealtimeEvents;
export type Listener<K extends EventKey> = RealtimeEvents[K];

export class TypedEventEmitter {
  private readonly listeners: Map<EventKey, Set<Function>> = new Map();

  public on<K extends EventKey>(event: K, listener: Listener<K>): this {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(listener);
    return this;
  }

  public off<K extends EventKey>(event: K, listener: Listener<K>): this {
    const set = this.listeners.get(event);
    if (set) {
      set.delete(listener);
      if (set.size === 0) {
        this.listeners.delete(event);
      }
    }
    return this;
  }

  public emit<K extends EventKey>(event: K, ...args: Parameters<RealtimeEvents[K]>): boolean {
    const set = this.listeners.get(event);
    if (!set || set.size === 0) return false;

    for (const listener of Array.from(set)) {
      try {
        (listener as any)(...args);
      } catch (err) {
        console.error(`[PIA-SDK] Unhandled error in '${event}' listener:`, err);
      }
    }
    return true;
  }

  public removeAllListeners(event?: EventKey): this {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
    return this;
  }
}
