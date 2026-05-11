#!/bin/bash
# parity_stats.sh — per-paper Rust vs Perl LaTeXML delta for a stage dir.
#
# Usage: parity_stats.sh <stage_dir>
#        parity_stats.sh ~/data/stage_01_failures/output/
#
# Workflow:
#   1. For each *.log under <stage_dir>: extract Rust error count + top
#      error category.
#   2. Locate the source zip in ~/data/arxmliv/<bucket>/<id>/<id>.zip
#   3. Run Perl LaTeXML on it (assumes `latexml` on PATH via local::lib);
#      extract Perl error count.
#   4. Output TSV:
#        arxiv_id  rust_err  perl_err  delta  verdict  top_category
#      sorted by ascending delta then ascending rust_err.
#
# Verdict:
#   BOTH-CLEAN     : rust_err == perl_err == 0
#   RUST-REGRESSION: (perl_err == 0 && rust_err > 0) OR delta > 1
#   SURPASS-PERL   : (rust_err == 0 && perl_err > 0) -- Rust fixed what Perl can't
#   RUST-CLEANER   : delta < -1                       -- Rust has fewer errors (both > 0)
#   SHARED-FAILURE : |delta| <= 1 AND both > 0        -- comparable failure counts
#
# Rows where verdict is RUST-REGRESSION come first — those are the
# actionable rows for the empirical-fix workflow.

set -euo pipefail

DIR="${1:-}"
if [[ -z "$DIR" || ! -d "$DIR" ]]; then
  echo "Usage: $0 <stage_dir>" >&2
  exit 1
fi

export PERL5LIB="${PERL5LIB:-$HOME/perl5/lib/perl5}"
LATEXML="${LATEXML:-$HOME/perl5/bin/latexml}"
if [[ ! -x "$LATEXML" ]]; then
  echo "Perl latexml not found at $LATEXML (set LATEXML env var to override)" >&2
  exit 1
fi

# Rust latexml-oxide targets the ar5iv configuration of Perl LaTeXML,
# not stock Perl. All parity runs preload ar5iv.sty from ar5iv-bindings
# so the comparison baseline matches what Rust emulates.
AR5IV_BINDINGS="${AR5IV_BINDINGS:-$HOME/git/ar5iv-bindings}"
LATEXML_PARITY_FLAGS=(--path="$AR5IV_BINDINGS" --preload=ar5iv.sty)

ARXMLIV="${ARXMLIV:-$HOME/data/arxmliv}"

# Collect data into a temp file, then sort.
TMP=$(mktemp)
trap "rm -f $TMP" EXIT

printf 'arxiv_id\trust_err\tperl_err\tdelta\tverdict\ttop_category\n'

count=0
for log in "$DIR"/*.log; do
  paper=$(basename "$log" .log)
  # Skip canvas-internal logs (canvas.stderr.log, etc.).
  [[ "$paper" == canvas* ]] && continue
  count=$((count + 1))

  # Rust error count + top category (non-cascade).
  CASCADE='\\@startsection|\\lx@begin@alignment|\\end\{equation\}|\\endgroup'
  rust_err=$(sed -E 's/\x1b\[[0-9]+m//g' "$log" | awk '/Error:[a-z_]+:|Fatal:[a-z_]+:/{c++} END{print c+0}')
  top=$(sed -E 's/\x1b\[[0-9]+m//g' "$log" \
        | grep -oE 'Error:[a-z_]+:[^ ]+' \
        | grep -vE "$CASCADE" \
        | sort | uniq -c | sort -rn | head -1 \
        | awk '{print $2}' || echo "-")
  [[ -z "$top" ]] && top="-"

  # Locate source zip.
  bucket=${paper:0:4}
  zip="$ARXMLIV/$bucket/$paper/$paper.zip"
  if [[ ! -f "$zip" ]]; then
    # Older buckets use non-numeric prefixes (astro-ph0001, etc.); search.
    zip=$(find "$ARXMLIV" -maxdepth 3 -name "$paper.zip" -print -quit 2>/dev/null)
  fi
  if [[ -z "$zip" || ! -f "$zip" ]]; then
    echo "$paper	$rust_err	-	-	NOZIP	$top" >> "$TMP"
    continue
  fi

  # Extract to temp dir, run Perl latexml.
  tdir=$(mktemp -d)
  unzip -o "$zip" -d "$tdir" > /dev/null 2>&1
  main_tex=$(find "$tdir" -maxdepth 1 -name '*.tex' | head -1)
  if [[ -z "$main_tex" ]]; then
    rm -rf "$tdir"
    echo "$paper	$rust_err	-	-	NOTEX	$top" >> "$TMP"
    continue
  fi

  # Perl LaTeXML refuses to use /dev/null as destination; give it a temp xml.
  perl_dest="$tdir/__parity_out.xml"
  perl_out=$(cd "$tdir" && timeout 90 "$LATEXML" "${LATEXML_PARITY_FLAGS[@]}" --destination="$perl_dest" "$(basename "$main_tex")" 2>&1 || true)
  rm -rf "$tdir"
  # Count both `Error:` and `Fatal:` events. User amendment: fatals matter.
  perl_err=$(echo "$perl_out" | awk '/Error:|Fatal:/{c++} END{print c+0}')
  rust_err=${rust_err:-0}
  perl_err=${perl_err:-0}

  delta=$((rust_err - perl_err))
  if (( rust_err == 0 && perl_err == 0 )); then
    verdict="BOTH-CLEAN"
  elif (( perl_err == 0 && rust_err > 0 )); then
    # Perl is clean, Rust has errors -> definite Rust regression regardless of count.
    verdict="RUST-REGRESSION"
  elif (( rust_err == 0 && perl_err > 0 )); then
    # Rust is clean, Perl has errors -> Rust fixed something Perl can't.
    verdict="SURPASS-PERL"
  elif (( delta > 1 )); then
    verdict="RUST-REGRESSION"
  elif (( delta < -1 )); then
    verdict="RUST-CLEANER"
  else
    verdict="SHARED-FAILURE"
  fi

  printf '%s\t%d\t%d\t%d\t%s\t%s\n' "$paper" "$rust_err" "$perl_err" "$delta" "$verdict" "$top" >> "$TMP"
done

# Sort priority: RUST-REGRESSION first (smallest err = simplest repro),
# then SHARED ascending err, then SURPASS-PERL, RUST-CLEANER, BOTH-CLEAN.
awk -F'\t' '
$5 == "RUST-REGRESSION" { print 0 "\t" $2 "\t" $0; next }
$5 == "SHARED-FAILURE"  { print 5 "\t" $2 "\t" $0; next }
$5 == "SURPASS-PERL"    { print 7 "\t" $3 "\t" $0; next }
$5 == "RUST-CLEANER"    { print 8 "\t" $4 "\t" $0; next }
$5 == "BOTH-CLEAN"      { print 9 "\t0\t" $0; next }
                          { print 5 "\t" $2 "\t" $0 }
' "$TMP" | sort -t$'\t' -k1n -k2n | cut -f3-
echo "" >&2
echo "Processed $count papers." >&2
