#!/bin/bash
# benchmark_canvas.sh — sandbox canvas benchmark runner for latexml-oxide
#
# Converts all ZIP archives in the input directory through cortex_worker
# standalone mode, with timeout/RAM guards, resumability, and structured logging.
#
# Stages: a large canvas (e.g. the 100k "no-problem" sandbox at
# ~/data/100k_noproblem_sandbox) can be partitioned into fixed-size stages
# via `--stage N --stage-size 10000`. Each stage gets its own output
# subdirectory `<OUTPUT_DIR>/stage_NN/` (and its own results.tsv) so stages
# can be processed independently and merged later.
#
# Usage:
#   ./tools/benchmark_canvas.sh                                # full run
#   ./tools/benchmark_canvas.sh --limit 50                     # dry run, first 50 files
#   ./tools/benchmark_canvas.sh --workers 8                    # 8 parallel workers
#   ./tools/benchmark_canvas.sh --rerun-failures               # only re-run previous failures
#   ./tools/benchmark_canvas.sh --input-dir ~/data/100k_noproblem_sandbox \
#                               --stage 1 --stage-size 10000   # first 10k slice
#   ./tools/benchmark_canvas.sh --input-dir ~/data/100k_noproblem_sandbox \
#                               --stage 5 --stage-size 10000   # 5th 10k slice

set -euo pipefail

# ─── Defaults ────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CUSTOM_WORKER_BIN=false
if [[ -n "${WORKER_BIN:-}" ]]; then
  CUSTOM_WORKER_BIN=true
fi

INPUT_DIR="${INPUT_DIR:-$HOME/data/10k_sandbox}"
OUTPUT_DIR_OVERRIDDEN=false
if [[ -n "${OUTPUT_DIR:-}" ]]; then
  OUTPUT_DIR_OVERRIDDEN=true
fi
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/data/10k_sandbox_html}"
WORKER_BIN="${WORKER_BIN:-$REPO_ROOT/target/release/cortex_worker}"
RESULTS_TSV=""  # set below after OUTPUT_DIR is finalized
WORKERS="${WORKERS:-16}"
TIMEOUT_S="${TIMEOUT_S:-120}"
MAX_RAM_KB="${MAX_RAM_KB:-8388608}"   # 8 GB in KB (for ulimit -v)
BUILD_JOBS="${BUILD_JOBS:-$(nproc)}"
LIMIT=0              # 0 = no limit
RERUN_FAILURES=false
# When CLEANUP_OK=true, after the run delete output zips/logs for rows whose
# category is "ok". Useful for bulk-canvas runs where only failure cases are
# kept for triage and disk space is the binding constraint.
CLEANUP_OK="${CLEANUP_OK:-false}"
MAX_OUTPUT_MB="${MAX_OUTPUT_MB:-200}"  # cap: skip output ZIPs larger than this
# Find depth for input ZIPs. The 10k canvas ships flat
# (`<dir>/<id>.zip`, depth 1). The 100k canvas at
# `~/data/100k_noproblem_sandbox/` is nested
# (`<dir>/arxmliv/<bucket>/<id>/<id>.zip`, depth 4). The 430k canvas
# at `~/data/430k_noproblem_sandbox/` adds an extra `data/` parent
# (`<dir>/data/arxmliv/<bucket>/<id>/<id>.zip`, depth 5). Default 6
# covers all known layouts with one level of headroom; tighten via
# env if you need to exclude deeper nesting.
INPUT_MAXDEPTH="${INPUT_MAXDEPTH:-6}"
# Stage selection: 0 means no slicing (process the whole canvas).
# When STAGE > 0, the sorted task list is sliced to
# [(STAGE-1)*STAGE_SIZE, STAGE*STAGE_SIZE) and OUTPUT_DIR is suffixed
# with /stage_NN/ unless the caller overrode --output-dir explicitly.
STAGE="${STAGE:-0}"
STAGE_SIZE="${STAGE_SIZE:-10000}"

