#!/usr/bin/env bash
#
# ATLSD Blue/Green deployment (CI-built images, VPS only pulls).
#
# Layout:
#   prod.infra.yml  – singletons: postgres/clickhouse/redis/nats  (project: atlsd-infra)
#   prod.edge.yml   – singletons: control-plane/analyzer/bot/...  (project: atlsd-edge)
#   prod.app.yml    – the COLOR stack: api/market/realtime/news/intel/web/sink
#                     (projects: atlsd-blue / atlsd-green)
#
# Flow: ensure infra+edge -> deploy INACTIVE color -> healthcheck it ->
#       point traffic-router at the new color -> switch ACTIVE marker -> stop old.
#
# Public ports NEVER move, so the Cloudflare tunnel config never changes:
#   host 8000 -> router :80 "/"      -> api-gateway        (active color)
#   host 8020 -> router :80 "/ws*"   -> realtime-gateway   (active color)
#   host 5173 -> router :81 "/"      -> public-web         (active color)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

ACTIVE_FILE="$ROOT_DIR/infra/env/active_target"
IMAGE_TAG="${ATLSD_IMAGE_TAG:-latest}"

COMPOSE="docker compose"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

color_ports() {
    # prints "<api> <rt> <web>" for a given color from its env file
    local envf="$ROOT_DIR/infra/env/.env.$1"
    local api rt web
    api="$(grep -E '^API_GATEWAY_PORT=' "$envf" | tail -1 | cut -d= -f2 | tr -d '[:space:]')"
    rt="$(grep -E '^REALTIME_GATEWAY_PORT=' "$envf" | tail -1 | cut -d= -f2 | tr -d '[:space:]')"
    web="$(grep -E '^PUBLIC_WEB_PORT=' "$envf" | tail -1 | cut -d= -f2 | tr -d '[:space:]')"
    echo "${api:-8000} ${rt:-8020} ${web:-5173}"
}

write_router_conf() {
    # $1 = api host port, $2 = realtime host port, $3 = public-web host port.
    # Unquoted EOF on purpose: $1/$2/$3 must expand NOW, while nginx vars
    # stay escaped as \$ so they survive into the generated conf.
    cat > "$ROOT_DIR/infra/compose/router/default.conf" <<EOF
# ATLSD traffic router — rewritten by deploy-blue-green.sh.
# Active color ports: api=$1 ws=$2 web=$3 (public ports stay fixed).

upstream api_gateway_backend {
    server 172.17.0.1:$1;
}

upstream realtime_gateway_backend {
    server 172.17.0.1:$2;
}

upstream public_web_backend {
    server 172.17.0.1:$3;
}

# ---- REST API + WebSocket (public hosts 8000 / 8020) ----------------------
server {
    listen 80;

    location = /healthz {
        access_log off;
        return 200 "router-ok\n";
    }

    # Realtime HTTP API (ticket issuance) + WS endpoints under /api/v1/ws
    location /api/v1/ws {
        proxy_pass http://realtime_gateway_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }

    location / {
        proxy_pass http://api_gateway_backend;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 60s;
    }

    location /ws {
        proxy_pass http://realtime_gateway_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}

# ---- Frontend (public host 5173) ------------------------------------------
server {
    listen 81;

    location / {
        proxy_pass http://public_web_backend;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_read_timeout 60s;
    }
}
EOF
}

switch_traffic() {
    # $1 = color
    read -r API_PORT RT_PORT WEB_PORT <<< "$(color_ports "$1")"
    echo "🔀  Pointing router at [$1] (api=$API_PORT, ws=$RT_PORT, web=$WEB_PORT)..."
    write_router_conf "$API_PORT" "$RT_PORT" "$WEB_PORT"
    docker exec atlsd-traffic-router nginx -s reload
}

stop_color() {
    # $1 = color
    $COMPOSE -p "atlsd-$1" \
        -f "$ROOT_DIR/infra/compose/prod.app.yml" \
        --env-file "$ROOT_DIR/infra/env/.env.shared" \
        --env-file "$ROOT_DIR/infra/env/.env.$1" \
        down --remove-orphans || true
}

deploy_color() {
    # $1 = color -> pull images (CI pushes them), run migrations, start stack
    $COMPOSE -p "atlsd-$1" \
        -f "$ROOT_DIR/infra/compose/prod.app.yml" \
        --env-file "$ROOT_DIR/infra/env/.env.shared" \
        --env-file "$ROOT_DIR/infra/env/.env.$1" \
        pull --ignore-buildable

    # Run migrations as an explicit one-shot FIRST: visible logs in CI,
    # retried on transient datastore hiccups (the --rm one-shot inside
    # `up --wait` hides its logs and one flaky connect kills the deploy).
    local attempt
    for attempt in 1 2 3; do
        if $COMPOSE -p "atlsd-$1" \
            -f "$ROOT_DIR/infra/compose/prod.app.yml" \
            --env-file "$ROOT_DIR/infra/env/.env.shared" \
            --env-file "$ROOT_DIR/infra/env/.env.$1" \
            run --rm db-migrate; then
            break
        fi
        if [ "$attempt" == "3" ]; then
            echo "\u274c db-migrate failed after 3 attempts."
            return 1
        fi
        echo "   [$1] db-migrate failed (attempt $attempt/3) \u2014 retrying in 10s..."
        sleep 10
    done

    # `up -d --wait` re-checks migrations (idempotent) while starting the
    # stack; retried too, since first-connect races have been observed.
    for attempt in 1 2 3; do
        if $COMPOSE -p "atlsd-$1" \
            -f "$ROOT_DIR/infra/compose/prod.app.yml" \
            --env-file "$ROOT_DIR/infra/env/.env.shared" \
            --env-file "$ROOT_DIR/infra/env/.env.$1" \
            up -d --wait; then
            return 0
        fi
        echo "   [$1] stack up failed (attempt $attempt/3) \u2014 retrying in 10s..."
        sleep 10
    done
    return 1
}

healthcheck_color() {
    # $1 = color; waits until its api-gateway answers on its own port
    read -r API_PORT _ <<< "$(color_ports "$1")"
    local max=30 attempt=0 code
    while [ $attempt -lt $max ]; do
        attempt=$((attempt + 1))
        code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 3 "http://127.0.0.1:$API_PORT/health" || true)"
        if [ "$code" == "200" ]; then
            return 0
        fi
        echo "   [$1] attempt $attempt/$max: gateway status ${code:-none}. retrying..."
        sleep 4
    done
    return 1
}

# ---------------------------------------------------------------------------
# Singleton guard: two overlapping deployments would fight over colors,
# ports and stacks (seen live: manual deploy stopping green mid-CI).
# ---------------------------------------------------------------------------
LOCK_FILE="/tmp/atlsd-deploy.lock"
exec 9>"$LOCK_FILE"
if ! flock -n 9; then
    echo "\u274c Another deployment is already running (lock held). Aborting."
    exit 1
fi

# ---------------------------------------------------------------------------
# Determine colors
# ---------------------------------------------------------------------------

if [ ! -f "$ACTIVE_FILE" ]; then
    echo "blue" > "$ACTIVE_FILE"
fi
ACTIVE_COLOR="$(tr -d ' \n\r' < "$ACTIVE_FILE")"
case "$ACTIVE_COLOR" in
    blue)  TARGET_COLOR="green" ;;
    green) TARGET_COLOR="blue" ;;
    *) echo "❌ active_target contains garbage: '$ACTIVE_COLOR'"; exit 1 ;;
