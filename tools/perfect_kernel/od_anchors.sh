#!/usr/bin/env bash
# od_anchors.sh: every `OXIDIZED_DESIGN #N` / `OXIDIZED_DESIGN_DIVERGENCES(.md) #N` cited in a tracked .rs or .md file
# resolves to a `### N.` heading (or stub) in docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md, or to a "formerly OD #N" entry
# in docs/math/OXIDIZED_DESIGN_MATH.md; and every `OXIDIZED_DESIGN_MATH(.md) #N` to a `### N.` there. OD numbers are
# permanent anchors (cited from code): a moved entry leaves a stub. Exit 1 and list the misses when one does not resolve.
# Usage: tools/perfect_kernel/od_anchors.sh [repo root, default: this checkout]
set -euo pipefail
root=${1:-$(git -C "$(dirname "$0")" rev-parse --show-toplevel)}
cd "$root"
od=docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md
math=docs/math/OXIDIZED_DESIGN_MATH.md
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
{ grep -oE '^### [0-9]+\.' "$od" | grep -oE '[0-9]+'
  grep -oE 'formerly OD #[0-9]+' "$math" | grep -oE '[0-9]+$'; } | sort -u > "$tmp/od"
grep -oE '^### [0-9]+\.' "$math" | grep -oE '[0-9]+' | sort -u > "$tmp/math"
git ls-files -z -- '*.rs' '*.md' | xargs -0 grep -noE \
  'OXIDIZED_DESIGN(_DIVERGENCES|_MATH)?(\.md)?[]`)]?[ ,]*#[0-9]+' 2>/dev/null > "$tmp/cites" || true
awk -v odf="$tmp/od" -v mf="$tmp/math" '
  BEGIN { while ((getline n < odf) > 0) od[n] = 1; while ((getline n < mf) > 0) m[n] = 1 }
  { cite = $0; sub(/^[^:]*:[0-9]+:/, "", cite); n = cite; sub(/.*#/, "", n)
    loc = $0; sub(/:[^:]*$/, "", loc)
    if (cite ~ /_MATH/) { if (!(n in m)) { print loc ": OXIDIZED_DESIGN_MATH #" n " has no heading"; bad++ } }
    else if (!(n in od)) { print loc ": OXIDIZED_DESIGN #" n " has no heading or stub"; bad++ }
    total++ }
  END { printf "od_anchors: %d citations, %d unresolved\n", total, bad + 0; exit (bad > 0) }' "$tmp/cites"
