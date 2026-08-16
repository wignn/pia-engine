CREATE TABLE IF NOT EXISTS market.why_move_explanations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol TEXT NOT NULL,
    time_window TEXT NOT NULL,
    evidence_hash TEXT NOT NULL UNIQUE,
    move_latest_at TIMESTAMPTZ,
    move_pct DOUBLE PRECISION,
    engine_version TEXT NOT NULL DEFAULT 'why-engine-v1',
    provider TEXT NOT NULL DEFAULT 'deterministic',
    model TEXT,
    status TEXT NOT NULL,
    response JSONB NOT NULL,
    evidence JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_why_move_explanations_symbol_window
ON market.why_move_explanations(symbol, time_window, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_why_move_explanations_expires_at
ON market.why_move_explanations(expires_at);
