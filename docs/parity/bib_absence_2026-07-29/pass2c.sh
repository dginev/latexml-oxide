#!/usr/bin/env bash
# pass2c.sh OUT.tsv < ids — TIGHT re-check of the legacy bibliography signals.
#
# pass2b matched `\references` inside longer macro names: 0704.0420 defines
# `\def\referencesz{...}` (a hand-rolled heading) and was counted as an aastex
# `references` environment. Every signal here requires a real control-sequence
# boundary, and the amsrefs `\bib` test requires its 3-argument shape.
set -u
OUTFILE="$1"

p2c_batch() {
  local id yymm digits src sig
  for id in "$@"; do
    if [[ "$id" =~ ^([0-9]{4})\.[0-9]+$ ]]; then yymm="${BASH_REMATCH[1]}"
    else digits="${id##*[!0-9]}"; yymm="${digits:0:4}"; fi
    src="/data/arxiv/$yymm/$id/$id.zip"
    [[ -f "$src" ]] || { printf '%s\t-\n' "$id"; continue; }
    sig=$(unzip -p "$src" '*.tex' '*.ltx' '*.TEX' 2>/dev/null | awk '
      /\\begin *\{ *biblist *\}/                { a = 1 }
      /\\bib\{[^}]*\}\{[^}]*\}\{/               { b = 1 }
      /\\begin *\{ *references *\}/             { d = 1 }
      /\\reference[^a-zA-Z]/                    { c = 1 }
      /\\listrefs([^a-zA-Z]|$)/                 { e = 1 }
      /\\Refs([^a-zA-Z]|$)/                     { f = 1 }
      END { s = ""
        if (a) s = s "biblist,"
        if (b) s = s "bibcs,"
        if (c) s = s "reference,"
        if (d) s = s "referencesenv,"
        if (e) s = s "listrefs,"
        if (f) s = s "Refs,"
        printf "%s", (s == "" ? "-" : substr(s, 1, length(s)-1)) }')
    printf '%s\t%s\n' "$id" "${sig:--}"
  done
}
export -f p2c_batch

xargs -d '\n' -P "${SCAN_P:-64}" -n 40 bash -c 'p2c_batch "$@"' _ >> "$OUTFILE"
