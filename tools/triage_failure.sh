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
# Companion to tools/benchmark_10k.sh, which is the Phase-1 canvas runner
# (release profile, no debug info, max throughput). See
# docs/SANDBOX_TRIAGE.md "Two-phase workflow" for the philosophy.
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

# If multiple, prefer one matching the arxiv id; else pick a likely main.
MAIN_TEX="${TEX_FILES[0]}"
for f in "${TEX_FILES[@]}"; do
  base=$(basename "$f" .tex)
  if [[ "$base" == "$ARXIV_ID" || "$base" == "ms" || "$base" == "main" || "$base" == "paper" ]]; then
    MAIN_TEX="$f"
    break
  fi
done

echo "[triage] arxiv_id : $ARXIV_ID"
echo "[triage] main tex : $MAIN_TEX"
echo "[triage] profile  : test (cargo default — debug=full, incremental)"
echo "[triage] running  : cargo run --bin latexml_oxide -- $MAIN_TEX $*"
echo

cd "$(dirname "$0")/.."
RUST_BACKTRACE="${RUST_BACKTRACE:-1}" \
  exec cargo run --bin latexml_oxide -- "$MAIN_TEX" "$@"
