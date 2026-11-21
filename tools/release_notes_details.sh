#!/usr/bin/env bash
# release_notes_details.sh [notes.md] — fold the "## What's Changed" pull-request list of GitHub's generated release
# notes into a <details> block, so a long list does not swamp the release page (DG 2026-11-21). Reads the notes (the
# `body` of `gh api repos/<owner>/<repo>/releases/generate-notes`) from the file or stdin; writes them to stdout with
# that section's `* …` items inside `<details><summary>N merged pull requests</summary> … </details>`. Everything
# else (New Contributors, the Full Changelog link) is passed through unchanged. Notes with no such section, or an
# empty one, are passed through unchanged. Self-test: tools/release_notes_details_test.sh.
set -euo pipefail
awk '
function flush() {
  if (n > 0) printf "\n<details>\n<summary>%d merged pull request%s</summary>\n\n%s\n</details>\n\n", n, (n == 1 ? "" : "s"), buf
  inwc = 0
}
/^## What.s Changed[[:space:]]*$/ { print; inwc = 1; n = 0; buf = ""; next }
inwc && /^\* / { n++; buf = buf $0 "\n"; next }
inwc && /^[[:space:]]*$/ { next }
inwc { flush() }
{ print }
END { if (inwc) flush() }
' "${1:-/dev/stdin}"
