"use client";

import React, { useState, useEffect, useCallback } from "react";
import { Bell, Plus, Trash2, CheckCircle2, AlertCircle, Volume2 } from "lucide-react";
import { PriceAlert } from "@/types";

interface AlertsPanelProps {
  symbol: string;
  livePrice: number | null;
  digits: number;
}

// Play Web Audio frequency chime on alert trigger
function playAlertChime() {
  try {
    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) return;
    const ctx = new AudioCtx();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(880, ctx.currentTime); // A5
    osc.frequency.exponentialRampToValueAtTime(1320, ctx.currentTime + 0.15); // E6

    gain.gain.setValueAtTime(0.3, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.4);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.4);
  } catch {
    // Ignore if audio context blocked by browser autoplay policy
  }
}

export const AlertsPanel: React.FC<AlertsPanelProps> = ({
  symbol,
  livePrice,
  digits,
}) => {
  const [alerts, setAlerts] = useState<PriceAlert[]>([]);
  const [targetPriceInput, setTargetPriceInput] = useState<string>("");
  const [condition, setCondition] = useState<"crossing_up" | "crossing_down">("crossing_up");

  // Load alerts from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem("atlsd_price_alerts");
      if (stored) {
        setAlerts(JSON.parse(stored));
      }
    } catch {
      // Ignore parse error
    }
  }, []);

  // Save alerts to localStorage
  const saveAlerts = (newAlerts: PriceAlert[]) => {
    setAlerts(newAlerts);
    try {
      localStorage.setItem("atlsd_price_alerts", JSON.stringify(newAlerts));
    } catch {
      // Ignore write error
    }
  };

  // Sync initial input price when live price changes if input is empty
  useEffect(() => {
    if (livePrice && !targetPriceInput) {
      setTargetPriceInput(livePrice.toFixed(digits));
    }
  }, [livePrice, digits, targetPriceInput]);

  // Monitor livePrice against alerts
  useEffect(() => {
    if (!livePrice || livePrice <= 0) return;

    let triggeredAny = false;
    const updated = alerts.map((alert) => {
      if (alert.triggered || alert.symbol !== symbol) return alert;

      const crossedUp = alert.condition === "crossing_up" && livePrice >= alert.targetPrice;
      const crossedDown = alert.condition === "crossing_down" && livePrice <= alert.targetPrice;

      if (crossedUp || crossedDown) {
        triggeredAny = true;
        return {
          ...alert,
          triggered: true,
          triggeredAt: Date.now(),
        };
      }
      return alert;
    });

    if (triggeredAny) {
      playAlertChime();
      saveAlerts(updated);
    }
  }, [livePrice, symbol, alerts]);

  const handleCreateAlert = (e: React.FormEvent) => {
    e.preventDefault();
    const target = parseFloat(targetPriceInput);
    if (isNaN(target) || target <= 0 || !livePrice) return;

    const newAlert: PriceAlert = {
      id: "alert_" + Date.now() + "_" + Math.random().toString(36).substring(2, 6),
      symbol,
      targetPrice: target,
      condition: target >= livePrice ? "crossing_up" : "crossing_down",
      createdPrice: livePrice,
      createdAt: Date.now(),
      triggered: false,
    };

    saveAlerts([newAlert, ...alerts]);
  };

  const handleDeleteAlert = (id: string) => {
    saveAlerts(alerts.filter((a) => a.id !== id));
  };

  const handlePreset = (percentDelta: number) => {
    if (!livePrice) return;
    const target = livePrice * (1 + percentDelta / 100);
    setTargetPriceInput(target.toFixed(digits));
    setCondition(percentDelta >= 0 ? "crossing_up" : "crossing_down");
  };

  const symbolAlerts = alerts.filter((a) => a.symbol === symbol);
  const activeAlerts = symbolAlerts.filter((a) => !a.triggered);
  const triggeredAlerts = symbolAlerts.filter((a) => a.triggered);

  return (
    <div className="flex h-full flex-col bg-[#1e222d] border-l border-[#2a2e39] text-xs text-[#d1d4dc] overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[#2a2e39] px-4 py-3 shrink-0">
        <div className="flex items-center gap-2">
          <Bell className="w-4 h-4 text-[#2962ff]" />
          <span className="font-bold text-white text-sm">Price Alerts</span>
        </div>
        <span className="font-mono text-[10px] text-[#787b86] uppercase bg-[#141722] px-2 py-0.5 rounded border border-[#2a2e39]">
          {symbol}
        </span>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Create Alert Form */}
        <form onSubmit={handleCreateAlert} className="space-y-3 rounded-lg border border-[#2a2e39] bg-[#141722] p-3">
          <div className="flex items-center justify-between text-[11px]">
            <span className="text-[#787b86]">Current Price:</span>
            <span className="font-mono font-bold text-white">
              {livePrice ? livePrice.toFixed(digits) : "—"}
            </span>
          </div>

          <div>
            <label className="block text-[11px] font-semibold text-[#787b86] mb-1">
              Target Price ({symbol})
            </label>
            <input
              type="number"
              step="any"
              value={targetPriceInput}
              onChange={(e) => setTargetPriceInput(e.target.value)}
              placeholder="Enter target price..."
              className="w-full rounded border border-[#2a2e39] bg-[#1e222d] px-2.5 py-1.5 font-mono text-xs text-white focus:border-[#2962ff] focus:outline-none"
            />
          </div>

          {/* Quick % Offset Presets */}
          <div className="flex gap-1.5">
            {[0.5, 1.0, -0.5, -1.0].map((pct) => (
              <button
                key={pct}
                type="button"
                onClick={() => handlePreset(pct)}
                className={`flex-1 rounded py-1 text-[10px] font-mono font-bold transition-colors ${
                  pct > 0
                    ? "bg-[#089981]/15 text-[#089981] hover:bg-[#089981]/25"
                    : "bg-[#f23645]/15 text-[#f23645] hover:bg-[#f23645]/25"
                }`}
              >
                {pct > 0 ? `+${pct}%` : `${pct}%`}
              </button>
            ))}
          </div>

          <button
            type="submit"
            disabled={!livePrice || !targetPriceInput}
            className="flex w-full items-center justify-center gap-1.5 rounded bg-[#2962ff] py-2 text-xs font-bold text-white hover:bg-[#1e53e5] transition-colors disabled:opacity-50"
          >
            <Plus className="w-3.5 h-3.5" /> Set Alert
          </button>
        </form>

        {/* Active Alerts List */}
        <div>
          <div className="text-[11px] font-bold uppercase tracking-wider text-[#787b86] mb-2">
            Active Alerts ({activeAlerts.length})
          </div>
          {activeAlerts.length === 0 ? (
            <div className="rounded border border-[#2a2e39] bg-[#181b27] p-3 text-center text-[11px] text-[#787b86]">
              No active alerts for {symbol}.
            </div>
          ) : (
            <div className="space-y-2">
              {activeAlerts.map((alert) => {
                const diff = livePrice ? alert.targetPrice - livePrice : 0;
                const diffPct = livePrice ? (diff / livePrice) * 100 : 0;
                return (
                  <div
                    key={alert.id}
                    className="flex items-center justify-between rounded border border-[#2a2e39] bg-[#181b27] p-2.5 transition-colors hover:border-[#363a45]"
                  >
                    <div>
                      <div className="flex items-center gap-1.5 font-mono font-bold text-white">
                        <span>{alert.targetPrice.toFixed(digits)}</span>
                        <span className="text-[9px] font-normal text-[#787b86]">
                          ({alert.condition === "crossing_up" ? "≥ Target" : "≤ Target"})
                        </span>
                      </div>
                      <div className="text-[10px] text-[#787b86] mt-0.5 font-mono">
                        Distance: {diff >= 0 ? "+" : ""}{diff.toFixed(digits)} ({diffPct >= 0 ? "+" : ""}{diffPct.toFixed(2)}%)
                      </div>
                    </div>
                    <button
                      onClick={() => handleDeleteAlert(alert.id)}
                      className="p-1 rounded text-[#787b86] hover:text-[#f23645] hover:bg-[#2a2e39] transition-colors"
                      title="Delete Alert"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Triggered Alerts History */}
        {triggeredAlerts.length > 0 && (
          <div>
            <div className="text-[11px] font-bold uppercase tracking-wider text-[#787b86] mb-2">
              Triggered History ({triggeredAlerts.length})
            </div>
            <div className="space-y-1.5">
              {triggeredAlerts.map((alert) => (
                <div
                  key={alert.id}
                  className="flex items-center justify-between rounded border border-[#2a2e39]/60 bg-[#141722] p-2 text-[11px]"
                >
                  <div className="flex items-center gap-2">
                    <CheckCircle2 className="w-3.5 h-3.5 text-[#089981]" />
                    <div>
                      <span className="font-mono font-semibold text-white">
                        {alert.targetPrice.toFixed(digits)}
                      </span>
                      <div className="text-[9px] text-[#787b86]">
                        {alert.triggeredAt ? new Date(alert.triggeredAt).toLocaleTimeString() : "Triggered"}
                      </div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleDeleteAlert(alert.id)}
                    className="p-1 text-[#787b86] hover:text-[#f23645]"
                    title="Remove"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