# ─── Parse arguments ─────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case "$1" in
    --input-dir)    INPUT_DIR="$2"; shift 2 ;;
    --output-dir)   OUTPUT_DIR="$2"; OUTPUT_DIR_OVERRIDDEN=true; shift 2 ;;
    --workers)      WORKERS="$2"; shift 2 ;;
    --timeout)      TIMEOUT_S="$2"; shift 2 ;;
    --limit)        LIMIT="$2"; shift 2 ;;
    --rerun-failures) RERUN_FAILURES=true; shift ;;
    --worker-bin)   WORKER_BIN="$2"; CUSTOM_WORKER_BIN=true; shift 2 ;;
    --stage)        STAGE="$2"; shift 2 ;;
    --stage-size)   STAGE_SIZE="$2"; shift 2 ;;
    --cleanup-ok)   CLEANUP_OK=true; shift ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --input-dir DIR       Input directory (default: \$HOME/data/10k_sandbox)"
      echo "  --output-dir DIR      Output directory (default: \$HOME/data/10k_sandbox_html)"
      echo "                        When --stage N (>0) is given without --output-dir,"
      echo "                        a /stage_NN/ subdirectory is appended automatically."
      echo "  --workers N           Parallel workers (default: 16)"
      echo "  --timeout SECS        Per-task wall-clock timeout (default: 120)"
      echo "  --limit N             Process only first N files of the (post-stage) task"
      echo "                        list (default: 0 = all)"
      echo "  --rerun-failures      Only re-run tasks that failed in previous run"
      echo "  --worker-bin PATH     Path to cortex_worker binary"
      echo "  --stage N             Process the Nth slice of the input directory"
      echo "                        (1-indexed; 0 = no slicing). Useful for the 100k"
      echo "                        canvas: --stage 1..10 with --stage-size 10000."
      echo "  --stage-size N        Slice size for --stage (default: 10000)"
      echo "  --cleanup-ok          After the run, delete output zips/logs for"
      echo "                        rows whose category is 'ok' (keeps only"
      echo "                        failure artifacts; reclaims disk for bulk"
      echo "                        canvas runs)"
      echo "  -h, --help            Show this help"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# ─── Stage handling ──────────────────────────────────────────────────────────
# When STAGE > 0, append /stage_NN/ to the output dir (unless caller passed
# --output-dir explicitly) so each stage gets its own results.tsv and ZIP set.

if (( STAGE < 0 )); then
  echo "ERROR: --stage must be >= 0 (got $STAGE)"
  exit 1
fi
if (( STAGE_SIZE <= 0 )); then
  echo "ERROR: --stage-size must be > 0 (got $STAGE_SIZE)"
  exit 1
fi

STAGE_LABEL=""
if (( STAGE > 0 )); then
  STAGE_LABEL=$(printf "stage_%02d" "$STAGE")
  if [[ "$OUTPUT_DIR_OVERRIDDEN" == false ]]; then
    OUTPUT_DIR="$OUTPUT_DIR/$STAGE_LABEL"
  fi
fi

RESULTS_TSV="$OUTPUT_DIR/results.tsv"
# Per-job telemetry JSONL accumulator (cortex_worker writes telemetry.json
# inside each output ZIP; we extract & append here). End of run: gzip.
# See docs/TELEMETRY.md.
TELEMETRY_JSONL="$OUTPUT_DIR/telemetry.jsonl"

# ─── Validate environment ────────────────────────────────────────────────────

if [[ ! -d "$INPUT_DIR" ]]; then
  echo "ERROR: Input directory not found: $INPUT_DIR"
  exit 1
fi

for required in cargo parallel timeout flock unzip awk find sort; do
  if ! command -v "$required" >/dev/null 2>&1; then
    echo "ERROR: required command not found: $required"
    exit 1
  fi
done

mkdir -p "$OUTPUT_DIR"

# Disable core dumps globally for this script and all children
ulimit -c 0

BUILD_RUSTFLAGS="${RUSTFLAGS:-}"
for flag in -Clinker-features=+lld -Zunstable-options -Ctarget-cpu=native; do
  case " $BUILD_RUSTFLAGS " in
    *" $flag "*) ;;
    *) BUILD_RUSTFLAGS="${BUILD_RUSTFLAGS:+$BUILD_RUSTFLAGS }$flag" ;;
  esac
done

echo "Building fresh release cortex_worker with ${BUILD_JOBS} cargo jobs ..."
(
  cd "$REPO_ROOT"
  RUSTFLAGS="$BUILD_RUSTFLAGS" cargo build --release --bin cortex_worker --features cortex --jobs "$BUILD_JOBS"
)

