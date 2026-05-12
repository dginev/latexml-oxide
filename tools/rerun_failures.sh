#!/bin/bash
# rerun_failures.sh — re-run the focused 181-paper failure sandbox.
#
# Wraps tools/benchmark_10k.sh against ~/data/10k_sandbox_failures
# (181 symlinks to the original paper zips that errored / fataled /
# aborted in sandbox_full_2026-04-26d_post_ar). Used to validate
# fixes without re-converting the whole 7898-paper sandbox.
#
# Each run gets a timestamped output dir under ~/data/sandbox_failures_<TS>
# so we can diff before/after.
#
# Usage:
#   ./tools/rerun_failures.sh             # full re-run, 20 workers, 60s timeout
#   ./tools/rerun_failures.sh --workers 8
#   ./tools/rerun_failures.sh --rerun-failures   # only re-run papers that failed last run
#
# To regenerate the failure list against a fresh sandbox run:
#   awk -F'\t' '$6 ~ /Status:conversion:[23]/ || $6 == "" {print $1}' \
#     <fresh-results.tsv> | grep -v '^arxiv_id$' > /tmp/failure_ids.txt
#   for id in $(cat /tmp/failure_ids.txt); do
#     ln -sf ~/data/10k_sandbox/${id}.zip ~/data/10k_sandbox_failures/${id}.zip
#   done

set -euo pipefail

INPUT_DIR="${INPUT_DIR:-$HOME/data/10k_sandbox_failures}"
TS="$(date +%Y%m%d_%H%M%S)"
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/data/sandbox_failures_$TS}"

if [[ ! -d "$INPUT_DIR" ]]; then
  echo "ERROR: failure sandbox not found at $INPUT_DIR"
  echo "Recreate it from a results.tsv — see header of this script."
  exit 1
fi

count=$(ls "$INPUT_DIR"/*.zip 2>/dev/null | wc -l)
echo "Re-running $count failure papers from $INPUT_DIR → $OUTPUT_DIR"

# Defaults: 20 workers, 6 GB RAM ceiling per worker, 60 s wall-clock
# per task. Override by exporting MAX_RAM_KB / WORKERS / TIMEOUT_S
# before invocation, or by passing flags through to benchmark_10k.sh.
export MAX_RAM_KB="${MAX_RAM_KB:-6291456}"   # 6 GB in KB

exec "$(dirname "$0")/benchmark_10k.sh" \
  --input-dir "$INPUT_DIR" \
  --output-dir "$OUTPUT_DIR" \
  --workers 20 \
  --timeout 60 \
  "$@"
