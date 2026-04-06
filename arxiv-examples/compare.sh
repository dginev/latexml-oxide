#!/bin/bash
# Compare latexml-oxide vs latexmlc output for arxiv papers
# Usage: ./compare.sh [paper_id]  — compares one paper
#        ./compare.sh              — compares all papers

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
RUST_BIN="$ROOT_DIR/target/release/latexml_oxide"
STYLESHEET="$ROOT_DIR/resources/XSLT/LaTeXML-html5.xsl"
OUTPUT_DIR="$ROOT_DIR/html/arxiv-examples"

# Build Rust binary if needed
if [ ! -f "$RUST_BIN" ]; then
  echo "Building latexml_oxide (release)..."
  cargo build --release --bin latexml_oxide --manifest-path "$ROOT_DIR/Cargo.toml"
fi

compare_paper() {
  local id="$1"
  local dir="$SCRIPT_DIR/$id"
  local outdir="$OUTPUT_DIR/$id"
  mkdir -p "$outdir"

  # Find main tex file
  local main=""
  for candidate in ms.tex paper.tex main.tex; do
    [ -f "$dir/$candidate" ] && main="$candidate" && break
  done
  if [ -z "$main" ]; then
    main=$(find "$dir" -maxdepth 1 -name "*.tex" | head -1 | xargs basename 2>/dev/null)
  fi
  if [ -z "$main" ]; then
    echo "SKIP $id: no .tex file found"
    return
  fi

  echo "=== $id ($main) ==="

  # Run Rust converter
  echo -n "  Rust: "
  local rust_start=$SECONDS
  (cd "$dir" && "$RUST_BIN" --format=html5 --nobibtex \
    --stylesheet="$STYLESHEET" \
    --dest="$outdir/rust.html" "$main" 2>"$outdir/rust.log") && \
    echo "$((SECONDS - rust_start))s" || echo "FAILED (see $outdir/rust.log)"

  # Run Perl converter
  echo -n "  Perl: "
  local perl_start=$SECONDS
  (cd "$dir" && latexmlc --format=html5 \
    --destination="$outdir/perl.html" "$main" 2>"$outdir/perl.log") && \
    echo "$((SECONDS - perl_start))s" || echo "FAILED (see $outdir/perl.log)"

  # Compare (strip known-intentional divergences for meaningful diff)
  if [ -f "$outdir/rust.html" ] && [ -f "$outdir/perl.html" ]; then
    local rust_size=$(wc -c < "$outdir/rust.html")
    local perl_size=$(wc -c < "$outdir/perl.html")
    # Raw diff
    local raw_diff=$(diff "$outdir/rust.html" "$outdir/perl.html" 2>/dev/null | wc -l)
    # Filtered diff (strip xml:id, tex= on picture, %&#10;, Generated timestamps)
    local filt_diff=$(diff \
      <(sed 's/xml:id="[^"]*"//g; s/ tex="[^"]*"//g; s/%&#10;//g; /Generated on/d' "$outdir/perl.html") \
      <(sed 's/xml:id="[^"]*"//g; s/ tex="[^"]*"//g; s/%&#10;//g; /Generated on/d' "$outdir/rust.html") \
      2>/dev/null | wc -l)
    echo "  Sizes: Rust ${rust_size}B, Perl ${perl_size}B"
    echo "  Raw diff: $raw_diff, Filtered: $filt_diff"
  fi
}

if [ -n "$1" ]; then
  compare_paper "$1"
else
  for dir in "$SCRIPT_DIR"/*/; do
    id=$(basename "$dir")
    [ "$id" = "output" ] && continue
    compare_paper "$id"
  done
fi
