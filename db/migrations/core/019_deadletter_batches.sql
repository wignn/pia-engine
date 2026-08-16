
CREATE TABLE IF NOT EXISTS platform.deadletter_batches (
    id BIGSERIAL PRIMARY KEY,
    target TEXT NOT NULL,
    payload JSONB NOT NULL,
    error TEXT NOT NULL,
    batch_size INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    replayed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_deadletter_unreplayed
    ON platform.deadletter_batches (id)
    WHERE replayed_at IS NULL;
