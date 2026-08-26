CREATE TABLE IF NOT EXISTS macro_bonds (
    id TEXT PRIMARY KEY,
    country TEXT NOT NULL,
    as_of DATE NOT NULL,
    raw_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_macro_bonds_country_as_of
    ON macro_bonds (country, as_of DESC);
