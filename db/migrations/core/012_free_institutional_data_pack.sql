-- Rates & Yield Curve
CREATE TABLE IF NOT EXISTS macro_rates (
    source TEXT NOT NULL,
    country TEXT NOT NULL,
    tenor TEXT NOT NULL,
    date DATE NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit TEXT NOT NULL DEFAULT 'percent',
    raw_series_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (source, country, tenor, date)
);

CREATE INDEX IF NOT EXISTS idx_macro_rates_country_tenor_date
    ON macro_rates (country, tenor, date DESC);

CREATE TABLE IF NOT EXISTS macro_rate_spreads (
    country TEXT NOT NULL,
    spread TEXT NOT NULL,
    date DATE NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (country, spread, date)
);

-- SEC EDGAR Filings
CREATE TABLE IF NOT EXISTS sec_companies (
    cik TEXT PRIMARY KEY,
    ticker TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    exchange TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sec_filings (
    accession_number TEXT PRIMARY KEY,
    cik TEXT NOT NULL,
    ticker TEXT,
    form_type TEXT NOT NULL,
    filing_date DATE NOT NULL,
    report_date DATE,
    primary_document TEXT NOT NULL,
    document_url TEXT NOT NULL,
    title TEXT NOT NULL,
    raw_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sec_filings_ticker_form_date
    ON sec_filings (ticker, form_type, filing_date DESC);

-- Central Bank Monitor
CREATE TABLE IF NOT EXISTS central_bank_sources (
    bank TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'rss',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (bank, url)
);

CREATE TABLE IF NOT EXISTS central_bank_documents (
    id TEXT PRIMARY KEY,
    bank TEXT NOT NULL,
    document_type TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    published_at TIMESTAMPTZ,
    summary TEXT,
    stance TEXT NOT NULL DEFAULT 'unknown',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    raw_text TEXT,
    raw_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT cb_docs_bank_url UNIQUE (bank, url)
);

CREATE INDEX IF NOT EXISTS idx_cb_docs_bank_published
    ON central_bank_documents (bank, published_at DESC);

-- GDELT Raw Events
CREATE TABLE IF NOT EXISTS geosignal_raw_events (
    source TEXT NOT NULL,
    event_id TEXT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    raw_json JSONB NOT NULL,
    PRIMARY KEY (source, event_id)
);

-- EIA Energy Data
CREATE TABLE IF NOT EXISTS energy_series (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL DEFAULT 'eia',
    name TEXT NOT NULL,
    commodity TEXT NOT NULL,
    unit TEXT NOT NULL,
    frequency TEXT NOT NULL DEFAULT 'weekly',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS energy_observations (
    series_id TEXT REFERENCES energy_series(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    raw_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (series_id, date)
);

CREATE INDEX IF NOT EXISTS idx_energy_obs_date
    ON energy_observations (series_id, date DESC);

-- CFTC COT Positioning
CREATE TABLE IF NOT EXISTS cot_reports (
    market_code TEXT NOT NULL,
    market_name TEXT NOT NULL,
    report_date DATE NOT NULL,
    report_type TEXT NOT NULL DEFAULT 'legacy_futures_only',
    commercial_long BIGINT,
    commercial_short BIGINT,
    noncommercial_long BIGINT,
    noncommercial_short BIGINT,
    nonreportable_long BIGINT,
    nonreportable_short BIGINT,
    open_interest BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (report_type, market_code, report_date)
);

CREATE TABLE IF NOT EXISTS cot_market_map (
    market_code TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    asset_class TEXT NOT NULL,
    display_name TEXT NOT NULL
);

-- Fear & Greed / Risk Regime Index
CREATE TABLE IF NOT EXISTS fear_greed_index (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    label TEXT NOT NULL,
    components JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_status JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fear_greed_scope_date UNIQUE (scope, date)
);

CREATE INDEX IF NOT EXISTS idx_fear_greed_scope_date
    ON fear_greed_index (scope, date DESC);
