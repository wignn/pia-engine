/**
 * PIA Terminal - Shared Communication Contracts
 * Pure TypeScript contract definitions for browser and server surfaces.
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
  IntelligenceAnalyzeResult,
  MarketInsightResult,
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
  SecCompanyData,
  SocialPostItemData,
  ChartLayoutData,
  MacroMapResult,
  TradingHaltItem,
  CorporateActionItem,
  VolatilityData,
  MarketSessionData,
  MarketDataQualityData,
  MarketSpikeData,
  ProviderAlertData,
  NewsDetailData,
  EnergySeriesData,
  EconomicMetadataData,
  RateLimitStatus,
  FixedIncomeRateData,
  FixedIncomeHistoryData,
  WsTicketData,
} from "./types";

export interface SaveApiKeyInput {
  apiKey: string;
  baseUrl?: string;
  wsUrl?: string;
  [key: string]: any;
}

export interface GetCandlesInput {
  symbol: string;
  timeframe: string;
  limit?: number;
  before?: number;
  after?: number;
  to?: number;
  from?: number;
  [key: string]: any;
}

export interface GetPricesInput {
  symbols?: string[];
}

export interface UploadSnapshotInput {
  image: string;
  symbol?: string;
  timeframe?: string;
}

export interface IntelligenceAnalyzeInput {
  symbol: string;
  timeframe?: string;
  prompt?: string;
}

export interface MacroMapInput {
  indicator?: string;
  period?: string;
}

export interface SecFilingsInput {
  symbol?: string;
  formType?: string;
  limit?: number;
  offset?: number;
}

export interface SocialPostsInput {
  symbol?: string;
  limit?: number;
}

export interface GetCalendarInput {
  fromDate?: string;
  toDate?: string;
  country?: string;
  importance?: string;
  impact?: string;
  limit?: number;
  [key: string]: any;
}

export interface GetNewsInput {
  symbol?: string;
  category?: string;
  limit?: number;
  offset?: number;
  [key: string]: any;
}

export interface TerminalAPI {
  market: {
    getSymbols: () => Promise<SymbolInfo[]>;
    getCandles: (params: GetCandlesInput) => Promise<any[]>;
    getPrices: (params?: GetPricesInput) => Promise<PriceQuote[]>;
    getPrice: (symbol: string) => Promise<PriceQuote | null>;
    getSession: (symbol: string) => Promise<MarketSessionData | null>;
    getDataQuality: () => Promise<MarketDataQualityData | null>;
    getSpikes: () => Promise<MarketSpikeData[]>;
    getAlerts: () => Promise<ProviderAlertData[]>;
    getSmartAlerts: () => Promise<ProviderAlertData[]>;
    subscribePrice: (symbol: string) => Promise<void>;
    unsubscribePrice: (symbol: string) => Promise<void>;
    onPriceUpdate: (callback: (quote: PriceQuote) => void) => () => void;
    onConnectionState: (callback: (state: ConnectionState) => void) => () => void;
    getTradingHalts: () => Promise<TradingHaltItem[]>;
    getCorporateActions: () => Promise<CorporateActionItem[]>;
    getRealizedVolatility: (symbol?: string) => Promise<VolatilityData | null>;
    getImpliedVolatility: (symbol?: string) => Promise<VolatilityData | null>;
    uploadSnapshot: (params: UploadSnapshotInput) => Promise<{ id: string; url: string } | null>;
  };
  orderbook: {
    get: (symbol: string) => Promise<OrderBookData | null>;
  };
  intelligence: {
    analyze: (params: IntelligenceAnalyzeInput) => Promise<IntelligenceAnalyzeResult | null>;
    getInsights: (symbol: string) => Promise<MarketInsightResult | null>;
  };
  options: {
    getChain: (symbol: string) => Promise<OptionChainData | null>;
    getGex: (symbol: string) => Promise<OptionGexData | null>;
    getSummary: () => Promise<OptionSummaryData | null>;
  };
  macro: {
    getFearGreed: () => Promise<FearGreedResult | null>;
    getFearGreedHistory: () => Promise<FearGreedHistoryItem[]>;
    getCot: (symbol: string) => Promise<CotReportResult | null>;
    getCentralBanks: (bank?: string) => Promise<CentralBankStanceResult[]>;
    getMap: (params?: MacroMapInput) => Promise<MacroMapResult | null>;
  };
  fixedIncome: {
    getYieldCurve: () => Promise<YieldCurveResult | null>;
    getSpreads: () => Promise<YieldSpreadResult | null>;
    getRate: (tenor: string) => Promise<FixedIncomeRateData | null>;
    getHistory: (tenor: string) => Promise<FixedIncomeHistoryData | null>;
  };
  geosignals: {
    getEvents: () => Promise<GeoSignalEventItem[]>;
    getMap: () => Promise<GeoSignalsMapRegion[]>;
    getAssetImpacts: () => Promise<GeoAssetImpactItem[]>;
  };
  energy: {
    getDashboard: () => Promise<EnergyDashboardData | null>;
    getSeries: (seriesId: string) => Promise<EnergySeriesData | null>;
  };
  sec: {
    getFilings: (params?: SecFilingsInput) => Promise<SecFilingItemData[]>;
    getCompany: (symbol: string) => Promise<SecCompanyData | null>;
  };
  social: {
    getPosts: (params?: SocialPostsInput) => Promise<SocialPostItemData[]>;
    getFeed: (params?: SocialPostsInput) => Promise<SocialPostItemData[]>;
  };
  drawings: {
    get: (params: { symbol: string; timeframe?: string }) => Promise<DrawingItem[]>;
    save: (drawing: DrawingItem) => Promise<boolean>;
    delete: (id: string) => Promise<boolean>;
    clear: (params?: { symbol?: string }) => Promise<boolean>;
  };
  layouts: {
    getAll: () => Promise<ChartLayoutData[]>;
    get: (id: string) => Promise<ChartLayoutData | null>;
    save: (layout: ChartLayoutData) => Promise<boolean>;
    delete: (id: string) => Promise<boolean>;
  };
  indicators: {
    get: (symbol: string) => Promise<IndicatorConfig[]>;
    save: (symbol: string, indicators: IndicatorConfig[]) => Promise<boolean>;
  };
  watchlist: {
    getAll: () => Promise<WatchlistGroup[]>;
    save: (watchlist: WatchlistGroup) => Promise<boolean>;
  };
  calendar: {
    get: (params?: GetCalendarInput) => Promise<EconomicEvent[]>;
  };
  news: {
    get: (params?: GetNewsInput) => Promise<NewsArticle[]>;
    getLatest: () => Promise<NewsArticle[]>;
    getById: (id: string) => Promise<NewsDetailData | null>;
  };
  economic: {
    getIndicators: () => Promise<EconomicMetadataData>;
    getCategories: () => Promise<EconomicMetadataData>;
    getCountries: () => Promise<EconomicMetadataData>;
  };
  alerts: {
    getAll: () => Promise<PriceAlert[]>;
    save: (alert: PriceAlert) => Promise<boolean>;
    delete: (id: string) => Promise<boolean>;
  };
  paper: {
    getAccount: () => Promise<PaperAccount>;
    saveAccount: (account: PaperAccount) => Promise<boolean>;
  };
  settings: {
    getCredentials: () => Promise<CredentialStatus>;
    saveApiKey: (input: SaveApiKeyInput) => Promise<boolean>;
    clearApiKey: () => Promise<boolean>;
  };
  system: {
    openExternal: (url: string) => Promise<boolean>;
    getRateLimit: () => Promise<RateLimitStatus>;
  };
  ws: {
    createTicket: () => Promise<WsTicketData | null>;
  };
}
