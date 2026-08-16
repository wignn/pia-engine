CREATE TABLE IF NOT EXISTS macro_series (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'macro',
    units TEXT,
    frequency TEXT,
    seasonal_adjustment TEXT,
    observation_start DATE,
    observation_end DATE,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS macro_observations (
    series_id TEXT NOT NULL REFERENCES macro_series(id) ON DELETE CASCADE,
    observation_date DATE NOT NULL,
    value DOUBLE PRECISION,
    raw_value TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (series_id, observation_date)
);

CREATE INDEX IF NOT EXISTS idx_macro_observations_date
    ON macro_observations(observation_date DESC);

CREATE INDEX IF NOT EXISTS idx_macro_series_provider_category
    ON macro_series(provider, category);

CREATE OR REPLACE VIEW news.macro_series AS
SELECT * FROM public.macro_series;

CREATE OR REPLACE VIEW news.macro_observations AS
SELECT * FROM public.macro_observations;

COMMENT ON TABLE macro_series IS 'Macroeconomic series metadata from providers such as FRED.';
COMMENT ON TABLE macro_observations IS 'Macroeconomic observations keyed by series and observation date.';