if [[ "$CUSTOM_WORKER_BIN" == true ]]; then
  echo "WARNING: using custom worker binary despite fresh release build: $WORKER_BIN"
fi

if [[ ! -x "$WORKER_BIN" ]]; then
  echo "ERROR: cortex_worker binary not found after release build: $WORKER_BIN"
  echo "  Expected build command: cargo build --release --bin cortex_worker --features cortex"
  exit 1
fi

# ─── Disk space check ────────────────────────────────────────────────────────

AVAIL_GB=$(df --output=avail "$OUTPUT_DIR" | tail -1 | awk '{printf "%.0f", $1/1048576}')
INPUT_COUNT=$(find "$INPUT_DIR" -maxdepth "$INPUT_MAXDEPTH" -name '*.zip' | wc -l)
# Conservative estimate: 1 MB average output per input
ESTIMATED_GB=$(( (INPUT_COUNT + 1023) / 1024 ))

echo "=== Sandbox Canvas Benchmark ==="
echo "Input:     $INPUT_DIR ($INPUT_COUNT ZIPs)"
echo "Output:    $OUTPUT_DIR"
if (( STAGE > 0 )); then
  STAGE_FROM=$(( (STAGE - 1) * STAGE_SIZE ))
  STAGE_TO=$(( STAGE * STAGE_SIZE ))
  echo "Stage:     $STAGE (slice [${STAGE_FROM}, ${STAGE_TO})) of size ${STAGE_SIZE}"
fi
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

# When --stage > 0, build a stage-restricted file list once. Both the
# rerun-failures and full-run branches below filter their candidate set
# through this list so a stage only ever sees its own slice of the canvas.
STAGE_FILE_LIST=""
if (( STAGE > 0 )); then
  STAGE_FILE_LIST=$(mktemp)
  STAGE_FROM=$(( (STAGE - 1) * STAGE_SIZE ))
  find "$INPUT_DIR" -maxdepth "$INPUT_MAXDEPTH" -type f -name '*.zip' -print | sort \
    | awk -v from="$STAGE_FROM" -v size="$STAGE_SIZE" \
        'NR > from && NR <= from + size' \
    > "$STAGE_FILE_LIST"
  STAGE_COUNT=$(wc -l < "$STAGE_FILE_LIST")
  echo "Stage filter: $STAGE_COUNT ZIPs in slice"
fi

valid_status_zip() {
  local zip_path="$1"
  local status

  [[ -f "$zip_path" ]] || return 1
  unzip -tq "$zip_path" >/dev/null 2>&1 || return 1

  status=$(unzip -p "$zip_path" status 2>/dev/null || true)
  case "$status" in
    Status:conversion:[0-3]) return 0 ;;
    *) return 1 ;;
  esac
}

has_completed_result() {
  local arxiv_id="$1"
  local row exit_code category output_zip

  [[ -f "$RESULTS_TSV" ]] || return 1

  row=$(awk -F'\t' -v id="$arxiv_id" 'NR>1 && $1 == id {line=$0} END {if (line != "") print line}' "$RESULTS_TSV")
  [[ -n "$row" ]] || return 1

  exit_code=$(awk -F'\t' '{print $3}' <<< "$row")
  category=$(awk -F'\t' '{print $7}' <<< "$row")
  output_zip="$OUTPUT_DIR/${arxiv_id}.zip"

  if [[ "$exit_code" == "0" ]] || [[ "$category" == "ok" ]] ||
     [[ "$category" == "conversion_error" ]] || [[ "$category" == "conversion_fatal" ]]; then
    if valid_status_zip "$output_zip"; then
      return 0
    fi
    return 1
  fi

  return 0
}

