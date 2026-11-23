#!/usr/bin/env bash
# Render-fleet platform check (SYNC_STATUS row R, RENDERPLATFORMS).
#
# Converts a generated 80-section split document twice with the given
# latexml_oxide binary — once with the automatic page-render fleet (its default
# for a split of 64+ pages) and once serial (LATEXML_RENDER_JOBS=1) — then
# converts the fleet run again into the same destination, so `writer::Staged`
# renames land on existing files. Fails unless every run exits 0 with no
# Error/Fatal, the fleet engaged in the parallel runs and not in the serial one,
# no run abandoned the fleet, the three sites are byte-identical, and no spill
# or handoff file is left behind.
#
# Needs no TeX Live: article + amsmath are compiled bindings.
# Usage: tools/render_platform_check.sh <latexml_oxide binary> <work dir>
set -euo pipefail

bin=$1
work=$2
mkdir -p "$work"
cd "$work"

{
  printf '%s\n' '\documentclass{article}' '\usepackage{amsmath}' '\begin{document}' \
    '\title{Render fleet check}' '\author{latexml-oxide}' '\maketitle' '\tableofcontents'
  for i in $(seq 1 80); do
    prev=$((i > 1 ? i - 1 : 1))
    printf '\\section{Section %d}\\label{s%d}\n' "$i" "$i"
    printf 'Text of section %d; see Section~\\ref{s%d}, Equation~\\eqref{e%d} and~\\cite{a}.\\footnote{Note %d.}\n' \
      "$i" "$prev" "$i" "$i"
    printf '\\begin{equation}\\label{e%d} x_{%d} = \\sum_{k=1}^{%d} \\frac{k^2}{k+1} \\end{equation}\n' "$i" "$i" "$i"
    printf '\\begin{itemize}\\item one %d \\item two %d\\end{itemize}\n' "$i" "$i"
  done
  printf '%s\n' '\begin{thebibliography}{9}' '\bibitem{a} A. Author, \emph{A Title}, 2026.' \
    '\end{thebibliography}' '\end{document}'
} > split.tex

fail=0
convert() { # <label> <dest dir> [env assignment]
  local label=$1 dest=$2
  shift 2
  local rc=0
  env "$@" "$bin" --splitat=section --format=html5 --dest="$dest/index.html" --log="$label.log" split.tex \
    > "$label.stdout" 2> "$label.stderr" || rc=$?
  # Strip ANSI into a file once and grep the FILE: `producer | grep -q` under
  # pipefail fails whenever grep's early exit SIGPIPEs the producer (a
  # false "did not engage" on macOS arm64, run 45119817997).
  sed 's/\x1b\[[0-9;]*m//g' "$label.log" > "$label.plain"
  local errors fatals
  errors=$(grep -c '^Error:' "$label.plain" || true)
  fatals=$(grep -c '^Fatal:' "$label.plain" || true)
  echo "$label: exit $rc, $errors error(s), $fatals fatal(s), $(find "$dest" -name '*.html' | wc -l | tr -d ' ') page(s)"
  if [ "$rc" -ne 0 ] || [ "$errors" -ne 0 ] || [ "$fatals" -ne 0 ]; then
    echo "::error::$label: the conversion failed"
    tail -20 "$label.stderr"
    fail=1
  fi
  if grep -q 'parallel render abandoned' "$label.plain"; then
    echo "::error::$label: $(grep -m1 'parallel render abandoned' "$label.plain")"
    fail=1
  fi
  if grep -q 'parallel page render engaged' "$label.plain"; then
    echo "$label: $(grep -m1 'parallel page render engaged' "$label.plain")"
  fi
}
engaged() { grep -q 'parallel page render engaged' "$1.plain"; }

convert fleet site-fleet
convert serial site-serial LATEXML_RENDER_JOBS=1
convert refleet site-fleet

engaged fleet || { echo "::error::fleet: the render fleet did not engage"; fail=1; }
engaged refleet || { echo "::error::refleet: the render fleet did not engage"; fail=1; }
if engaged serial; then echo "::error::serial: LATEXML_RENDER_JOBS=1 still engaged the fleet"; fail=1; fi

if diff -r site-fleet site-serial > site.diff; then
  echo "sites: byte-identical ($(find site-fleet -type f | wc -l | tr -d ' ') files)"
else
  echo "::error::the fleet and serial sites differ"
  head -40 site.diff
  fail=1
fi

find . -name '.latexml-spill*' -o -name '*latexml-post-handoff*' > leftovers.txt
if [ -s leftovers.txt ]; then
  echo "::error::spill or handoff files left behind: $(head -5 leftovers.txt | tr '\n' ' ')"
  fail=1
fi

exit "$fail"
