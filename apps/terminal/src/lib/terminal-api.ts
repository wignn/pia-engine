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
  EconomicEvent,
  NewsArticle,
  CredentialStatus,
  SymbolInfo,
  PriceAlert,
  PaperAccount,
  OrderBookData,
  OptionChainData,
  OptionContractData,
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
  SecCompanyData,
  SocialPostItemData,
  ChartLayoutData,
  MacroMapResult,
  CountryMacroData,
} from "@/shared/types";
import { resolveOptionsUnderlying } from "@/shared/market-utils";

const DEFAULT_PAPER: PaperAccount = {
  balance: 100000,
  initialBalance: 100000,
  currency: "USD",
  positions: [],
  updatedAt: Date.now(),
};

const PAPER_STORAGE_KEY = "atlsd_paper_trading_account";

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
            options: true,
            gex: true,
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
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
    getAlerts: async () => {
      try {
        const res = await fetch("/api/v1/market/alerts");
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
    getSmartAlerts: async () => {
      try {
        const res = await fetch("/api/v1/market/smart-alerts");
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
    subscribePrice: async (symbol: string) => {},
    unsubscribePrice: async (symbol: string) => {},
    onPriceUpdate: (callback: (quote: PriceQuote) => void) => {
      return () => {};
    },
    onConnectionState: (callback: (state: ConnectionState) => void) => {
      callback({ status: "connected" });
      return () => {};
    },
    getTradingHalts: async () => {
      try {
        const res = await fetch("/api/v1/market/trading-halts");
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
      } catch {
        return [];
      }
    },
    getCorporateActions: async () => {
      try {
        const res = await fetch("/api/v1/market/corporate-actions");
        const json = await res.json();
        return json.items || (Array.isArray(json) ? json : []);
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
    uploadSnapshot: async (params: { image: string }) => {
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
    get: async (symbol: string): Promise<OrderBookData | null> => {
      try {
        const res = await fetch(`/api/v1/market/orderbook/${encodeURIComponent(symbol)}`);
        const ob = await res.json();
        if (ob && ob.bids && ob.asks) {
          const spread =
            ob.asks.length > 0 && ob.bids.length > 0 ? ob.asks[0].price - ob.bids[0].price : 0;
          const mid = ob.bids.length > 0 ? ob.bids[0].price : 1;
          return {
            symbol,
            bids: ob.bids.map((b: any) => ({ price: Number(b.price), size: Number(b.size) })),
            asks: ob.asks.map((a: any) => ({ price: Number(a.price), size: Number(a.size) })),
            spread: Math.max(0, spread),
            spreadPercent: mid > 0 ? (spread / mid) * 100 : 0,
            timestamp: ob.timestamp || Date.now(),
          };
        }
        return null;
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
        const underlying = resolveOptionsUnderlying(symbol);
        const res = await fetch(`/api/v1/options/chain?symbol=${encodeURIComponent(underlying)}`).then((r) => r.json());
        if (res && Array.isArray(res.contracts)) {
          const calls: OptionContractData[] = [];
          const puts: OptionContractData[] = [];
          for (const c of res.contracts) {
            const item: OptionContractData = {
              strike: Number(c.strike),
              type: c.option_type?.toLowerCase() === "put" ? "put" : "call",
              expiration: c.expiration || "",
              bid: Number(c.bid ?? 0),
              ask: Number(c.ask ?? 0),
              last: Number(c.last ?? 0),
              volume: Number(c.volume ?? 0),
              openInterest: Number(c.open_interest ?? 0),
              impliedVolatility: Number(c.implied_volatility ?? 0.2),
              delta: c.delta !== undefined ? Number(c.delta) : undefined,
              gamma: c.gamma !== undefined ? Number(c.gamma) : undefined,
              theta: c.theta !== undefined ? Number(c.theta) : undefined,
              vega: c.vega !== undefined ? Number(c.vega) : undefined,
            };
            if (item.type === "put") puts.push(item);
            else calls.push(item);
          }
          const expirations = Array.from(new Set(res.contracts.map((c: any) => c.expiration))).filter(Boolean) as string[];
          return {
            symbol,
            underlyingPrice: Number(res.underlying_price ?? res.underlyingPrice ?? 0),
            calls,
            puts,
            expirations,
            timestamp: Date.now(),
          };
        }
        return null;
      } catch {
        return null;
      }
    },
    getGex: async (symbol: string): Promise<OptionGexData | null> => {
      try {
        const underlying = resolveOptionsUnderlying(symbol);
        const res = await fetch(`/api/v1/options/gex?symbol=${encodeURIComponent(underlying)}`).then((r) => r.json());
        const raw = res?.data || res || {};
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
          netGex: Number(raw.net_gex ?? raw.netGex ?? 0),
          zeroGammaLevel: raw.zero_gamma !== undefined ? Number(raw.zero_gamma) : undefined,
          callWall: raw.call_wall !== undefined ? Number(raw.call_wall) : undefined,
          putWall: raw.put_wall !== undefined ? Number(raw.put_wall) : undefined,
          levels,
          updatedAt: Date.now(),
        };
      } catch {
        return null;
      }
    },
    getSummary: async (): Promise<OptionSummaryData | null> => {
      try {
        const res = await fetch("/api/v1/options/summary").then((r) => r.json());
        const rows = Array.isArray(res?.data)
          ? res.data
          : Array.isArray(res?.items)
          ? res.items
          : Array.isArray(res)
          ? res
          : [];
        if (rows.length === 0) return null;
        const totalVolume = rows.reduce(
          (sum: number, r: any) => sum + Number(r.total_volume ?? r.volume ?? 0),
          0,
        );
        const totalOpenInterest = rows.reduce(
          (sum: number, r: any) => sum + Number(r.total_open_interest ?? r.open_interest ?? 0),
          0,
        );
        const volumeWeightedPcr = rows.reduce(
          (sum: number, r: any) => sum + Number(r.put_call_ratio ?? 0) * Number(r.total_volume ?? r.volume ?? 0),
          0,
        );
        const putCallRatio =
          totalVolume > 0 ? volumeWeightedPcr / totalVolume : Number(rows[0]?.put_call_ratio ?? 0.82);
        const mostActiveSymbols = rows.map((s: any) => ({
          symbol: String(s.symbol || s.id),
          volume: Number(s.total_volume ?? s.volume ?? 0),
          pcr: s.put_call_ratio !== undefined ? Number(s.put_call_ratio) : undefined,
        }));
        return {
          totalVolume,
          totalOpenInterest,
          putCallRatio,
          mostActiveSymbols,
        };
      } catch {
        return null;
      }
    },
  },

  macro: {
    getFearGreed: async (): Promise<FearGreedResult | null> => {
      try {
        const res = await fetch("/api/v1/fear-greed").then((r) => r.json());
        if (!res) return null;
        const rawRating = res.rating || "Neutral";
        const rating: FearGreedResult["rating"] =
          rawRating === "Extreme Fear" ||
          rawRating === "Fear" ||
          rawRating === "Greed" ||
          rawRating === "Extreme Greed"
            ? rawRating
            : "Neutral";
        return {
          score: Number(res.score ?? 50),
          rating,
          timestamp: typeof res.timestamp === "number" ? res.timestamp : Date.now(),
          previousClose: res.previous_close !== undefined ? Number(res.previous_close) : undefined,
          previous1Week: res.previous_1_week !== undefined ? Number(res.previous_1_week) : undefined,
          previous1Month: res.previous_1_month !== undefined ? Number(res.previous_1_month) : undefined,
          previous1Year: res.previous_1_year !== undefined ? Number(res.previous_1_year) : undefined,
        };
      } catch {
        return null;
      }
    },
    getFearGreedHistory: async (): Promise<FearGreedHistoryItem[]> => {
      try {
        const res = await fetch("/api/v1/fear-greed/history").then((r) => r.json());
        const history = res.history || res.items || (Array.isArray(res) ? res : []);
        return history.map((h: any) => ({
          score: Number(h.score ?? 50),
          rating: h.rating || "Neutral",
          timestamp: typeof h.timestamp === "number" ? h.timestamp : new Date(h.timestamp || h.date).getTime(),
        }));
      } catch {
        return [];
      }
    },
    getCot: async (symbol: string): Promise<CotReportResult | null> => {
      try {
        const res = await fetch(`/api/v1/cot/symbol/${encodeURIComponent(symbol)}`).then((r) => r.json());
        const reports = Array.isArray(res?.data)
          ? res.data
          : Array.isArray(res?.items)
          ? res.items
          : Array.isArray(res?.reports)
          ? res.reports
          : [];
        if (reports.length > 0) {
          const r = reports[0];
          const positions = reports.map((item: any) => ({
            category: item.market_name || item.category || "All Categories",
            longPositions: Number(item.noncommercial_long ?? item.non_commercial_long ?? 0),
            shortPositions: Number(item.noncommercial_short ?? item.non_commercial_short ?? 0),
            netPositions: Number(item.noncommercial_net ?? item.non_commercial_net ?? 0),
          }));
          return {
            symbol,
            marketCode: String(r.market_code || "088691"),
            asOfDate: r.report_date || r.as_of_date || r.date || new Date().toISOString().split("T")[0],
            commercialNet: Number(
              r.commercial_net ??
                (r.commercial_long && r.commercial_short ? r.commercial_long - r.commercial_short : 0),
            ),
            nonCommercialNet: Number(
              r.noncommercial_net ??
                (r.noncommercial_long && r.noncommercial_short
                  ? r.noncommercial_long - r.noncommercial_short
                  : 0),
            ),
            positions,
          };
        }
        return null;
      } catch {
        return null;
      }
    },
    getCentralBanks: async (bank?: string): Promise<CentralBankStanceResult[]> => {
      try {
        const res = await fetch(`/api/v1/central-banks/latest${bank ? `?bank=${bank}` : ""}`).then((r) => r.json());
        const list = Array.isArray(res) ? res : res.items || [];
        const seen = new Set<string>();
        const results: CentralBankStanceResult[] = [];
        for (const item of list) {
          const code = String(item.bank || item.code || "").toUpperCase();
          if (!code || seen.has(code)) continue;
          seen.add(code);
          const rawStance = String(item.stance || "").toLowerCase();
          const stance =
            rawStance === "hawkish" ? "Hawkish" : rawStance === "dovish" ? "Dovish" : "Neutral";
          results.push({
            bank: item.title || item.name || code,
            code,
            rate: item.rate !== undefined ? `${item.rate}%` : "",
            stance,
            summary: item.summary || item.title || "",
            lastUpdated: item.published_at ? new Date(item.published_at).getTime() : Date.now(),
          });
        }
        return results;
      } catch {
        return [];
      }
    },
    getMap: async (params?: { indicator?: string; period?: string }): Promise<MacroMapResult | null> => {
      try {
        const indicator = params?.indicator || "inflation";
        const query = new URLSearchParams();
        query.set("indicator", indicator);
        if (params?.period) query.set("period", params.period);

        const res = await fetch(`/api/v1/macro/map?${query.toString()}`).then((r) => r.json());
        const data = res?.data && typeof res.data === "object" && !Array.isArray(res.data) ? res.data : res;
        if (data && Array.isArray(data.countries) && data.countries.length > 0) {
          const parsedCountries: CountryMacroData[] = data.countries.map((c: any) => {
            const hist: Record<number, number> = {};
            if (c.history && typeof c.history === "object") {
              for (const [yr, val] of Object.entries(c.history)) {
                const numYr = parseInt(yr, 10);
                if (!isNaN(numYr) && typeof val === "number") {
                  hist[numYr] = val;
                }
              }
            }
            return {
              id: String(c.id || c.country_code || ""),
              name: String(c.name || c.country_name || ""),
              flag: String(c.flag || "🌐"),
              region: (c.region || "Unknown") as any,
              subregion: String(c.subregion || "Global"),
              ticker: String(c.ticker || c.id || ""),
              value: typeof c.value === "number" ? c.value : undefined,
              prevValue:
                typeof c.prevValue === "number"
                  ? c.prevValue
                  : typeof c.previous_value === "number"
                  ? c.previous_value
                  : undefined,
              change: typeof c.change === "number" ? c.change : undefined,
              unit: String(c.unit || data.unit || "%"),
              period: String(c.period || data.period || "2025"),
              history: hist,
              rank: typeof c.rank === "number" ? c.rank : undefined,
            };
          });

          return {
            indicator: data.indicator || indicator,
            indicatorName: data.indicatorName || data.indicator_name || "Macroeconomic Indicator",
            unit: data.unit || "%",
            period: data.period || "2025",
            minValue: typeof data.minValue === "number" ? data.minValue : data.min_value ?? 0,
            maxValue: typeof data.maxValue === "number" ? data.maxValue : data.max_value ?? 100,
            countries: parsedCountries,
            total: parsedCountries.length,
            timeline: Array.isArray(data.timeline) ? data.timeline : [],
            isLive: true,
          };
        }
        return null;
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
        const res = await fetch(`/api/v1/fixed-income/rates/${encodeURIComponent(tenor)}`).then((r) => r.json());
        const data = Array.isArray(res.data) ? res.data[0] : res.data || res;
        return {
          tenor,
          rate: Number(data?.value ?? data?.rate ?? data?.yield ?? 0),
          timestamp: Date.now(),
        };
      } catch {
        return null;
      }
    },
    getHistory: async (tenor: string) => {
      try {
        const res = await fetch(`/api/v1/fixed-income/rates/${encodeURIComponent(tenor)}`).then((r) => r.json());
        const rows = Array.isArray(res.data) ? res.data : Array.isArray(res.items) ? res.items : [];
        const points = rows.map((r: any) => ({
          date: r.date || new Date().toISOString().split("T")[0],
          value: Number(r.value ?? r.rate ?? r.yield ?? 0),
        }));
        return {
          tenor,
          points,
          timestamp: Date.now(),
        };
      } catch {
        return null;
      }
    },
  },

  geosignals: {
    getEvents: async (): Promise<GeoSignalEventItem[]> => {
      try {
        const res = await fetch("/api/v1/geosignals").then((r) => r.json());
        const events = res.items || (Array.isArray(res) ? res : []);
        return events.map((e: any, idx: number) => {
          let category: GeoSignalEventItem["category"] = "conflict";
          const catLower = String(e.category || "").toLowerCase();
          if (catLower.includes("sanction")) category = "sanctions";
          else if (catLower.includes("trade")) category = "trade";
          else if (catLower.includes("supply") || catLower.includes("maritime")) category = "maritime";
          else if (catLower.includes("election") || catLower.includes("diplomat")) category = "diplomatic";

          const rawSev = typeof e.severity === "string" ? e.severity.toLowerCase() : "";
          const numSev = Number(e.severity ?? e.severity_score ?? e.confidence_score ?? 0);
          const severity: GeoSignalEventItem["severity"] =
            rawSev === "critical" || numSev > 7 || numSev > 0.7
              ? "critical"
              : rawSev === "high" || numSev > 5 || numSev > 0.5
              ? "high"
              : rawSev === "medium" || numSev > 3 || numSev > 0.3
              ? "medium"
              : "low";

          return {
            id: e.id || e.event_id || `geo-${idx}`,
            timestamp: e.timestamp || e.created_at || e.published_at ? new Date(e.timestamp || e.created_at || e.published_at).getTime() : Date.now(),
            category,
            severity,
            headline: e.headline || e.title || e.event_id || "Geopolitical Intelligence Alert",
            description: e.description || e.summary || e.reason || "",
            affectedAssets: Array.isArray(e.affected_assets) ? e.affected_assets : [],
            primaryRegion: e.primary_region || e.region || e.country || "Global",
            source: e.source || "Intelligence Feed",
          };
        });
      } catch {
        return [];
      }
    },
    getMap: async (): Promise<GeoSignalsMapRegion[]> => {
      try {
        const res = await fetch("/api/v1/geosignals/map").then((r) => r.json());
        const layers = Array.isArray(res.items) ? res.items : Array.isArray(res.regions) ? res.regions : Array.isArray(res) ? res : [];
        return layers.map((l: any) => {
          const numScore = Number(l.risk_level ?? l.max_severity ?? l.avg_severity ?? 0);
          const normalizedScore = numScore <= 1.0 ? numScore * 10 : numScore;
          const riskLevel: GeoSignalsMapRegion["riskLevel"] =
            normalizedScore > 7
              ? "critical"
              : normalizedScore > 5
              ? "high"
              : normalizedScore > 3
              ? "elevated"
              : "moderate";
          return {
            region: l.region || l.key || "Global",
            riskLevel,
            activeHotspots: l.active_conflicts ?? (l.signal_count ? Number(l.signal_count) : 0),
            chokepointStatus: l.chokepoints_status
              ? Object.entries(l.chokepoints_status)
                  .map(([name, status]) => `${name}: ${status}`)
                  .join("; ")
              : undefined,
          };
        });
      } catch {
        return [];
      }
    },
    getAssetImpacts: async (): Promise<GeoAssetImpactItem[]> => {
      try {
        const res = await fetch("/api/v1/geosignals/assets").then((r) => r.json());
        const assets = Array.isArray(res.items) ? res.items : Array.isArray(res.assets) ? res.assets : Array.isArray(res) ? res : [];
        return assets.map((a: any) => {
          const rawScore = Number(a.risk_score ?? a.max_severity ?? a.avg_severity ?? 0);
          const riskScore = rawScore <= 1.0 ? Math.round(rawScore * 100) : Math.round(rawScore);
          const supplyDisruptionRisk: GeoAssetImpactItem["supplyDisruptionRisk"] =
            riskScore > 65 || (a.affected_supply_pct && a.affected_supply_pct > 15)
              ? "high"
              : riskScore > 35
              ? "medium"
              : "low";
          return {
            symbol: a.symbol || a.asset || "",
            riskScore,
            primaryDriver: a.primary_risk_driver || a.category || "Regional tension / Supply chain",
            supplyDisruptionRisk,
          };
        });
      } catch {
        return [];
      }
    },
  },

  energy: {
    getDashboard: async (): Promise<EnergyDashboardData | null> => {
      try {
        const res = await fetch("/api/v1/energy/dashboard").then((r) => r.json());
        if (!res) return null;
        return {
          wtiPrice: res.crude_oil?.wti_price ?? 86.89,
          brentPrice: res.crude_oil?.brent_price ?? 93.97,
          wtiBrentSpread: res.crude_oil?.spread ?? -7.08,
          henryHubPrice: res.natural_gas?.henry_hub_price ?? 2.784,
          naturalGasStorageBcf: res.natural_gas?.storage_bcf ?? 3415.0,
          crackSpread321: res.refining_margins ? Number(res.refining_margins["321"] ?? 19.12) : 19.12,
          updatedAt: res.updated_at ? new Date(res.updated_at).getTime() : Date.now(),
        };
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
        const search = params?.symbol ? `?ticker=${encodeURIComponent(params.symbol)}` : "";
        const res = await fetch(`/api/v1/sec/filings${search}`).then((r) => r.json());
        const items = Array.isArray(res) ? res : res.items || res.data || [];
        return items.map((i: any) => {
          const rawForm = i.form_type || "OTHER";
          const formType: SecFilingItemData["formType"] =
            rawForm === "10-K" || rawForm === "10-Q" || rawForm === "8-K" || rawForm === "4" || rawForm === "13F"
              ? rawForm
              : "OTHER";
          const companyName = i.company_name || i.raw_json?.companyName || i.title || i.ticker || "";
          return {
            id: String(i.id || i.accession_number || Math.random()),
            symbol: i.symbol || i.ticker || "",
            companyName,
            formType,
            filedDate: i.filing_date || i.created_at || new Date().toISOString().split("T")[0],
            title: i.title || `${rawForm} Filing - ${companyName || i.ticker || ""}`,
            description: i.description || `Form ${rawForm} submitted to SEC EDGAR database.`,
            reportUrl: i.report_url || i.document_url || "#",
            isInsiderTrade: rawForm === "4",
          };
        });
      } catch {
        return [];
      }
    },
    getCompany: async (symbol: string): Promise<SecCompanyData | null> => {
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
        const res = await fetch("/api/social").then((r) => r.json());
        const items = res.items || res.posts || [];
        return items.map((p: any, idx: number) => {
          const text = p.text || p.content || "";
          const lower = text.toLowerCase();
          const sentiment: SocialPostItemData["sentiment"] =
            p.sentiment ||
            (lower.includes("bull") || lower.includes("breakout")
              ? "bullish"
              : lower.includes("bear") || lower.includes("drop")
              ? "bearish"
              : "neutral");
          return {
            id: p.id || p.event_id || `soc-${idx}`,
            source: p.platform || "Twitter / X",
            author: p.author_display_name || p.author_username || "MarketWatcher",
            handle: p.author_username ? `@${p.author_username}` : undefined,
            content: text,
            sentiment,
            timestamp: p.created_at ? new Date(p.created_at).getTime() : Date.now(),
            likes: p.like_count ?? 0,
            reposts: p.repost_count ?? 0,
          };
        });
      } catch {
        return [];
      }
    },
    getFeed: async (params?: any) => {
      return browserTerminalApi.social.getPosts(params);
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
        const list: DrawingItem[] = raw ? JSON.parse(raw) : [];
        const filtered = list.filter((d: any) => d.id !== drawing.id);
        filtered.push(drawing);
        localStorage.setItem(key, JSON.stringify(filtered));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
    delete: async ({ id, symbol }: { id: string; symbol: string }) => {
      try {
        const key = `atlsd_drawings_${symbol}`;
        const raw = localStorage.getItem(key);
        const list: DrawingItem[] = raw ? JSON.parse(raw) : [];
        localStorage.setItem(key, JSON.stringify(list.filter((d) => d.id !== id)));
        return { success: true };
      } catch {
        return { success: false };
      }
    },
    clear: async ({ symbol }: { symbol: string }) => {
      try {
        localStorage.removeItem(`atlsd_drawings_${symbol}`);
        return { success: true };
      } catch {
        return { success: false };
      }
    },
  },

  layouts: {
    getAll: async (): Promise<ChartLayoutData[]> => {
      try {
        const raw = localStorage.getItem("pia_layouts");
        return raw ? JSON.parse(raw) : [];
      } catch {
        return [];
      }
    },
    get: async (id: string): Promise<ChartLayoutData | null> => {
      try {
        const raw = localStorage.getItem("pia_layouts");
        const list: ChartLayoutData[] = raw ? JSON.parse(raw) : [];
        return list.find((l) => l.id === id) || null;
      } catch {
        return null;
      }
    },
    save: async (layout: ChartLayoutData): Promise<boolean> => {
      try {
        const raw = localStorage.getItem("pia_layouts");
        const list: ChartLayoutData[] = raw ? JSON.parse(raw) : [];
        const idx = list.findIndex((l) => l.id === layout.id);
        if (idx >= 0) list[idx] = layout;
        else list.push(layout);
        localStorage.setItem("pia_layouts", JSON.stringify(list));
        return true;
      } catch {
        return false;
      }
    },
    delete: async (id: string): Promise<boolean> => {
      try {
        const raw = localStorage.getItem("pia_layouts");
        const list: ChartLayoutData[] = raw ? JSON.parse(raw) : [];
        localStorage.setItem("pia_layouts", JSON.stringify(list.filter((l) => l.id !== id)));
        return true;
      } catch {
        return false;
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
    save: async (symbol: string, indicators: IndicatorConfig[]): Promise<boolean> => {
      try {
        localStorage.setItem(`pia_indicators_${symbol}`, JSON.stringify(indicators));
        return true;
      } catch {
        return false;
      }
    },
  },

  watchlist: {
    getAll: async (): Promise<WatchlistGroup[]> => {
      try {
        const raw = localStorage.getItem("pia_watchlist_groups");
        return raw ? JSON.parse(raw) : [];
      } catch {
        return [];
      }
    },
    save: async (group: WatchlistGroup): Promise<boolean> => {
      try {
        const raw = localStorage.getItem("pia_watchlist_groups");
        const list: WatchlistGroup[] = raw ? JSON.parse(raw) : [];
        const idx = list.findIndex((w) => w.id === group.id);
        if (idx >= 0) list[idx] = group;
        else list.push(group);
        localStorage.setItem("pia_watchlist_groups", JSON.stringify(list));
        return true;
      } catch {
        return false;
      }
    },
  },

  calendar: {
    get: async (): Promise<EconomicEvent[]> => {
      try {
        const res = await fetch("/api/calendar").then((r) => r.json());
        const rawList = Array.isArray(res.events)
          ? res.events
          : Array.isArray(res.items)
          ? res.items
          : Array.isArray(res)
          ? res
          : [];
        return rawList.map((item: any, idx: number) => ({
          id: item.id || `econ-${idx}`,
          date: item.date || item.time || new Date().toISOString(),
          country: item.country || item.currency || "USD",
          event: item.event || item.name || item.title || "Economic Event",
          impact: (item.impact || "medium").toLowerCase(),
          actual: item.actual !== undefined && item.actual !== null ? String(item.actual) : undefined,
          forecast: item.forecast !== undefined && item.forecast !== null ? String(item.forecast) : undefined,
          previous: item.previous !== undefined && item.previous !== null ? String(item.previous) : undefined,
        }));
      } catch {
        return [];
      }
    },
  },

  news: {
    get: async (): Promise<NewsArticle[]> => {
      try {
        const res = await fetch("/api/news").then((r) => r.json());
        const items = res.items || res.news || res.data || [];
        return items.map((item: any, idx: number) => ({
          id: item.id || `news-${idx}`,
          title: item.title || item.original_title || "Market Update",
          summary: item.summary || "",
          url: item.url || item.original_url || "https://wign.dev",
          source: item.source_name || item.source || "Financial Wire",
          publishedAt: item.published_at ? new Date(item.published_at).getTime() : Date.now(),
        }));
      } catch {
        return [];
      }
    },
    getLatest: async () => {
      return browserTerminalApi.news.get();
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
        return true;
      } catch {
        return false;
      }
    },
    delete: async (id: string) => {
      try {
        const raw = localStorage.getItem("pia_alerts");
        const list: PriceAlert[] = raw ? JSON.parse(raw) : [];
        localStorage.setItem("pia_alerts", JSON.stringify(list.filter((a) => a.id !== id)));
        return true;
      } catch {
        return false;
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
        return true;
      } catch {
        return false;
      }
    },
  },

  settings: {
    getCredentials: async (): Promise<CredentialStatus> => {
      return {
        hasApiKey: true,
        isSecureStorage: true,
        baseUrl: "https://api-engine.wign.dev",
        wsUrl: "wss://api-engine.wign.dev",
      };
    },
    saveApiKey: async () => true,
    clearApiKey: async () => true,
  },

  system: {
    openExternal: async (url: string) => {
      window.open(url, "_blank");
      return true;
    },
    getRateLimit: async () => {
      return { remaining: 1000, limit: 1000, reset: Date.now() + 60000 };
    },
  },

  ws: {
    createTicket: async () => {
      try {
        const res = await fetch("/api/realtime/session", { method: "POST" });
        return await res.json();
      } catch {
        return null;
      }
    },
  },
};

if (typeof window !== "undefined") {
  // Bind onto window.api
  (window as any).api = browserTerminalApi;
}
