#!/bin/bash
# benchmark_10k.sh — 10k sandbox benchmark runner for latexml-oxide
#
# Converts all ZIP archives in the input directory through cortex_worker
# standalone mode, with timeout/RAM guards, resumability, and structured logging.
#
# Usage:
#   ./tools/benchmark_10k.sh                    # full run, 4 workers
#   ./tools/benchmark_10k.sh --limit 50         # dry run, first 50 files
#   ./tools/benchmark_10k.sh --workers 8        # 8 parallel workers
#   ./tools/benchmark_10k.sh --rerun-failures   # only re-run previous failures

set -euo pipefail

# ─── Defaults ────────────────────────────────────────────────────────────────

INPUT_DIR="${INPUT_DIR:-$HOME/data/10k_sandbox}"
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/data/10k_sandbox_html}"
WORKER_BIN="${WORKER_BIN:-$(dirname "$0")/../target/release/cortex_worker}"
RESULTS_TSV=""  # set below after OUTPUT_DIR is finalized
WORKERS="${WORKERS:-16}"
TIMEOUT_S="${TIMEOUT_S:-120}"
MAX_RAM_KB="${MAX_RAM_KB:-8388608}"   # 8 GB in KB (for ulimit -v)
LIMIT=0              # 0 = no limit
RERUN_FAILURES=false
MAX_OUTPUT_MB=200    # cap: skip output ZIPs larger than this

# ─── Parse arguments ─────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case "$1" in
    --input-dir)    INPUT_DIR="$2"; shift 2 ;;
    --output-dir)   OUTPUT_DIR="$2"; shift 2 ;;
    --workers)      WORKERS="$2"; shift 2 ;;
    --timeout)      TIMEOUT_S="$2"; shift 2 ;;
    --limit)        LIMIT="$2"; shift 2 ;;
    --rerun-failures) RERUN_FAILURES=true; shift ;;
    --worker-bin)   WORKER_BIN="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --input-dir DIR       Input directory (default: \$HOME/data/10k_sandbox)"
      echo "  --output-dir DIR      Output directory (default: \$HOME/data/10k_sandbox_html)"
      echo "  --workers N           Parallel workers (default: 4)"
      echo "  --timeout SECS        Per-task wall-clock timeout (default: 120)"
      echo "  --limit N             Process only first N files (default: 0 = all)"
      echo "  --rerun-failures      Only re-run tasks that failed in previous run"
      echo "  --worker-bin PATH     Path to cortex_worker binary"
      echo "  -h, --help            Show this help"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

RESULTS_TSV="$OUTPUT_DIR/results.tsv"

# ─── Validate environment ────────────────────────────────────────────────────

if [[ ! -d "$INPUT_DIR" ]]; then
  echo "ERROR: Input directory not found: $INPUT_DIR"
  exit 1
fi

if [[ ! -x "$WORKER_BIN" ]]; then
  # Try resolving relative to script dir
  WORKER_BIN="$(cd "$(dirname "$0")/.." && pwd)/target/release/cortex_worker"
  if [[ ! -x "$WORKER_BIN" ]]; then
    echo "ERROR: cortex_worker binary not found. Run: cargo build --release --bin cortex_worker"
    exit 1
  fi
fi

mkdir -p "$OUTPUT_DIR"

# Disable core dumps globally for this script and all children
ulimit -c 0

# ─── Disk space check ────────────────────────────────────────────────────────

AVAIL_GB=$(df --output=avail "$OUTPUT_DIR" | tail -1 | awk '{printf "%.0f", $1/1048576}')
INPUT_COUNT=$(find "$INPUT_DIR" -maxdepth 1 -name '*.zip' | wc -l)
# Conservative estimate: 1 MB average output per input
ESTIMATED_GB=$(( (INPUT_COUNT + 1023) / 1024 ))

