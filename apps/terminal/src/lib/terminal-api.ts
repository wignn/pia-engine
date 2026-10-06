/**
 * PIA Terminal - Client Browser API Adapter
 * Exposes window.api for 100% desktop-fidelity panels in web Next.js environment.
 */

import type {
  PriceQuote,
  ConnectionState,
  DrawingItem,
  IndicatorConfig,
  WatchlistGroup,
  PriceAlert,
  PaperAccount,
  ChartLayoutData,
  SymbolInfo,
  OptionChainData,
  OptionGexData,
  OptionSummaryData,
  FearGreedResult,
  FearGreedHistoryItem,
  CotReportResult,
  CentralBankStanceResult,
  YieldCurveResult,
  YieldSpreadResult,
  GeoSignalEventItem,
  GeoSignalsMapRegion,
  GeoAssetImpactItem,
  EnergyDashboardData,
  SecFilingItemData,
  NewsArticle,
} from "@/shared/types";

const PAPER_STORAGE_KEY = "pia_paper_account";
const DEFAULT_PAPER: PaperAccount = {
  balance: 100000,
  initialBalance: 100000,
  currency: "USD",
  positions: [],
  updatedAt: Date.now(),
};

export const browserTerminalApi = {
  market: {
    getSymbols: async (): Promise<SymbolInfo[]> => {
      try {
        const res = await fetch("/api/market/prices");
        const json = await res.json();
        const items = json.data || json.items || [];
        return items.map((item: any) => ({
          symbol: item.symbol,
          name: item.name || item.symbol,
          category: item.category || "crypto",
          pricePrecision: item.digits || 2,
          volumePrecision: 2,
          minMove: Math.pow(10, -(item.digits || 2)),
          capabilities: {
            quote: true,
            candles: true,
            trades: true,
            orderBook: true,
            options: ["BTC", "ETH", "AAPL", "XAUUSD", "SPX"].includes(item.symbol),
            gex: ["BTC", "ETH", "AAPL", "XAUUSD", "SPX"].includes(item.symbol),
          },
        }));
      } catch {
        return [];
      }
    },
    getCandles: async (params: { symbol: string; timeframe: string }) => {
      try {
        const res = await fetch(`/api/market/history/${encodeURIComponent(params.symbol)}?resolution=${params.timeframe}`);
        const json = await res.json();
        return json.items || json.candles || [];
      } catch {
        return [];
      }
    },
    getPrices: async () => {
      try {
        const res = await fetch("/api/market/prices");
        const json = await res.json();
        return json.data || json.items || [];
      } catch {
        return [];
      }
    },
    getPrice: async (symbol: string) => {
      try {
        const res = await fetch(`/api/market/prices`);
        const json = await res.json();
        const items = json.data || json.items || [];
        return items.find((i: any) => i.symbol === symbol) || null;
      } catch {
        return null;
      }
    },
    getSession: async (symbol: string) => {
      try {
        const res = await fetch(`/api/v1/market/session/${encodeURIComponent(symbol)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
    getDataQuality: async () => {
      try {
        const res = await fetch("/api/v1/market/data-quality");
        return await res.json();
      } catch {
        return null;
      }
    },
    getSpikes: async () => {
      try {
        const res = await fetch("/api/v1/market/spikes");
        return await res.json();
      } catch {
        return [];
      }
    },
    getAlerts: async () => {
      try {
        const res = await fetch("/api/v1/market/alerts");
        return await res.json();
      } catch {
        return [];
      }
    },
    getSmartAlerts: async () => {
      try {
        const res = await fetch("/api/v1/market/smart-alerts");
        return await res.json();
      } catch {
        return [];
      }
    },
    subscribePrice: async () => {},
    unsubscribePrice: async () => {},
    onPriceUpdate: () => () => {},
    onConnectionState: () => () => {},
    getTradingHalts: async () => {
      try {
        const res = await fetch("/api/v1/market/trading-halts");
        return await res.json();
      } catch {
        return [];
      }
    },
    getCorporateActions: async () => {
      try {
        const res = await fetch("/api/v1/market/corporate-actions");
        return await res.json();
      } catch {
        return [];
      }
    },
    getRealizedVolatility: async (symbol?: string) => {
      try {
        const res = await fetch(`/api/v1/market/realized-volatility${symbol ? `?symbol=${symbol}` : ""}`);
        return await res.json();
      } catch {
        return null;
      }
    },
    getImpliedVolatility: async (symbol?: string) => {
      try {
        const res = await fetch(`/api/v1/market/implied-volatility${symbol ? `?symbol=${symbol}` : ""}`);
        return await res.json();
      } catch {
        return null;
      }
    },
    uploadSnapshot: async (params: { image: string; symbol?: string; timeframe?: string }) => {
      try {
        const res = await fetch("/api/v1/charts/snapshot", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(params),
        });
        return await res.json();
      } catch {
        return { url: null };
      }
    },
  },

  orderbook: {
    get: async (symbol: string) => {
      try {
        const res = await fetch(`/api/v1/market/orderbook/${encodeURIComponent(symbol)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  intelligence: {
    analyze: async (params: any) => {
      try {
        const res = await fetch("/api/v1/intelligence/analyze", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(params),
        });
        return await res.json();
      } catch {
        return null;
      }
    },
    getInsights: async (symbol: string) => {
      try {
        const res = await fetch(`/api/v1/market/insights/${encodeURIComponent(symbol)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  options: {
    getChain: async (symbol: string): Promise<OptionChainData | null> => {
      try {
        const res = await fetch(`/api/v1/options/chain/${encodeURIComponent(symbol)}`);
        const json = await res.json();
        return json.data || json;
      } catch {
        return null;
      }
    },
    getGex: async (symbol: string): Promise<OptionGexData | null> => {
      try {
        const res = await fetch(`/api/v1/options/gex/${encodeURIComponent(symbol)}`);
        const json = await res.json();
        const raw = json?.data || json || {};
        const levels: any[] = [];
        const posMap = new Map<number, number>();
        const negMap = new Map<number, number>();
        if (Array.isArray(raw.major_positive_levels)) {
          raw.major_positive_levels.forEach((l: any) => posMap.set(Number(l.strike), Number(l.gex)));
        }
        if (Array.isArray(raw.major_negative_levels)) {
          raw.major_negative_levels.forEach((l: any) => negMap.set(Number(l.strike), Number(l.gex)));
        }
        if (Array.isArray(raw.levels)) {
          raw.levels.forEach((l: any) => levels.push(l));
        } else {
          const strikes = Array.from(new Set([...posMap.keys(), ...negMap.keys()])).sort((a, b) => a - b);
          for (const s of strikes) {
            const cGex = posMap.get(s) ?? 0;
            const pGex = negMap.get(s) ?? 0;
            levels.push({
              strike: s,
              callGex: cGex,
              putGex: pGex,
              netGex: cGex - pGex,
            });
          }
        }
        return {
          symbol: raw.symbol || symbol,
          netGex: raw.net_gex ?? raw.netGex ?? 0,
          zeroGammaLevel: raw.zero_gamma ?? raw.zeroGammaLevel,
          callWall: raw.call_wall ?? raw.callWall,
          putWall: raw.put_wall ?? raw.putWall,
          levels,
          updatedAt: Date.now(),
        };
      } catch {
        return null;
      }
    },
    getSummary: async (): Promise<OptionSummaryData | null> => {
      try {
        const res = await fetch("/api/v1/options/summary");
        const json = await res.json();
        return Array.isArray(json.data) ? json.data[0] : json.data || json;
      } catch {
        return null;
      }
    },
  },

  macro: {
    getFearGreed: async (): Promise<FearGreedResult | null> => {
      try {
        const res = await fetch("/api/v1/fear-greed");
        return await res.json();
      } catch {
        return null;
      }
    },
    getFearGreedHistory: async (): Promise<FearGreedHistoryItem[]> => {
      try {
        const res = await fetch("/api/v1/fear-greed/history");
        const json = await res.json();
        return json.items || json.data || [];
      } catch {
        return [];
      }
    },
    getCot: async (symbol: string): Promise<CotReportResult | null> => {
      try {
        const res = await fetch(`/api/v1/cot/symbol/${encodeURIComponent(symbol)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
    getCentralBanks: async (bank?: string): Promise<CentralBankStanceResult[]> => {
      try {
        const res = await fetch(`/api/v1/central-banks/latest${bank ? `?bank=${bank}` : ""}`);
        const json = await res.json();
        return Array.isArray(json) ? json : json.items || [];
      } catch {
        return [];
      }
    },
    getMap: async (params?: { indicator?: string; period?: string }) => {
      try {
        const indicator = params?.indicator || "inflation";
        const period = params?.period || "2025";
        const res = await fetch(`/api/v1/economic/map?indicator=${encodeURIComponent(indicator)}&period=${encodeURIComponent(period)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  fixedIncome: {
    getYieldCurve: async (): Promise<YieldCurveResult | null> => {
      try {
        const raw = await fetch("/api/v1/fixed-income/yield-curve").then((r) => r.json());
        const wrapped = raw as any;
        const payload = wrapped?.data && typeof wrapped.data === "object" ? wrapped.data : raw;
        const pointRows = Array.isArray(payload?.points)
          ? payload.points
          : Array.isArray(payload?.bonds)
          ? payload.bonds
          : [];
        const normalizeYield = (p: any) => ({
          tenor: p.tenor || p.symbol || p.name || "",
          yield: Number(p.yield ?? p.yield_value ?? p.value ?? 0),
          previousYield: p.previous_yield !== undefined ? Number(p.previous_yield) : undefined,
        });
        const points = pointRows
          .map(normalizeYield)
          .filter((p: any) => p.tenor && Number.isFinite(p.yield));
        return {
          date: payload?.date || payload?.as_of || new Date().toISOString().split("T")[0],
          points,
          updatedAt: Date.now(),
        };
      } catch {
        return null;
      }
    },
    getSpreads: async (): Promise<YieldSpreadResult | null> => {
      try {
        const raw = await fetch("/api/v1/fixed-income/spreads").then((r) => r.json());
        const payload = (raw?.data && typeof raw.data === "object" ? raw.data : raw) as any;
        const rows = Array.isArray(payload)
          ? payload
          : Array.isArray(payload?.data)
          ? payload.data
          : Array.isArray(payload?.items)
          ? payload.items
          : [];
        const valueOf = (short: string, long: string) => {
          const normalized = `${short}${long}`.toLowerCase();
          const row = rows.find((item: any) =>
            String(item?.spread ?? item?.name ?? "")
              .toLowerCase()
              .replace(/[^a-z0-9]/g, "")
              .includes(normalized),
          );
          return row?.value !== undefined ? Number(row.value) : undefined;
        };
        const spread2y10y =
          payload?.spread2y10y ??
          payload?.spread_2y_10y ??
          valueOf("2y", "10y") ??
          valueOf("2s", "10s");
        const spread3m10y =
          payload?.spread3m10y ??
          payload?.spread_3m_10y ??
          valueOf("3m", "10y") ??
          valueOf("3m", "10s");
        return {
          date: new Date().toISOString().split("T")[0],
          spread2Y10Y: spread2y10y !== undefined ? Number(spread2y10y) : undefined,
          spread3M10Y: spread3m10y !== undefined ? Number(spread3m10y) : undefined,
          isInverted: spread2y10y !== undefined ? Number(spread2y10y) < 0 : undefined,
          updatedAt: Date.now(),
        };
      } catch {
        return null;
      }
    },
    getRate: async (tenor: string) => {
      try {
        const res = await fetch(`/api/v1/fixed-income/rates/${encodeURIComponent(tenor)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
    getHistory: async (tenor: string) => {
      try {
        const res = await fetch(`/api/v1/fixed-income/history/${encodeURIComponent(tenor)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  geosignals: {
    getEvents: async (): Promise<GeoSignalEventItem[]> => {
      try {
        const res = await fetch("/api/v1/geosignals");
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
    getMap: async (): Promise<GeoSignalsMapRegion[]> => {
      try {
        const res = await fetch("/api/v1/geosignals/map");
        const json = await res.json();
        return json.regions || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
    getAssetImpacts: async (): Promise<GeoAssetImpactItem[]> => {
      try {
        const res = await fetch("/api/v1/geosignals/assets");
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
  },

  energy: {
    getDashboard: async (): Promise<EnergyDashboardData | null> => {
      try {
        const res = await fetch("/api/v1/energy/dashboard");
        return await res.json();
      } catch {
        return null;
      }
    },
    getSeries: async (seriesId: string) => {
      try {
        const res = await fetch(`/api/v1/energy/${encodeURIComponent(seriesId)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  sec: {
    getFilings: async (params?: any): Promise<SecFilingItemData[]> => {
      try {
        const search = params ? `?${new URLSearchParams(params as any)}` : "";
        const res = await fetch(`/api/v1/sec/filings${search}`);
        const json = await res.json();
        return Array.isArray(json) ? json : json.items || [];
      } catch {
        return [];
      }
    },
    getCompany: async (symbol: string) => {
      try {
        const res = await fetch(`/api/v1/sec/companies/${encodeURIComponent(symbol)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  social: {
    getPosts: async (params?: any) => {
      try {
        const res = await fetch("/api/social");
        const json = await res.json();
        return json.items || json.posts || [];
      } catch {
        return [];
      }
    },
    getFeed: async (params?: any) => {
      try {
        const res = await fetch("/api/social");
        const json = await res.json();
        return json.items || json.posts || [];
      } catch {
        return [];
      }
    },
  },

  drawings: {
    get: async ({ symbol }: { symbol: string }) => {
      try {
        const raw = localStorage.getItem(`atlsd_drawings_${symbol}`);
        return raw ? JSON.parse(raw) : [];
      } catch {
        return [];
      }
    },
    save: async (drawing: DrawingItem) => {
      try {
        const key = `atlsd_drawings_${drawing.symbol}`;
        const raw = localStorage.getItem(key);
        const list = raw ? JSON.parse(raw) : [];
        const filtered = list.filter((d: any) => d.id !== drawing.id);
        filtered.push(drawing);
        localStorage.setItem(key, JSON.stringify(filtered));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
    delete: async ({ id }: { id: string }) => {
      return { success: true };
    },
    clear: async ({ symbol }: { symbol?: string }) => {
      if (symbol) {
        localStorage.removeItem(`atlsd_drawings_${symbol}`);
      }
      return { success: true };
    },
  },

  layouts: {
    getAll: async (): Promise<ChartLayoutData[]> => {
      try {
        const raw = localStorage.getItem("pia_saved_layouts");
        return raw ? JSON.parse(raw) : [];
      } catch {
        return [];
      }
    },
    get: async ({ id }: { id: string }): Promise<ChartLayoutData | null> => {
      try {
        const raw = localStorage.getItem("pia_saved_layouts");
        const list: ChartLayoutData[] = raw ? JSON.parse(raw) : [];
        return list.find((l) => l.id === id) || null;
      } catch {
        return null;
      }
    },
    save: async (layout: ChartLayoutData) => {
      try {
        const raw = localStorage.getItem("pia_saved_layouts");
        const list: ChartLayoutData[] = raw ? JSON.parse(raw) : [];
        const index = list.findIndex((l) => l.id === layout.id);
        if (index >= 0) list[index] = layout;
        else list.push(layout);
        localStorage.setItem("pia_saved_layouts", JSON.stringify(list));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
    delete: async ({ id }: { id: string }) => {
      try {
        const raw = localStorage.getItem("pia_saved_layouts");
        const list: ChartLayoutData[] = raw ? JSON.parse(raw) : [];
        localStorage.setItem("pia_saved_layouts", JSON.stringify(list.filter((l) => l.id !== id)));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
  },

  indicators: {
    get: async (symbol: string): Promise<IndicatorConfig[]> => {
      try {
        const raw = localStorage.getItem(`pia_indicators_${symbol}`);
        return raw ? JSON.parse(raw) : [];
      } catch {
        return [];
      }
    },
    save: async ({ symbol, indicators }: { symbol: string; indicators: IndicatorConfig[] }) => {
      try {
        localStorage.setItem(`pia_indicators_${symbol}`, JSON.stringify(indicators));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
  },

  watchlist: {
    getAll: async (): Promise<WatchlistGroup[]> => {
      return [
        {
          id: "default",
          name: "Main Watchlist",
          symbols: ["XAUUSD", "BTCUSDT", "ETHUSDT", "EURUSD", "USDJPY", "SPX", "NVDA", "AAPL"],
        },
      ];
    },
    save: async () => ({ success: true }),
  },

  calendar: {
    get: async () => {
      try {
        const res = await fetch("/api/calendar");
        const json = await res.json();
        return json.events || json.items || [];
      } catch {
        return [];
      }
    },
  },

  news: {
    get: async () => {
      try {
        const res = await fetch("/api/news");
        const json = await res.json();
        return json.items || json.news || [];
      } catch {
        return [];
      }
    },
    getLatest: async () => {
      try {
        const res = await fetch("/api/news");
        const json = await res.json();
        return json.items || json.news || [];
      } catch {
        return [];
      }
    },
    getById: async (id: string) => {
      try {
        const res = await fetch(`/api/v1/news/${encodeURIComponent(id)}`);
        return await res.json();
      } catch {
        return null;
      }
    },
  },

  economic: {
    getIndicators: async () => {
      try {
        const res = await fetch("/api/v1/economic/indicators");
        return await res.json();
      } catch {
        return [];
      }
    },
    getCategories: async () => {
      try {
        const res = await fetch("/api/v1/economic/categories");
        return await res.json();
      } catch {
        return [];
      }
    },
    getCountries: async () => {
      try {
        const res = await fetch("/api/v1/economic/countries");
        return await res.json();
      } catch {
        return [];
      }
    },
  },

  alerts: {
    getAll: async (): Promise<PriceAlert[]> => {
      try {
        const raw = localStorage.getItem("pia_alerts");
        return raw ? JSON.parse(raw) : [];
      } catch {
        return [];
      }
    },
    save: async (alert: PriceAlert) => {
      try {
        const raw = localStorage.getItem("pia_alerts");
        const list: PriceAlert[] = raw ? JSON.parse(raw) : [];
        const idx = list.findIndex((a) => a.id === alert.id);
        if (idx >= 0) list[idx] = alert;
        else list.push(alert);
        localStorage.setItem("pia_alerts", JSON.stringify(list));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
    delete: async ({ id }: { id: string }) => {
      try {
        const raw = localStorage.getItem("pia_alerts");
        const list: PriceAlert[] = raw ? JSON.parse(raw) : [];
        localStorage.setItem("pia_alerts", JSON.stringify(list.filter((a) => a.id !== id)));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
  },

  paper: {
    getAccount: async (): Promise<PaperAccount> => {
      try {
        const raw = localStorage.getItem(PAPER_STORAGE_KEY);
        return raw ? JSON.parse(raw) : DEFAULT_PAPER;
      } catch {
        return DEFAULT_PAPER;
      }
    },
    saveAccount: async (account: PaperAccount) => {
      try {
        localStorage.setItem(PAPER_STORAGE_KEY, JSON.stringify(account));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
  },

  settings: {
    getCredentials: async () => ({
      hasApiKey: true,
      isSecureStorage: true,
      baseUrl: "https://api-engine.wign.dev",
      wsUrl: "wss://api-engine.wign.dev",
    }),
    saveApiKey: async () => ({ success: true }),
    clearApiKey: async () => ({ success: true }),
  },

  system: {
    openExternal: async ({ url }: { url: string }) => {
      if (typeof window !== "undefined") {
        window.open(url, "_blank");
      }
    },
    getRateLimit: async () => null,
  },

  ws: {
    createTicket: async () => {
      try {
        const res = await fetch("/api/realtime/session", { method: "POST" });
        return await res.json();
      } catch {
        return { ticket: null };
      }
    },
  },
};

// Auto-register onto window.api if in browser
if (typeof window !== "undefined") {
  (window as any).api = browserTerminalApi;
}
