/**
 * Official PIA SDK - WebSocket Handshake Ticket Resource
 */

import type { HttpTransport } from "../http/transport";
import type { RequestOptions, WsTicketResponse } from "../types";

export class WsResource {
  constructor(private readonly transport: HttpTransport) {}

  /**
   * Generates an ephemeral single-use WebSocket ticket for browser authentication.
   */
  public async createTicket(options?: RequestOptions): Promise<WsTicketResponse> {
    return this.transport.request<WsTicketResponse>(
      "/api/v1/ws/ticket",
      "POST",
      {},
      options
    );
  }
}
