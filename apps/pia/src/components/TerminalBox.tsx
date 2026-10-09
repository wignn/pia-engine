"use client";

import { useState } from "react";

type Service = "ws" | "rest" | "bot";
const endpoints: Record<Service, string> = {
  ws: "wss://api.slv.engine/v1/realtime?ticks=canonical",
  rest: "https://api.slv.engine/v1/market/gex?symbol=SPX",
  bot: "soon!"
};

export function TerminalBox() {
  const [service, setService] = useState<Service>("ws");
  const [copied, setCopied] = useState(false);

  async function copyEndpoint() {
    try {
      await navigator.clipboard.writeText(endpoints[service]);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch { setCopied(false); }
  }

  return (
    <div className="terminal-box-exact">
      <div className="terminal-nav-exact" role="tablist" aria-label="Service endpoints">
        <button
          role="tab"
          aria-selected={service === "ws"}
          aria-controls="panel-ws"
          id="tab-ws"
          className={service === "ws" ? "active" : ""}
          onClick={() => setService("ws")}
        >
          WebSocket Stream
        </button>
        <button
          role="tab"
          aria-selected={service === "rest"}
          aria-controls="panel-rest"
          id="tab-rest"
          className={service === "rest" ? "active" : ""}
          onClick={() => setService("rest")}
        >
          REST API
        </button>
        <button
          role="tab"
          aria-selected={service === "bot"}
          aria-controls="panel-bot"
          id="tab-bot"
          className={service === "bot" ? "active" : ""}
          onClick={() => setService("bot")}
        >
          Discord Bot (Rust)
        </button>
      </div>
      <div
        className="terminal-cmd-exact"
        role="tabpanel"
        id={`panel-${service}`}
        aria-labelledby={`tab-${service}`}
      >
        <code>{endpoints[service]}</code>
        <button
          className="copy-icon-btn"
          onClick={copyEndpoint}
          aria-label={copied ? "Copied endpoint to clipboard" : "Copy endpoint"}
          title={copied ? "Copied!" : "Copy endpoint"}
        >
          {copied ? "✓" : (
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          )}
        </button>
      </div>
      <span className="sr-only" aria-live="polite">
        {copied ? "Endpoint copied to clipboard" : ""}
      </span>
    </div>
  );
}
