#!/usr/bin/env bash
# sweep_worker_count.sh — re-measure the throughput-optimal harness worker count under the CURRENT
# limits (2048-recycle default). Worker count is the ONLY variable. Fair-comparison methodology:
# the instantaneous rate is NOT flat (it falls from ~35/s while cheap papers drain to ~14/s on the
# hard residual), and a single rerun depletes the cheap fraction across points — so each point gets
# a FRESH rerun (also clears orphaned leases) and is timed over the SAME done-range [LO,HI]. That
# measures the identical representative bulk at every worker count, long enough for aggregate RSS to
# build and expose any high-count memory pressure. Reports bulk tasks/s + peak RSS + governor sheds.
# Supersedes the old report_summary-capped ~14/s table in memory [[cortex-fleet-worker-count]].
set -uo pipefail
BIN=~/git/latexml-oxide/target/release/cortex_worker
CORTEX=~/git/cortex/target/release/cortex
DB=postgres://cortex:cortex@localhost/cortex
COUNTS="${COUNTS:-72 96 120 136}"
LO="${LO:-1000}"; HI="${HI:-6000}"; CAP="${CAP:-600}"
q() { env PGPASSWORD=cortex psql -h localhost -U cortex -d cortex -tAq -c \
       "select count(*) filter (where status<0) from tasks where corpus_id=2 and service_id=4"; }
mem() { awk '/MemAvailable/{printf "%.0f",$2/1024}' /proc/meminfo; }  # kB → MiB (printf later → GiB)
rss() { ps --no-headers -o rss -C cortex_worker 2>/dev/null | awk '{s+=$1}END{printf "%d",s/1024}'; }

printf '%-8s %-9s %-10s %-11s %-6s %-7s\n' workers tasks_s peakRSSgb minMemAvgb sheds wall_s
for N in $COUNTS; do
  ( cd ~/git/cortex && DATABASE_URL="$DB" "$CORTEX" rerun \
      sandbox-arxiv-10k-shuffle oxidized-tex-to-html --yes --owner claude-agent \
      --description "wc-sweep N=$N" ) >/dev/null 2>&1
  log="/tmp/swc_N${N}.log"
  setsid "$BIN" --harness --workers "$N" >"$log" 2>&1 &
  PG=$!
  s=$(date +%s); peak=0; minmem=999999
  while :; do [ "$(q)" -ge "$LO" ] && break; [ $(($(date +%s)-s)) -ge "$CAP" ] && break; sleep 3; done
  t0=$(date +%s)
  while :; do
    d=$(q); r=$(rss); m=$(mem)
    [ "${r:-0}" -gt "$peak" ] && peak=$r; [ "${m:-0}" -lt "$minmem" ] && minmem=$m
    [ "${d:-0}" -ge "$HI" ] && break
    [ $(($(date +%s)-t0)) -ge "$CAP" ] && break
    sleep 5
  done
  t1=$(date +%s); wall=$((t1-t0))
  rate=$(awk -v a="$LO" -v b="$HI" -v t="$wall" 'BEGIN{printf "%.1f",(b-a)/t}')
  sheds=$(grep -ciE 'shedding|shed .*largest|mem pressure' "$log")
  printf '%-8s %-9s %-10s %-11s %-6s %-7s\n' "$N" "$rate" \
    "$(awk -v p=$peak 'BEGIN{printf "%.0f",p/1024}')" \
    "$(awk -v m=$minmem 'BEGIN{printf "%.0f",m/1024}')" "$sheds" "$wall"
  kill -TERM -"$PG" 2>/dev/null; sleep 3; kill -KILL -"$PG" 2>/dev/null
  for _ in $(seq 1 20); do [ "$(pgrep -c cortex_worker || echo 0)" = 0 ] && break; sleep 1; done
done
echo "=== worker-count sweep (rerun-per-point, fixed [$LO,$HI] bulk) done ==="
