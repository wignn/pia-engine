CREATE TABLE IF NOT EXISTS macro_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    series_id TEXT NOT NULL REFERENCES macro_series(id) ON DELETE CASCADE,
    category TEXT NOT NULL,
    signal_date DATE NOT NULL,
    latest_value DOUBLE PRECISION,
    previous_value DOUBLE PRECISION,
    change_1d DOUBLE PRECISION,
    change_7d DOUBLE PRECISION,
    change_30d DOUBLE PRECISION,
    direction TEXT NOT NULL,
    severity TEXT NOT NULL,
    narrative TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (series_id, signal_date)
);

CREATE INDEX IF NOT EXISTS idx_macro_signals_date
    ON macro_signals(signal_date DESC);

CREATE INDEX IF NOT EXISTS idx_macro_signals_category_severity
    ON macro_signals(category, severity, signal_date DESC);

CREATE OR REPLACE VIEW news.macro_signals AS
SELECT * FROM public.macro_signals;

COMMENT ON TABLE macro_signals IS 'Derived macro trend signals generated from macro observations.';
