"use client";

import React from "react";

interface InstrumentHeaderProps {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  digits?: number;
  open?: number;
  high?: number;
  low?: number;
  volume?: number;
  sessionStatus?: string;
  theme?: "dark" | "light";
}

export const InstrumentHeader: React.FC<InstrumentHeaderProps> = ({
  symbol,
  name,
  price,
  change,
  changePercent,
  digits = 2,
  open,
  high,
  low,
  volume,
  sessionStatus,
}) => {
  const isUp = change >= 0;
  const changeSign = isUp ? "+" : "";

  const formatNum = (val: number | undefined): string => {
    if (val === undefined || isNaN(val) || val === 0) return "—";
    return val.toLocaleString("en-US", {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });
  };

  const formatVol = (val: number | undefined): string => {
    if (val === undefined || isNaN(val) || val <= 0) return "—";
    if (val >= 1_000_000_000) return (val / 1_000_000_000).toFixed(2) + "B";
    if (val >= 1_000_000) return (val / 1_000_000).toFixed(2) + "M";
    if (val >= 1_000) return (val / 1_000).toFixed(1) + "K";
    return val.toLocaleString("en-US");
  };

  const todayStr = React.useMemo(() => {
    if (sessionStatus) return sessionStatus;
    const now = new Date();
    return `Market close · ${now.toLocaleDateString("en-US", {
      day: "numeric",
      month: "long",
      year: "numeric",
    })}`;
  }, [sessionStatus]);

  return (
    <div className="border-b border-border bg-card/40 transition-colors">
      <div className="flex flex-wrap items-end justify-between gap-x-8 gap-y-3 px-4 py-2.5 sm:px-6 [@media(max-height:500px)]:py-1">
        <div className="min-w-0">
          <h1 className="flex min-w-0 items-baseline gap-2">
            <span className="font-mono text-lg font-semibold tracking-wide text-primary">
              {symbol}
            </span>
            <span className="truncate text-sm text-foreground">{name}</span>
          </h1>
          <div className="flex items-baseline gap-3 mt-0.5">
            <span className="tabular-nums font-mono text-2xl sm:text-3xl font-medium tracking-tight text-foreground">
              {formatNum(price)}
            </span>
            <span
              className={`tabular-nums font-mono text-sm font-medium ${
                isUp ? "text-up" : "text-down"
              }`}
            >
              {changeSign}
              {formatNum(change)} ({changeSign}
              {changePercent.toFixed(2)}%)
            </span>
          </div>
          <p className="text-xs text-muted-foreground mt-0.5">{todayStr}</p>
        </div>

        <dl className="grid grid-cols-4 gap-x-6 gap-y-1 text-xs sm:flex sm:gap-x-6 [@media(max-height:500px)]:hidden">
          <div>
            <dt className="text-muted-foreground">Open</dt>
            <dd className="tabular-nums font-mono text-foreground font-medium">
              {formatNum(open ?? price - change)}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">High</dt>
            <dd className="tabular-nums font-mono text-foreground font-medium">
              {formatNum(high ?? Math.max(price, price - change))}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Low</dt>
            <dd className="tabular-nums font-mono text-foreground font-medium">
              {formatNum(low ?? Math.min(price, price - change))}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Volume</dt>
            <dd className="tabular-nums font-mono text-foreground font-medium">
              {formatVol(volume)}
            </dd>
          </div>
        </dl>
      </div>
    </div>
  );
};
