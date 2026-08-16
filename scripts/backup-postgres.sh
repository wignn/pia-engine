#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backup}"
RETENTION_DAYS="${RETENTION_DAYS:-14}"
PG_USER="${POSTGRES_USER:-atlsd}"
PG_DB="${POSTGRES_DB:-core}"

mkdir -p "$BACKUP_DIR"
STAMP="$(date -u +%Y%m%d-%H%M%S)"
OUT="$BACKUP_DIR/$PG_DB-$STAMP.sql.gz"

pg_dump -U "$PG_USER" -d "$PG_DB" --no-owner --no-privileges | gzip -9 > "$OUT"

# An empty or truncated dump is worse than no backup: it would fail silently
# at restore time. Verify before keeping it.
if [ ! -s "$OUT" ] || ! gzip -t "$OUT"; then
  echo "[atlsd-backup] ERROR: $OUT is empty or corrupt; removing" >&2
  rm -f "$OUT"
  exit 1
fi
echo "[atlsd-backup] wrote $OUT ($(du -h "$OUT" | cut -f1))"

find "$BACKUP_DIR" -name "$PG_DB-*.sql.gz" -mtime +"$RETENTION_DAYS" -print -delete | \
  sed 's/^/[atlsd-backup] pruned /'
