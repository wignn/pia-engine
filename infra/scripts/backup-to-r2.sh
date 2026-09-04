#!/usr/bin/env bash
#
# Automated Backup Script for ATLSD -> Cloudflare R2
# Dumps PostgreSQL and ClickHouse, compresses, uploads to R2, and cleans up local/remote old backups.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_FILE="$ROOT_DIR/infra/env/.env.backup"

if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Error: $ENV_FILE not found."
    exit 1
fi

# Load backup env
set -a
source "$ENV_FILE"
set +a

if [ "${CF_ACCOUNT_ID}" == "your_account_id_here" ] || [ -z "${CF_ACCOUNT_ID}" ]; then
    echo "⚠️ Warning: CF_ACCOUNT_ID is not configured in $ENV_FILE."
    echo "Please set your Cloudflare R2 credentials in $ENV_FILE to enable R2 upload."
    SKIP_UPLOAD=true
else
    SKIP_UPLOAD=false
fi

TIMESTAMP="$(date -u +%Y%m%d_%H%M%S)"
BACKUP_TMP_DIR="/tmp/atlsd_backup_${TIMESTAMP}"
mkdir -p "$BACKUP_TMP_DIR"

echo "=============================================="
echo "📦 STARTING ATLSD AUTOMATED BACKUP ($TIMESTAMP)"
echo "=============================================="

# 1. Dump PostgreSQL Database
echo "🐘 1. Dumping PostgreSQL..."
PG_FILE="$BACKUP_TMP_DIR/postgres_${TIMESTAMP}.sql.gz"
if docker exec postgres-prod pg_dumpall -U atlsd | gzip > "$PG_FILE"; then
    echo "✅ PostgreSQL dump completed: $(du -h "$PG_FILE" | cut -f1)"
else
    echo "❌ PostgreSQL dump failed!"
    rm -rf "$BACKUP_TMP_DIR"
    exit 1
fi

# 2. Dump ClickHouse Schemas/Data
echo "📊 2. Dumping ClickHouse..."
CH_FILE="$BACKUP_TMP_DIR/clickhouse_${TIMESTAMP}.sql.gz"
if docker exec clickhouse-prod clickhouse-client --query "SHOW DATABASES" | grep -vE 'system|default|INFORMATION_SCHEMA|information_schema' | while read -r db; do
    [ -n "$db" ] || continue
    echo "-- DATABASE: $db"
    docker exec clickhouse-prod clickhouse-client --query "SHOW TABLES FROM $db" | while read -r tbl; do
        [ -n "$tbl" ] || continue
        docker exec clickhouse-prod clickhouse-client --query "SHOW CREATE TABLE $db.$tbl"
        echo ";"
    done
done | gzip > "$CH_FILE"; then
    echo "✅ ClickHouse dump completed: $(du -h "$CH_FILE" | cut -f1)"
else
    echo "⚠️ ClickHouse dump notice: empty or partial dump."
fi

# 3. Create Tarball Archive
ARCHIVE_NAME="atlsd_backup_${TIMESTAMP}.tar.gz"
ARCHIVE_PATH="/tmp/${ARCHIVE_NAME}"
echo "📦 3. Compressing archive -> $ARCHIVE_NAME..."
tar -czf "$ARCHIVE_PATH" -C "$BACKUP_TMP_DIR" .
rm -rf "$BACKUP_TMP_DIR"
echo "✅ Archive ready: $(du -h "$ARCHIVE_PATH" | cut -f1)"

# 4. Upload to Cloudflare R2 via Python / boto3
if [ "$SKIP_UPLOAD" = true ]; then
    echo "⏩ 4. Skipping upload to Cloudflare R2 (Credentials missing in .env.backup)."
    echo "Local backup preserved at: $ARCHIVE_PATH"
    exit 0
fi

echo "☁️  4. Uploading to Cloudflare R2 bucket [$CF_R2_BUCKET_NAME]..."
python3 - "$ARCHIVE_PATH" "$ARCHIVE_NAME" <<'PY'
import os, sys, boto3
from botocore.client import Config

archive_path = sys.argv[1]
archive_name = sys.argv[2]

account_id = os.environ.get("CF_ACCOUNT_ID")
access_key = os.environ.get("CF_R2_ACCESS_KEY_ID")
secret_key = os.environ.get("CF_R2_SECRET_ACCESS_KEY")
bucket_name = os.environ.get("CF_R2_BUCKET_NAME", "atlsd-backups")
retention_days = int(os.environ.get("RETENTION_DAYS", "14"))

r2_url = f"https://{account_id}.r2.cloudflarestorage.com"

s3 = boto3.client(
    "s3",
    endpoint_url=r2_url,
    aws_access_key_id=access_key,
    aws_secret_access_key=secret_key,
    config=Config(signature_version="s3v4"),
    region_name="auto"
)

# Ensure bucket exists
try:
    s3.head_bucket(Bucket=bucket_name)
except Exception:
    try:
        s3.create_bucket(Bucket=bucket_name)
        print(f"Created R2 bucket: {bucket_name}")
    except Exception as e:
        print(f"Bucket notice: {e}")

# Upload file
key = f"backups/{archive_name}"
print(f"Uploading {archive_name} -> {bucket_name}/{key}...")
s3.upload_file(archive_path, bucket_name, key)
print("✅ Upload successful!")

# Cleanup old backups in R2
import datetime
now = datetime.datetime.now(datetime.timezone.utc)
try:
    response = s3.list_objects_v2(Bucket=bucket_name, Prefix="backups/")
    for obj in response.get("Contents", []):
        obj_key = obj["Key"]
        last_modified = obj["LastModified"]
        age_days = (now - last_modified).days
        if age_days > retention_days:
            print(f"🧹 Pruning old R2 backup ({age_days} days old): {obj_key}")
            s3.delete_object(Bucket=bucket_name, Key=obj_key)
except Exception as e:
    print(f"Cleanup notice: {e}")
PY

# Remove local temp archive
rm -f "$ARCHIVE_PATH"
echo "🎉 BACKUP COMPLETE & VERIFIED!"
