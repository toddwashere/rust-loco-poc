#!/usr/bin/env bash
# Side-by-side benchmark: Loco (release binary) vs Next.js (next start, production build).
# Measures cold start, idle RSS, peak RSS under load, throughput and latency per endpoint.
# Usage: bench/run.sh [duration=15s] [concurrency=50]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DUR="${1:-15s}"; CONC="${2:-50}"
OUT="$ROOT/bench/results"; mkdir -p "$OUT"
export PATH="$HOME/.cargo/bin:$PATH"
# Never leave a server holding the port if a run aborts.
trap 'pkill -f "loco_crm-cli start" 2>/dev/null; pkill -f "next start -p 3100" 2>/dev/null; true' EXIT

# Sum RSS (MB) of a process and all its descendants (Next spawns a child next-server).
tree_rss() {
  local pids="$1" all="$1" kids
  while [ -n "$pids" ]; do
    kids=$(for p in $pids; do pgrep -P "$p" || true; done | xargs)
    all="$all $kids"; pids="$kids"
  done
  ps -o rss= -p $(echo $all | tr ' ' ',') 2>/dev/null | awk '{s+=$1} END {printf "%.1f", s/1024}'
}

wait_up() { # prints ms until URL answers 200
  local url="$1" t0; t0=$(python3 -c 'import time;print(int(time.time()*1000))')
  until curl -sf -o /dev/null "$url"; do sleep 0.02; done
  echo $(( $(python3 -c 'import time;print(int(time.time()*1000))') - t0 ))
}

load() { # name url label
  local name="$1" url="$2" label="$3" pid="$4" peak=0 rss
  oha -z "$DUR" -c "$CONC" --no-tui --output-format json "$url" > "$OUT/$name-$label.json" &
  local oha_pid=$!
  while kill -0 $oha_pid 2>/dev/null; do
    rss=$(tree_rss "$pid"); peak=$(python3 -c "print(max($peak,$rss))"); sleep 0.5
  done
  python3 - "$OUT/$name-$label.json" "$name" "$label" "$peak" <<'PY'
import json,sys
d=json.load(open(sys.argv[1])); s=d["summary"]; p=d["latencyPercentiles"]
if not s["successRate"]:
    sys.exit(f'{sys.argv[2]} {sys.argv[3]}: all requests failed: {d.get("errorDistribution")}')
codes=d.get("statusCodeDistribution",{})
print(f'{sys.argv[2]:10} {sys.argv[3]:14} rps={s["requestsPerSec"]:>9.0f}  p50={p["p50"]*1000:6.2f}ms  p99={p["p99"]*1000:7.2f}ms  peakRSS={float(sys.argv[4]):6.1f}MB  codes={codes}')
PY
}

run_app() { # name start_cmd health_url api_url page_url cwd
  local name="$1" cmd="$2" health="$3" api="$4" page="$5" dir="$6"
  (cd "$dir" && exec bash -c "$cmd") > "$OUT/$name-server.log" 2>&1 &
  local pid=$!
  local cold; cold=$(wait_up "$health")
  sleep 3; for u in "$api" "$page"; do curl -sf -o /dev/null "$u"; done; sleep 2
  echo "$name  cold_start=${cold}ms  idle_RSS=$(tree_rss $pid)MB"
  load "$name" "$health" health "$pid"
  load "$name" "$api" api_contacts "$pid"
  load "$name" "$page" ssr_contacts "$pid"
  echo "$name  post-load RSS=$(tree_rss $pid)MB"
  pkill -P "$pid" 2>/dev/null || true; kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true
}

# Production config for Loco: fresh SQLite files, seeded with the same data the twin seeds itself with.
export DATABASE_URL="sqlite://bench.sqlite?mode=rwc" QUEUE_URL="sqlite://bench_queue.sqlite?mode=rwc" \
  JWT_SECRET="bench-only-secret" HOST="http://localhost" MAILER_HOST="localhost" MAILER_USER="bench" MAILER_PASSWORD="bench" LOG_LEVEL="warn"
(cd "$ROOT/loco-crm" && rm -f bench*.sqlite* && ./target/release/loco_crm-cli db migrate -e production >/dev/null 2>&1 \
  && ./target/release/loco_crm-cli task seed_demo -e production 2>&1 | tail -1)
(cd "$ROOT/nextjs-twin" && rm -f twin.sqlite*)

echo "== $(date)  duration=$DUR concurrency=$CONC  $(sysctl -n machdep.cpu.brand_string) $(sysctl -n hw.ncpu) cores"
run_app loco "./target/release/loco_crm-cli start --environment production" \
  http://127.0.0.1:5150/bench/health "http://127.0.0.1:5150/bench/contacts.json?page=3" "http://127.0.0.1:5150/bench/contacts?page=3" "$ROOT/loco-crm"
# Same binary, SQLite pool of 1: sqlx+SQLite loses throughput as the pool grows (see report).
run_app loco-pool1 "DB_MAX_CONNECTIONS=1 DB_MIN_CONNECTIONS=1 ./target/release/loco_crm-cli start --environment production" \
  http://127.0.0.1:5150/bench/health "http://127.0.0.1:5150/bench/contacts.json?page=3" "http://127.0.0.1:5150/bench/contacts?page=3" "$ROOT/loco-crm"
run_app next "./node_modules/.bin/next start -p 3100" \
  http://127.0.0.1:3100/api/health "http://127.0.0.1:3100/api/contacts?page=3" "http://127.0.0.1:3100/contacts?page=3" "$ROOT/nextjs-twin"
