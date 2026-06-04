CREATE TABLE IF NOT EXISTS economic_calendar_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source TEXT NOT NULL,
    event_hash TEXT NOT NULL,
    country TEXT NOT NULL,
    event_name TEXT NOT NULL,
    impact TEXT,
    unit TEXT,
    actual DOUBLE PRECISION,
    forecast DOUBLE PRECISION,
    previous DOUBLE PRECISION,
    event_time TIMESTAMPTZ,
    raw_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (source, event_hash)
);

CREATE INDEX IF NOT EXISTS idx_economic_calendar_events_time
    ON economic_calendar_events(event_time DESC NULLS LAST);

CREATE INDEX IF NOT EXISTS idx_economic_calendar_events_country
    ON economic_calendar_events(country, event_time DESC NULLS LAST);

CREATE OR REPLACE VIEW news.economic_calendar_events AS
SELECT * FROM public.economic_calendar_events;

COMMENT ON TABLE economic_calendar_events IS 'Global macroeconomic calendar events collected from API providers such as Finnhub.';
