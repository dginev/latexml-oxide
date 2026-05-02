#!/bin/bash
# =====================================================================
# compileschema.sh — Rust-side port of LaTeXML/tools/compileschema
# =====================================================================
# Stage 1: rnc → rng via trang + URN-rewrite sed fix-up.
# Stage 2: rng → LaTeXML.model via `latexml_oxide --dump-model`. Output is
#          byte-identical to Perl `bin/compileschema --schema=LaTeXML`
#          (mirrors LaTeXML/tools/compileschema lines 66-85).
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
echo "==============================="

# =====================================================================
# rng → .model — invokes the Rust binary's --dump-model flag, which
# loads the embedded LaTeXML schema (the same one runtime sees) and
# serialises it via Model::dump_compiled_schema(). Mirrors the Perl
# compileSchema print loop (Model.pm L121-136). Output is byte-for-byte
# identical to Perl's `bin/compileschema --schema=LaTeXML > LaTeXML.model`.
# =====================================================================
LATEXML_BIN=""
for candidate in "release" "debug"; do
  if [ -x "$LATEXMLDIR/target/$candidate/latexml_oxide" ]; then
    LATEXML_BIN="$LATEXMLDIR/target/$candidate/latexml_oxide"
    break
  fi
done
if [ -z "$LATEXML_BIN" ]; then
  echo "warning: latexml_oxide not built — skipping stage 2." >&2
  echo "  Build with: cargo build --bin latexml_oxide        # local debug" >&2
  echo "  (or:        cargo build --release --bin latexml_oxide  # publish-grade)" >&2
  exit 0
fi
MODEL_OUT="$RELAXNGDIR/$SCHEMA.model"
echo "Generating $SCHEMA.model via --dump-model"
"$LATEXML_BIN" --dump-model > "$MODEL_OUT"
echo "$MODEL_OUT updated."