if [[ "$RERUN_FAILURES" == true ]]; then
  if [[ ! -f "$RESULTS_TSV" ]]; then
    echo "ERROR: --rerun-failures requested, but no previous results TSV exists: $RESULTS_TSV"
    exit 1
  fi
  # Re-run any non-OK paper, sorted by name. Includes both:
  #   - exit_code != 0 (panics, OOM, timeouts, aborts) AND
  #   - exit_code == 0 with category != "ok" (conversion_error, _fatal, etc.)
  # The earlier `$3 != "0"` filter only captured the first kind, leaving
  # in-process status:2 / status:3 papers untouched on rerun.
  echo "Mode: re-running previous failures only"
  awk -F'\t' 'NR>1 && $7 != "ok" {print $1}' "$RESULTS_TSV" | sort | while read -r arxiv_id; do
    input_zip="$INPUT_DIR/${arxiv_id}.zip"
    if [[ -f "$input_zip" ]]; then
      # When staged, only emit failures that fall inside the current
      # stage's slice — other stages handle their own failures.
      if [[ -n "$STAGE_FILE_LIST" ]] \
         && ! grep -Fxq "$input_zip" "$STAGE_FILE_LIST"; then
        continue
      fi
      echo "$input_zip"
    fi
  done > "$TASK_LIST"
else
  # Full run with resumability: skip files only when both results and artifacts validate.
  echo "Mode: full run (skipping completed validated rows)"
  SKIPPED=0
  if [[ -n "$STAGE_FILE_LIST" ]]; then
    INPUT_LIST_SOURCE="$STAGE_FILE_LIST"
  else
    INPUT_LIST_SOURCE=$(mktemp)
    find "$INPUT_DIR" -maxdepth "$INPUT_MAXDEPTH" -type f -name '*.zip' -print | sort > "$INPUT_LIST_SOURCE"
  fi
  while IFS= read -r zip; do
    arxiv_id=$(basename "$zip" .zip)
    if has_completed_result "$arxiv_id"; then
      SKIPPED=$((SKIPPED + 1))
      continue
    fi
    echo "$zip"
  done < "$INPUT_LIST_SOURCE" > "$TASK_LIST"
  if [[ "$INPUT_LIST_SOURCE" != "$STAGE_FILE_LIST" ]]; then
    rm -f "$INPUT_LIST_SOURCE"
  fi
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

# Track current-run results separately for accurate summary
RUN_RESULTS=$(mktemp)
trap "rm -f '$TASK_LIST' '$RUN_RESULTS' ${STAGE_FILE_LIST:+'$STAGE_FILE_LIST'}" EXIT

# ─── Worker function ─────────────────────────────────────────────────────────
# Exported so GNU parallel can call it in subshells.

