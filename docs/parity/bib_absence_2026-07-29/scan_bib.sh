#!/usr/bin/env bash
# scan_bib.sh OUT.tsv SUFFIX DIR...
#
# Bibliography-presence audit over cortex result zips.
# For each paper dir under the given corpus DIRs, picks
# oxidized_tex_to_html${SUFFIX}.zip (falling back to the plain
# oxidized_tex_to_html.zip), streams every *.html member plus the
# `status` member through ONE unzip|awk pass, and emits:
#   id <TAB> verdict <TAB> cortex_status
# verdicts:
#   ok        - at least one ltx_bibitem present
#   empty_bib - a bibliography/biblist section exists but zero ltx_bibitem
#   no_bib    - HTML present, no bibliography markup at all
#   no_html   - result zip has no readable HTML bytes (incl. corrupt zip)
#   no_result - no result zip on disk
# Bias is fail-safe: anything unreadable lands in a flagged bucket.
set -u
OUT="$1"; SUFFIX="$2"; shift 2

scan_batch() {
  local suffix="$1"; shift
  local d id zip
  for d in "$@"; do
    id="${d##*/}"
    zip="$d/oxidized_tex_to_html${suffix}.zip"
    [[ -f "$zip" ]] || zip="$d/oxidized_tex_to_html.zip"
    if [[ ! -f "$zip" ]]; then
      printf '%s\tno_result\t-\n' "$id"
      continue
    fi
    unzip -p "$zip" '*.html' status 2>/dev/null | awk -v id="$id" '
      /ltx_bibitem/            { bi = 1 }
      /ltx_biblist|ltx_bibliography/ { bl = 1 }
      /^Status:conversion:/    { split($0, a, ":"); st = a[3]; next }
      { hn += 1 }
      END {
        if (hn == 0)      v = "no_html"
        else if (bi)      v = "ok"
        else if (bl)      v = "empty_bib"
        else              v = "no_bib"
        if (st == "") st = "-"
        printf "%s\t%s\t%s\n", id, v, st
      }'
  done
}
export -f scan_batch

find "$@" -mindepth 1 -maxdepth 1 \( -type d -o -type l \) -print0 |
  xargs -0 -P "${SCAN_P:-32}" -n 100 bash -c 'scan_batch "$@"' _ "$SUFFIX" >> "$OUT"
