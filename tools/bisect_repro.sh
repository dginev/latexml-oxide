#!/bin/bash
# bisect_repro.sh — extract a minimal repro from a paper that triggers
# a specific error.
#
# Usage: bisect_repro.sh <paper_id> <canary_pattern>
#        bisect_repro.sh astro-ph0103460 'unexpected:\\end\{equation\}'
#        bisect_repro.sh hep-th0005159 'Fatal:'
#
# Algorithm (simple coarse-bisection — works well for cascade-style
# errors where the trigger is a single localized construct):
#   1. Extract source from ~/data/arxmliv/<bucket>/<id>/<id>.zip.
#   2. Confirm canary fires on the full file.
#   3. Find the first error's source line N.
#   4. Build a candidate minimal file:
#        \documentclass{...} ... \begin{document}
#        <lines [N-30 .. N+10] from source>
#        \end{document}
#      Carry over `\def...` `\newcommand...` `\usepackage...` from preamble.
#   5. If canary fires on this minimal, output it as the repro.
#      If it doesn't fire, expand by ±20 lines and retry.
#   6. Cap at 5 expansions; emit whatever last fired or report failure.
#
# Output: minimal .tex file at /tmp/repro_<paper>.tex, plus a diagnostic
# line on stdout.

set -euo pipefail

PAPER="${1:-}"
CANARY="${2:-Error:}"
if [[ -z "$PAPER" ]]; then
  echo "Usage: $0 <paper_id> [canary_pattern]" >&2
  echo "  canary_pattern defaults to 'Error:' (any error)" >&2
  exit 1
fi

ARXMLIV="${ARXMLIV:-$HOME/data/arxmliv}"
RUST_BIN="${RUST_BIN:-$HOME/git/latexml-oxide/target/debug/latexml_oxide}"

bucket=${PAPER:0:4}
zip="$ARXMLIV/$bucket/$PAPER/$PAPER.zip"
if [[ ! -f "$zip" ]]; then
  zip=$(find "$ARXMLIV" -maxdepth 3 -name "$PAPER.zip" -print -quit 2>/dev/null)
fi
[[ -z "$zip" || ! -f "$zip" ]] && { echo "no zip for $PAPER" >&2; exit 1; }

work=$(mktemp -d)
trap "rm -rf $work" EXIT
unzip -o "$zip" -d "$work" > /dev/null 2>&1

main=$(find "$work" -maxdepth 1 -name '*.tex' | head -1)
[[ -z "$main" ]] && { echo "no .tex in $zip" >&2; exit 1; }

# Confirm canary fires on full file.
echo "Step 1: confirm canary fires on full $PAPER..." >&2
full_out=$(cd "$work" && timeout 60 "$RUST_BIN" "$(basename "$main")" 2>&1 | sed -E 's/\x1b\[[0-9]+m//g' || true)
if ! echo "$full_out" | grep -qE "$CANARY"; then
  echo "Canary '$CANARY' does NOT fire on full $PAPER." >&2
  exit 1
fi

# Find first canary error's source line.
first_loc=$(echo "$full_out" | awk -v re="$CANARY" '
  $0 ~ re { in_err=1; next }
  in_err && /^\s+at .*line [0-9]+/ { print; exit }')

if [[ -z "$first_loc" ]]; then
  echo "Could not locate canary source line." >&2
  exit 1
fi
line_n=$(echo "$first_loc" | sed -E 's/.*line ([0-9]+).*/\1/')
echo "Step 2: canary fires at line $line_n" >&2

# Extract preamble: everything before \begin{document} or first
# non-comment non-\def line beyond first 40 lines.
preamble_end=$(awk '/^\\begin\{document\}/ { print NR; exit }' "$main")
[[ -z "$preamble_end" ]] && preamble_end=40

# Bisection loop: start with ±20 lines around the error, expand if needed.
for span in 20 40 80 160 320; do
  start=$((line_n - span))
  end=$((line_n + 10))
  (( start < preamble_end + 1 )) && start=$((preamble_end + 1))

  repro="/tmp/repro_${PAPER}.tex"
  {
    awk -v end="$preamble_end" 'NR <= end' "$main"
    [[ "$preamble_end" != "40" ]] || echo "\\begin{document}"
    awk -v s="$start" -v e="$end" 'NR >= s && NR <= e' "$main"
    echo "\\end{document}"
  } > "$repro"

  # Check if canary still fires.
  out=$(cd "$work" && timeout 30 "$RUST_BIN" "$repro" 2>&1 | sed -E 's/\x1b\[[0-9]+m//g' || true)
  if echo "$out" | grep -qE "$CANARY"; then
    lines=$(wc -l < "$repro")
    echo "Step 3: minimal repro at $repro ($lines lines, span ±$span)" >&2
    echo "$repro"
    exit 0
  fi
done

echo "Bisection failed — canary doesn't fire on any span around line $line_n." >&2
echo "The error may depend on cumulative state. Try the full file at /tmp/wit_${PAPER}/" >&2
mkdir -p "/tmp/wit_${PAPER}"
cp -r "$work"/* "/tmp/wit_${PAPER}/" 2>/dev/null
exit 1
