#!/usr/bin/env bash
# S3 spot-audit: content-completeness of a converted core XML against the
# golden PDF shipped beside the manual.
#
# Usage: s3_audit.sh <bundle>/<name> [outroot]
#
# Method (word-recall): extract text from BOTH artifacts (pdftotext /
# xmllint), normalize to lowercase word lists, and report what fraction of
# the PDF's DISTINCT words (len>=4, alphabetic) also occur in the XML.
# This is a RECALL screen, not equality: hyphenation, ligatures, math
# rendering and page furniture (headers, page numbers) legitimately differ.
# Guideline: >=90% recall = content-complete for S3 purposes; below that,
# inspect the missing-word sample this script prints.
set -uo pipefail

DOC="$1"
OUTROOT="${2:-$HOME/data/perfect_kernel}"
name=$(basename "$DOC")
xml="$OUTROOT/$DOC/$name.xml"
# The golden PDF sits in the source bundle dir.
DOCROOT="${DOCROOT:-$(kpsewhich -var-value=TEXMFDIST)/doc/latex}"
pdf="$DOCROOT/$DOC.pdf"
[[ -f "$xml" ]] || { echo "no XML: $xml" >&2; exit 1; }
[[ -f "$pdf" ]] || { echo "no PDF: $pdf" >&2; exit 1; }

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
pdftotext -q "$pdf" "$tmp/pdf.txt"
# Tag-strip with a SPACE per tag (a bare string(/) glues text across element
# boundaries — `Wolczko<break/>mario` read as "wolczkomario" and produced a
# false missing-word). Entities are then decoded by xmllint on the wrapped
# remainder.
sed 's/<[^>]*>/ /g' "$xml" > "$tmp/xml.txt" 2>/dev/null

words() {
  tr -cs '[:alpha:]' '\n' < "$1" | tr '[:upper:]' '[:lower:]' |
    awk 'length($0)>=4' | sort -u
}
words "$tmp/pdf.txt" > "$tmp/pdf.words"
words "$tmp/xml.txt" > "$tmp/xml.words"

total=$(wc -l < "$tmp/pdf.words")
missing=$(comm -23 "$tmp/pdf.words" "$tmp/xml.words" | wc -l)
found=$((total - missing))
pct=$(awk -v f="$found" -v t="$total" 'BEGIN{printf "%.1f", t? 100*f/t : 0}')
printf '%s\trecall=%s%%\t(%d/%d distinct pdf words; %d missing)\n' \
  "$DOC" "$pct" "$found" "$total" "$missing"
if [[ "$missing" -gt 0 ]]; then
  echo "  missing sample:" $(comm -23 "$tmp/pdf.words" "$tmp/xml.words" | head -15)
fi
