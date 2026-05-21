-- =============================================================================
-- Migration: Add URL Analysis Cache Table
-- =============================================================================

CREATE TABLE IF NOT EXISTS url_analysis_cache (
    url          TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    content      TEXT NOT NULL,
    raw_response JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
