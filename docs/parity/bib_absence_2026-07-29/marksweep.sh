#!/usr/bin/env bash
# marksweep.sh OUT.tsv < ids — per-paper bibliography log-marker flags (corpus zips)
set -u
OUTFILE="$1"

mark_batch() {
  local id yymm digits z m
  for id in "$@"; do
    if [[ "$id" =~ ^([0-9]{4})\.[0-9]+$ ]]; then yymm="${BASH_REMATCH[1]}"
    else digits="${id##*[!0-9]}"; yymm="${digits:0:4}"; fi
    z="/data/arxiv/$yymm/$id/oxidized_tex_to_html.zip"
    [[ -f "$z" ]] || { printf '%s\t-\n' "$id"; continue; }
    m=$(unzip -p "$z" cortex.log 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -aoE 'Error:bibliography:convert|Couldn.t find usable bibliography|bibentries, 0 cited|bibliography:missing_keys|Missing Entry for citation|\\end\{bibtex@bibliography\}' | sort -u | tr '\n' ';')
    printf '%s\t%s\n' "$id" "${m:--}"
  done
}
export -f mark_batch

xargs -d '\n' -P "${SCAN_P:-64}" -n 40 bash -c 'mark_batch "$@"' _ >> "$OUTFILE"
