#!/usr/bin/env bash
# characterize.sh OUT.tsv RUNDIR... — full accounting of the residual.
#
# For every paper with ZERO ltx_bibitem in a bib_recheck run, emit one row:
#   id  first_error_class  nerr  status  html_bytes  src_ratio_pct  bibsig  biblog
# Every field is derived from the run's own artifacts, so every residual paper
# lands in exactly one bucket — no paper is dropped.
#
# NOTE: use printf, never `echo -e`: an error class like `\endgroup` contains
# `\e`, which `echo -e` turns into an ESC byte and silently truncates the label
# to "ndgroup".
set -uo pipefail
OUT="$1"; shift

for dir in "$@"; do
  [[ -f "$dir/results.tsv" ]] || continue
  while IFS=$'\t' read -r id now want cited miss base status nerr verdict; do
    [[ "$verdict" == "EMPTY" || "$verdict" == "NOHTML" ]] || continue
    log="$dir/$id/cortex.log"
    html="$dir/$id/out.html"

    cls=$(grep -m1 -aE '^(Error|Fatal):' "$log" 2>/dev/null | awk '{print $1}')
    [[ -n "$cls" ]] || cls="-none-"

    hb=$(stat -c%s "$html" 2>/dev/null || echo 0)
    yymm=$(printf '%s' "$id" | grep -oE '^[0-9]{4}')
    [[ -n "$yymm" ]] || yymm=$(printf '%s' "${id##*[!0-9]}" | cut -c1-4)
    src="/data/arxiv/$yymm/$id/$id.zip"
    sb=$(unzip -p "$src" '*.tex' 2>/dev/null | wc -c)
    ratio=$(( sb > 0 ? hb * 100 / sb : -1 ))

    # what the source offers as a bibliography
    names=$(unzip -Z1 "$src" 2>/dev/null)
    sig=""
    grep -qiE '\.bbl$' <<<"$names" && sig+="bbl,"
    grep -qiE '\.bib$' <<<"$names" && sig+="bib,"
    body=$(unzip -p "$src" '*.tex' 2>/dev/null)
    grep -qa 'begin{thebibliography}' <<<"$body" && sig+="thebib,"
    grep -qaE '\\printbibliography|\\addbibresource' <<<"$body" && sig+="biblatex,"
    sig="${sig%,}"; sig="${sig:--}"

    # what the bibliography machinery said
    bl="-"
    grep -qa 'bibentries, 0 cited' "$log" 2>/dev/null && bl="0cited"
    grep -qa "Couldn't find usable bibliography" "$log" 2>/dev/null && bl="nobibfile"
    grep -qa 'Error:bibliography:convert' "$log" 2>/dev/null && bl="convertfail"
    grep -qa 'MakeBibliography: no entries' "$log" 2>/dev/null && bl="${bl}/noentries"

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$id" "$cls" "$nerr" "$status" "$hb" "$ratio" "$sig" "$bl"
  done < "$dir/results.tsv"
done | sort > "$OUT"
wc -l < "$OUT"
