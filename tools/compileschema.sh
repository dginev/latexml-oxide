#!/bin/bash
# =====================================================================
# compileschema.sh — Rust-side port of LaTeXML/tools/compileschema
# =====================================================================
# Stage 1: rnc → rng via trang + URN-rewrite sed fix-up.
# Stage 2: rng → LaTeXML.model — deferred pending a `--dump-model` flag
#          on latexml_oxide; until then, regenerate LaTeXML.model with
#          the upstream Perl toolchain (see LaTeXML/tools/compileschema
#          lines 66-85).
# =====================================================================

set -eu

TOOLSDIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
LATEXMLDIR="$TOOLSDIR/.."
RESOURCEDIR="$LATEXMLDIR/resources"
RELAXNGDIR="$RESOURCEDIR/RelaxNG"
CATALOG="$RESOURCEDIR/LaTeXML.catalog"

if ! command -v trang >/dev/null 2>&1; then
  echo "error: trang not found on PATH (install via apt/brew/tlmgr)." >&2
  exit 1
fi

TRANG="trang -C $CATALOG"

TMP=$(mktemp -d "/tmp/LaTeXML.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

SCHEMA=LaTeXML

# trang resolves urn:x-LaTeXML:RelaxNG:... includes via the catalog.
export XML_CATALOG_FILES="$CATALOG"

# =====================================================================
# rnc → rng (the important part).
# =====================================================================
echo "Converting $SCHEMA.rnc to $SCHEMA.rng"
$TRANG "$RELAXNGDIR/$SCHEMA.rnc" "$TMP/$SCHEMA.rng"

# trang drops every converted rng module into the same output dir and
# strips the urn: prefix from include hrefs. Restore the urn form and
# move modules back into the schema tree.
for RNG in "$TMP/LaTeXML"*.rng; do
  [ -e "$RNG" ] || continue
  sed \
    -e 's|include href="LaTeXML-|include href="urn:x-LaTeXML:RelaxNG:LaTeXML-|' \
    -e 's|include href="svg|include href="urn:x-LaTeXML:RelaxNG:svg:svg|' \
    "$RNG" > "$RELAXNGDIR/$(basename "$RNG")"
done

for RNG in "$TMP/svg"*.rng; do
  [ -e "$RNG" ] || continue
  sed \
    -e 's|include href="LaTeXML-|include href="urn:x-LaTeXML:RelaxNG:LaTeXML-|' \
    -e 's|include href="svg|include href="urn:x-LaTeXML:RelaxNG:svg:svg|' \
    "$RNG" > "$RELAXNGDIR/svg/$(basename "$RNG")"
done

echo "==============================="
echo "rnc → rng conversion complete."
echo "Stage 2 (rng → .model) still requires the Perl toolchain — see"
echo "LaTeXML/tools/compileschema and SYNC_STATUS.md §Work Plan."
echo "==============================="
