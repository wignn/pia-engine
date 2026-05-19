#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-$ROOT_DIR/infrastructure/compose.yml}"
DB_ENV_FILE="${DB_ENV_FILE:-$ROOT_DIR/infrastructure/env/.env.db}"

usage() {
  cat <<'USAGE'
Usage:
  scripts/create-admin.sh [email] [name] [password]

Environment variables:
  ADMIN_EMAIL        Admin email. Defaults to admin@example.com.
  ADMIN_NAME         Admin display name. Defaults to Administrator.
  ADMIN_PASSWORD     Admin password. If empty, a random password is generated.
  CONTROL_PLANE_URL  Control plane base URL. Defaults to http://localhost:8081.
  COMPOSE_FILE       Docker Compose file. Defaults to infrastructure/compose.yml.
  DB_ENV_FILE        Postgres env file. Defaults to infrastructure/env/.env.db.
  POSTGRES_USER      Postgres user fallback. Defaults to atlsd.
  POSTGRES_DB        Postgres database fallback. Defaults to core.

Examples:
  scripts/create-admin.sh admin@example.com "Admin" "change-me-now"
  ADMIN_PASSWORD="change-me-now" scripts/create-admin.sh admin@example.com
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

ADMIN_EMAIL="${1:-${ADMIN_EMAIL:-admin@example.com}}"
ADMIN_NAME="${2:-${ADMIN_NAME:-Administrator}}"
ADMIN_PASSWORD="${3:-${ADMIN_PASSWORD:-}}"

if [[ -f "$DB_ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  . "$DB_ENV_FILE"
  set +a
fi

POSTGRES_USER="${POSTGRES_USER:-atlsd}"
POSTGRES_DB="${POSTGRES_DB:-core}"
CONTROL_PLANE_URL="${CONTROL_PLANE_URL:-http://localhost:${CONTROL_PLANE_PORT:-8081}}"
API_BASE="${CONTROL_PLANE_URL%/}/api/v1"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

generate_password() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -base64 24 | tr -d '\n'
  else
    local password
    password="$(LC_ALL=C tr -dc 'A-Za-z0-9_@%+=:,.!' </dev/urandom | head -c 24 || true)"
    if [[ -z "$password" ]]; then
      echo "Failed to generate password. Set ADMIN_PASSWORD manually." >&2
      exit 1
    fi
    printf '%s' "$password"
  fi
}

compose_cmd() {
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    docker compose -f "$COMPOSE_FILE" "$@"
  elif command -v docker-compose >/dev/null 2>&1; then
    docker-compose -f "$COMPOSE_FILE" "$@"
  else
    echo "Missing Docker Compose. Install docker compose or docker-compose." >&2
    exit 1
  fi
}

require_cmd curl
require_cmd sed

generated_password=false
if [[ -z "$ADMIN_PASSWORD" ]]; then
  ADMIN_PASSWORD="$(generate_password)"
  generated_password=true
fi

if (( ${#ADMIN_PASSWORD} < 6 )); then
  echo "ADMIN_PASSWORD must be at least 6 characters." >&2
  exit 1
fi

payload="$(printf '{"email":"%s","name":"%s","password":"%s"}' \
  "$(json_escape "$ADMIN_EMAIL")" \
  "$(json_escape "$ADMIN_NAME")" \
  "$(json_escape "$ADMIN_PASSWORD")")"

tmp_response="$(mktemp)"
cleanup() {
  rm -f "$tmp_response"
}
trap cleanup EXIT

echo "Registering admin user through $API_BASE/auth/register ..."
created_user=false
existing_user=false
http_code="$(
  curl -sS -o "$tmp_response" -w '%{http_code}' \
    -H 'Content-Type: application/json' \
    -X POST "$API_BASE/auth/register" \
    --data "$payload" || true
)"

case "$http_code" in
  200|201)
    echo "User created."
    created_user=true
    ;;
  409)
    echo "User already exists, promoting existing account."
    existing_user=true
    ;;
  000)
    echo "Cannot reach control plane at $CONTROL_PLANE_URL." >&2
    echo "Start it first, for example: docker compose -f infrastructure/compose.yml up -d postgres redis control-plane" >&2
    exit 1
    ;;
  *)
    echo "Register request failed with HTTP $http_code:" >&2
    cat "$tmp_response" >&2
    echo >&2
    exit 1
    ;;
esac

echo "Promoting $ADMIN_EMAIL to enterprise plan and verified status ..."
compose_cmd exec -T postgres psql \
  -U "$POSTGRES_USER" \
  -d "$POSTGRES_DB" \
  -v ON_ERROR_STOP=1 \
  -v admin_email="$ADMIN_EMAIL" <<'SQL'
UPDATE users
SET
  plan = 'enterprise',
  is_active = TRUE,
  email_verified = TRUE,
  verify_token = NULL,
  updated_at = NOW()
WHERE lower(email) = lower(:'admin_email')
RETURNING id, email, name, plan, is_active, email_verified;
SQL

echo
echo "Admin account ready:"
echo "  email: $ADMIN_EMAIL"
if [[ "$generated_password" == true && "$created_user" == true ]]; then
  echo "  password: $ADMIN_PASSWORD"
elif [[ "$existing_user" == true ]]; then
  echo "  password: unchanged (account already existed)"
fi
echo
echo "Note: control-plane /admin/* endpoints currently require ADMIN_API_KEY authentication."
echo "This account is promoted to enterprise so it can enter the portal admin route, but API admin calls still need the configured admin key."
