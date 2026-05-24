CREATE SCHEMA IF NOT EXISTS control;
CREATE SCHEMA IF NOT EXISTS news;
CREATE SCHEMA IF NOT EXISTS market;
CREATE SCHEMA IF NOT EXISTS ops;

COMMENT ON SCHEMA control IS 'SaaS control-plane data: users, API keys, tenant configs, plans, usage, billing/auth.';
COMMENT ON SCHEMA news IS 'News ingestion data: sources, articles, analyses, and future search/indexing tables.';
COMMENT ON SCHEMA market IS 'Market data and time-series data: candles, latest prices, and future tick storage.';
COMMENT ON SCHEMA ops IS 'Operational telemetry: events, health snapshots, alerts, and audit trails.';

CREATE OR REPLACE VIEW market.ohlcv_candles AS
SELECT * FROM public.ohlcv_candles;

CREATE OR REPLACE VIEW market.market_latest_prices AS
SELECT * FROM public.market_latest_prices;

CREATE OR REPLACE VIEW news.forex_news_sources AS
SELECT * FROM public.forex_news_sources;

CREATE OR REPLACE VIEW news.forex_news_articles AS
SELECT * FROM public.forex_news_articles;

CREATE OR REPLACE VIEW news.forex_news_analyses AS
SELECT * FROM public.forex_news_analyses;

CREATE OR REPLACE VIEW news.stock_news AS
SELECT * FROM public.stock_news;

CREATE INDEX IF NOT EXISTS idx_ohlcv_candles_symbol_resolution_time_desc
    ON public.ohlcv_candles(symbol, resolution, time DESC);

COMMENT ON VIEW market.ohlcv_candles IS 'Compatibility view for future market schema migration; base table remains public.ohlcv_candles in phase 1.';
COMMENT ON VIEW market.market_latest_prices IS 'Compatibility view for future market schema migration; base table remains public.market_latest_prices in phase 1.';
COMMENT ON VIEW news.forex_news_sources IS 'Compatibility view for future news schema migration; base table remains public.forex_news_sources in phase 1.';
COMMENT ON VIEW news.forex_news_articles IS 'Compatibility view for future news schema migration; base table remains public.forex_news_articles in phase 1.';
COMMENT ON VIEW news.forex_news_analyses IS 'Compatibility view for future news schema migration; base table remains public.forex_news_analyses in phase 1.';
COMMENT ON VIEW news.stock_news IS 'Compatibility view for future news schema migration; base table remains public.stock_news in phase 1.';
