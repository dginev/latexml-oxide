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
# S3_EXT=html audits the post-processed page (post_sweep.sh output root).
S3_EXT="${S3_EXT:-xml}"
xml="$OUTROOT/$DOC/$name.$S3_EXT"
# The golden PDF sits in the source bundle dir.
# The golden PDFs live in the SWEEP's TeX tree: honour TL_ROOT (as run_doc.sh
# does — the distro kpsewhich on PATH has only 32 doc PDFs, so every audit
# read "no PDF") before falling back to the ambient tree.
if [[ -z "${DOCROOT:-}" && -n "${TL_ROOT:-}" && -d "$TL_ROOT/texmf-dist/doc/latex" ]]; then
  DOCROOT="$TL_ROOT/texmf-dist/doc/latex"
fi
DOCROOT="${DOCROOT:-$(kpsewhich -var-value=TEXMFDIST)/doc/latex}"
pdf="$DOCROOT/$DOC.pdf"
[[ -f "$xml" ]] || { echo "no XML: $xml" >&2; exit 1; }
[[ -f "$pdf" ]] || { echo "no PDF: $pdf" >&2; exit 1; }

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
pdftotext -q "$pdf" "$tmp/pdf.txt"
# Tag-strip: a block or layout element is a SPACE (a bare string(/) glues text
# across element boundaries — `Wolczko<break/>mario` read as "wolczkomario" and
# produced a false missing word).
# MathML annotations carry the TeX source; drop them so HTML recall counts
# rendered text only (they could only inflate "found").
# An inline text element (`span`, `em`, `a`, `sup`, the core XML's `text`/`emph`,
# …) glues its text to its neighbours, as a browser renders it: `\LaTeX` is
# `L<span>a</span>T<span>e</span>X`, and a space per tag made "latex" a missing
# word in 335 manuals (stream A, sweep #124). Spans whose LaTeXML.css class is
# a block or inline-block (tabular cells `ltx_td`, inline-blocks, minipages,
# paragraphs, lists, notes, pubnotes, tags, listings, framed boxes) and anything
# with a `display:` style still separate, as does every other element (SVG
# `text` too: the core XML's `text`/`emph` glue only with S3_EXT=xml); a closing
# tag does what its opening tag did, and an HTML void element (`br`, `img`, …)
# opens nothing. A tag's quoted attributes may hold a raw `<`/`>` (libxml2's
# `href="mailto:<a@b>"`). Text under `visibility:hidden` or
# `display:none` is not rendered and is dropped (`\phantom{pf}`'s content
# would otherwise glue to the next word).
XMLMODE=$([[ $S3_EXT == xml ]] && echo 1 || echo 0) perl -0777 -ne '
  my $xmlmode = $ENV{XMLMODE};
  s{<(m:)?annotation\b.*?</(m:)?annotation>}{ }gs;
  s{<!--.*?-->}{ }gs;
  my (@stack, $out);
  my $hidden = 0;
  for my $piece (split /(<(?:[^>"]|"[^"]*")*>)/) {
    if ($piece =~ m{^<(/?)([A-Za-z][\w:.-]*)((?:[^>"]|"[^"]*")*?)(/?)>$}s) {
      my ($close, $name, $attrs, $empty) = ($1, lc $2, $3, $4);
      my ($g, $h) = (0, 0);
      if ($close) {
        ($g, $h) = @{ pop(@stack) // [0, 0] };
        $hidden -= $h;
      } else {
        $name =~ s/^\w+://;
        $empty ||= $name =~ /^(?:area|base|br|col|embed|hr|img|input|link|meta|param|source|track|wbr)$/;
        $g = (($name =~ /^(?:span|a|em|strong|b|i|u|s|sub|sup|small|big|code|tt|abbr|cite|q|del|ins|mark|kbd|samp|var|font)$/
          || ($xmlmode && $name =~ /^(?:text|emph)$/))
          && $attrs !~ /class="[^"]*\bltx_(?:td|th|tr|tabular|tbody|thead|tfoot|inline-block|inline-logical-block|logical-block|minipage|parbox|p|para|block|item|itemize|enumerate|description|quote|centering|tag|listing|listingline|bibblock|note|pubnotes|pubnote|author_notes|contact|transformed_inner|transformed_outer|framed|ERROR)\b/
          && $attrs !~ /display\s*:/) ? 1 : 0;
        $h = ($attrs =~ /visibility\s*:\s*hidden|display\s*:\s*none/) ? 1 : 0;
        unless ($empty) { push @stack, [$g, $h]; $hidden += $h; }
      }
      $out .= $g ? "" : " ";
    } elsif ($piece =~ /^</) {
      $out .= " ";
    } elsif (!$hidden) {
      $out .= $piece;
    }
  }
  print $out;
' "$xml" > "$tmp/xml.txt" 2>/dev/null

# Word extraction is Unicode-aware on BOTH sides (the byte-wise `tr -cs
# '[:alpha:]'` split every non-ASCII letter: "Schriftgröße" → "schriftgr",
# "e" — a false missing word per umlaut, the artifact that dominated the
# 90–95 % band of the s105 reading): NFKC folds ligatures (ﬁ → fi) and
# compatibility forms, the pdftotext line-break hyphen ("in-\nput") is
# rejoined, then lowercase letter runs of length ≥ 4 in any script.
words() {
  perl -CSD -MUnicode::Normalize -0777 -ne '
    $_ = NFKC($_);
    s/(\p{L})-\n(\p{L})/$1$2/g;
    print lc($_) =~ s/[^\p{L}]+/\n/gr;
  ' "$1" | awk 'length($0)>=4' | LC_ALL=C sort -u
}
words "$tmp/pdf.txt" > "$tmp/pdf.words"
words "$tmp/xml.txt" > "$tmp/xml.words"

total=$(wc -l < "$tmp/pdf.words")
missing=$(LC_ALL=C comm -23 "$tmp/pdf.words" "$tmp/xml.words" | wc -l)
found=$((total - missing))
pct=$(awk -v f="$found" -v t="$total" 'BEGIN{printf "%.1f", t? 100*f/t : 0}')
printf '%s\trecall=%s%%\t(%d/%d distinct pdf words; %d missing)\n' \
  "$DOC" "$pct" "$found" "$total" "$missing"
if [[ "$missing" -gt 0 ]]; then
  echo "  missing sample:" $(LC_ALL=C comm -23 "$tmp/pdf.words" "$tmp/xml.words" | head -15)
fi
