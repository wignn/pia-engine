"use client";

import React, { useMemo } from "react";
import { CandleData, IndicatorState, TerminalSettings } from "@/types";

interface OscillatorPaneProps {
  candles: CandleData[];
  indicators: IndicatorState;
  settings?: TerminalSettings;
  theme?: "dark" | "light";
}

export const OscillatorPane: React.FC<OscillatorPaneProps> = ({
  candles,
  indicators,
  settings,
  theme = "dark",
}) => {
  const showRsi = indicators.rsi;
  const showMacd = indicators.macd;
  const showAtr = indicators.atr;
  const isLight = (settings?.theme || theme) === "light";

  const rsiPeriod = settings?.indicatorParams?.rsiPeriod || 14;
  const macdFast = settings?.indicatorParams?.macdFast || 12;
  const macdSlow = settings?.indicatorParams?.macdSlow || 26;
  const macdSignal = settings?.indicatorParams?.macdSignal || 9;
  const atrPeriod = settings?.indicatorParams?.atrPeriod || 14;

  // --- Calculate RSI series ---
  const rsiSeries = useMemo(() => {
    if (!showRsi || candles.length < rsiPeriod + 2) return [];
    const period = rsiPeriod;
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
  }, [candles, showRsi, rsiPeriod]);

  // --- Calculate MACD series ---
  const macdSeries = useMemo(() => {
    if (!showMacd || candles.length < macdSlow + macdSignal) return [];
    const fastP = macdFast;
    const slowP = macdSlow;
    const sigP = macdSignal;

    const closes = candles.map((c) => c.close);
    const times = candles.map((c) => c.time);

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

    const emaFast = calcEMA(closes, fastP);
    const emaSlow = calcEMA(closes, slowP);

    const macdLine: number[] = [];
    const macdTimes: number[] = [];

    for (let i = slowP - 1; i < closes.length; i++) {
      macdLine.push(emaFast[i] - emaSlow[i]);
      macdTimes.push(times[i]);
    }

    const signalLine = calcEMA(macdLine, sigP);

    const result: { time: number; macd: number; signal: number; hist: number }[] = [];
    for (let i = sigP - 1; i < macdLine.length; i++) {
      const m = macdLine[i];
      const s = signalLine[i];
      result.push({
        time: macdTimes[i],
        macd: m,
        signal: s,
        hist: m - s,
      });
    }

    return result;
  }, [candles, showMacd, macdFast, macdSlow, macdSignal]);

  // --- Calculate ATR (Average True Range) series ---
  const atrSeries = useMemo(() => {
    if (!showAtr || candles.length < atrPeriod + 2) return [];
    const trArr: { time: number; tr: number }[] = [];

    for (let i = 1; i < candles.length; i++) {
      const prev = candles[i - 1].close;
      const cur = candles[i];
      const tr = Math.max(
        cur.high - cur.low,
        Math.abs(cur.high - prev),
        Math.abs(cur.low - prev)
      );
      trArr.push({ time: cur.time, tr });
    }

    if (trArr.length < atrPeriod) return [];
    const result: { time: number; value: number }[] = [];
    let sum = 0;
    for (let i = 0; i < atrPeriod; i++) sum += trArr[i].tr;
    let atr = sum / atrPeriod;
    result.push({ time: trArr[atrPeriod - 1].time, value: atr });

    for (let i = atrPeriod; i < trArr.length; i++) {
      atr = (atr * (atrPeriod - 1) + trArr[i].tr) / atrPeriod;
      result.push({ time: trArr[i].time, value: atr });
    }

    return result;
  }, [candles, showAtr, atrPeriod]);

  if (!showRsi && !showMacd && !showAtr) return null;

  const currentRsi = rsiSeries.length > 0 ? rsiSeries[rsiSeries.length - 1].value : null;
  const currentMacd = macdSeries.length > 0 ? macdSeries[macdSeries.length - 1] : null;
  const currentAtr = atrSeries.length > 0 ? atrSeries[atrSeries.length - 1].value : null;

  return (
    <div
      className={`border-t flex flex-col divide-y transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] divide-[#e0e3eb]" : "bg-[#131722] border-[#2a2e39] divide-[#2a2e39]"
      }`}
    >
      {/* 1. RSI (Relative Strength Index) Pane */}
      {showRsi && (
        <div className="relative px-3 py-1.5 select-none shrink-0">
          <div className="flex items-center justify-between text-[11px] font-mono">
            <div className="flex items-center gap-2">
              <span className="font-bold text-[#787b86]">RSI ({rsiPeriod})</span>
              {currentRsi !== null && (
                <span
                  className={`font-black ${
                    currentRsi >= 70
                      ? "text-[#f23645]"
                      : currentRsi <= 30
                      ? "text-[#089981]"
                      : "text-[#2962ff]"
                  }`}
                >
                  {currentRsi.toFixed(2)}
                </span>
              )}
            </div>
            <div className="flex items-center gap-2 text-[9px] text-[#787b86]">
              <span className="text-[#f23645]/80">OB 70</span>
              <span className="text-[#089981]/80">OS 30</span>
            </div>
          </div>

          <div className="relative h-[56px] w-full mt-1">
            <svg className="w-full h-full" preserveAspectRatio="none" viewBox="0 0 1000 100">
              <rect x="0" y="30" width="1000" height="40" fill={isLight ? "rgba(41, 98, 255, 0.04)" : "rgba(41, 98, 255, 0.06)"} />
              <line x1="0" y1="30" x2="1000" y2="30" stroke="#f2364555" strokeDasharray="3 3" strokeWidth="1" />
              <line x1="0" y1="50" x2="1000" y2="50" stroke="#787b8633" strokeDasharray="2 2" strokeWidth="1" />
              <line x1="0" y1="70" x2="1000" y2="70" stroke="#08998155" strokeDasharray="3 3" strokeWidth="1" />

              {rsiSeries.length > 1 && (
                <polyline
                  fill="none"
                  stroke="#a855f7"
                  strokeWidth="1.8"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  points={rsiSeries
                    .slice(-100)
                    .map((pt, idx, arr) => {
                      const x = (idx / (arr.length - 1)) * 1000;
                      const y = 100 - pt.value;
                      return `${x.toFixed(1)},${y.toFixed(1)}`;
                    })
                    .join(" ")}
                />
              )}
            </svg>
          </div>
        </div>
      )}

      {/* 2. ATR (Average True Range) Pane */}
      {showAtr && (
        <div className="relative px-3 py-1.5 select-none shrink-0">
          <div className="flex items-center justify-between text-[11px] font-mono">
            <div className="flex items-center gap-2">
              <span className="font-bold text-[#787b86]">ATR ({atrPeriod})</span>
              {currentAtr !== null && (
                <span className="font-black text-[#f59e0b]">
                  {currentAtr.toFixed(4)}
                </span>
              )}
            </div>
            <span className="text-[9px] text-[#787b86]">Volatility Measure</span>
          </div>

          <div className="relative h-[48px] w-full mt-1">
            <svg className="w-full h-full" preserveAspectRatio="none" viewBox="0 0 1000 100">
              {atrSeries.length > 1 && (() => {
                const slice = atrSeries.slice(-100);
                const minVal = Math.min(...slice.map((p) => p.value)) * 0.95;
                const maxVal = Math.max(...slice.map((p) => p.value)) * 1.05;
                const range = maxVal - minVal || 1;

                const points = slice
                  .map((pt, idx) => {
                    const x = (idx / (slice.length - 1)) * 1000;
                    const y = 90 - ((pt.value - minVal) / range) * 80;
                    return `${x.toFixed(1)},${y.toFixed(1)}`;
                  })
                  .join(" ");

                return (
                  <polyline
                    fill="none"
                    stroke="#f59e0b"
                    strokeWidth="1.8"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    points={points}
                  />
                );
              })()}
            </svg>
          </div>
        </div>
      )}

      {/* 3. MACD Pane */}
      {showMacd && (
        <div className="relative px-3 py-1.5 select-none shrink-0">
          <div className="flex items-center justify-between text-[11px] font-mono">
            <div className="flex items-center gap-2">
              <span className="font-bold text-[#787b86]">
                MACD ({macdFast}, {macdSlow}, {macdSignal})
              </span>
              {currentMacd && (
                <>
                  <span className="font-bold text-[#2962ff]">
                    {currentMacd.macd.toFixed(2)}
                  </span>
                  <span className="font-bold text-[#f5b942]">
                    {currentMacd.signal.toFixed(2)}
                  </span>
                  <span
                    className={`font-black ${
                      currentMacd.hist >= 0 ? "text-[#089981]" : "text-[#f23645]"
                    }`}
                  >
                    {currentMacd.hist.toFixed(2)}
                  </span>
                </>
              )}
            </div>
          </div>

          <div className="relative h-[68px] w-full mt-1">
            <svg className="w-full h-full" preserveAspectRatio="none" viewBox="0 0 1000 100">
              <line x1="0" y1="50" x2="1000" y2="50" stroke="#787b8644" strokeWidth="1" />

              {/* Histogram Bars */}
              {macdSeries.slice(-100).map((pt, idx, arr) => {
                const maxVal = Math.max(0.01, ...arr.map((p) => Math.abs(p.hist)));
                const x = (idx / arr.length) * 1000;
                const barWidth = (1000 / arr.length) * 0.7;
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
