#!/usr/bin/env bash
# maketests.sh — regenerate (".bless") the golden test XML files.
#
# The Rust equivalent of Perl LaTeXML's `tools/maketests`. It runs the normal
# test harness with `LATEXML_BLESS=1`, which makes every `…_ok` test OVERWRITE
# its golden `<name>.xml` with the actual conversion output instead of
# comparing+asserting. Because it reuses the exact harness conversion and
# serialization path (`latexml_oxide/src/util/test.rs::process_texfile`), the
# regenerated goldens are byte-identical to what the comparison expects — there
# is no risk of config/serialization drift between "generate" and "check".
#
# Git is the backup: ALWAYS review `git diff` afterwards and commit deliberately.
# A clean working tree before running is strongly recommended so the diff is
# exactly the intended golden churn.
#
# Usage:
#   tools/maketests.sh                 # regenerate ALL goldens (whole suite)
#   tools/maketests.sh <filter>        # regenerate only tests matching <filter>
#   tools/maketests.sh -- <cargo args> # pass extra args through to cargo test
#
# Examples:
#   tools/maketests.sh                       # bless everything
#   tools/maketests.sh 06_cluster            # bless one test binary
#   tools/maketests.sh array_pcolumn         # bless tests matching a name
#
# After running, verify nothing unexpected changed:
#   git diff --stat
#   cargo test --tests        # should now be green
set -euo pipefail

cd "$(dirname "$0")/.."

if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
  echo "WARNING: working tree is not clean; the blessed diff will mix with your"
  echo "         existing changes. Consider stashing first. Continuing in 3s…" >&2
  sleep 3
fi

filter=()
if [ "$#" -ge 1 ] && [ "$1" != "--" ]; then
  filter=("$1"); shift
fi
# Drop a leading `--` separator if present.
[ "${1:-}" = "--" ] && shift || true

echo "==> Regenerating goldens (LATEXML_BLESS=1 cargo test --tests ${filter[*]} $*)" >&2
LATEXML_BLESS=1 cargo test --tests --no-fail-fast "${filter[@]}" "$@" -- --nocapture 2>&1 \
  | grep -E '^BLESS ' || true

echo "" >&2
echo "==> Done. Review the golden churn:" >&2
echo "      git diff --stat" >&2
echo "    then re-run the suite to confirm green:" >&2
echo "      cargo test --tests" >&2
