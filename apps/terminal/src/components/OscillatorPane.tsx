"use client";

import React, { useMemo } from "react";
import { CandleData, IndicatorState } from "@/types";

interface OscillatorPaneProps {
  candles: CandleData[];
  indicators: IndicatorState;
}

export const OscillatorPane: React.FC<OscillatorPaneProps> = ({
  candles,
  indicators,
}) => {
  const showRsi = indicators.rsi;
  const showMacd = indicators.macd;

  // --- Calculate RSI (14) series ---
  const rsiSeries = useMemo(() => {
    if (!showRsi || candles.length < 16) return [];
    const period = 14;
    const result: { time: number; value: number }[] = [];

    let gains = 0;
    let losses = 0;

    for (let i = 1; i <= period; i++) {
      const diff = candles[i].close - candles[i - 1].close;
      if (diff >= 0) gains += diff;
      else losses += Math.abs(diff);
    }

    let avgGain = gains / period;
    let avgLoss = losses / period;

    const rsi0 = avgLoss === 0 ? 100 : 100 - 100 / (1 + avgGain / avgLoss);
    result.push({ time: candles[period].time, value: rsi0 });

    for (let i = period + 1; i < candles.length; i++) {
      const diff = candles[i].close - candles[i - 1].close;
      if (diff >= 0) {
        avgGain = (avgGain * (period - 1) + diff) / period;
        avgLoss = (avgLoss * (period - 1)) / period;
      } else {
        avgGain = (avgGain * (period - 1)) / period;
        avgLoss = (avgLoss * (period - 1) + Math.abs(diff)) / period;
      }
      const rsi = avgLoss === 0 ? 100 : 100 - 100 / (1 + avgGain / avgLoss);
      result.push({ time: candles[i].time, value: rsi });
    }

    return result;
  }, [candles, showRsi]);

  // --- Calculate MACD (12, 26, 9) series ---
  const macdSeries = useMemo(() => {
    if (!showMacd || candles.length < 35) return [];
    const fastP = 12;
    const slowP = 26;
    const sigP = 9;

    const closes = candles.map((c) => c.close);
    const times = candles.map((c) => c.time);

    // Calculate EMA helper
    const calcEMA = (data: number[], period: number): number[] => {
      const k = 2 / (period + 1);
      const emaArr: number[] = new Array(data.length);
      let sum = 0;
      for (let i = 0; i < period; i++) sum += data[i];
      let prev = sum / period;
      emaArr[period - 1] = prev;

      for (let i = period; i < data.length; i++) {
        const val = data[i] * k + prev * (1 - k);
        emaArr[i] = val;
        prev = val;
      }
      return emaArr;
    };

    const fastEma = calcEMA(closes, fastP);
    const slowEma = calcEMA(closes, slowP);

    const macdLine: (number | null)[] = new Array(closes.length).fill(null);
    const macdRaw: number[] = [];
    const macdIndices: number[] = [];

    for (let i = slowP - 1; i < closes.length; i++) {
      const diff = fastEma[i] - slowEma[i];
      macdLine[i] = diff;
      macdRaw.push(diff);
      macdIndices.push(i);
    }

    if (macdRaw.length < sigP) return [];

    const signalRaw = calcEMA(macdRaw, sigP);
    const result: { time: number; macd: number; signal: number; hist: number }[] = [];

    for (let j = sigP - 1; j < macdRaw.length; j++) {
      const idx = macdIndices[j];
      const mVal = macdRaw[j];
      const sVal = signalRaw[j];
      result.push({
        time: times[idx],
        macd: mVal,
        signal: sVal,
        hist: mVal - sVal,
      });
    }

    return result;
  }, [candles, showMacd]);

  if (!showRsi && !showMacd) return null;

  const currentRsi = rsiSeries.length > 0 ? rsiSeries[rsiSeries.length - 1].value : null;
  const currentMacd = macdSeries.length > 0 ? macdSeries[macdSeries.length - 1] : null;

  return (
    <div className="flex flex-col border-t border-[#2a2e39] bg-[#141722] text-xs select-none">
      {/* RSI Sub-pane */}
      {showRsi && (
        <div className="relative h-[95px] w-full border-b border-[#2a2e39] px-3 py-1">
          {/* Header & Badges */}
          <div className="flex items-center gap-2">
            <span className="font-mono text-[11px] font-bold text-[#9c27b0]">RSI (14)</span>
            {currentRsi !== null && (
              <span
                className={`font-mono text-[10px] font-semibold px-1.5 py-0.2 rounded border ${
                  currentRsi >= 70
                    ? "bg-[#f23645]/15 border-[#f23645]/30 text-[#f23645]"
                    : currentRsi <= 30
                    ? "bg-[#089981]/15 border-[#089981]/30 text-[#089981]"
                    : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
                }`}
              >
                {currentRsi.toFixed(1)} {currentRsi >= 70 ? "Overbought" : currentRsi <= 30 ? "Oversold" : "Neutral"}
              </span>
            )}
          </div>

          {/* SVG Canvas for RSI */}
          <div className="relative h-[68px] w-full mt-1">
            <svg className="w-full h-full" preserveAspectRatio="none" viewBox="0 0 1000 100">
              {/* Overbought / Oversold Zone (30-70) */}
              <rect x="0" y="30" width="1000" height="40" fill="rgba(156, 39, 176, 0.05)" />
              {/* Level 70 */}
              <line x1="0" y1="30" x2="1000" y2="30" stroke="#f2364566" strokeWidth="1" strokeDasharray="4 3" />
              {/* Level 50 */}
              <line x1="0" y1="50" x2="1000" y2="50" stroke="#787b8633" strokeWidth="1" strokeDasharray="2 2" />
              {/* Level 30 */}
              <line x1="0" y1="70" x2="1000" y2="70" stroke="#08998166" strokeWidth="1" strokeDasharray="4 3" />

              {/* RSI Curve */}
              {rsiSeries.length > 1 && (
                <polyline
                  fill="none"
                  stroke="#ab47bc"
                  strokeWidth="1.8"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  points={rsiSeries
                    .slice(-120)
                    .map((pt, idx, arr) => {
                      const x = (idx / (arr.length - 1)) * 1000;
                      const y = 100 - pt.value; // Invert: 100 is top (y=0), 0 is bottom (y=100)
                      return `${x.toFixed(1)},${y.toFixed(1)}`;
                    })
                    .join(" ")}
                />
              )}
            </svg>

            {/* Labels right */}
            <div className="absolute right-1 top-0 bottom-0 flex flex-col justify-between text-[9px] font-mono text-[#787b86] pointer-events-none">
              <span>70</span>
              <span>50</span>
              <span>30</span>
            </div>
          </div>
        </div>
      )}

      {/* MACD Sub-pane */}
      {showMacd && (
        <div className="relative h-[95px] w-full px-3 py-1">
          {/* Header & Badges */}
          <div className="flex items-center gap-2">
            <span className="font-mono text-[11px] font-bold text-[#2962ff]">MACD (12, 26, 9)</span>
            {currentMacd && (
              <div className="flex items-center gap-2 font-mono text-[10px]">
                <span className="text-[#2962ff]">MACD: {currentMacd.macd.toFixed(2)}</span>
                <span className="text-[#f5b942]">Signal: {currentMacd.signal.toFixed(2)}</span>
                <span className={currentMacd.hist >= 0 ? "text-[#089981]" : "text-[#f23645]"}>
                  Hist: {currentMacd.hist >= 0 ? "+" : ""}{currentMacd.hist.toFixed(2)}
                </span>
              </div>
            )}
          </div>

          {/* SVG Canvas for MACD */}
          <div className="relative h-[68px] w-full mt-1">
            <svg className="w-full h-full" preserveAspectRatio="none" viewBox="0 0 1000 100">
              {/* Zero Line */}
              <line x1="0" y1="50" x2="1000" y2="50" stroke="#787b8644" strokeWidth="1" />

              {/* Histogram Bars */}
              {macdSeries.slice(-100).map((pt, idx, arr) => {
                const maxVal = Math.max(0.01, ...arr.map((p) => Math.abs(p.hist)));
                const x = (idx / arr.length) * 1000;
                const barWidth = 1000 / arr.length * 0.7;
                const barHeight = Math.min(45, (Math.abs(pt.hist) / maxVal) * 45);
                const y = pt.hist >= 0 ? 50 - barHeight : 50;
                const isPositive = pt.hist >= 0;

                return (
                  <rect
                    key={idx}
                    x={x}
                    y={y}
                    width={Math.max(1, barWidth)}
                    height={Math.max(1, barHeight)}
                    fill={isPositive ? "#089981bb" : "#f23645bb"}
                  />
                );
              })}

              {/* MACD Line */}
              {macdSeries.length > 1 && (
                <polyline
                  fill="none"
                  stroke="#2962ff"
                  strokeWidth="1.6"
                  strokeLinecap="round"
                  points={macdSeries
                    .slice(-100)
                    .map((pt, idx, arr) => {
                      const maxVal = Math.max(0.01, ...arr.map((p) => Math.max(Math.abs(p.macd), Math.abs(p.signal))));
                      const x = (idx / (arr.length - 1)) * 1000;
                      const y = 50 - (pt.macd / maxVal) * 40;
                      return `${x.toFixed(1)},${y.toFixed(1)}`;
                    })
                    .join(" ")}
                />
              )}

              {/* Signal Line */}
              {macdSeries.length > 1 && (
                <polyline
                  fill="none"
                  stroke="#f5b942"
                  strokeWidth="1.4"
                  strokeLinecap="round"
                  points={macdSeries
                    .slice(-100)
                    .map((pt, idx, arr) => {
                      const maxVal = Math.max(0.01, ...arr.map((p) => Math.max(Math.abs(p.macd), Math.abs(p.signal))));
                      const x = (idx / (arr.length - 1)) * 1000;
                      const y = 50 - (pt.signal / maxVal) * 40;
                      return `${x.toFixed(1)},${y.toFixed(1)}`;
                    })
                    .join(" ")}
                />
              )}
            </svg>
          </div>
        </div>
      )}
    </div>
  );
};
