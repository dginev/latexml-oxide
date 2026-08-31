#!/usr/bin/env bash
# S2 pass: RelaxNG-validate every surviving core XML in the sweep output.
#
# Usage: validate.sh [outroot]
# Writes <outroot>/validate_verdicts.tsv:  bundle name rng_errors
# (rng_errors = count of jing error lines; 0 = schema-valid) and prints a
# tally. Uses jing against the repo's authoritative LaTeXML.rng.
set -uo pipefail

OUTROOT="${1:-$HOME/data/perfect_kernel}"
JOBS="${JOBS:-12}"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
V="$OUTROOT/validate_verdicts.tsv"
: >"$V"

# jing can't resolve the schema's `urn:x-LaTeXML:RelaxNG:*` includes — build a
# temp copy of the whole RelaxNG directory with those rewritten to relative
# sibling hrefs.
RNGSRC="$REPO/latexml_core/resources/RelaxNG"
RNGDIR="$(mktemp -d)"
cp "$RNGSRC"/*.rng "$RNGDIR"/
sed -i 's|urn:x-LaTeXML:RelaxNG:||g' "$RNGDIR"/*.rng
RNG="$RNGDIR/LaTeXML.rng"
export RNG
trap 'rm -rf "$RNGDIR"' EXIT

validate_one() {
  local xml="$1"
  local name bundle n
  name=$(basename "$xml" .xml)
  bundle=$(basename "$(dirname "$(dirname "$xml")")")
  n=$(jing "$RNG" "$xml" 2>&1 | grep -c ':' || true)
  printf '%s\t%s\t%s\n' "$bundle" "$name" "$n"
}
export -f validate_one

find "$OUTROOT" -mindepth 3 -maxdepth 3 -name '*.xml' -print0 |
  xargs -0 -P "$JOBS" -n1 -I{} bash -c 'validate_one "$@"' _ {} | sort >"$V"

awk -F'\t' '{t++; if ($3==0) c++} END {printf "schema-valid: %d / %d\n", c, t}' "$V"
