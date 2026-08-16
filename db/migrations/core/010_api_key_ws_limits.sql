ALTER TABLE api_keys
ADD COLUMN IF NOT EXISTS max_ws_connections INT;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'api_keys_max_ws_connections_positive'
  ) THEN
    ALTER TABLE api_keys
    ADD CONSTRAINT api_keys_max_ws_connections_positive
    CHECK (max_ws_connections IS NULL OR max_ws_connections > 0);
  END IF;
END$$;