convert_one() {
  local input_zip="$1"
  local arxiv_id
  arxiv_id=$(basename "$input_zip" .zip)
  local output_zip="$OUTPUT_DIR/${arxiv_id}.zip"
  local output_tmp="$OUTPUT_DIR/.${arxiv_id}.zip.${BASHPID}.${RANDOM}.tmp"
  local log_file="$OUTPUT_DIR/${arxiv_id}.log"
  local log_tmp="$OUTPUT_DIR/.${arxiv_id}.log.${BASHPID}.${RANDOM}.tmp"

  # Per-task temp directory (cleaned up even on kill)
  local task_tmp
  task_tmp=$(mktemp -d "${TMPDIR:-/tmp}/latexml_bench_${arxiv_id}_XXXXXX")
  trap 'rm -rf "$task_tmp"; rm -f "$output_tmp" "$log_tmp"' RETURN

  local start_time wall_time exit_code output_size status_line category
  category=""

  start_time=$(date +%s%N)

  # Run with timeout + RAM guard
  # timeout sends SIGTERM, then SIGKILL after 10s grace
  exit_code=0
  TMPDIR="$task_tmp" timeout --kill-after=10 "$TIMEOUT_S" \
    bash -c "ulimit -v $MAX_RAM_KB 2>/dev/null; exec \"\$@\"" -- \
    "$WORKER_BIN" --standalone --input "$input_zip" --output "$output_tmp" \
    --timeout "$TIMEOUT_S" \
    2>"$log_tmp" || exit_code=$?

  wall_time=$(( ($(date +%s%N) - start_time) / 1000000 ))  # milliseconds

  # Determine output size
  if [[ -f "$output_tmp" ]]; then
    output_size=$(stat --format=%s "$output_tmp" 2>/dev/null || echo 0)

    # Cap: remove oversized outputs (likely blowup)
    local output_mb=$(( output_size / 1048576 ))
    if (( output_mb > MAX_OUTPUT_MB )); then
      echo "WARNING: $arxiv_id output ${output_mb}MB exceeds ${MAX_OUTPUT_MB}MB cap, removing" >&2
      rm -f "$output_tmp"
      output_size=0
      category="oversized"
    fi
  else
    output_size=0
  fi

  # Extract status line from output ZIP if present
  status_line=""
  if [[ -f "$output_tmp" ]] && (( output_size > 0 )); then
    if unzip -tq "$output_tmp" >/dev/null 2>&1; then
      status_line=$(unzip -p "$output_tmp" status 2>/dev/null || echo "")
    else
      category="invalid_output"
    fi
  fi
  status_line=${status_line//$'\t'/ }
  status_line=${status_line//$'\n'/ }

  # Categorize result
  # Status codes: 0=no_problem, 1=warnings, 2=errors, 3=fatal
  if [[ -z "$category" ]]; then
    case "$exit_code" in
      0)
        if (( output_size == 0 )); then
          category="empty_output"
        else
          case "$status_line" in
            Status:conversion:0|Status:conversion:1) category="ok" ;;
            Status:conversion:2) category="conversion_error" ;;
            Status:conversion:3) category="conversion_fatal" ;;
            "") category="missing_status" ;;
            *) category="invalid_status" ;;
          esac
        fi
        ;;
      124) category="timeout" ;;    # timeout(1) exit code
      137) category="oom_or_kill" ;; # SIGKILL (from timeout --kill-after or OOM)
      139) category="segfault" ;;    # SIGSEGV
      134) category="abort" ;;       # SIGABRT (panic)
      *)   category="error" ;;
    esac
  fi

  case "$category" in
    ok|conversion_error|conversion_fatal)
      mv -f "$output_tmp" "$output_zip"
      # Extract telemetry.json (single-line JSON) from the output ZIP
      # and append to the corpus-wide JSONL accumulator. Failure is
      # non-fatal — older binaries won't have the member.
      local telem_line
      if telem_line=$(unzip -p "$output_zip" telemetry.json 2>/dev/null) && [[ -n "$telem_line" ]]; then
        (
          flock 201
          printf '%s\n' "$telem_line" >> "$TELEMETRY_JSONL"
        ) 201>"${TELEMETRY_JSONL}.lock"
      fi
      ;;
    *)
      rm -f "$output_tmp"
      output_size=0
      ;;
  esac

  if [[ -f "$log_tmp" ]]; then
    mv -f "$log_tmp" "$log_file"
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
    echo "$result_line" >> "$RUN_RESULTS"
  ) 200>"${RESULTS_TSV}.lock"

  # Progress indicator to stderr
  echo "[$category] $arxiv_id  ${wall_time_s}s  ${output_size}B" >&2
}

export -f convert_one
export OUTPUT_DIR WORKER_BIN TIMEOUT_S MAX_RAM_KB MAX_OUTPUT_MB RESULTS_TSV RUN_RESULTS TELEMETRY_JSONL

# ─── Run ──────────────────────────────────────────────────────────────────────

echo "Starting at $(date '+%Y-%m-%d %H:%M:%S') ..."
echo ""

RUN_START=$(date +%s)

PARALLEL_EXIT=0
parallel --will-cite \
  --jobs "$WORKERS" \
  convert_one {} \
  < "$TASK_LIST" || PARALLEL_EXIT=$?

if (( PARALLEL_EXIT != 0 )); then
  echo "Note: parallel exited with code $PARALLEL_EXIT (some tasks failed, see results)" >&2
fi

# Retry transient categories serially. Under 16-worker contention some
# papers hit timeout/abort/oom_or_kill/error not because they fail but
# because their share of CPU was insufficient (see resource-exhaustion
# analysis in commit history). Retry pass runs each candidate once with
# all cores free; results that flip to ok/conversion_* replace the
# transient row in RUN_RESULTS.
RETRY_LIST=$(mktemp)
RETRY_CANDIDATES_RE='^(timeout|abort|oom_or_kill|error|missing_status|invalid_status|invalid_output|empty_output)$'
awk -F'\t' -v re="$RETRY_CANDIDATES_RE" '$7 ~ re {print $1"\t"$7}' "$RUN_RESULTS" \
  | sort -u > "$RETRY_LIST"
