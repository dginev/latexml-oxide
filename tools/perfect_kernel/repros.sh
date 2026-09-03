#!/usr/bin/env bash
# Run one topic of the perfect-kernel repro corpus (tools/perfect_kernel/repros/<topic>)
# and print an errors table per repro.
#
#   tools/perfect_kernel/repros.sh <topic> [--bin PATH] [--perl] [--pdflatex] [--out DIR]
#
# For every <topic>/*.tex: converts with the Rust binary (default
# ~/data/pk_target2/debug/latexml_oxide; the `% preload:` header overrides the
# default raw-load preload), counts ANSI-stripped `Error:`/`Fatal:` lines, and
# prints `status expect rust [perl] [pdflatex] name -- first error`. Optional
# same-host Perl (`latexml`, same preload) and pdflatex (`grep -c '^!'` on the
# log) columns give the SHARED / oracle verdicts. Exit status is 1 when a
# repro marked `% status: GREEN` errors (regression) or one marked `CONTROL`
# converts clean (the boundary moved) — RED repros are informational.
set -uo pipefail

usage() { sed -n '2,15p' "$0"; exit 0; }
[[ $# -ge 1 ]] || usage
[[ $1 == --help || $1 == -h ]] && usage

topic=$1; shift
BIN="$HOME/data/pk_target2/debug/latexml_oxide"
PERL=0; PDF=0
OUT=""
while [[ $# -gt 0 ]]; do
  case $1 in
    --bin) BIN=$2; shift 2 ;;
    --perl) PERL=1; shift ;;
    --pdflatex) PDF=1; shift ;;
    --out) OUT=$2; shift 2 ;;
    *) echo "unknown option $1" >&2; exit 2 ;;
  esac
done

here=$(cd "$(dirname "$0")" && pwd)
dir="$here/repros/$topic"
[[ -d "$dir" ]] || { echo "no such topic: $dir" >&2; exit 2; }
[[ -x "$BIN" ]] || { echo "no binary: $BIN" >&2; exit 2; }
OUT=${OUT:-${TMPDIR:-/tmp}/pk_repros_$topic}
mkdir -p "$OUT"

strip() { sed 's/\x1b\[[0-9;]*m//g'; }
errs() { strip <"$1" | grep -cE '^(Error|Fatal):' ; }
first() { strip <"$1" | grep -E '^(Error|Fatal):' | head -1 | cut -c1-90; }

rc=0
printf '%-8s %-8s %5s' status expect rust
[[ $PERL == 1 ]] && printf ' %5s' perl
[[ $PDF == 1 ]] && printf ' %5s' pdftex
printf '  %s\n' name
for tex in "$dir"/*.tex; do
  [[ -e "$tex" ]] || continue
  name=$(basename "$tex" .tex)
  status=$(grep -m1 -oP '^% status:\s*\K\w+' "$tex" || echo '?')
  expect=$(grep -m1 -oP '^% expect:\s*\K[0-9]+' "$tex" || echo 0)
  preload=$(grep -m1 -oP '^% preload:\s*\K\S+' "$tex" || echo '[rawstyles,rawclasses]latexml.sty')
  ( cd "$dir" && timeout 120 "$BIN" --nocomments --timeout=100 --preload="$preload" \
      --dest="$OUT/$name.xml" "$name.tex" >"$OUT/$name.stderr" 2>&1 )
  rust=$(errs "$OUT/$name.stderr")
  line=$(printf '%-8s %-8s %5s' "$status" "$expect" "$rust")
  if [[ $PERL == 1 ]]; then
    ( cd "$OUT" && cp "$tex" . && timeout 120 latexml --nocomments --preload="$preload" \
        --dest="$OUT/$name.perl.xml" "$name.tex" >"$OUT/$name.perl.stderr" 2>&1 )
    if grep -q 'Conversion complete' "$OUT/$name.perl.stderr"; then
      perl=$(errs "$OUT/$name.perl.stderr")
    else
      perl='n/a'
    fi
    line+=$(printf ' %5s' "$perl")
  fi
  if [[ $PDF == 1 ]]; then
    ( cd "$OUT" && cp "$tex" . && timeout 120 pdflatex -interaction=batchmode "$name.tex" >/dev/null 2>&1 )
    pdf=$(grep -c '^!' "$OUT/$name.log" 2>/dev/null || echo '?')
    line+=$(printf ' %5s' "$pdf")
  fi
  printf '%s  %s' "$line" "$name"
  [[ $rust -gt 0 ]] && printf ' -- %s' "$(first "$OUT/$name.stderr")"
  printf '\n'
  case $status in
    GREEN) [[ $rust -gt $expect ]] && rc=1 ;;
    CONTROL) [[ $rust -eq 0 ]] && rc=1 ;;
  esac
done
exit $rc
