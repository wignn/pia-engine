"use client";

import React, { useState, useEffect, useMemo } from "react";
import { Layers, Activity, ArrowUpRight, ArrowDownRight } from "lucide-react";

interface OrderBookPanelProps {
  symbol: string;
  livePrice: number | null;
  digits: number;
  theme?: "dark" | "light";
}

interface OrderBookLevel {
  price: number;
  size: number;
  total: number;
}

interface TradeTapeItem {
  id: string;
  price: number;
  size: number;
  side: "buy" | "sell";
  time: string;
}

export const OrderBookPanel: React.FC<OrderBookPanelProps> = ({
  symbol,
  livePrice,
  digits,
  theme = "dark",
}) => {
  const isLight = theme === "light";
  const [activeTab, setActiveTab] = useState<"book" | "tape">("book");
  const [trades, setTrades] = useState<TradeTapeItem[]>([]);

  const price = livePrice && livePrice > 0 ? livePrice : 100.0;
  const tickStep = useMemo(() => Math.pow(10, -digits), [digits]);

  // Generate dynamic Level-2 Depth based on current price
  const { asks, bids, totalBid, totalAsk, spread } = useMemo(() => {
    const levelsCount = 8;
    const askList: OrderBookLevel[] = [];
    const bidList: OrderBookLevel[] = [];

    let cumAsk = 0;
    for (let i = levelsCount; i >= 1; i--) {
      const p = price + i * tickStep * (digits <= 2 ? 1 : 10);
      const size = Math.floor(Math.abs(Math.sin(p * 13) * 80) + 15);
      cumAsk += size;
      askList.push({ price: p, size, total: cumAsk });
    }

    let cumBid = 0;
    for (let i = 1; i <= levelsCount; i++) {
      const p = price - i * tickStep * (digits <= 2 ? 1 : 10);
      const size = Math.floor(Math.abs(Math.cos(p * 17) * 85) + 18);
      cumBid += size;
      bidList.push({ price: p, size, total: cumBid });
    }

    const lowestAsk = askList[askList.length - 1]?.price ?? price;
    const highestBid = bidList[0]?.price ?? price;
    const sp = Math.max(0, lowestAsk - highestBid);

    return {
      asks: askList,
      bids: bidList,
      totalBid: cumBid,
      totalAsk: cumAsk,
      spread: sp,
    };
  }, [price, tickStep, digits]);

  // Generate mock real-time trade tape
  useEffect(() => {
    const initialTrades: TradeTapeItem[] = [];
    const now = Date.now();
    for (let i = 0; i < 15; i++) {
      const side = Math.random() > 0.48 ? "buy" : "sell";
      const p = price + (Math.random() - 0.5) * tickStep * 5;
      const size = Number((Math.random() * 2.5 + 0.1).toFixed(3));
      const d = new Date(now - i * 1500);
      initialTrades.push({
        id: Math.random().toString(),
        price: p,
        size,
        side,
        time: d.toTimeString().split(" ")[0],
      });
    }
    setTrades(initialTrades);

    const interval = setInterval(() => {
      const side = Math.random() > 0.48 ? "buy" : "sell";
      const p = price + (Math.random() - 0.5) * tickStep * 4;
      const size = Number((Math.random() * 2.5 + 0.1).toFixed(3));
      const newTrade: TradeTapeItem = {
        id: Math.random().toString(),
        price: p,
        size,
        side,
        time: new Date().toTimeString().split(" ")[0],
      };
      setTrades((prev) => [newTrade, ...prev.slice(0, 30)]);
    }, 1800);

    return () => clearInterval(interval);
  }, [price, tickStep]);

  const maxDepth = Math.max(totalBid, totalAsk, 1);
  const totalVolume = totalBid + totalAsk;
  const bidRatio = totalVolume > 0 ? Math.round((totalBid / totalVolume) * 100) : 50;
  const askRatio = 100 - bidRatio;

  return (
    <aside
      className={`w-full flex flex-col h-full select-none text-xs overflow-hidden transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Header */}
      <div
        className={`flex items-center justify-between border-b px-4 py-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-2">
          <Layers className="w-4 h-4 text-[#2962ff]" />
          <span className={`font-bold text-sm ${isLight ? "text-[#131722]" : "text-white"}`}>Order Book & Tape</span>
        </div>
        <span
          className={`font-mono text-[10px] uppercase px-2 py-0.5 rounded border ${
            isLight ? "bg-[#f0f3fa] text-[#5d606b] border-[#e0e3eb]" : "bg-[#141722] text-[#787b86] border-[#2a2e39]"
          }`}
        >
          {symbol}
        </span>
      </div>

      {/* Sub tabs: Book vs Tape */}
      <div className={`flex border-b ${isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"}`}>
        <button
          onClick={() => setActiveTab("book")}
          className={`flex-1 py-2 text-center text-[11px] font-bold transition-colors cursor-pointer ${
            activeTab === "book"
              ? isLight
                ? "border-b-2 border-[#2962ff] text-[#131722] bg-white font-bold"
                : "border-b-2 border-[#2962ff] text-white bg-[#1e222d]"
              : isLight
              ? "text-[#5d606b] hover:text-[#131722]"
              : "text-[#787b86] hover:text-[#d1d4dc]"
          }`}
        >
          Depth (DOM)
        </button>
        <button
          onClick={() => setActiveTab("tape")}
          className={`flex-1 py-2 text-center text-[11px] font-bold transition-colors cursor-pointer ${
            activeTab === "tape"
              ? isLight
                ? "border-b-2 border-[#2962ff] text-[#131722] bg-white font-bold"
                : "border-b-2 border-[#2962ff] text-white bg-[#1e222d]"
              : isLight
              ? "text-[#5d606b] hover:text-[#131722]"
              : "text-[#787b86] hover:text-[#d1d4dc]"
          }`}
        >
          Time & Sales ({trades.length})
        </button>
      </div>

      {activeTab === "book" ? (
        <div className="flex-1 flex flex-col overflow-hidden p-3 font-mono text-[11px]">
          {/* Depth Ratio Bar */}
          <div className="mb-2 shrink-0">
            <div className="flex justify-between text-[10px] font-bold mb-1">
              <span className="text-[#089981]">Bids: {bidRatio}%</span>
              <span className="text-[#f23645]">Asks: {askRatio}%</span>
            </div>
            <div className={`flex h-1.5 w-full overflow-hidden rounded ${isLight ? "bg-[#e0e3eb]" : "bg-[#141722]"}`}>
              <div style={{ width: `${bidRatio}%` }} className="bg-[#089981]" />
              <div style={{ width: `${askRatio}%` }} className="bg-[#f23645]" />
            </div>
          </div>

          <div
            className={`grid grid-cols-3 text-[10px] font-semibold border-b pb-1.5 mb-1 px-1 ${
              isLight ? "text-[#5d606b] border-[#e0e3eb]" : "text-[#787b86] border-[#2a2e39]"
            }`}
          >
            <span>Price</span>
            <span className="text-right">Size</span>
            <span className="text-right">Total</span>
          </div>

          {/* Asks (Sell Orders) */}
          <div className="flex-1 flex flex-col justify-end space-y-0.5 overflow-hidden">
            {asks.map((a, idx) => {
              const widthPct = Math.min(100, (a.total / maxDepth) * 100);
              return (
                <div key={idx} className="relative grid grid-cols-3 px-1 py-0.5 rounded overflow-hidden">
                  <div
                    style={{ width: `${widthPct}%` }}
                    className="absolute inset-y-0 right-0 bg-[#f23645]/15 rounded-l pointer-events-none"
                  />
                  <span className="text-[#f23645] font-semibold z-10">{a.price.toFixed(digits)}</span>
                  <span className={`text-right z-10 ${isLight ? "text-[#131722]" : "text-white"}`}>{a.size}</span>
                  <span className="text-right text-[#787b86] z-10">{a.total}</span>
                </div>
              );
            })}
          </div>

          {/* Spread / Mid-Market Bar */}
          <div
            className={`my-1.5 py-1 px-2 border-y flex items-center justify-between font-sans text-[11px] shrink-0 ${
              isLight ? "bg-[#f8f9fc] border-[#e0e3eb]" : "bg-[#141722] border-[#2a2e39]"
            }`}
          >
            <div className="flex items-center gap-1.5 font-bold">
              <Activity className="w-3.5 h-3.5 text-[#2962ff]" />
              <span className={isLight ? "text-[#131722]" : "text-white"}>{price.toFixed(digits)}</span>
            </div>
            <span className="text-[10px] text-[#787b86] font-mono">
              Spread: {spread.toFixed(digits)}
            </span>
          </div>

          {/* Bids (Buy Orders) */}
          <div className="flex-1 flex flex-col space-y-0.5 overflow-hidden">
            {bids.map((b, idx) => {
              const widthPct = Math.min(100, (b.total / maxDepth) * 100);
              return (
                <div key={idx} className="relative grid grid-cols-3 px-1 py-0.5 rounded overflow-hidden">
                  <div
                    style={{ width: `${widthPct}%` }}
                    className="absolute inset-y-0 right-0 bg-[#089981]/15 rounded-l pointer-events-none"
                  />
                  <span className="text-[#089981] font-semibold z-10">{b.price.toFixed(digits)}</span>
                  <span className={`text-right z-10 ${isLight ? "text-[#131722]" : "text-white"}`}>{b.size}</span>
                  <span className="text-right text-[#787b86] z-10">{b.total}</span>
                </div>
              );
            })}
          </div>
        </div>
      ) : (
        /* Time & Sales (Trade Tape) */
        <div className="flex-1 flex flex-col overflow-hidden p-3 font-mono text-[11px]">
          <div
            className={`grid grid-cols-3 text-[10px] font-semibold border-b pb-1.5 mb-1 px-1 ${
              isLight ? "text-[#5d606b] border-[#e0e3eb]" : "text-[#787b86] border-[#2a2e39]"
            }`}
          >
            <span>Price</span>
            <span className="text-right">Size</span>
            <span className="text-right">Time</span>
          </div>

          <div className="flex-1 overflow-y-auto space-y-1">
            {trades.map((t) => {
              const isBuy = t.side === "buy";
              return (
                <div
                  key={t.id}
                  className={`grid grid-cols-3 items-center px-1 py-0.5 rounded transition-colors ${
                    isLight ? "hover:bg-[#f0f3fa]" : "hover:bg-[#262b37]"
                  }`}
                >
                  <div className="flex items-center gap-1 font-semibold">
                    {isBuy ? (
                      <ArrowUpRight className="w-3 h-3 text-[#089981]" />
                    ) : (
                      <ArrowDownRight className="w-3 h-3 text-[#f23645]" />
                    )}
                    <span className={isBuy ? "text-[#089981]" : "text-[#f23645]"}>
                      {t.price.toFixed(digits)}
                    </span>
                  </div>
                  <span className={`text-right font-medium ${isLight ? "text-[#131722]" : "text-white"}`}>
                    {t.size}
                  </span>
                  <span className="text-right text-[#787b86] text-[10px]">{t.time}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </aside>
  );
};
