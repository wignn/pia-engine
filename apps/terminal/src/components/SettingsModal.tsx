"use client";

import React, { useState } from "react";
import { X, Moon, Sun, Volume2, VolumeX, Grid, Clock, Check, RotateCcw } from "lucide-react";
import { TerminalSettings, Timeframe } from "@/types";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  settings: TerminalSettings;
  onSaveSettings: (settings: TerminalSettings) => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  settings,
  onSaveSettings,
}) => {
  const [form, setForm] = useState<TerminalSettings>(settings);

  if (!isOpen) return null;

  const isLight = form.theme === "light";

  const handleSave = () => {
    onSaveSettings(form);
    onClose();
  };

  const handleReset = () => {
    const defaultSettings: TerminalSettings = {
      theme: "dark",
      upColor: "#089981",
      downColor: "#f23645",
      gridVisible: true,
      timezone: "UTC",
      audioAlerts: true,
      defaultTimeframe: "15m",
    };
    setForm(defaultSettings);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 backdrop-blur-xs p-4 select-none"
      onClick={onClose}
    >
      <div
        className={`w-full max-w-[520px] rounded-xl border shadow-2xl overflow-hidden flex flex-col transition-colors ${
          isLight
            ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
            : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className={`flex items-center justify-between px-5 py-3.5 border-b ${
            isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
          }`}
        >
          <div className="flex items-center gap-2">
            <span className="font-bold text-sm">Chart & Terminal Settings</span>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 text-muted-foreground transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form Body */}
        <div className="p-5 space-y-5 text-xs overflow-y-auto max-h-[70vh]">
          {/* Theme Mode Toggle */}
          <div>
            <label className="font-bold uppercase tracking-wider text-[11px] opacity-70 block mb-2">
              Appearance / Theme
            </label>
            <div className="grid grid-cols-2 gap-3">
              <button
                type="button"
                onClick={() => setForm({ ...form, theme: "dark" })}
                className={`flex items-center justify-center gap-2 py-2.5 px-3 rounded-lg border font-bold cursor-pointer transition-all ${
                  form.theme === "dark"
                    ? "border-[#2962ff] bg-[#2962ff]/15 text-[#2962ff] ring-1 ring-[#2962ff]"
                    : "border-[#2a2e39] bg-[#131722] text-[#787b86] hover:text-white hover:border-[#363a45]"
                }`}
              >
                <Moon className="w-4 h-4" />
                <span>Dark Mode</span>
              </button>
              <button
                type="button"
                onClick={() => setForm({ ...form, theme: "light" })}
                className={`flex items-center justify-center gap-2 py-2.5 px-3 rounded-lg border font-bold cursor-pointer transition-all ${
                  form.theme === "light"
                    ? "border-[#2962ff] bg-[#2962ff]/15 text-[#2962ff] ring-1 ring-[#2962ff]"
                    : "border-[#e0e3eb] bg-[#f0f3fa] text-[#5d606b] hover:text-black hover:border-[#b2b5be]"
                }`}
              >
                <Sun className="w-4 h-4" />
                <span>Light Mode</span>
              </button>
            </div>
          </div>

          {/* Candle Colors */}
          <div>
            <label className="font-bold uppercase tracking-wider text-[11px] opacity-70 block mb-2">
              Candle Palette
            </label>
            <div className="grid grid-cols-2 gap-3">
              <div
                className={`flex items-center justify-between p-3 rounded-lg border ${
                  isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
                }`}
              >
                <span className="font-semibold">Bullish (Up)</span>
                <div className="flex items-center gap-2">
                  <input
                    type="color"
                    value={form.upColor}
                    onChange={(e) => setForm({ ...form, upColor: e.target.value })}
                    className="w-7 h-7 rounded border-none cursor-pointer bg-transparent"
                  />
                  <span className="font-mono text-[11px] font-bold">{form.upColor}</span>
                </div>
              </div>

              <div
                className={`flex items-center justify-between p-3 rounded-lg border ${
                  isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
                }`}
              >
                <span className="font-semibold">Bearish (Down)</span>
                <div className="flex items-center gap-2">
                  <input
                    type="color"
                    value={form.downColor}
                    onChange={(e) => setForm({ ...form, downColor: e.target.value })}
                    className="w-7 h-7 rounded border-none cursor-pointer bg-transparent"
                  />
                  <span className="font-mono text-[11px] font-bold">{form.downColor}</span>
                </div>
              </div>
            </div>
          </div>

          {/* Grid Lines Toggle */}
          <div
            className={`flex items-center justify-between p-3 rounded-lg border ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="flex items-center gap-2.5">
              <Grid className="w-4 h-4 text-[#2962ff]" />
              <div>
                <div className="font-bold text-xs">Background Grid Lines</div>
                <div className="text-[11px] opacity-60">Show vertical and horizontal price/time lines</div>
              </div>
            </div>
            <input
              type="checkbox"
              checked={form.gridVisible}
              onChange={(e) => setForm({ ...form, gridVisible: e.target.checked })}
              className="w-4 h-4 rounded accent-[#2962ff] cursor-pointer"
            />
          </div>

          {/* Timezone */}
          <div>
            <label className="font-bold uppercase tracking-wider text-[11px] opacity-70 block mb-1.5">
              Chart Timezone
            </label>
            <select
              value={form.timezone}
              onChange={(e) => setForm({ ...form, timezone: e.target.value })}
              className={`w-full rounded-lg border p-2 text-xs font-semibold focus:outline-none cursor-pointer ${
                isLight
                  ? "bg-[#f8f9fc] border-[#e0e3eb] text-[#131722]"
                  : "bg-[#141722] border-[#2a2e39] text-[#d1d4dc]"
              }`}
            >
              <option value="UTC">UTC (Universal Coordinated Time)</option>
              <option value="America/New_York">America / New York (EST / EDT)</option>
              <option value="Asia/Jakarta">Asia / Jakarta (WIB +7)</option>
              <option value="Asia/Tokyo">Asia / Tokyo (JST +9)</option>
              <option value="Europe/London">Europe / London (GMT / BST)</option>
            </select>
          </div>

          {/* Audio Alerts */}
          <div
            className={`flex items-center justify-between p-3 rounded-lg border ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="flex items-center gap-2.5">
              {form.audioAlerts ? (
                <Volume2 className="w-4 h-4 text-[#089981]" />
              ) : (
                <VolumeX className="w-4 h-4 text-[#787b86]" />
              )}
              <div>
                <div className="font-bold text-xs">Price Alert Chime</div>
                <div className="text-[11px] opacity-60">Play audio ping when price crosses alert target</div>
              </div>
            </div>
            <input
              type="checkbox"
              checked={form.audioAlerts}
              onChange={(e) => setForm({ ...form, audioAlerts: e.target.checked })}
              className="w-4 h-4 rounded accent-[#2962ff] cursor-pointer"
            />
          </div>
        </div>

        {/* Footer Buttons */}
        <div
          className={`flex items-center justify-between px-5 py-3 border-t ${
            isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
          }`}
        >
          <button
            type="button"
            onClick={handleReset}
            className="flex items-center gap-1 px-3 py-1.5 rounded font-semibold text-xs opacity-70 hover:opacity-100 hover:bg-black/5 dark:hover:bg-white/5 cursor-pointer transition-all"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span>Reset Defaults</span>
          </button>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              className={`px-3 py-1.5 rounded font-semibold text-xs border cursor-pointer ${
                isLight ? "border-[#e0e3eb] hover:bg-black/5" : "border-[#2a2e39] hover:bg-[#2a2e39]"
              }`}
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleSave}
              className="flex items-center gap-1.5 px-4 py-1.5 rounded bg-[#2962ff] text-white font-bold text-xs hover:bg-[#1e53e5] shadow-sm cursor-pointer transition-colors"
            >
              <Check className="w-3.5 h-3.5" />
              <span>Apply Settings</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
