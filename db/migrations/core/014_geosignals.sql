CREATE SCHEMA IF NOT EXISTS news;

CREATE TABLE IF NOT EXISTS news.geosignals (
    event_id TEXT PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    source_url TEXT,
    title TEXT NOT NULL,
    summary TEXT,
    category TEXT NOT NULL,
    country TEXT,
    region TEXT,
    location_scope TEXT NOT NULL,
    severity_score DOUBLE PRECISION NOT NULL CHECK (severity_score >= 0 AND severity_score <= 1),
    sentiment_score DOUBLE PRECISION NOT NULL CHECK (sentiment_score >= -1 AND sentiment_score <= 1),
    confidence_score DOUBLE PRECISION NOT NULL CHECK (confidence_score >= 0 AND confidence_score <= 1),
    affected_assets TEXT[] NOT NULL DEFAULT '{}',
    asset_impact JSONB NOT NULL DEFAULT '{}'::jsonb,
    freshness TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT geosignals_event_id_len CHECK (char_length(event_id) <= 255),
    CONSTRAINT geosignals_source_len CHECK (char_length(source) <= 255),
    CONSTRAINT geosignals_category_len CHECK (char_length(category) <= 100),
    CONSTRAINT geosignals_country_len CHECK (country IS NULL OR char_length(country) <= 100),
    CONSTRAINT geosignals_region_len CHECK (region IS NULL OR char_length(region) <= 100),
    CONSTRAINT geosignals_location_scope_len CHECK (char_length(location_scope) <= 50),
    CONSTRAINT geosignals_freshness_len CHECK (char_length(freshness) <= 50)
);

ALTER TABLE news.geosignals
    ALTER COLUMN freshness DROP DEFAULT;

ALTER TABLE news.geosignals
    ALTER COLUMN freshness TYPE TEXT USING CASE
        WHEN lower(freshness::text) IN ('fresh', 'stale', 'partial') THEN lower(freshness::text)
        ELSE 'fresh'
    END;

ALTER TABLE news.geosignals
    ALTER COLUMN source SET DEFAULT 'unknown',
    ALTER COLUMN title SET DEFAULT '',
    ALTER COLUMN category SET DEFAULT 'market_news',
    ALTER COLUMN location_scope SET DEFAULT 'global',
    ALTER COLUMN severity_score SET DEFAULT 0,
    ALTER COLUMN sentiment_score SET DEFAULT 0,
    ALTER COLUMN confidence_score SET DEFAULT 0,
    ALTER COLUMN affected_assets SET DEFAULT '{}',
    ALTER COLUMN asset_impact SET DEFAULT '{}'::jsonb,
    ALTER COLUMN freshness SET DEFAULT 'fresh';

UPDATE news.geosignals
SET source = COALESCE(source, 'unknown'),
    title = COALESCE(title, event_id),
    category = COALESCE(category, 'market_news'),
    location_scope = COALESCE(location_scope, 'global'),
    severity_score = COALESCE(severity_score, 0),
    sentiment_score = COALESCE(sentiment_score, 0),
    confidence_score = COALESCE(confidence_score, 0),
    affected_assets = COALESCE(affected_assets, '{}'),
    asset_impact = COALESCE(asset_impact, '{}'::jsonb),
    freshness = COALESCE(NULLIF(freshness, ''), 'fresh');

ALTER TABLE news.geosignals
    ALTER COLUMN source SET NOT NULL,
    ALTER COLUMN title SET NOT NULL,
    ALTER COLUMN category SET NOT NULL,
    ALTER COLUMN location_scope SET NOT NULL,
    ALTER COLUMN severity_score SET NOT NULL,
    ALTER COLUMN sentiment_score SET NOT NULL,
    ALTER COLUMN confidence_score SET NOT NULL,
    ALTER COLUMN affected_assets SET NOT NULL,
    ALTER COLUMN asset_impact SET NOT NULL,
    ALTER COLUMN freshness SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_geosignals_timestamp_desc
    ON news.geosignals(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_geosignals_category_timestamp_desc
    ON news.geosignals(category, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_geosignals_country_timestamp_desc
    ON news.geosignals(country, timestamp DESC)
    WHERE country IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_geosignals_region_timestamp_desc
    ON news.geosignals(region, timestamp DESC)
    WHERE region IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_geosignals_severity_score_desc
    ON news.geosignals(severity_score DESC);

CREATE INDEX IF NOT EXISTS idx_geosignals_affected_assets_gin
    ON news.geosignals USING gin(affected_assets);

CREATE INDEX IF NOT EXISTS idx_geosignals_asset_impact_gin
    ON news.geosignals USING gin(asset_impact);

-- Table comment
COMMENT ON TABLE news.geosignals IS 'Geo-spatial signals tracking geopolitical, environmental, and regional events with impact scoring.';
COMMENT ON COLUMN news.geosignals.event_id IS 'Unique event identifier from source.';
COMMENT ON COLUMN news.geosignals.timestamp IS 'Event occurrence timestamp.';
COMMENT ON COLUMN news.geosignals.source IS 'Data source identifier.';
COMMENT ON COLUMN news.geosignals.source_url IS 'URL reference to original source.';
COMMENT ON COLUMN news.geosignals.title IS 'Event title or headline.';
COMMENT ON COLUMN news.geosignals.summary IS 'Event summary or description.';
COMMENT ON COLUMN news.geosignals.category IS 'Event category (e.g., geopolitical, environmental, health, economic).';
COMMENT ON COLUMN news.geosignals.country IS 'ISO country code or country name.';
COMMENT ON COLUMN news.geosignals.region IS 'Geographic region or sub-national area.';
COMMENT ON COLUMN news.geosignals.location_scope IS 'Geographic scope (e.g., global, regional, local).';
COMMENT ON COLUMN news.geosignals.severity_score IS 'Event severity score [0-1].';
COMMENT ON COLUMN news.geosignals.sentiment_score IS 'Market sentiment impact [-1,1], -1=negative, 0=neutral, 1=positive.';
COMMENT ON COLUMN news.geosignals.confidence_score IS 'Data confidence/reliability score [0-1].';
COMMENT ON COLUMN news.geosignals.affected_assets IS 'Array of affected asset identifiers or classes.';
COMMENT ON COLUMN news.geosignals.asset_impact IS 'JSONB object with detailed asset impact analysis.';
COMMENT ON COLUMN news.geosignals.freshness IS 'Data freshness timestamp.';
COMMENT ON COLUMN news.geosignals.created_at IS 'Record creation timestamp.';
