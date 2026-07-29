#!/usr/bin/env bash
# pass2b.sh OUT.tsv < ids   — recheck source bib intent with EXTENDED signals
# (amsrefs \begin{biblist}/\bib{, aastex \reference, \begin{references},
#  harvmac \listrefs/\lref, amstex \Refs). Emits: id <TAB> sig2 (comma list, or "-").
set -u
OUT="$1"

p2b_batch() {
  local id d src yymm digits sig
  for id in "$@"; do
    if [[ "$id" =~ ^([0-9]{4})\.[0-9]+$ ]]; then yymm="${BASH_REMATCH[1]}"
    else digits="${id##*[!0-9]}"; yymm="${digits:0:4}"; fi
    d="/data/arxiv/$yymm/$id"; src="$d/$id.zip"
    sig="-"
    if [[ -f "$src" ]]; then
      sig=$(unzip -p "$src" '*.tex' '*.ltx' '*.TEX' 2>/dev/null | awk '
        /\\begin *\{ *biblist/    { a = 1 }
        /\\bib *\{/               { b = 1 }
        /\\reference[^s]/         { c = 1 }
        /\\begin *\{ *references *\}|\\references/ { d = 1 }
        /\\listrefs|\\lref\\/     { e = 1 }
        /\\Refs([^a-zA-Z]|$)/     { f = 1 }
        END { s = ""
          if (a) s = s "biblist,"
          if (b) s = s "bibcs,"
          if (c) s = s "reference,"
          if (d) s = s "referencesenv,"
          if (e) s = s "listrefs,"
          if (f) s = s "Refs,"
          if (s == "") s = "-,"
          printf "%s", substr(s, 1, length(s)-1) }')
      [[ -n "$sig" ]] || sig="-"
    fi
    printf '%s\t%s\n' "$id" "$sig"
  done
}
export -f p2b_batch

xargs -d '\n' -P "${SCAN_P:-64}" -n 40 bash -c 'p2b_batch "$@"' _ >> "$OUT"