echo "=== 10k Sandbox Benchmark ==="
echo "Input:     $INPUT_DIR ($INPUT_COUNT ZIPs)"
echo "Output:    $OUTPUT_DIR"
echo "Workers:   $WORKERS"
echo "Timeout:   ${TIMEOUT_S}s per task"
echo "RAM limit: $((MAX_RAM_KB / 1048576)) GB per task"
echo "Disk:      ${AVAIL_GB} GB available, ~${ESTIMATED_GB} GB estimated output"
echo "Binary:    $WORKER_BIN"

if (( AVAIL_GB < ESTIMATED_GB * 2 )); then
  echo "WARNING: Available disk (${AVAIL_GB} GB) is less than 2x estimated output (${ESTIMATED_GB} GB)"
  echo "         Consider freeing space or reducing --limit"
fi

# ─── Build task list ──────────────────────────────────────────────────────────

TASK_LIST=$(mktemp)
# trap set below after RUN_RESULTS is created

if [[ "$RERUN_FAILURES" == true ]] && [[ -f "$RESULTS_TSV" ]]; then
  # Re-run any non-OK paper, sorted by name. Includes both:
  #   - exit_code != 0 (panics, OOM, timeouts, aborts) AND
  #   - exit_code == 0 with category != "ok" (conversion_error, _fatal, etc.)
  # The earlier `$3 != "0"` filter only captured the first kind, leaving
  # in-process status:2 / status:3 papers untouched on rerun.
  echo "Mode: re-running previous failures only"
  awk -F'\t' 'NR>1 && $7 != "ok" {print $1}' "$RESULTS_TSV" | sort | while read -r arxiv_id; do
    input_zip="$INPUT_DIR/${arxiv_id}.zip"
    if [[ -f "$input_zip" ]]; then
      echo "$input_zip"
    fi
  done > "$TASK_LIST"
  # Remove old results for these failures so they get fresh entries
  if [[ -s "$TASK_LIST" ]]; then
    # Build grep pattern to exclude re-run IDs
    EXCLUDE_PATTERN=$(sed 's|.*/||; s|\.zip$||' "$TASK_LIST" | paste -sd'|')
    grep -vE "^(${EXCLUDE_PATTERN})\t" "$RESULTS_TSV" > "${RESULTS_TSV}.tmp" || true
    # Keep header
    head -1 "$RESULTS_TSV" > "${RESULTS_TSV}.new"
    tail -n +2 "${RESULTS_TSV}.tmp" >> "${RESULTS_TSV}.new" 2>/dev/null || true
    mv "${RESULTS_TSV}.new" "$RESULTS_TSV"
    rm -f "${RESULTS_TSV}.tmp"
  fi
