#!/usr/bin/env bash
# List the tests a CI test step skipped because their requirements were not met
# (`latexml::util::test::check_requirements`, Perl's `check_requirements`): one
# `<test>\t<reason>` line each in the skip log. No log means no test skipped — a
# TeX Live holding every package (macOS's Homebrew TeX Live) writes none.
# Usage: tools/test_skip_summary.sh [log]   (default target/test-skips.log)
set -euo pipefail
log=${1:-target/test-skips.log}
count=0
if [ -f "$log" ]; then count=$(wc -l < "$log" | tr -d ' '); fi
echo "$count skipped test(s)"
if [ "$count" -gt 0 ]; then
  sort "$log"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    { echo "### $count skipped test(s)"; echo '```'; sort "$log"; echo '```'; } >> "$GITHUB_STEP_SUMMARY"
  fi
fi
