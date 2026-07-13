#!/usr/bin/env bash
# Pre-release reproducer sweep: convert every self-contained reproducer under
# docs/reproducers/ and report the current Rust Error/Fatal count. Reproducers
# are minimal witnesses distilled from real papers (see each file's header for
# the symptom + pdflatex/Perl ground truth). A FIXED reproducer converts with 0
# errors; an OPEN one still shows its failure. Run this before a release to catch
# regressions (a fixed repro going non-zero) and to review the open long-tail.
#
# Usage:  tools/repro_sweep.sh [--bin PATH] [--filter GLOB]
#   --bin     latexml_oxide binary (default: target/release/latexml_oxide)
#   --filter  only run reproducers whose basename matches GLOB (default: *)
#
# All reproducers run with --includestyles so raw-loaded packages
# (IEEEtrantools, tikz, …) resolve; that is the strictest, most faithful path.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/latexml_oxide"
FILTER='*'
while [ $# -gt 0 ]; do
  case "$1" in
    --bin) BIN="$2"; shift 2;;
    --filter) FILTER="$2"; shift 2;;
    *) echo "unknown arg: $1" >&2; exit 2;;
  esac
done
[ -x "$BIN" ] || { echo "binary not found/executable: $BIN" >&2; exit 2; }

REPRO_DIR="$ROOT/docs/reproducers"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

printf '%-56s %6s %6s  %s\n' "REPRODUCER" "ERR" "FATAL" "STATUS"
printf '%.0s-' {1..86}; echo
total=0; clean=0; failing=0
for f in "$REPRO_DIR"/$FILTER.tex; do
  [ -f "$f" ] || continue
  base="$(basename "$f")"
  # Skip fragments (no \documentclass — not standalone-runnable).
  grep -qE '\\documentclass' "$f" || { printf '%-56s %6s %6s  %s\n' "$base" "-" "-" "SKIP(fragment)"; continue; }
  total=$((total+1))
  log="$TMP/$base.log"
  timeout 180 "$BIN" --includestyles --dest="$TMP/$base.html" "$f" >"$log" 2>&1
  rc=$?
  err=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -acE '^Error:')
  fat=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -acE '^Fatal:')
  if [ "$rc" = 124 ]; then status="TIMEOUT";  failing=$((failing+1))
  elif [ "$fat" -gt 0 ]; then status="FATAL";  failing=$((failing+1))
  elif [ "$err" -gt 0 ]; then status="errors"; failing=$((failing+1))
  else status="clean"; clean=$((clean+1)); fi
  printf '%-56s %6s %6s  %s\n' "$base" "$err" "$fat" "$status"
done
printf '%.0s-' {1..86}; echo
echo "swept=$total  clean=$clean  with-errors/fatal/timeout=$failing"
