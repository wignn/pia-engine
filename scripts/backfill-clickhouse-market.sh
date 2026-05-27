#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-infra/compose/prod.yml}"
POSTGRES_SERVICE="${POSTGRES_SERVICE:-postgres}"
CLICKHOUSE_SERVICE="${CLICKHOUSE_SERVICE:-clickhouse}"
POSTGRES_USER="${POSTGRES_USER:-atlsd}"
POSTGRES_DB="${POSTGRES_DB:-core}"
SYMBOL_FILTER="${SYMBOL_FILTER:-}"

where_clause="WHERE resolution = '1m'"
if [ -n "$SYMBOL_FILTER" ]; then
  escaped_symbol=$(printf "%s" "$SYMBOL_FILTER" | sed "s/'/''/g")
  where_clause="$where_clause AND symbol = '$escaped_symbol'"
fi

query="COPY (
  SELECT
    symbol,
    resolution,
    to_char(time AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS.MS') AS time,
    open,
    high,
    low,
    close,
    COALESCE(volume, 0) AS volume,
    to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS.MS') AS updated_at
  FROM market.ohlcv_candles
  $where_clause
  ORDER BY symbol, time
) TO STDOUT WITH CSV"

if [ ! -f "$COMPOSE_FILE" ]; then
  echo "Compose file not found: $COMPOSE_FILE" >&2
  exit 1
fi

echo "Backfilling Postgres market.ohlcv_candles into ClickHouse market.ohlcv_candles"
if [ -n "$SYMBOL_FILTER" ]; then
  echo "Symbol filter: $SYMBOL_FILTER"
fi

docker compose -f "$COMPOSE_FILE" exec -T "$POSTGRES_SERVICE" \
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "$query" \
| docker compose -f "$COMPOSE_FILE" exec -T "$CLICKHOUSE_SERVICE" \
  clickhouse-client --query "INSERT INTO market.ohlcv_candles FORMAT CSV"

echo "Backfill complete"
