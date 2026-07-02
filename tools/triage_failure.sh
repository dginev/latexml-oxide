#!/usr/bin/env bash
# triage_failure.sh — Phase-2 triage entry point for a single sandbox failure.
#
# Looks up <arxiv_id> in $HOME/data/10k_sandbox/<arxiv_id>.zip, unzips it
# under a temp directory, finds the main .tex file, and runs
# `cargo run --bin latexml_oxide` against it under the **test profile**
# (debug=full, debug-assertions, overflow-checks, incremental, unwind) so
# the operator gets full backtraces, line numbers, and a 5-second
# turnaround on subsequent rebuilds.
#
# Companion to tools/benchmark_canvas.sh, which is the Phase-1 canvas runner
# (release profile, no debug info, max throughput). See
# docs/archive/SANDBOX_TRIAGE_2026-05-21.md "Two-phase workflow" for the philosophy.
#
# Usage:
#   tools/triage_failure.sh 1407.5769
#   tools/triage_failure.sh 1407.5769 --                 # extra args to latexml_oxide
#   tools/triage_failure.sh 1407.5769 -- --timeout 600
#   SANDBOX_DIR=~/data/sandbox_failures tools/triage_failure.sh 0704.0192
#   KEEP_TMP=1 tools/triage_failure.sh 1407.5769         # keep extracted dir
#
# Env:
#   SANDBOX_DIR   default $HOME/data/10k_sandbox
#   KEEP_TMP      if set, leave the extracted directory after exit
#                 (path is printed to stderr at the start of the run)

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <arxiv_id> [-- extra args to latexml_oxide]" >&2
  exit 2
fi

ARXIV_ID="$1"
shift || true
# Drop a leading -- so callers can pass through extra latexml_oxide args
if [[ "${1:-}" == "--" ]]; then
  shift
fi

SANDBOX_DIR="${SANDBOX_DIR:-$HOME/data/10k_sandbox}"
ZIP="$SANDBOX_DIR/${ARXIV_ID}.zip"

if [[ ! -f "$ZIP" ]]; then
  echo "ERROR: $ZIP not found" >&2
  echo "       Set SANDBOX_DIR or check the arxiv_id." >&2
  exit 1
fi

# Temp directory — kept on KEEP_TMP, otherwise removed on exit.
WORK=$(mktemp -d "${TMPDIR:-/tmp}/triage_${ARXIV_ID//[^A-Za-z0-9]/_}_XXXXXX")
if [[ -z "${KEEP_TMP:-}" ]]; then
  trap 'rm -rf "$WORK"' EXIT
else
  echo "[triage] KEEP_TMP=1 — extracted dir: $WORK" >&2
fi

unzip -q -o "$ZIP" -d "$WORK"

# Prefer a single top-level .tex; otherwise fall back to the first one.
mapfile -t TEX_FILES < <(find "$WORK" -maxdepth 3 -name '*.tex' -type f | sort)
if [[ ${#TEX_FILES[@]} -eq 0 ]]; then
  echo "ERROR: no .tex file found inside $ZIP" >&2
  exit 1
fi

# If multiple, prefer one matching the arxiv id or a "main"-ish basename.
# Otherwise scan for the file containing \documentclass / \documentstyle —
# subsidiary figure includes (e.g. nkfig1.tex in hep-ph0003141) come up
# first alphabetically and would otherwise be picked by mistake.
MAIN_TEX="${TEX_FILES[0]}"
named_match=
for f in "${TEX_FILES[@]}"; do
  base=$(basename "$f" .tex)
  if [[ "$base" == "$ARXIV_ID" || "$base" == "ms" || "$base" == "main" || "$base" == "paper" ]]; then
    named_match="$f"
    break
  fi
done
if [[ -n "$named_match" ]]; then
  MAIN_TEX="$named_match"
elif [[ ${#TEX_FILES[@]} -gt 1 ]]; then
  for f in "${TEX_FILES[@]}"; do
    if grep -lE '\\documentclass|\\documentstyle' "$f" >/dev/null 2>&1; then
      MAIN_TEX="$f"
      break
    fi
  done
fi

echo "[triage] arxiv_id : $ARXIV_ID"
echo "[triage] main tex : $MAIN_TEX"

cd "$(dirname "$0")/.."

# Use cortex_worker (the same binary the canvas uses), so triage results
# match what tools/benchmark_canvas.sh observed. Prefer the existing
# release build; fall back to a fresh build if missing or stale.
WORKER_BIN="${WORKER_BIN:-./target/release/cortex_worker}"
if [[ ! -x "$WORKER_BIN" ]]; then
  echo "[triage] building cortex_worker (release, --features cortex) ..."
  cargo build --release --bin cortex_worker --features cortex
fi

# cortex_worker --standalone takes the .zip as input, not the .tex; emits
# a result archive into --output. We run against the original .zip so
# the worker's own unzip + cwd handling matches the canvas exactly.
INPUT_ZIP="$SANDBOX_DIR/${ARXIV_ID}.zip"
OUTPUT_TMP="${OUTPUT_TMP:-/tmp/triage_${ARXIV_ID}_$$.zip}"

echo "[triage] worker   : $WORKER_BIN --standalone"
echo "[triage] input    : $INPUT_ZIP"
echo "[triage] output   : $OUTPUT_TMP"
echo "[triage] running  : $WORKER_BIN --standalone --input $INPUT_ZIP --output $OUTPUT_TMP --timeout 120 $*"
echo

RUST_BACKTRACE="${RUST_BACKTRACE:-1}" \
  exec "$WORKER_BIN" --standalone --input "$INPUT_ZIP" --output "$OUTPUT_TMP" --timeout 120 "$@"