RETRY_COUNT=$(wc -l < "$RETRY_LIST" | tr -d ' ')
if (( RETRY_COUNT > 0 )); then
  echo ""
  echo "Retry pass: $RETRY_COUNT transient result(s) — running serially..." >&2
  RETRY_FLIPPED=0
  RETRY_RESULTS=$(mktemp)
  while IFS=$'\t' read -r arxiv_id orig_category; do
    # Locate the original input zip path from the original task list
    input_zip=$(awk -v id="$arxiv_id" '{
      n = split($0, parts, "/"); fname = parts[n]; sub(/\.zip$/, "", fname);
      if (fname == id) { print; exit }
    }' "$TASK_LIST")
    [[ -z "$input_zip" ]] && continue
    convert_one "$input_zip"
  done < "$RETRY_LIST"
  # Diff old vs new categories for retried IDs to count flips
  RETRY_FLIPPED=$(awk -F'\t' -v list="$RETRY_LIST" '
    BEGIN { while ((getline line < list) > 0) { split(line, a, "\t"); orig[a[1]] = a[2] } }
    $1 in orig {
      if (latest[$1] == "" || $2+0 > latest_id[$1]+0) { latest[$1] = $7; latest_id[$1] = $2 }
    }
    END {
      flipped = 0
      for (id in orig) {
        new = latest[id]
        if (new == "ok" || new == "conversion_error" || new == "conversion_fatal") flipped++
      }
      print flipped
    }' "$RUN_RESULTS")
  echo "Retry pass: $RETRY_FLIPPED of $RETRY_COUNT recovered to ok/conversion_*" >&2
  rm -f "$RETRY_RESULTS"
fi
rm -f "$RETRY_LIST"

# Atomically merge current-run rows into the cumulative TSV after workers finish.
# Existing rows are kept unless the current run produced a replacement row.
MERGED_RESULTS=$(mktemp)
RUN_IDS=$(mktemp)
awk -F'\t' '{print $1}' "$RUN_RESULTS" | sort -u > "$RUN_IDS"
{
  head -1 "$RESULTS_TSV"
  awk -F'\t' 'FNR == NR {ids[$1] = 1; next} FNR > 1 && !($1 in ids)' "$RUN_IDS" "$RESULTS_TSV"
  cat "$RUN_RESULTS"
} > "$MERGED_RESULTS"
mv "$MERGED_RESULTS" "$RESULTS_TSV"

# Avoid leaving stale success artifacts behind after a paper now fails before
# producing a valid output ZIP.
while IFS=$'\t' read -r arxiv_id _entry_id _exit_code _wall_time _output_size _status_line category; do
  case "$category" in
    ok|conversion_error|conversion_fatal) ;;
    *) rm -f "$OUTPUT_DIR/${arxiv_id}.zip" ;;
  esac
done < "$RUN_RESULTS"

# When --cleanup-ok is set, delete output zips and logs for "ok" rows so only
# failure artifacts remain on disk. Telemetry and the row in results.tsv are
# preserved (telemetry was already extracted during the run).
if [[ "$CLEANUP_OK" == "true" ]]; then
  CLEANUP_OK_COUNT=0
  while IFS=$'\t' read -r arxiv_id _entry_id _exit_code _wall_time _output_size _status_line category; do
    if [[ "$category" == "ok" ]]; then
      rm -f "$OUTPUT_DIR/${arxiv_id}.zip" "$OUTPUT_DIR/${arxiv_id}.log"
      CLEANUP_OK_COUNT=$((CLEANUP_OK_COUNT + 1))
    fi
  done < "$RUN_RESULTS"
  echo "cleanup-ok: deleted artifacts for ${CLEANUP_OK_COUNT} ok rows"
fi

rm -f "$RUN_IDS"

RUN_END=$(date +%s)
RUN_DURATION=$(( RUN_END - RUN_START ))

# Compress the per-job telemetry JSONL accumulator. Removes uncompressed
# original. tools/perf_phase_summary.py reads the .gz directly.
if [[ -s "$TELEMETRY_JSONL" ]]; then
  gzip -f "$TELEMETRY_JSONL" 2>/dev/null && \
    echo "Telemetry: $(zcat "${TELEMETRY_JSONL}.gz" | wc -l) records → ${TELEMETRY_JSONL}.gz"
fi
rm -f "${TELEMETRY_JSONL}.lock"

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
