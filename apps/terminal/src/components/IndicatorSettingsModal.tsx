"use client";

import React, { useState, useEffect } from "react";
import { X, Sliders, RotateCcw, Check } from "lucide-react";
import { IndicatorParameters, DEFAULT_INDICATOR_PARAMS } from "@/types";

interface IndicatorSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  params: IndicatorParameters;
  onSave: (newParams: IndicatorParameters) => void;
  theme?: "dark" | "light";
}

export const IndicatorSettingsModal: React.FC<IndicatorSettingsModalProps> = ({
  isOpen,
  onClose,
  params,
  onSave,
  theme = "dark",
}) => {
  const isLight = theme === "light";
  const [localParams, setLocalParams] = useState<IndicatorParameters>(params || DEFAULT_INDICATOR_PARAMS);

  useEffect(() => {
    if (isOpen) {
      setLocalParams(params || DEFAULT_INDICATOR_PARAMS);
    }
  }, [isOpen, params]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(localParams);
    onClose();
  };

  const handleReset = () => {
    setLocalParams(DEFAULT_INDICATOR_PARAMS);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4"
      onClick={onClose}
    >
      <div
        className={`w-full max-w-[480px] rounded-2xl shadow-2xl flex flex-col overflow-hidden text-xs transition-colors ${
          isLight
            ? "bg-[#ffffff] border border-[#e0e3eb] text-[#131722]"
            : "bg-[#1e222d] border border-[#2a2e39] text-[#d1d4dc]"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Modal Header */}
        <div
          className={`h-12 px-4 border-b flex items-center justify-between shrink-0 ${
            isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-center gap-2">
            <Sliders className="w-4 h-4 text-[#2962ff]" />
            <span className={`font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>
              Indicator Parameters
            </span>
          </div>
          <button
            onClick={onClose}
            className={`p-1.5 rounded cursor-pointer transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
            }`}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form Body */}
        <form onSubmit={handleSubmit} className="p-4 space-y-4 max-h-[70vh] overflow-y-auto">
          {/* Moving Averages */}
          <div
            className={`p-3 rounded-xl border space-y-3 ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="font-bold text-xs flex items-center justify-between">
              <span>Moving Averages</span>
              <span className="text-[10px] text-[#787b86] font-normal">Main Chart Overlays</span>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-[11px] text-[#787b86] font-medium block mb-1">
                  SMA Length (Bars)
                </label>
                <input
                  type="number"
                  min="2"
                  max="500"
                  value={localParams.smaPeriod}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      smaPeriod: Math.max(2, parseInt(e.target.value) || 20),
                    }))
                  }
                  className={`w-full rounded-lg px-2.5 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>

              <div>
                <label className="text-[11px] text-[#787b86] font-medium block mb-1">
                  EMA Length (Bars)
                </label>
                <input
                  type="number"
                  min="2"
                  max="500"
                  value={localParams.emaPeriod}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      emaPeriod: Math.max(2, parseInt(e.target.value) || 50),
                    }))
                  }
                  className={`w-full rounded-lg px-2.5 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>
            </div>
          </div>

          {/* Bollinger Bands */}
          <div
            className={`p-3 rounded-xl border space-y-3 ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="font-bold text-xs flex items-center justify-between">
              <span>Bollinger Bands</span>
              <span className="text-[10px] text-[#787b86] font-normal">Volatility Envelope</span>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-[11px] text-[#787b86] font-medium block mb-1">
                  Length (Period)
                </label>
                <input
                  type="number"
                  min="5"
                  max="200"
                  value={localParams.bollingerPeriod}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      bollingerPeriod: Math.max(5, parseInt(e.target.value) || 20),
                    }))
                  }
                  className={`w-full rounded-lg px-2.5 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>

              <div>
                <label className="text-[11px] text-[#787b86] font-medium block mb-1">
                  StdDev Multiplier
                </label>
                <input
                  type="number"
                  step="0.1"
                  min="0.5"
                  max="5.0"
                  value={localParams.bollingerStdDev}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      bollingerStdDev: Math.max(0.5, parseFloat(e.target.value) || 2.0),
                    }))
                  }
                  className={`w-full rounded-lg px-2.5 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>
            </div>
          </div>

          {/* RSI & ATR */}
          <div
            className={`p-3 rounded-xl border space-y-3 ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="font-bold text-xs flex items-center justify-between">
              <span>Oscillators (RSI &amp; ATR)</span>
              <span className="text-[10px] text-[#787b86] font-normal">Momentum &amp; Volatility</span>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-[11px] text-[#787b86] font-medium block mb-1">
                  RSI Period
                </label>
                <input
                  type="number"
                  min="2"
                  max="100"
                  value={localParams.rsiPeriod}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      rsiPeriod: Math.max(2, parseInt(e.target.value) || 14),
                    }))
                  }
                  className={`w-full rounded-lg px-2.5 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>

              <div>
                <label className="text-[11px] text-[#787b86] font-medium block mb-1">
                  ATR Period
                </label>
                <input
                  type="number"
                  min="2"
                  max="100"
                  value={localParams.atrPeriod}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      atrPeriod: Math.max(2, parseInt(e.target.value) || 14),
                    }))
                  }
                  className={`w-full rounded-lg px-2.5 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>
            </div>
          </div>

          {/* MACD */}
          <div
            className={`p-3 rounded-xl border space-y-3 ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="font-bold text-xs flex items-center justify-between">
              <span>MACD (Moving Average Convergence Divergence)</span>
            </div>
            <div className="grid grid-cols-3 gap-2">
              <div>
                <label className="text-[10px] text-[#787b86] font-medium block mb-1">
                  Fast EMA
                </label>
                <input
                  type="number"
                  min="2"
                  max="100"
                  value={localParams.macdFast}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      macdFast: Math.max(2, parseInt(e.target.value) || 12),
                    }))
                  }
                  className={`w-full rounded-lg px-2 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>

              <div>
                <label className="text-[10px] text-[#787b86] font-medium block mb-1">
                  Slow EMA
                </label>
                <input
                  type="number"
                  min="2"
                  max="200"
                  value={localParams.macdSlow}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      macdSlow: Math.max(2, parseInt(e.target.value) || 26),
                    }))
                  }
                  className={`w-full rounded-lg px-2 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>

              <div>
                <label className="text-[10px] text-[#787b86] font-medium block mb-1">
                  Signal Period
                </label>
                <input
                  type="number"
                  min="2"
                  max="100"
                  value={localParams.macdSignal}
                  onChange={(e) =>
                    setLocalParams((prev) => ({
                      ...prev,
                      macdSignal: Math.max(2, parseInt(e.target.value) || 9),
                    }))
                  }
                  className={`w-full rounded-lg px-2 py-1.5 border font-mono text-xs focus:outline-hidden ${
                    isLight ? "bg-white border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-white"
                  }`}
                />
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex items-center justify-between pt-2">
            <button
              type="button"
              onClick={handleReset}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-xs font-semibold cursor-pointer transition-colors ${
                isLight ? "border-[#e0e3eb] text-[#5d606b] hover:bg-[#f0f3fa]" : "border-[#2a2e39] text-[#787b86] hover:bg-[#2a2e39]"
              }`}
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>Reset Defaults</span>
            </button>

            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={onClose}
                className={`px-3 py-1.5 rounded-lg border text-xs font-semibold cursor-pointer transition-colors ${
                  isLight ? "border-[#e0e3eb] text-[#5d606b] hover:bg-[#f0f3fa]" : "border-[#2a2e39] text-[#787b86] hover:bg-[#2a2e39]"
                }`}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg bg-[#2962ff] text-white font-semibold hover:bg-[#1e4bd8] shadow-sm cursor-pointer transition-colors"
              >
                <Check className="w-3.5 h-3.5" />
                <span>Apply Parameters</span>
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
};
