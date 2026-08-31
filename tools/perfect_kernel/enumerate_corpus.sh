#!/usr/bin/env bash
# Enumerate the "perfect kernel" corpus: TeX Live documentation bundles whose
# manual is a compilable LaTeX source with a golden PDF next to it.
#
# A candidate is a `<dir>/<name>.tex` under $DOCROOT that
#   - contains \documentclass (i.e. is a standalone LaTeX document, not a
#     fragment or a plain-TeX/docstrip source), and
#   - has a sibling `<name>.pdf` (the golden rendering).
#
# Output: TSV lines  `bundle<TAB>texfile<TAB>pdffile<TAB>lines`  on stdout,
# sorted by bundle. Use `-c` to just print the count.
set -euo pipefail

DOCROOT="${DOCROOT:-$(kpsewhich -var-value=TEXMFDIST)/doc/latex}"
COUNT_ONLY=0
[[ "${1:-}" == "-c" ]] && COUNT_ONLY=1

emit() {
  local tex="$1"
  local pdf="${tex%.tex}.pdf"
  [[ -f "$pdf" ]] || return 0
  # Cheap standalone-document test: \documentclass in the first 2000 lines,
  # not commented out.
  if head -n 2000 "$tex" 2>/dev/null | grep -qm1 '^[^%]*\\documentclass'; then
    local bundle
    bundle=$(basename "$(dirname "$tex")")
    printf '%s\t%s\t%s\t%s\n' "$bundle" "$tex" "$pdf" "$(wc -l <"$tex")"
  fi
}

export -f emit
find "$DOCROOT" -maxdepth 2 -name '*.tex' -print0 |
  xargs -0 -n1 -P8 bash -c 'emit "$0"' |
  sort -t$'\t' -k1,1 -k2,2 |
  if [[ $COUNT_ONLY == 1 ]]; then wc -l; else cat; fi
