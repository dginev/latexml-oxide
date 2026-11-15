#!/usr/bin/env bash
# od_anchors.sh: every OD citation in a tracked .rs or .md file — `OXIDIZED_DESIGN #N`,
# `OXIDIZED_DESIGN_DIVERGENCES(.md) #N`, and the short forms `DIVERGENCES #N` and `OD #N` — resolves to a `### N.`
# heading (or stub) in docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md, or to a "formerly OD #N" entry in
# docs/math/OXIDIZED_DESIGN_MATH.md; every `OXIDIZED_DESIGN_MATH(.md) #N` to a `### N.` there; and every OD stub's target
# exists: `→ KNOWN_PERL_ERRORS #M` a `## M.` in KNOWN_PERL_ERRORS.md, `→ OXIDIZED_DESIGN_MATH #M` a `### M.` in the math
# doc, `→ #M` an OD entry that is no stub, `→ Bindings beyond Perl's tree` a `#### N.` in OD's catalog, `→ SYNC_STATUS`
# a "formerly OD #N" line in docs/SYNC_STATUS.md. OD numbers are permanent anchors (cited from code): a moved entry
# leaves a stub. Exit 1 and list the misses when one does not resolve.
# Usage: tools/perfect_kernel/od_anchors.sh [repo root, default: this checkout]
set -euo pipefail
root=${1:-$(git -C "$(dirname "$0")" rev-parse --show-toplevel)}
cd "$root"
od=docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md
math=docs/math/OXIDIZED_DESIGN_MATH.md
kpe=docs/parity/KNOWN_PERL_ERRORS.md
sync=docs/SYNC_STATUS.md
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
{ grep -oE '^### [0-9]+\.' "$od" | grep -oE '[0-9]+'
  grep -oE 'formerly OD #[0-9]+' "$math" | grep -oE '[0-9]+$'; } | sort -u > "$tmp/od"
grep -oE '^### [0-9]+\.' "$math" | grep -oE '[0-9]+' | sort -u > "$tmp/math"
grep -oE '^## [0-9]+\.' "$kpe" | grep -oE '[0-9]+' | sort -u > "$tmp/kpe"
grep -oE '^#### [0-9]+\.' "$od" | grep -oE '[0-9]+' | sort -u > "$tmp/catalog"
grep -oE 'formerly OD #[0-9]+' "$sync" | grep -oE '[0-9]+$' | sort -u > "$tmp/sync"
grep -nE '^### [0-9]+\. ' "$od" > "$tmp/heads"
git ls-files -z -- '*.rs' '*.md' | xargs -0 grep -noE \
  '(OXIDIZED_DESIGN(_DIVERGENCES|_MATH)?(\.md)?|DIVERGENCES(\.md)?|\<OD)[]`)]?[ ,]*#[0-9]+' 2>/dev/null > "$tmp/cites" || true
awk -v odf="$tmp/od" -v mf="$tmp/math" -v kf="$tmp/kpe" -v cf="$tmp/catalog" -v sf="$tmp/sync" -v hf="$tmp/heads" \
    -v odpath="$od" '
  BEGIN {
    while ((getline n < odf) > 0) od[n] = 1; while ((getline n < mf) > 0) m[n] = 1
    while ((getline n < kf) > 0) k[n] = 1; while ((getline n < cf) > 0) c[n] = 1; while ((getline n < sf) > 0) s[n] = 1
    while ((getline h < hf) > 0) {
      ln = h; sub(/:.*/, "", ln); t = h; sub(/^[0-9]+:### /, "", t); n = t; sub(/\..*/, "", n); t = substr(t, length(n) + 3)
      head[n] = t; line[n] = ln }
    for (n in head) {
      t = head[n]; if (t !~ /^→ /) continue
      stubs++; where = odpath ":" line[n] ": stub #" n
      x = t; sub(/^→ [^#(]*#/, "", x); sub(/[^0-9].*$/, "", x)
      if (t ~ /^→ KNOWN_PERL_ERRORS #[0-9]/) { if (!(x in k)) { print where " → KNOWN_PERL_ERRORS #" x " has no heading"; bad++ } }
      else if (t ~ /^→ OXIDIZED_DESIGN_MATH #[0-9]/) { if (!(x in m)) { print where " → OXIDIZED_DESIGN_MATH #" x " has no heading"; bad++ } }
      else if (t ~ /^→ #[0-9]/) { if (!(x in head) || head[x] ~ /^(→|\(removed)/) { print where " → #" x " is no entry"; bad++ } }
      else if (t ~ /^→ Bindings beyond Perl.s tree/) { if (!(n in c)) { print where " has no #### " n ". in the catalog"; bad++ } }
      else if (t ~ /^→ SYNC_STATUS/) { if (!(n in s)) { print where " has no \"formerly OD #" n "\" in SYNC_STATUS"; bad++ } }
      else { print where ": unrecognized stub target"; bad++ } } }
  { cite = $0; sub(/^[^:]*:[0-9]+:/, "", cite); n = cite; sub(/.*#/, "", n)
    loc = $0; sub(/:[^:]*$/, "", loc)
    if (cite ~ /_MATH/) { if (!(n in m)) { print loc ": OXIDIZED_DESIGN_MATH #" n " has no heading"; bad++ } }
    else if (!(n in od)) { print loc ": OXIDIZED_DESIGN #" n " has no heading or stub"; bad++ }
    total++ }
  END { printf "od_anchors: %d citations, %d stubs, %d unresolved\n", total, stubs, bad + 0; exit (bad > 0) }' "$tmp/cites"
