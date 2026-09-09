"use client";

import React, { useState, useEffect, useCallback } from "react";
import { Bell, Plus, Trash2, CheckCircle2, AlertCircle, Volume2 } from "lucide-react";
import { PriceAlert } from "@/types";

interface AlertsPanelProps {
  symbol: string;
  livePrice: number | null;
  digits: number;
  theme?: "dark" | "light";
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
  theme = "dark",
}) => {
  const isLight = theme === "light";
  const [alerts, setAlerts] = useState<PriceAlert[]>([]);
  const [targetInput, setTargetInput] = useState<string>("");
  const [conditionInput, setConditionInput] = useState<"crossing_up" | "crossing_down">("crossing_up");
  const [showAddForm, setShowAddForm] = useState(false);

  // Load alerts from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem("atlsd_price_alerts");
      if (stored) {
        setAlerts(JSON.parse(stored));
      }
    } catch {
      // ignore
    }
  }, []);

  // Save alerts to localStorage
  const saveAlerts = (newAlerts: PriceAlert[]) => {
    setAlerts(newAlerts);
    try {
      localStorage.setItem("atlsd_price_alerts", JSON.stringify(newAlerts));
    } catch {
      // ignore
    }
  };

  // Check alert conditions against live price
  useEffect(() => {
    if (!livePrice || livePrice <= 0) return;

    let hasUpdates = false;
    const updated = alerts.map((a) => {
      if (a.triggered || a.symbol !== symbol) return a;

      let isTriggered = false;
      if (a.condition === "crossing_up" && livePrice >= a.targetPrice && a.createdPrice < a.targetPrice) {
        isTriggered = true;
      } else if (a.condition === "crossing_down" && livePrice <= a.targetPrice && a.createdPrice > a.targetPrice) {
        isTriggered = true;
      }

      if (isTriggered) {
        hasUpdates = true;
        playAlertChime();
        return { ...a, triggered: true, triggeredAt: Date.now() };
      }
      return a;
    });

    if (hasUpdates) {
      saveAlerts(updated);
    }
  }, [livePrice, symbol, alerts]);

  const handleCreateAlert = (e: React.FormEvent) => {
    e.preventDefault();
    const val = parseFloat(targetInput);
    if (isNaN(val) || val <= 0) return;

    const newAlert: PriceAlert = {
      id: `alert-${Date.now()}`,
      symbol,
      targetPrice: val,
      condition: conditionInput,
      createdPrice: livePrice || val,
      createdAt: Date.now(),
      triggered: false,
    };

    saveAlerts([newAlert, ...alerts]);
    setTargetInput("");
    setShowAddForm(false);
  };

  const handleDeleteAlert = (id: string) => {
    saveAlerts(alerts.filter((a) => a.id !== id));
  };

  const symbolAlerts = alerts.filter((a) => a.symbol === symbol);

  return (
    <aside
      className={`w-full flex flex-col h-full select-none text-xs overflow-hidden transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Header */}
      <div
        className={`h-[44px] border-b flex items-center justify-between px-4 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-2">
          <Bell className="w-4 h-4 text-[#2962ff]" />
          <span className={`font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>Price Alerts</span>
        </div>
        <button
          onClick={() => setShowAddForm((v) => !v)}
          className="flex items-center gap-1 bg-[#2962ff] text-white px-2 py-1 rounded text-[11px] font-semibold hover:bg-[#1e4bd8] transition-colors cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>New Alert</span>
        </button>
      </div>

      {/* Quick Add Form */}
      {showAddForm && (
        <form
          onSubmit={handleCreateAlert}
          className={`p-3 border-b flex flex-col gap-2.5 shrink-0 ${
            isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="font-semibold text-[11px] text-[#787b86]">Symbol:</span>
            <span className={`font-mono font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>{symbol}</span>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[10px] uppercase font-bold text-[#787b86]">Condition</label>
            <select
              value={conditionInput}
              onChange={(e) => setConditionInput(e.target.value as any)}
              className={`rounded px-2 py-1.5 border text-xs focus:outline-hidden ${
                isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
              }`}
            >
              <option value="crossing_up">Crossing Up (&gt;=)</option>
              <option value="crossing_down">Crossing Down (&lt;=)</option>
            </select>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[10px] uppercase font-bold text-[#787b86]">Target Price</label>
            <input
              type="number"
              step="any"
              placeholder={livePrice ? livePrice.toFixed(digits) : "0.00"}
              value={targetInput}
              onChange={(e) => setTargetInput(e.target.value)}
              className={`rounded px-2 py-1.5 border text-xs font-mono focus:outline-hidden ${
                isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
              }`}
              autoFocus
            />
          </div>

          <div className="flex items-center gap-2 mt-1">
            <button
              type="submit"
              className="flex-1 bg-[#2962ff] text-white py-1.5 rounded font-bold hover:bg-[#1e4bd8] transition-colors cursor-pointer"
            >
              Set Alert
            </button>
            <button
              type="button"
              onClick={() => setShowAddForm(false)}
              className={`px-3 py-1.5 rounded border transition-colors cursor-pointer ${
                isLight ? "border-[#e0e3eb] text-[#5d606b] hover:bg-[#e0e3eb]" : "border-[#2a2e39] text-[#787b86] hover:bg-[#2a2e39]"
              }`}
            >
              Cancel
            </button>
          </div>
        </form>
      )}

      {/* Alerts List */}
      <div className={`flex-1 overflow-y-auto divide-y ${isLight ? "divide-[#e0e3eb]" : "divide-[#2a2e39]/50"}`}>
        {symbolAlerts.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">
            <AlertCircle className="w-8 h-8 mx-auto mb-2 opacity-40" />
            <p className="font-medium">No alerts for {symbol}</p>
            <p className="text-[11px] opacity-75 mt-1">Click "New Alert" above to create one</p>
          </div>
        ) : (
          symbolAlerts.map((a) => (
            <div
              key={a.id}
              className={`p-3 flex items-center justify-between transition-colors ${
                a.triggered
                  ? "bg-amber-500/10 border-l-2 border-amber-500"
                  : isLight
                  ? "hover:bg-[#f8f9fc]"
                  : "hover:bg-[#262b37]"
              }`}
            >
              <div className="flex flex-col gap-0.5">
                <div className="flex items-center gap-1.5">
                  <span className={`font-mono font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>
                    {a.targetPrice.toFixed(digits)}
                  </span>
                  <span
                    className={`text-[9px] font-mono px-1.5 py-0.2 rounded font-bold uppercase ${
                      a.condition === "crossing_up"
                        ? "bg-[#089981]/15 text-[#089981]"
                        : "bg-[#f23645]/15 text-[#f23645]"
                    }`}
                  >
                    {a.condition === "crossing_up" ? "Cross Up" : "Cross Down"}
                  </span>
                </div>
                <div className="text-[10px] text-[#787b86]">
                  {a.triggered ? (
                    <span className="text-amber-500 font-semibold flex items-center gap-1">
                      <CheckCircle2 className="w-3 h-3" />
                      Triggered
                    </span>
                  ) : (
                    `Active · Created at ${a.createdPrice.toFixed(digits)}`
                  )}
                </div>
              </div>

              <button
                onClick={() => handleDeleteAlert(a.id)}
                className={`p-1.5 rounded transition-colors cursor-pointer ${
                  isLight ? "text-[#5d606b] hover:text-[#f23645] hover:bg-[#f0f3fa]" : "text-[#787b86] hover:text-[#f23645] hover:bg-[#2a2e39]"
                }`}
                title="Delete Alert"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ))
        )}
      </div>
    </aside>
  );
};
