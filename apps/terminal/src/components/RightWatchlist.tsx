"use client";

import { useEffect, useMemo, useState } from "react";
import { ArrowDown, ArrowUp, Search } from "lucide-react";
import { WatchlistItem } from "@/types";

interface RightWatchlistProps {
  items: WatchlistItem[];
  selectedSymbol: string;
  onSelectSymbol: (item: WatchlistItem) => void;
  onOpenNews?: () => void;
  theme?: "dark" | "light";
}

type Category = WatchlistItem["category"] | "all";
type SortField = "symbol" | "price" | "change";

const CATEGORIES: { value: Category; label: string }[] = [
  { value: "all", label: "All markets" },
  { value: "indices", label: "Indices" },
  { value: "stocks", label: "Stocks" },
  { value: "forex", label: "Forex" },
  { value: "commodities", label: "Commodities" },
  { value: "crypto", label: "Crypto" },
];

export function RightWatchlist({ items, selectedSymbol, onSelectSymbol, onOpenNews }: RightWatchlistProps) {
  const [category, setCategory] = useState<Category>("all");
  const [query, setQuery] = useState("");
  const [sortField, setSortField] = useState<SortField>("symbol");
  const [sortAscending, setSortAscending] = useState(true);
  const [headline, setHeadline] = useState<string | null>(null);

  useEffect(() => {
    setHeadline(null);
    if (!selectedSymbol) return;
    const controller = new AbortController();
    fetch(`/api/news?symbol=${encodeURIComponent(selectedSymbol)}&limit=1`, { signal: controller.signal })
      .then((response) => response.ok ? response.json() : null)
      .then((data) => {
        const article = data?.articles?.[0];
        if (article) setHeadline(article.title || article.headline || null);
      })
      .catch(() => {});
    return () => controller.abort();
  }, [selectedSymbol]);

  const visibleItems = useMemo(() => {
    const search = query.trim().toLowerCase();
    const filtered = items.filter((item) =>
      (category === "all" || item.category === category) &&
      (!search || item.symbol.toLowerCase().includes(search) || item.name.toLowerCase().includes(search)),
    );
    filtered.sort((a, b) => {
      const comparison = sortField === "symbol"
        ? a.symbol.localeCompare(b.symbol)
        : sortField === "price"
          ? a.price - b.price
          : a.changePercent - b.changePercent;
      return sortAscending ? comparison : -comparison;
    });
    return filtered;
  }, [items, category, query, sortField, sortAscending]);

  const setSort = (field: SortField) => {
    if (sortField === field) setSortAscending((current) => !current);
    else { setSortField(field); setSortAscending(field === "symbol"); }
  };
  const sortIcon = (field: SortField) => sortField === field
    ? (sortAscending ? <ArrowUp className="size-3" /> : <ArrowDown className="size-3" />)
    : null;
  const formatPrice = (item: WatchlistItem) => {
    if (item.price <= 0) return "—";
    const decimals = item.category === "forex" ? 5 : item.category === "stocks" ? 3 : 2;
    return item.price.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: decimals });
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-card">
      <div className="shrink-0 border-b border-border px-3 pb-2.5 pt-3">
        <div className="mb-2.5 flex items-center justify-between gap-2">
          <div className="flex min-w-0 items-baseline gap-2">
            <h2 className="text-[11px] font-bold uppercase tracking-[0.12em] text-foreground">Watchlist</h2>
            <span className="truncate font-mono text-[10px] text-muted-foreground">{items.length} instruments</span>
          </div>
          <span className="size-1.5 rounded-full bg-primary" aria-hidden="true" />
        </div>
        <div className="flex gap-2">
          <label className="relative min-w-0 flex-1">
            <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <span className="sr-only">Search watchlist</span>
            <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Filter instruments" className="h-8 w-full rounded border border-border bg-background pl-8 pr-2 text-[11px] text-foreground placeholder:text-muted-foreground" />
          </label>
          <select aria-label="Market category" value={category} onChange={(event) => setCategory(event.target.value as Category)} className="h-8 max-w-[104px] rounded border border-border bg-background px-1.5 text-[10px] text-foreground">
            {CATEGORIES.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
          </select>
        </div>
      </div>

      <div className="grid h-7 shrink-0 grid-cols-[minmax(0,1fr)_82px_67px] items-center gap-1 border-b border-border bg-muted/50 px-3 text-[9px] font-semibold uppercase tracking-wider text-muted-foreground">
        <button type="button" onClick={() => setSort("symbol")} className="flex items-center gap-1 text-left hover:text-foreground">Symbol {sortIcon("symbol")}</button>
        <button type="button" onClick={() => setSort("price")} className="flex items-center justify-end gap-1 hover:text-foreground">Last {sortIcon("price")}</button>
        <button type="button" onClick={() => setSort("change")} className="flex items-center justify-end gap-1 hover:text-foreground">Chg % {sortIcon("change")}</button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {visibleItems.length === 0 ? (
          <div className="px-4 py-8 text-center text-xs text-muted-foreground">No symbols match this filter.</div>
        ) : visibleItems.map((item) => {
          const active = item.symbol === selectedSymbol;
          return (
            <button
              type="button"
              key={item.symbol}
              onClick={() => onSelectSymbol(item)}
              className={`grid w-full grid-cols-[minmax(0,1fr)_82px_67px] items-center gap-1 border-b border-border/50 px-3 py-2 text-left transition-colors ${active ? "border-l-2 border-l-primary bg-primary/10 pl-[10px]" : "hover:bg-muted/60"}`}
              aria-current={active ? "true" : undefined}
            >
              <span className="min-w-0">
                <span className={`block truncate font-mono text-[11px] font-semibold ${active ? "text-primary" : "text-foreground"}`}>{item.symbol}</span>
                <span className="block truncate text-[10px] text-muted-foreground">{item.name}</span>
              </span>
              <span className="text-right font-mono text-[11px] font-medium tabular-nums text-foreground">{formatPrice(item)}</span>
              <span className={`text-right font-mono text-[10px] font-medium tabular-nums ${item.price <= 0 || item.changePercent === 0 ? "text-muted-foreground" : item.changePercent > 0 ? "text-up" : "text-down"}`}>
                {item.price > 0 ? `${item.changePercent > 0 ? "+" : ""}${item.changePercent.toFixed(2)}%` : "—"}
              </span>
            </button>
          );
        })}
      </div>

      {headline && (
        <button type="button" onClick={onOpenNews} className="flex shrink-0 items-center gap-2 border-t border-border bg-muted/40 px-3 py-2 text-left hover:bg-muted">
          <span className="shrink-0 text-[9px] font-bold uppercase tracking-wider text-primary">News</span>
          <span className="truncate text-[10px] text-muted-foreground">{headline}</span>
        </button>
      )}
    </div>
  );
}
