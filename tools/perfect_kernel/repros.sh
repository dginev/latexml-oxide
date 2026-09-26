#!/usr/bin/env bash
# Run one topic of the perfect-kernel repro corpus (tools/perfect_kernel/repros/<topic>)
# and print an errors table per repro.
#
#   tools/perfect_kernel/repros.sh <topic> [--bin PATH] [--perl] [--pdflatex] [--recall] [--out DIR]
#
# For every <topic>/*.tex: converts with the Rust binary (default
# ~/data/pk_target2/debug/latexml_oxide; the `% preload:` header overrides the
# default raw-load preload, `none` for no preload), counts ANSI-stripped `Error:`/`Fatal:` lines, and
# prints `status expect rust [perl] [pdflatex] name -- first error`. Optional
# same-host Perl (`latexml`, same preload) and pdflatex (`grep -c '^!'` on the
# log) columns give the SHARED / oracle verdicts. `--recall` compiles each repro
# with its INTENDED engine (`% engine:` header; else lualatex when the preload has
# `luatex`, else pdflatex), and reports the PDF-to-XML content recall
# (`pdf_recall.py`: every word that reaches the PDF must reach the XML) with the
# missing words. Exit status is 1 when a
# repro marked `% status: GREEN` errors (regression) or one marked `CONTROL`
# converts clean (the boundary moved) — RED repros are informational.
set -uo pipefail

usage() { sed -n '2,15p' "$0"; exit 0; }
[[ $# -ge 1 ]] || usage
[[ $1 == --help || $1 == -h ]] && usage

topic=$1; shift
BIN="$HOME/data/pk_target2/debug/latexml_oxide"
PERL=0; PDF=0; RECALL=0
OUT=""
while [[ $# -gt 0 ]]; do
  case $1 in
    --bin) BIN=$2; shift 2 ;;
    --perl) PERL=1; shift ;;
    --pdflatex) PDF=1; shift ;;
    --recall) RECALL=1; shift ;;
    --out) OUT=$2; shift 2 ;;
    *) echo "unknown option $1" >&2; exit 2 ;;
  esac
done

here=$(cd "$(dirname "$0")" && pwd)
dir="$here/repros/$topic"
[[ -d "$dir" ]] || { echo "no such topic: $dir" >&2; exit 2; }
[[ -x "$BIN" ]] || { echo "no binary: $BIN" >&2; exit 2; }
# The conversion runs from the topic directory: a relative --bin must not break it
# (it did — every repro "converted" with 0 errors and no XML, and read GREEN).
BIN=$(readlink -f "$BIN")
OUT=${OUT:-${TMPDIR:-/tmp}/pk_repros_$topic}
mkdir -p "$OUT"

strip() { sed 's/\x1b\[[0-9;]*m//g'; }
errs() { strip <"$1" | grep -cE '(Error|Fatal):[A-Za-z_]+:' ; }
first() { strip <"$1" | grep -E '(Error|Fatal):[A-Za-z_]+:' | head -1 | cut -c1-90; }

rc=0
printf '%-8s %-8s %5s' status expect rust
[[ $PERL == 1 ]] && printf ' %5s' perl
[[ $PDF == 1 ]] && printf ' %5s' pdftex
[[ $RECALL == 1 ]] && printf ' %7s' recall
printf '  %s\n' name
for tex in "$dir"/*.tex; do
  [[ -e "$tex" ]] || continue
  name=$(basename "$tex" .tex)
  # The status is the line's LAST state: "RED (56jh) -> GREEN (56jk)" is GREEN
  # (a first-word read made such repros RED and skipped their regression check).
  statline=$(grep -m1 -oP '^% status:\s*\K.*' "$tex" || echo '?')
  status=$(grep -oP '(?:^|->\s*)\K[A-Z][A-Z-]*' <<<"$statline" | tail -1)
  [[ -n $status ]] || status=$(grep -oP '^\w+' <<<"$statline" || echo '?')
  expect=$(grep -m1 -oP '^% expect:\s*\K[0-9]+' "$tex" || echo 0)
  preload=$(grep -m1 -oP '^% preload:\s*\K\S+' "$tex" || echo '[rawstyles,rawclasses]latexml.sty')
  # `% preload: none` converts with no preload, as a guard's `convert(tex, false)` does.
  preload_args=(--preload="$preload")
  [[ $preload == none ]] && preload_args=()
  ( cd "$dir" && timeout 120 "$BIN" --nocomments --timeout=100 "${preload_args[@]}" \
      --dest="$OUT/$name.xml" "$name.tex" >"$OUT/$name.stderr" 2>&1 )
  rust=$(errs "$OUT/$name.stderr")
  # No XML is a failure, never a clean pass (fail toward flagging).
  [[ -s "$OUT/$name.xml" ]] || rust="noxml"
  line=$(printf '%-8s %-8s %5s' "$status" "$expect" "$rust")
  if [[ $PERL == 1 ]]; then
    ( cd "$OUT" && cp "$tex" . && timeout 120 latexml --nocomments "${preload_args[@]}" \
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
  missing=""
  if [[ $RECALL == 1 ]]; then
    engine=$(grep -m1 -oP '^% engine:\s*\K\w+' "$tex" || true)
    [[ -z $engine ]] && { [[ $preload == *luatex* ]] && engine=lualatex || engine=pdflatex; }
    mkdir -p "$OUT/$name.pdfrun"
    ( cd "$OUT/$name.pdfrun" && cp "$tex" . \
        && timeout 120 "$engine" -interaction=batchmode "$name.tex" >/dev/null 2>&1 )
    if [[ -s "$OUT/$name.pdfrun/$name.pdf" && -s "$OUT/$name.xml" ]]; then
      rec=$(python3 "$here/pdf_recall.py" "$OUT/$name.xml" "$OUT/$name.pdfrun/$name.pdf")
      line+=$(printf ' %7s' "$(grep -oP 'recall=\K[0-9.]+%' <<<"$rec")")
      missing=$(grep -oP 'missing: \K.*' <<<"$rec" || true)
    else
      line+=$(printf ' %7s' 'n/a')
    fi
  fi
  printf '%s  %s' "$line" "$name"
  if [[ $rust == noxml ]]; then
    printf ' -- no XML written: %s' "$(tail -1 "$OUT/$name.stderr" | cut -c1-90)"
  elif [[ $rust -gt 0 ]]; then
    printf ' -- %s' "$(first "$OUT/$name.stderr")"
  fi
  [[ -n $missing ]] && printf ' -- pdf words missing from xml: %s' "$missing"
  printf '\n'
  if [[ $rust == noxml ]]; then
    rc=1
  else
    case $status in
      GREEN) [[ $rust -gt $expect ]] && rc=1 ;;
      CONTROL) [[ $rust -eq 0 ]] && rc=1 ;;
    esac
  fi
done
exit $rc
