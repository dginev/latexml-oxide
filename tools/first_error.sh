#!/bin/bash
# first_error.sh — show the FIRST non-cascade error in a canvas log,
# with source context.
#
# Usage: first_error.sh <log_file>
#        first_error.sh ~/data/staged_canvas_*/astro-ph0103460.log
#
# Cascade categories (errors that are downstream of other errors and
# carry little independent diagnostic value) are filtered out:
#   \@startsection, \lx@begin@alignment, \end{equation}, \endgroup,
#   \@personname, \@add@frontmatter@now
#
# Output: error message + locator + ±5 source lines around the trigger.

set -euo pipefail

LOG="${1:-}"
if [[ -z "$LOG" || ! -f "$LOG" ]]; then
  echo "Usage: $0 <log_file>" >&2
  exit 1
fi

# Patterns that are usually cascade noise, not root causes.
CASCADE_RE='\\@startsection|\\lx@begin@alignment|\\end\{equation\}|\\endgroup|\\@personname|\\@add@frontmatter@now|\\lx@end@inline@math|\\lx@end@display@math'

# Strip ANSI colour, grep for Error:..., filter cascades, take first.
first=$(sed -E 's/\x1b\[[0-9]+m//g' "$LOG" \
  | grep -E '(Error|Fatal):[a-z_]+:[^ ]+' \
  | grep -vE "$CASCADE_RE" \
  | head -1 || true)

if [[ -z "$first" ]]; then
  echo "(no non-cascade error found)" >&2
  exit 0
fi

# Locator is on the next line(s), prefixed with TAB.
err_line_num=$(sed -E 's/\x1b\[[0-9]+m//g' "$LOG" \
  | grep -nE '(Error|Fatal):[a-z_]+:[^ ]+' \
  | grep -vE "$CASCADE_RE" \
  | head -1 | cut -d: -f1)

# Extract the "at <file>; line N col M" locator following this error
locator=$(sed -E 's/\x1b\[[0-9]+m//g' "$LOG" \
  | awk -v start="$err_line_num" 'NR>=start && /^\s+at .*line [0-9]+/ {print; exit}')

echo "=== $first"
[[ -n "$locator" ]] && echo "   $(echo "$locator" | sed 's/^\s*//')"
echo ""

# If we can resolve the source file, show ±5 lines of context.
if [[ -n "$locator" ]]; then
  # locator format: "at <name>; line N col M - line N col M"
  # need to find the source file. assume the canvas extracted it under /tmp/latexml_bench_<id>_*/ or similar
  name=$(echo "$locator" | sed -E 's/.*at ([^;]+);.*/\1/')
  line_n=$(echo "$locator" | sed -E 's/.*line ([0-9]+).*/\1/')

  # Try common locations: the working repro dir if it exists, else nothing.
  paper_id=$(basename "$LOG" .log)
  for dir in /tmp/wit*/${paper_id} /tmp/wit*/  ~/data/staged_canvas_*/extracted/${paper_id}; do
    if [[ -d "$dir" ]]; then
      tex=$(find "$dir" -maxdepth 2 -name '*.tex' | head -1)
      if [[ -n "$tex" ]]; then
        echo "=== source context: $tex (line $line_n ±5)"
        awk -v ln="$line_n" 'NR>=ln-5 && NR<=ln+5 {printf "%5d %s%s\n", NR, (NR==ln?"> ":"  "), $0}' "$tex"
        exit 0
      fi
    fi
  done
  echo "(no extracted source found for $paper_id; extract its zip to /tmp/<dir>/ to see context)"
fi
