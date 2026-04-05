#!/bin/bash
# Generate both Rust and Perl HTML for all arxiv-examples, then screenshot
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
RUST_BIN="$ROOT_DIR/target/release/latexml_oxide"
OUTPUT_DIR="$ROOT_DIR/html/arxiv-examples"

find_main_tex() {
  local dir="$1"
  for candidate in ms.tex paper.tex main.tex _main.tex; do
    [ -f "$dir/$candidate" ] && echo "$candidate" && return
  done
  # Check for single .tex file
  local count=$(find "$dir" -maxdepth 1 -name "*.tex" | wc -l)
  if [ "$count" -eq 1 ]; then
    find "$dir" -maxdepth 1 -name "*.tex" -exec basename {} \;
    return
  fi
  # Check for .tex file matching directory name
  local id=$(basename "$dir")
  for f in "$dir"/*.tex; do
    local base=$(basename "$f" .tex)
    if [ "$base" = "$id" ] || [ "$base" = "${id//./_}" ]; then
      echo "$(basename "$f")"
      return
    fi
  done
  # Find .tex file containing \documentclass (the actual main file)
  local docclass=$(grep -l '\\documentclass' "$dir"/*.tex 2>/dev/null | head -1)
  if [ -n "$docclass" ]; then
    basename "$docclass"
    return
  fi
  # Fall back to first .tex
  find "$dir" -maxdepth 1 -name "*.tex" | head -1 | xargs basename 2>/dev/null
}

generate_paper() {
  local id="$1"
  local mode="$2"  # "rust" or "perl"
  local dir="$SCRIPT_DIR/$id"
  local outdir="$OUTPUT_DIR/$id"
  mkdir -p "$outdir"

  local main=$(find_main_tex "$dir")
  if [ -z "$main" ]; then
    echo "SKIP $id: no .tex file"
    return 1
  fi

  if [ "$mode" = "rust" ]; then
    if [ -f "$outdir/rust.html" ] && [ $(wc -c < "$outdir/rust.html") -gt 100 ]; then
      echo "CACHED $id rust ($(wc -c < "$outdir/rust.html")B)"
      return 0
    fi
    echo -n "RUST $id ($main)... "
    timeout 120 bash -c "cd '$dir' && '$RUST_BIN' --format=html5 --nobibtex \
      --nodefaultresources \
      --css=https://cdn.jsdelivr.net/gh/dginev/ar5iv-css@0.8.5/css/ar5iv.css \
      --css=https://cdn.jsdelivr.net/gh/dginev/ar5iv-css@0.8.5/css/ar5iv-fonts.css \
      --preload=ar5iv.sty \
      --dest='$outdir/rust.html' '$main'" 2>"$outdir/rust.log" && \
      echo "OK ($(wc -c < "$outdir/rust.html")B)" || echo "FAILED"
  elif [ "$mode" = "perl" ]; then
    if [ -f "$outdir/perl.html" ] && [ $(wc -c < "$outdir/perl.html") -gt 100 ]; then
      echo "CACHED $id perl ($(wc -c < "$outdir/perl.html")B)"
      return 0
    fi
    echo -n "PERL $id ($main)... "
    timeout 300 bash -c "cd '$dir' && latexmlc --format=html5 \
      --nodefaultresources \
      --css=https://cdn.jsdelivr.net/gh/dginev/ar5iv-css@0.8.5/css/ar5iv.css \
      --css=https://cdn.jsdelivr.net/gh/dginev/ar5iv-css@0.8.5/css/ar5iv-fonts.css \
      --path='$HOME/git/ar5iv-bindings/bindings/' \
      --preload=ar5iv.sty \
      --destination='$outdir/perl.html' '$main'" 2>"$outdir/perl.log" && \
      echo "OK ($(wc -c < "$outdir/perl.html")B)" || echo "FAILED"
  fi
}

# Generate for all papers
for dir in "$SCRIPT_DIR"/*/; do
  id=$(basename "$dir")
  [ "$id" = "output" ] && continue
  [ -d "$dir" ] || continue
  generate_paper "$id" "$1"
done
