#!/usr/bin/env bash
# Same-TL LaTeX oracle for the perfect-kernel corpus.
#
# Many TL doc manuals were authored against the CONTEMPORARY version of their
# package; the shipped .tex may no longer compile with today's TeX Live (e.g.
# a4wide.tex uses siunitx-v1 \newunit — 39 pdflatex errors on TL2025). Holding
# latexml-oxide to zero errors on such a document is meaningless, so every doc
# gets an oracle verdict from the real engines first.
#
# Usage: oracle.sh <corpus.tsv> [outroot]
# For each doc, compiles in a throwaway dir with TEXINPUTS pointing at the
# source dir. Engine: lualatex when the source names lualatex/fontspec/
# unicode-math in its head, else pdflatex; on a pdflatex FATAL (exit != 0 with
# no PDF), lualatex is tried as a fallback.
# Writes <outroot>/oracle_verdicts.tsv:  bundle name engine exit errors
#   errors = count of '^!' lines in the engine log (0 = clean oracle)
#   exit 124 = timeout; a doc is DOCUMENT-STALE when errors > 0 / no PDF.
# Resumable: docs already present in oracle_verdicts.tsv are skipped.
set -uo pipefail

CORPUS="$1"
OUTROOT="${2:-$HOME/data/perfect_kernel}"
JOBS="${JOBS:-12}"
export TIMEOUT_S="${TIMEOUT_S:-90}"
V="$OUTROOT/oracle_verdicts.tsv"
export V OUTROOT
touch "$V"

oracle_one() {
  local tex="$1"
  local name bundle srcdir tmp engine exit_code errors log
  name=$(basename "$tex" .tex)
  bundle=$(basename "$(dirname "$tex")")
  grep -qm1 "^$bundle	$name	" "$V" && return 0
  srcdir=$(dirname "$tex")
  tmp=$(mktemp -d)
  engine=pdflatex
  if head -50 "$tex" | grep -qim1 'lualatex\|fontspec\|unicode-math\|directlua'; then
    engine=lualatex
  fi
  run_engine() {
    ( cd "$tmp" &&
      TEXINPUTS="$srcdir:" timeout "$TIMEOUT_S" \
        "$1" -interaction=nonstopmode -halt-on-error "$tex" \
        >/dev/null 2>&1 )
  }
  run_engine "$engine"
  exit_code=$?
  if [[ $exit_code -ne 0 && ! -f "$tmp/$name.pdf" && $engine == pdflatex ]]; then
    engine=lualatex
    run_engine "$engine"
    exit_code=$?
  fi
  log="$tmp/$name.log"
  errors=0
  [[ -f "$log" ]] && errors=$(grep -c '^!' "$log" || true)
  printf '%s\t%s\t%s\t%s\t%s\n' "$bundle" "$name" "$engine" "$exit_code" "$errors" >>"$V"
  rm -rf "$tmp"
}
export -f oracle_one

cut -f2 "$CORPUS" | xargs -P "$JOBS" -n1 -I{} bash -c 'oracle_one "$@"' _ {}

sort -o "$V" "$V"
awk -F'\t' '{t++; if ($4==0 && $5==0) c++} END {printf "oracle clean: %d / %d\n", c, t}' "$V"
