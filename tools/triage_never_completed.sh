#!/usr/bin/env bash
# Solo-triage the run-170 `never_completed_with_retries` papers: convert each ALONE with the
# release worker under GENEROUS limits (300s timeout = 2.5× prod, 32 GB RSS cap, no contention),
# so we can tell a contention/shed casualty (converts fine given room) from a genuinely
# resource-bound or pathological paper (dies even solo). One process at a time → clean per-paper
# wall + peak-RSS, and no risk of co-scheduled hungry papers OOMing the box.
set -uo pipefail
BIN=~/git/latexml-oxide/target/release/cortex_worker
IDS="1707.01231 1711.02901 1402.1906 1208.4948 0803.1344 1810.05740 1201.5525 1701.06513 1602.03591"
printf '%-12s %-7s %-9s %-9s %-7s %s\n' id exit wall_s rss_gb out class
for id in $IDS; do
  src="/data/arxiv_shuffle_1902/$id/$id.zip"; out="/tmp/triage_$id.zip"; log="/tmp/triage_$id.log"
  rm -f "$out"
  /usr/bin/time -v "$BIN" --standalone --input "$src" --output "$out" \
      --timeout 300 --max-rss-mb 32768 >"$log" 2>"$log.time"
  ec=$?
  wall=$(awk -F': ' '/Elapsed \(wall/{print $2}' "$log.time" | awk -F: '{if(NF==3)print $1*3600+$2*60+$3; else print $1*60+$2}')
  rsskb=$(awk -F': ' '/Maximum resident set/{print $2}' "$log.time")
  rssgb=$(awk -v k="${rsskb:-0}" 'BEGIN{printf "%.1f", k/1048576}')
  if [ -f "$out" ]; then
    st=$(unzip -p "$out" cortex.log 2>/dev/null | grep -oiE 'Status:conversion:[0-9]+' | tail -1)
    outc="${st:-zip_no_log}"
  else
    outc="NO_OUTPUT"
  fi
  # classify by exit code + output presence
  case "$ec" in
    0) class="CONVERTS_SOLO (contention/shed casualty)";;
    124|137) class="RESOURCE_BOUND (timeout/oom even solo)";;
    134|139) class="CRASH (abort/segv — engine pathology)";;
    *) class="exit=$ec (investigate)";;
  esac
  [ "$outc" != "NO_OUTPUT" ] && [ "$ec" != 0 ] && class="COMPLETED_FATAL_SOLO ($outc)"
  printf '%-12s %-7s %-9s %-9s %-7s %s\n' "$id" "$ec" "${wall:-?}" "$rssgb" "${outc:0:18}" "$class"
done
echo "=== triage done ==="