else
  # Full run with resumability: skip files that already have output, sorted by name
  echo "Mode: full run (skipping completed)"
  SKIPPED=0
  for zip in $(ls "$INPUT_DIR"/*.zip | sort); do
    arxiv_id=$(basename "$zip" .zip)
    output_zip="$OUTPUT_DIR/${arxiv_id}.zip"
    if [[ -f "$output_zip" ]]; then
      SKIPPED=$((SKIPPED + 1))
      continue
    fi
    echo "$zip"
  done > "$TASK_LIST"
  if (( SKIPPED > 0 )); then
    echo "Skipped:   $SKIPPED already completed"
  fi
fi

TOTAL=$(wc -l < "$TASK_LIST")
if (( LIMIT > 0 && TOTAL > LIMIT )); then
  head -n "$LIMIT" "$TASK_LIST" > "${TASK_LIST}.limited"
  mv "${TASK_LIST}.limited" "$TASK_LIST"
  TOTAL=$LIMIT
fi

echo "To process: $TOTAL files"
echo ""

if (( TOTAL == 0 )); then
  echo "Nothing to do."
  exit 0
fi

# ─── Initialize results TSV ──────────────────────────────────────────────────

if [[ ! -f "$RESULTS_TSV" ]]; then
  printf "arxiv_id\tentry_id\texit_code\twall_time_s\toutput_size_bytes\tstatus_line\tcategory\n" > "$RESULTS_TSV"
fi

# Remove any stale entries for files we're about to (re-)process
# This prevents duplicates when re-running failed tasks in full-run mode
REPROCESS_IDS=$(sed 's|.*/||; s|\.zip$||' "$TASK_LIST" | paste -sd'|')
if [[ -n "$REPROCESS_IDS" ]]; then
  head -1 "$RESULTS_TSV" > "${RESULTS_TSV}.dedup"
  tail -n +2 "$RESULTS_TSV" | grep -vE "^(${REPROCESS_IDS})\t" >> "${RESULTS_TSV}.dedup" 2>/dev/null || true
  mv "${RESULTS_TSV}.dedup" "$RESULTS_TSV"
fi

# Track current-run results separately for accurate summary
RUN_RESULTS=$(mktemp)
trap "rm -f '$TASK_LIST' '$RUN_RESULTS'" EXIT

# ─── Worker function ─────────────────────────────────────────────────────────
# Exported so GNU parallel can call it in subshells.

convert_one() {
  local input_zip="$1"
  local arxiv_id
  arxiv_id=$(basename "$input_zip" .zip)
  local output_zip="$OUTPUT_DIR/${arxiv_id}.zip"
  local log_file="$OUTPUT_DIR/${arxiv_id}.log"

  # Per-task temp directory (cleaned up even on kill)
  local task_tmp
  task_tmp=$(mktemp -d "${TMPDIR:-/tmp}/latexml_bench_${arxiv_id}_XXXXXX")

  local start_time wall_time exit_code output_size status_line category

  start_time=$(date +%s%N)

  # Run with timeout + RAM guard
  # timeout sends SIGTERM, then SIGKILL after 10s grace
  exit_code=0
  TMPDIR="$task_tmp" timeout --kill-after=10 "$TIMEOUT_S" \
    bash -c "ulimit -v $MAX_RAM_KB 2>/dev/null; exec \"\$@\"" -- \
    "$WORKER_BIN" --standalone --input "$input_zip" --output "$output_zip" \
    --timeout "$TIMEOUT_S" \
    2>"$log_file" || exit_code=$?

  wall_time=$(( ($(date +%s%N) - start_time) / 1000000 ))  # milliseconds

  # Determine output size
  if [[ -f "$output_zip" ]]; then
    output_size=$(stat --format=%s "$output_zip" 2>/dev/null || echo 0)

    # Cap: remove oversized outputs (likely blowup)
    local output_mb=$(( output_size / 1048576 ))
    if (( output_mb > MAX_OUTPUT_MB )); then
      echo "WARNING: $arxiv_id output ${output_mb}MB exceeds ${MAX_OUTPUT_MB}MB cap, removing" >&2
      rm -f "$output_zip"
      output_size=0
      category="oversized"
    fi
  else
    output_size=0
  fi

  # Extract status line from output ZIP if present
  status_line=""
  if [[ -f "$output_zip" ]] && (( output_size > 0 )); then
    status_line=$(unzip -p "$output_zip" status 2>/dev/null || echo "")
  fi

  # Categorize result
  # Status codes: 0=no_problem, 1=warnings, 2=errors, 3=fatal
  if [[ -z "$category" ]]; then
    case "$exit_code" in
      0)
        if (( output_size == 0 )); then
          category="empty_output"
        elif [[ "$status_line" == *"conversion:2"* ]]; then
          category="conversion_error"
        elif [[ "$status_line" == *"conversion:3"* ]]; then
          category="conversion_fatal"
        else
          category="ok"
        fi
        ;;
      124) category="timeout" ;;    # timeout(1) exit code
      137) category="oom_or_kill" ;; # SIGKILL (from timeout --kill-after or OOM)
      139) category="segfault" ;;    # SIGSEGV
      134) category="abort" ;;       # SIGABRT (panic)
      *)   category="error" ;;
    esac
  fi

  # Wall time in seconds (with 1 decimal)
  local wall_time_s
  wall_time_s=$(awk "BEGIN {printf \"%.1f\", $wall_time / 1000}")

  # Append to results (atomic via lock file)
  local result_line
  result_line=$(printf "%s\t%s\t%d\t%s\t%d\t%s\t%s" \
    "$arxiv_id" "$arxiv_id" "$exit_code" "$wall_time_s" "$output_size" "$status_line" "$category")
  (
    flock 200
    echo "$result_line" >> "$RESULTS_TSV"
    echo "$result_line" >> "$RUN_RESULTS"
  ) 200>"${RESULTS_TSV}.lock"

  # Cleanup per-task temp directory
  rm -rf "$task_tmp"

  # Progress indicator to stderr
  echo "[$category] $arxiv_id  ${wall_time_s}s  ${output_size}B" >&2
}

