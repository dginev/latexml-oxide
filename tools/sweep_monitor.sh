#!/usr/bin/env bash
# sweep_monitor.sh — sample a live cortex sandbox sweep (perf + robustness) until it drains.
# Records a 10s time-series of task progress, the per-severity distribution, system memory
# headroom, aggregate worker RSS, and fleet size — then exits when no task is left TODO/Queued
# (the same drain predicate run-completion-on-drain uses). One row per sample to $CSV.
#
# Usage: CORPUS_ID=2 SERVICE_ID=4 CSV=/tmp/sweep_monitor.csv tools/sweep_monitor.sh
set -uo pipefail
CORPUS_ID="${CORPUS_ID:-2}"
SERVICE_ID="${SERVICE_ID:-4}"
CSV="${CSV:-/tmp/sweep_monitor.csv}"
INTERVAL="${INTERVAL:-10}"
PSQL=(env PGPASSWORD=cortex psql -h localhost -U cortex -d cortex -tAq -F,)

T0=$(date +%s)
echo "elapsed_s,todo,queued,done,no_problem,warning,error,fatal,invalid,mem_avail_mb,worker_rss_mb,workers" >"$CSV"
while true; do
  now=$(date +%s); el=$((now - T0))
  row=$("${PSQL[@]}" -c "select
      count(*) filter (where status=0),
      count(*) filter (where status>0),
      count(*) filter (where status<0),
      count(*) filter (where status=-1),
      count(*) filter (where status=-2),
      count(*) filter (where status=-3),
      count(*) filter (where status=-4),
      count(*) filter (where status=-5)
    from tasks where corpus_id=$CORPUS_ID and service_id=$SERVICE_ID;" 2>/dev/null)
  todo=${row%%,*}; rest=${row#*,}; queued=${rest%%,*}
  memavail=$(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo)
  rss=$(ps --no-headers -o rss -C cortex_worker 2>/dev/null | awk '{s+=$1} END{print int(s/1024)}')
  [ -z "$rss" ] && rss=0
  nw=$(pgrep -c cortex_worker 2>/dev/null || echo 0)
  echo "$el,$row,$memavail,$rss,$nw" >>"$CSV"
  # Drained: nothing TODO and nothing leased/Queued (matches HistoricalRun::complete_if_drained).
  if [ "${todo:-1}" = 0 ] && [ "${queued:-1}" = 0 ]; then
    echo "DRAINED after ${el}s ($(date '+%H:%M:%S'))"
    break
  fi
  sleep "$INTERVAL"
done
