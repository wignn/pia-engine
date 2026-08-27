#!/usr/bin/env bash
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

MONITORING_ENV="$ROOT_DIR/infra/env/.env.monitoring"
EXAMPLE_ENV="$ROOT_DIR/infra/env/monitoring.env.example"

echo "=========================================="
echo "📊 ATLSD MONITORING STACK RUNNER"
echo "=========================================="

if [ ! -f "$MONITORING_ENV" ]; then
    if [ -f "$EXAMPLE_ENV" ]; then
        echo "📋 Copying default monitoring environment from example..."
        cp "$EXAMPLE_ENV" "$MONITORING_ENV"
    else
        echo "⚠️ Creating default .env.monitoring..."
        touch "$MONITORING_ENV"
    fi
fi

if ! docker network inspect atlsd_private >/dev/null 2>&1; then
    echo "❌ atlsd_private network not found. Start the production infrastructure stack first."
    exit 1
fi

echo "🚀 Starting Grafana, Prometheus, Alertmanager, Loki, Promtail, Node Exporter..."
docker compose \
    -f "$ROOT_DIR/infra/compose/monitoring.yml" \
    --env-file "$MONITORING_ENV" \
    up -d --remove-orphans

echo "⏳ Waiting for Grafana & Prometheus healthchecks..."
sleep 5

if curl -s http://127.0.0.1:3000/api/health >/dev/null; then
    echo "✅ Grafana is running at http://127.0.0.1:3000"
else
    echo "⚠️ Grafana is starting up or unavailable at localhost:3000"
fi

if curl -s http://127.0.0.1:9090/-/healthy >/dev/null; then
    echo "✅ Prometheus is running at http://127.0.0.1:9090"
else
    echo "⚠️ Prometheus is starting up or unavailable at localhost:9090"
fi

echo "🎉 Monitoring stack successfully launched!"
