#!/usr/bin/env bash
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

ACTIVE_FILE="$ROOT_DIR/infra/env/active_target"
ACTIVE_COLOR="blue"

if [ -f "$ACTIVE_FILE" ]; then
    ACTIVE_COLOR="$(cat "$ACTIVE_FILE" | tr -d ' \n\r')"
fi

if [ "$ACTIVE_COLOR" == "blue" ]; then
    TARGET_COLOR="green"
    TARGET_API_PORT=8001
    TARGET_RT_PORT=8021
else
    TARGET_COLOR="blue"
    TARGET_API_PORT=8000
    TARGET_RT_PORT=8020
fi

echo "=========================================="
echo "🚀 ATLSD BLUE-GREEN DEPLOYMENT"
echo "Active Color: [$ACTIVE_COLOR]"
echo "Target Color: [$TARGET_COLOR]"
echo "Target API Port: $TARGET_API_PORT"
echo "=========================================="

echo "📦 1. Building and starting [$TARGET_COLOR] environment..."
docker compose \
    -f "$ROOT_DIR/infra/compose/prod.yml" \
    --env-file "$ROOT_DIR/infra/env/.env.shared" \
    --env-file "$ROOT_DIR/infra/env/.env.$TARGET_COLOR" \
    up -d --build --remove-orphans

echo "⏳ 2. Running healthcheck on [$TARGET_COLOR] (Port $TARGET_API_PORT)..."
MAX_ATTEMPTS=20
ATTEMPT=0
HEALTHY=false

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    ATTEMPT=$((ATTEMPT + 1))
    STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$TARGET_API_PORT/health" || true)
    if [ "$STATUS_CODE" == "200" ]; then
        HEALTHY=true
        break
    fi
    echo "Attempt $ATTEMPT/$MAX_ATTEMPTS: API Gateway status $STATUS_CODE. Waiting 3s..."
    sleep 3
done

if [ "$HEALTHY" = false ]; then
    echo "❌ ERROR: [$TARGET_COLOR] deployment healthcheck failed!"
    echo "Stopping failed [$TARGET_COLOR] containers..."
    docker compose \
        -f "$ROOT_DIR/infra/compose/prod.yml" \
        --env-file "$ROOT_DIR/infra/env/.env.shared" \
        --env-file "$ROOT_DIR/infra/env/.env.$TARGET_COLOR" \
        down
    exit 1
fi

echo "✅ [$TARGET_COLOR] is Healthy!"

echo "🔀 3. Switching Nginx traffic to [$TARGET_COLOR]..."
UPSTREAM_CONF="/etc/nginx/conf.d/upstream.conf"
if [ -w "$UPSTREAM_CONF" ] || [ -w "/etc/nginx/conf.d" ]; then
    cat <<EOF > "$UPSTREAM_CONF"
upstream api_gateway_backend {
    server 127.0.0.1:$TARGET_API_PORT;
}

upstream realtime_gateway_backend {
    server 127.0.0.1:$TARGET_RT_PORT;
}
EOF
    sudo nginx -s reload || nginx -s reload
    echo "Nginx successfully reloaded to upstream port $TARGET_API_PORT!"
else
    echo "⚠️ Warning: $UPSTREAM_CONF is not writable. Please update Nginx upstream manually to port $TARGET_API_PORT."
fi

echo "$TARGET_COLOR" > "$ACTIVE_FILE"
echo "🎉 DEPLOYMENT COMPLETE! [$TARGET_COLOR] is now live."