esac

echo "=============================================="
echo "🚀 ATLSD BLUE-GREEN DEPLOYMENT"
echo "   Active: [$ACTIVE_COLOR] -> Target: [$TARGET_COLOR] (tag: $IMAGE_TAG)"
echo "=============================================="

# ---------------------------------------------------------------------------
# 0. One-time migration: retire the legacy single-stack (project "compose")
# ---------------------------------------------------------------------------

LEGACY_IDS="$(docker ps -q --filter 'label=com.docker.compose.project=compose')"
if [ -n "$LEGACY_IDS" ]; then
    echo "🧟 0. Legacy stack detected (project 'compose'). Stopping it once..."
    docker compose -p compose -f "$ROOT_DIR/infra/compose/prod.yml" down --remove-orphans || \
        docker stop $LEGACY_IDS || true
    echo "   Legacy stack retired. Blue/green takes over from here."
fi

# ---------------------------------------------------------------------------
# 1. Shared infra (datastores) + edge singletons
# ---------------------------------------------------------------------------

echo "📦 1. Ensuring shared infrastructure..."
$COMPOSE -p atlsd-infra -f "$ROOT_DIR/infra/compose/prod.infra.yml" up -d

echo "🔌 2. Ensuring edge singletons..."
$COMPOSE -p atlsd-edge -f "$ROOT_DIR/infra/compose/prod.edge.yml" \
    --env-file "$ROOT_DIR/infra/env/.env.shared" pull --ignore-buildable
$COMPOSE -p atlsd-edge -f "$ROOT_DIR/infra/compose/prod.edge.yml" \
    --env-file "$ROOT_DIR/infra/env/.env.shared" up -d --remove-orphans

# First-ever deployment: router conf still points at default blue ports,
# which is fine — we flip it after the target becomes healthy.

# ---------------------------------------------------------------------------
# 2. Deploy target color
# ---------------------------------------------------------------------------

echo "🏗️  3. Deploying [$TARGET_COLOR] (pull + migrate + up)..."
if ! deploy_color "$TARGET_COLOR"; then
    echo "❌ [$TARGET_COLOR] failed to come up. Keeping [$ACTIVE_COLOR] untouched."
    stop_color "$TARGET_COLOR"
    exit 1
fi

echo "⏳ 4. Healthchecking [$TARGET_COLOR]..."
if ! healthcheck_color "$TARGET_COLOR"; then
    echo "❌ [$TARGET_COLOR] UNHEALTHY. Rolling back to [$ACTIVE_COLOR]..."
    stop_color "$TARGET_COLOR"
    exit 1
fi
echo "✅ [$TARGET_COLOR] is healthy."

# ---------------------------------------------------------------------------
# 3. Cut traffic over, then retire the old color
# ---------------------------------------------------------------------------

echo "🔀 5. Switching traffic [$ACTIVE_COLOR] -> [$TARGET_COLOR]..."
switch_traffic "$TARGET_COLOR"
echo "$TARGET_COLOR" > "$ACTIVE_FILE"

echo "🧹 6. Stopping old [$ACTIVE_COLOR] stack..."
stop_color "$ACTIVE_COLOR"

echo "🎉 DEPLOYMENT COMPLETE! [$TARGET_COLOR] is live."
$COMPOSE -p "atlsd-$TARGET_COLOR" -f "$ROOT_DIR/infra/compose/prod.app.yml" ps
