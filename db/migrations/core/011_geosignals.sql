-- Create schema if needed (should already exist from 005_schema_foundation.sql)
CREATE SCHEMA IF NOT EXISTS news;

-- Create geosignals table
CREATE TABLE IF NOT EXISTS news.geosignals (
    event_id TEXT PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    source TEXT,
    source_url TEXT,
    title TEXT,
    summary TEXT,
    category TEXT,
    country TEXT,
    region TEXT,
    location_scope TEXT,
    severity_score DOUBLE PRECISION,
    sentiment_score DOUBLE PRECISION,
    confidence_score DOUBLE PRECISION,
    affected_assets TEXT[] DEFAULT '{}',
    asset_impact JSONB DEFAULT '{}'::jsonb,
    freshness TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- CHECK constraints for score ranges
    CONSTRAINT severity_score_range CHECK (severity_score >= 0 AND severity_score <= 1),
    CONSTRAINT sentiment_score_range CHECK (sentiment_score >= -1 AND sentiment_score <= 1),
    CONSTRAINT confidence_score_range CHECK (confidence_score >= 0 AND confidence_score <= 1)
);

-- Index on timestamp (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_geosignals_timestamp_desc
    ON news.geosignals(timestamp DESC);

-- Composite index on category + timestamp
CREATE INDEX IF NOT EXISTS idx_geosignals_category_timestamp_desc
    ON news.geosignals(category, timestamp DESC);

-- Partial index on country + timestamp (where country is not null)
CREATE INDEX IF NOT EXISTS idx_geosignals_country_timestamp_desc
    ON news.geosignals(country, timestamp DESC)
    WHERE country IS NOT NULL;

-- Partial index on region + timestamp (where region is not null)
CREATE INDEX IF NOT EXISTS idx_geosignals_region_timestamp_desc
    ON news.geosignals(region, timestamp DESC)
    WHERE region IS NOT NULL;

-- Index on severity_score for filtering high-impact events
CREATE INDEX IF NOT EXISTS idx_geosignals_severity_score_desc
    ON news.geosignals(severity_score DESC);

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
