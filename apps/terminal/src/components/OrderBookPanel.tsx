"use client";

import React, { useState, useEffect, useMemo } from "react";
import { Layers, Activity, ArrowUpRight, ArrowDownRight } from "lucide-react";

interface OrderBookPanelProps {
  symbol: string;
  livePrice: number | null;
  digits: number;
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
}) => {
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

  // Add a trade when price updates or pulse every 2.5s
  useEffect(() => {
    if (!livePrice || livePrice <= 0) return;

    const now = new Date();
    const timeStr = now.toTimeString().split(" ")[0] + "." + Math.floor(now.getMilliseconds() / 100);
    const side: "buy" | "sell" = Math.random() > 0.48 ? "buy" : "sell";
    const delta = (Math.random() - 0.5) * tickStep * 2;
    const tPrice = livePrice + delta;
    const size = Math.floor(Math.random() * 45) + 5;

    const newTrade: TradeTapeItem = {
      id: "trade_" + Date.now() + "_" + Math.random().toString(36).substring(2, 5),
      price: tPrice,
      size,
      side,
      time: timeStr,
    };

    setTrades((prev) => [newTrade, ...prev.slice(0, 40)]);
  }, [livePrice, tickStep]);

  const maxDepth = Math.max(totalAsk, totalBid, 1);
  const bidRatio = Math.round((totalBid / (totalBid + totalAsk || 1)) * 100);
  const askRatio = 100 - bidRatio;

  return (
    <div className="flex h-full flex-col bg-[#1e222d] border-l border-[#2a2e39] text-xs text-[#d1d4dc] overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[#2a2e39] px-4 py-3 shrink-0">
        <div className="flex items-center gap-2">
          <Layers className="w-4 h-4 text-[#2962ff]" />
          <span className="font-bold text-white text-sm">Order Book & Tape</span>
        </div>
        <span className="font-mono text-[10px] text-[#787b86] uppercase bg-[#141722] px-2 py-0.5 rounded border border-[#2a2e39]">
          {symbol}
        </span>
      </div>

      {/* Sub tabs: Book vs Tape */}
      <div className="flex border-b border-[#2a2e39] bg-[#141722]">
        <button
          onClick={() => setActiveTab("book")}
          className={`flex-1 py-2 text-center text-[11px] font-bold transition-colors ${
            activeTab === "book"
              ? "border-b-2 border-[#2962ff] text-white bg-[#1e222d]"
              : "text-[#787b86] hover:text-[#d1d4dc]"
          }`}
        >
          Depth (DOM)
        </button>
        <button
          onClick={() => setActiveTab("tape")}
          className={`flex-1 py-2 text-center text-[11px] font-bold transition-colors ${
            activeTab === "tape"
              ? "border-b-2 border-[#2962ff] text-white bg-[#1e222d]"
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
            <div className="flex h-1.5 w-full overflow-hidden rounded bg-[#141722]">
              <div style={{ width: `${bidRatio}%` }} className="bg-[#089981]" />
              <div style={{ width: `${askRatio}%` }} className="bg-[#f23645]" />
            </div>
          </div>

          <div className="grid grid-cols-3 text-[10px] text-[#787b86] font-semibold border-b border-[#2a2e39] pb-1.5 mb-1 px-1">
            <span>Price</span>
            <span className="text-right">Size</span>
            <span className="text-right">Total</span>
          </div>

          {/* Asks (Sell Orders) */}
          <div className="flex-1 flex flex-col justify-end space-y-0.5 overflow-hidden">
            {asks.map((a, idx) => {
              const widthPct = Math.min(100, (a.total / maxDepth) * 100);
              return (
                <div
                  key={idx}
                  className="relative grid grid-cols-3 px-1 py-0.5 items-center hover:bg-[#2a2e39]/40 rounded"
                >
                  <div
                    className="absolute right-0 top-0 bottom-0 bg-[#f23645]/15 pointer-events-none rounded-r"
                    style={{ width: `${widthPct}%` }}
                  />
                  <span className="text-[#f23645] font-semibold z-1">
                    {a.price.toFixed(digits)}
                  </span>
                  <span className="text-right text-[#d1d4dc] z-1">{a.size}</span>
                  <span className="text-right text-[#787b86] z-1">{a.total}</span>
                </div>
              );
            })}
          </div>

          {/* Spread & Live Mid Price */}
          <div className="my-2 flex items-center justify-between rounded bg-[#141722] border border-[#2a2e39] px-2.5 py-1.5 shrink-0">
            <div className="flex items-center gap-1.5">
              <span className="font-bold text-sm text-white">
                {price.toFixed(digits)}
              </span>
              <span className="text-[9px] text-[#089981] font-bold">MID</span>
            </div>
            <div className="text-[10px] text-[#787b86]">
              Spread: <span className="text-white font-semibold">{spread.toFixed(digits)}</span>
            </div>
          </div>

          {/* Bids (Buy Orders) */}
          <div className="flex-1 flex flex-col space-y-0.5 overflow-hidden">
            {bids.map((b, idx) => {
              const widthPct = Math.min(100, (b.total / maxDepth) * 100);
              return (
                <div
                  key={idx}
                  className="relative grid grid-cols-3 px-1 py-0.5 items-center hover:bg-[#2a2e39]/40 rounded"
                >
                  <div
                    className="absolute right-0 top-0 bottom-0 bg-[#089981]/15 pointer-events-none rounded-r"
                    style={{ width: `${widthPct}%` }}
                  />
                  <span className="text-[#089981] font-semibold z-1">
                    {b.price.toFixed(digits)}
                  </span>
                  <span className="text-right text-[#d1d4dc] z-1">{b.size}</span>
                  <span className="text-right text-[#787b86] z-1">{b.total}</span>
                </div>
              );
            })}
          </div>
        </div>
      ) : (
        /* Time & Sales (Trade Tape) */
        <div className="flex-1 overflow-y-auto p-3 font-mono text-[11px]">
          <div className="grid grid-cols-3 text-[10px] text-[#787b86] font-semibold border-b border-[#2a2e39] pb-1.5 mb-1 px-1">
            <span>Price</span>
            <span className="text-right">Size</span>
            <span className="text-right">Time</span>
          </div>

          <div className="space-y-1">
            {trades.map((t) => (
              <div
                key={t.id}
                className="grid grid-cols-3 px-1 py-1 rounded items-center hover:bg-[#2a2e39]/30"
              >
                <div className="flex items-center gap-1 font-semibold">
                  {t.side === "buy" ? (
                    <ArrowUpRight className="w-3 h-3 text-[#089981]" />
                  ) : (
                    <ArrowDownRight className="w-3 h-3 text-[#f23645]" />
                  )}
                  <span className={t.side === "buy" ? "text-[#089981]" : "text-[#f23645]"}>
                    {t.price.toFixed(digits)}
                  </span>
                </div>
                <span className="text-right text-[#d1d4dc]">{t.size}</span>
                <span className="text-right text-[10px] text-[#787b86]">{t.time}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
