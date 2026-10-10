import { TrendingDown, TrendingUp } from "lucide-react";

interface InstrumentHeaderProps {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  digits?: number;
  category?: string;
  provider?: string;
}

export function InstrumentHeader({
  symbol,
  name,
  price,
  change,
  changePercent,
  digits = 2,
  category,
  provider,
}: InstrumentHeaderProps) {
  const hasPrice = Number.isFinite(price) && price > 0;
  const rising = change >= 0;
  const format = (value: number) =>
    value.toLocaleString("en-US", {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });

  return (
    <div className="flex min-h-[60px] shrink-0 items-center justify-between gap-3 border-b border-border bg-card px-4 sm:px-5">
      <div className="min-w-0 border-l-2 border-primary pl-3">
        <div className="flex min-w-0 items-center gap-2">
          <h1 className="font-mono text-base font-bold tracking-tight text-foreground sm:text-lg">{symbol}</h1>
          {category && <span className="hidden rounded-sm border border-border bg-muted px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-muted-foreground sm:inline">{category}</span>}
        </div>
        <p className="truncate text-[11px] text-muted-foreground">{name}{provider ? ` · ${provider}` : ""}</p>
      </div>
      <div className="shrink-0 border-l border-border pl-3 text-right sm:pl-5">
        <div className="font-mono text-lg font-semibold tabular-nums tracking-tight text-foreground sm:text-xl">
          {hasPrice ? format(price) : "—"}
        </div>
        <div className={`flex items-center justify-end gap-1 font-mono text-[11px] font-medium tabular-nums ${rising ? "text-up" : "text-down"}`}>
          {hasPrice ? (
            <>
              {rising ? <TrendingUp className="size-3.5" /> : <TrendingDown className="size-3.5" />}
              {rising ? "+" : ""}{format(change)} ({rising ? "+" : ""}{changePercent.toFixed(2)}%)
            </>
          ) : "Waiting for market data"}
        </div>
      </div>
    </div>
  );
}
