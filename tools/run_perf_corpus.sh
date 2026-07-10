#!/bin/bash
# Standing perf corpus runner — unzips each Tier A paper + complex/si.tex
# into a tmpdir, converts with the release binary, records wall-clock and
# exit code. Idle-serial (no parallelism) — use for regression triage
# against the baseline in docs/performance/PERFORMANCE.md.
#
# Usage:
#   tools/run_perf_corpus.sh               # full corpus, serial
#   tools/run_perf_corpus.sh 0906.1883     # single paper

set -u
BIN=${LATEXML_OXIDE_BIN:-/home/deyan/git/latexml-oxide/target/release/latexml_oxide}
BIND=${AR5IV_BINDINGS:-/home/deyan/git/ar5iv-bindings/bindings}
SANDBOX=${SANDBOX_ZIPS:-/home/deyan/data/10k_sandbox}
SI_TEX=/home/deyan/git/latexml-oxide/latexml_oxide/tests/complex/si.tex

# Tier A slow-paper set (session 120 baseline; resolved sessions 116-124).
PAPERS=(0906.1883 1011.1955 1009.1431 1008.4386 0909.2656 0911.4739 1005.1610 0803.0466)

run_zip() {
  local id=$1
  local zip="$SANDBOX/$id.zip"
  if [[ ! -f "$zip" ]]; then echo "$id  MISSING_ZIP"; return 1; fi
  local work
  work=$(mktemp -d)
  unzip -q "$zip" -d "$work" 2>/dev/null
  local main
  main=$(grep -l '^[^%]*\\documentclass\|^[^%]*\\documentstyle' "$work"/*.tex 2>/dev/null | head -1)
  [[ -z "$main" ]] && main=$(ls "$work"/*.tex 2>/dev/null | head -1)
  if [[ -z "$main" ]]; then echo "$id  NO_TEX"; rm -rf "$work"; return 1; fi
  local t0 t1 dt rc
  t0=$(date +%s.%N)
  "$BIN" --preload=ar5iv.sty --path="$BIND" --dest="$work/out.html" --timeout=60 "$main" >/dev/null 2>&1
  rc=$?
  t1=$(date +%s.%N)
  dt=$(awk -v a="$t1" -v b="$t0" 'BEGIN{printf "%.2f", a-b}')
  printf "%-15s %-40s exit=%d  dt=%ss\n" "$id" "$(basename "$main")" "$rc" "$dt"
  rm -rf "$work"
}

run_si() {
  [[ -f "$SI_TEX" ]] || { echo "si.tex  MISSING"; return 1; }
  local work
  work=$(mktemp -d)
  cp "$SI_TEX" "$work/"
  local t0 t1 dt rc
  t0=$(date +%s.%N)
  ( cd "$work" && "$BIN" --preload=ar5iv.sty --path="$BIND" --dest=out.html --timeout=60 si.tex ) >/dev/null 2>&1
  rc=$?
  t1=$(date +%s.%N)
  dt=$(awk -v a="$t1" -v b="$t0" 'BEGIN{printf "%.2f", a-b}')
  printf "%-15s %-40s exit=%d  dt=%ss\n" "complex/si.tex" "si.tex" "$rc" "$dt"
  rm -rf "$work"
}

if [[ $# -eq 0 ]]; then
  for id in "${PAPERS[@]}"; do run_zip "$id"; done
  run_si
elif [[ "$1" == "si.tex" || "$1" == "complex/si.tex" ]]; then
  run_si
else
  run_zip "$1"
fi
