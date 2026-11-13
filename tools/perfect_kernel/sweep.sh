#!/usr/bin/env bash
# Sweep the perfect-kernel corpus (or a subset) through run_doc.sh.
#
# Usage: sweep.sh <corpus.tsv> [outroot]
#   corpus.tsv: output of enumerate_corpus.sh (bundle \t tex \t pdf \t lines)
#   outroot defaults to ~/data/perfect_kernel
#
# Env: JOBS (default 8 — docs/THERMALS.md), TIMEOUT_S (default 120), WORKER_BIN.
# Skips documents that already have a verdict.tsv (delete it to re-run).
# Writes <outroot>/sweep_verdicts.tsv (rebuilt from all verdict.tsv at the end).
set -uo pipefail

CORPUS="$1"
OUTROOT="${2:-$HOME/data/perfect_kernel}"
JOBS="${JOBS:-8}"  # docs/THERMALS.md: 8 alone, 4 beside anything else
export TIMEOUT_S="${TIMEOUT_S:-120}"
HERE="$(cd "$(dirname "$0")" && pwd)"
export OUTROOT HERE

run_one() {
  local tex="$1"
  local name bundle out
  name=$(basename "$tex" .tex)
  bundle=$(basename "$(dirname "$tex")")
  out="$OUTROOT/$bundle/$name"
  if [[ -f "$out/verdict.tsv" ]]; then
    return 0
  fi
  "$HERE/run_doc.sh" "$tex" "$OUTROOT" >/dev/null 2>&1
  # Record a verdict even if run_doc.sh itself was killed abnormally.
  if [[ ! -f "$out/verdict.tsv" ]]; then
    printf '%s\t%s\t137\t137\t0\t0\t0\t0\n' "$bundle" "$name" >"$out/verdict.tsv"
  fi
}
export -f run_one

cut -f2 "$CORPUS" | xargs -P "$JOBS" -n1 -I{} bash -c 'run_one "$@"' _ {}

find "$OUTROOT" -mindepth 3 -maxdepth 3 -name verdict.tsv -print0 |
  xargs -0 cat | sort >"$OUTROOT/sweep_verdicts.tsv"
awk -F'\t' '{n[$3]++} END {for (s in n) printf "status %s: %d\n", s, n[s]}' \
  "$OUTROOT/sweep_verdicts.tsv" | sort
