#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-/home/wign/atlsd/infra/monitoring/textfile}"
TMP_FILE="$OUT_DIR/docker_stats.prom.$$"
OUT_FILE="$OUT_DIR/docker_stats.prom"
mkdir -p "$OUT_DIR"

escape_label() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

{
  echo '# HELP atlsd_docker_container_cpu_percent Docker container CPU usage percent from docker stats.'
  echo '# TYPE atlsd_docker_container_cpu_percent gauge'
  echo '# HELP atlsd_docker_container_memory_usage_bytes Docker container memory usage bytes from docker stats.'
  echo '# TYPE atlsd_docker_container_memory_usage_bytes gauge'
  echo '# HELP atlsd_docker_container_memory_limit_bytes Docker container memory limit bytes from docker stats.'
  echo '# TYPE atlsd_docker_container_memory_limit_bytes gauge'
  echo '# HELP atlsd_docker_container_up Docker container visible in docker stats.'
  echo '# TYPE atlsd_docker_container_up gauge'

  docker stats --no-stream --format '{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}' | while IFS=$'\t' read -r name cpu mem; do
    [ -n "$name" ] || continue
    cpu_value="${cpu%%%}"
    usage_raw="${mem%% / *}"
    limit_raw="${mem##* / }"

    to_bytes() {
      local value="$1"
      python3 - "$value" <<'PY'
import re, sys
s=sys.argv[1].strip().replace('B','')
m=re.match(r'^([0-9.]+)\s*([KMGTPE]?i?)?$', s)
if not m:
    print(0); raise SystemExit
n=float(m.group(1)); unit=(m.group(2) or '').lower()
scale={'':1,'k':1e3,'m':1e6,'g':1e9,'t':1e12,'p':1e15,'e':1e18,'ki':1024,'mi':1024**2,'gi':1024**3,'ti':1024**4,'pi':1024**5,'ei':1024**6}
print(int(n*scale.get(unit,1)))
PY
    }

    usage_bytes="$(to_bytes "$usage_raw")"
    limit_bytes="$(to_bytes "$limit_raw")"
    label="container=\"$(escape_label "$name")\""
    echo "atlsd_docker_container_cpu_percent{$label} $cpu_value"
    echo "atlsd_docker_container_memory_usage_bytes{$label} $usage_bytes"
    echo "atlsd_docker_container_memory_limit_bytes{$label} $limit_bytes"
    echo "atlsd_docker_container_up{$label} 1"
  done
} > "$TMP_FILE"

mv "$TMP_FILE" "$OUT_FILE"