export -f convert_one
export OUTPUT_DIR WORKER_BIN TIMEOUT_S MAX_RAM_KB MAX_OUTPUT_MB RESULTS_TSV RUN_RESULTS

# ─── Run ──────────────────────────────────────────────────────────────────────

echo "Starting at $(date '+%Y-%m-%d %H:%M:%S') ..."
echo ""

RUN_START=$(date +%s)

parallel --will-cite \
  --jobs "$WORKERS" \
  convert_one {} \
  < "$TASK_LIST"

PARALLEL_EXIT=$?
if (( PARALLEL_EXIT != 0 )); then
  echo "Note: parallel exited with code $PARALLEL_EXIT (some tasks failed, see results)" >&2
fi

RUN_END=$(date +%s)
RUN_DURATION=$(( RUN_END - RUN_START ))

# ─── Summary ─────────────────────────────────────────────────────────────────

echo ""
echo "=== Benchmark Complete ==="
echo "Duration: $((RUN_DURATION / 60))m $((RUN_DURATION % 60))s"
echo "Results:  $RESULTS_TSV"
echo ""

# ─── Current run summary (from RUN_RESULTS) ─────────────────────────────────

RUN_TOTAL=$(wc -l < "$RUN_RESULTS")
RUN_OK=$(awk -F'\t' '$7 == "ok"' "$RUN_RESULTS" | wc -l)
RUN_FAIL=$((RUN_TOTAL - RUN_OK))

echo "This run: ${RUN_OK}/${RUN_TOTAL} OK (${RUN_FAIL} failures)"
echo ""

# Category breakdown (this run)
echo "Category breakdown (this run):"
awk -F'\t' '{cats[$7]++} END {for (c in cats) printf "  %-15s %d\n", c, cats[c]}' \
  "$RUN_RESULTS" | sort -t' ' -k2 -rn

echo ""

# Slow tasks (>60s, this run)
SLOW=$(awk -F'\t' '$4+0 > 60 {printf "  %-20s %ss\n", $1, $4}' "$RUN_RESULTS" | sort -t' ' -k2 -rn)
if [[ -n "$SLOW" ]]; then
  echo "Slow tasks (>60s):"
  echo "$SLOW"
else
  echo "No tasks exceeded 60s."
fi

echo ""

# Failures (this run)
FAILURES=$(awk -F'\t' '$7 != "ok" {printf "  %-20s %s (exit %s)\n", $1, $7, $3}' "$RUN_RESULTS")
if [[ -n "$FAILURES" ]] && (( RUN_FAIL <= 100 )); then
  echo "Failures:"
  echo "$FAILURES"
elif (( RUN_FAIL > 100 )); then
  echo "(${RUN_FAIL} failures — see $RESULTS_TSV for full list)"
fi

# ─── Cumulative totals ───────────────────────────────────────────────────────

CUM_TOTAL=$(awk -F'\t' 'NR>1' "$RESULTS_TSV" | wc -l)
CUM_OK=$(awk -F'\t' 'NR>1 && $7 == "ok"' "$RESULTS_TSV" | wc -l)
echo ""
echo "Cumulative: ${CUM_OK}/${CUM_TOTAL} OK across all runs"
