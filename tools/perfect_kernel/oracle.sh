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
# unicode-math in its head, else pdflatex. When that run is not clean, the golden
# PDF's Producer engine is tried next (pdfTeX → pdflatex, XeTeX/xdvipdfmx →
# xelatex, LuaTeX/LuaHBTeX → lualatex), then the remaining engines, and the first
# clean one is recorded. xelatex was never tried before, so run_doc.sh's xetex
# persona never fired (the 2026-09-25 refresh: 54 docs newly clean, 24 of them
# under xelatex). A clean first run is kept as before. Producer-first for such runs
# would change the persona of ~750 manuals whose golden came from another engine;
# that is a measured experiment of its own (roadmap stream A, char-list).
# When no engine is clean, a pdflatex run without a PDF records lualatex, as before
# batch 56is (run_doc.sh's luatex retry reads the column as the required engine).
# REFRESH=<file of tex paths>: drop those docs' rows and re-run only them.
# Writes <outroot>/oracle_verdicts.tsv:  bundle name engine exit errors
#   errors = count of '^!' lines in the engine log (0 = clean oracle)
#   exit 124 = timeout; a doc is DOCUMENT-STALE when errors > 0 / no PDF.
# Resumable: docs already present in oracle_verdicts.tsv are skipped.
set -uo pipefail

CORPUS="$1"
OUTROOT="${2:-$HOME/data/perfect_kernel}"
JOBS="${JOBS:-8}"  # docs/THERMALS.md: 8 alone, 4 beside anything else
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
  tmp=$(mktemp -d) || return 1
  engine=pdflatex
  if head -50 "$tex" | grep -qim1 'lualatex\|fontspec\|unicode-math\|directlua'; then
    engine=lualatex
  fi
  local golden="${tex%.tex}.pdf" producer="" produced=""
  [[ -f "$golden" ]] && producer=$(pdfinfo "$golden" 2>/dev/null | sed -n 's/^Producer: *//p')
  case "$producer" in
    *pdfTeX*|*pdfeTeX*) produced=pdflatex ;;
    *XeTeX*|*xdvipdfmx*) produced=xelatex ;;
    *LuaTeX*|*LuaHBTeX*) produced=lualatex ;;
  esac
  # The Producer engine goes first in the fallback order, without repeating it.
  local order="$produced" e
  for e in pdflatex lualatex xelatex; do
    [[ $e == "$produced" ]] || order="$order $e"
  done
  run_engine() {
    rm -rf -- "${tmp:?}"/*
    ( cd "$tmp" &&
      TEXINPUTS="$srcdir:" timeout "$TIMEOUT_S" \
        "$1" -interaction=nonstopmode -halt-on-error "$tex" \
        >/dev/null 2>&1 )
    exit_code=$?
    errors=0
    [[ -f "$tmp/$name.log" ]] && errors=$(grep -c '^!' "$tmp/$name.log" || true)
    has_pdf=0
    [[ -f "$tmp/$name.pdf" ]] && has_pdf=1
  }
  run_engine "$engine"
  local first="$engine" first_exit=$exit_code first_errors=$errors first_pdf=$has_pdf
  local lua_exit="" lua_errors=""
  [[ $first == lualatex ]] && lua_exit=$exit_code lua_errors=$errors
  if [[ $exit_code -ne 0 || $errors -ne 0 ]]; then
    for e in $order; do
      [[ $e == "$first" ]] && continue
      run_engine "$e"
      [[ $e == lualatex ]] && lua_exit=$exit_code lua_errors=$errors
      if [[ $exit_code -eq 0 && $errors -eq 0 ]]; then
        engine=$e
        break
      fi
    done
    if [[ $exit_code -ne 0 || $errors -ne 0 ]]; then
      # No engine is clean. The engine column still names the engine the
      # document needs, which run_doc.sh reads for its luatex retry: as before
      # batch 56is, a pdflatex run that makes no PDF records lualatex (with its
      # own result), otherwise the first engine. Recording pdflatex there (56is)
      # dropped the retry for 76 documents (sweep #124: +568 errors).
      if [[ $first == pdflatex && $first_pdf -eq 0 && -n $lua_exit ]]; then
        engine=lualatex exit_code=$lua_exit errors=$lua_errors
      else
        engine=$first exit_code=$first_exit errors=$first_errors
      fi
    fi
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$bundle" "$name" "$engine" "$exit_code" "$errors" >>"$V"
  rm -rf "$tmp"
}
export -f oracle_one

if [[ -n "${REFRESH:-}" ]]; then
  # Drop the listed docs' rows, then run only those. An empty or missing list
  # would drop nothing and run nothing; refuse it rather than rewrite $V.
  [[ -s "$REFRESH" ]] || { echo "REFRESH list '$REFRESH' is empty or missing" >&2; exit 1; }
  awk -F'\t' 'NR==FNR { n=split($0,p,"/"); b=p[n-1]; d=p[n]; sub(/\.tex$/,"",d); drop[b"\t"d]=1; next }
               !(($1"\t"$2) in drop)' "$REFRESH" "$V" >"$V.tmp" && mv "$V.tmp" "$V"
  xargs -P "$JOBS" -n1 -I{} bash -c 'oracle_one "$@"' _ {} <"$REFRESH"
else
  cut -f2 "$CORPUS" | xargs -P "$JOBS" -n1 -I{} bash -c 'oracle_one "$@"' _ {}
fi

sort -o "$V" "$V"
awk -F'\t' '{t++; if ($4==0 && $5==0) c++} END {printf "oracle clean: %d / %d\n", c, t}' "$V"
