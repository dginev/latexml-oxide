#!/usr/bin/env bash
# pass2.sh OUT.tsv SUFFIX ROOT < flagged_ids
#   ROOT = a corpus dir (/data/arxiv/2605) or AUTO to derive /data/arxiv/<yymm>
#          from each id (new style 2101.00001 -> 2101, old style math0203001 -> 0203).
# For each flagged paper id (stdin, one per line), emit:
#   id <TAB> expect <TAB> srcsig <TAB> category <TAB> outbytes <TAB> firsterr
# expect: yes|no|auto_ignore|no_tex|no_src  (does the SOURCE ask for a bibliography?)
# srcsig: comma list of matched source signals (bbl,bib,stub,thebib,bibcmd,printbib,bibitem)
# category/outbytes from telemetry.json; firsterr = first ANSI-stripped Error:/Fatal: line.
set -u
OUT="$1"; SUFFIX="$2"; ROOT="$3"

pass2_batch() {
  local suffix="$1" root="$2"; shift 2
  local id d src zip
  for id in "$@"; do
    if [[ "$root" == "AUTO" ]]; then
      local yymm
      if [[ "$id" =~ ^([0-9]{4})\.[0-9]+$ ]]; then yymm="${BASH_REMATCH[1]}"
      else local digits="${id##*[!0-9]}"; yymm="${digits:0:4}"; fi
      d="/data/arxiv/$yymm/$id"
    else
      d="$root/$id"
    fi
    src="$d/$id.zip"
    zip="$d/oxidized_tex_to_html${suffix}.zip"
    [[ -f "$zip" ]] || zip="$d/oxidized_tex_to_html.zip"
    # --- source-side intent ---
    local sig="" expect="no"
    if [[ ! -f "$src" ]]; then
      expect="no_src"
    else
      local names; names=$(unzip -Z1 "$src" 2>/dev/null)
      grep -qiE '\.bbl$' <<<"$names" && sig+="bbl,"
      grep -qiE '\.bib$' <<<"$names" && sig+="bib,"
      if grep -qiE '\.(tex|ltx)$' <<<"$names"; then
        local texsig
        texsig=$(unzip -p "$src" '*.tex' '*.ltx' '*.TEX' 2>/dev/null | awk '
          /%auto-ignore/                  { s = 1 }
          /\\begin *\{ *thebibliography/  { t = 1 }
          /\\bibliography *[{[]/          { b = 1 }
          /\\printbibliography|\\addbibresource/ { p = 1 }
          /\\bibitem/                     { i = 1 }
          END { printf "%d%d%d%d%d", s+0, t+0, b+0, p+0, i+0 }')
        [[ ${texsig:0:1} == 1 ]] && sig+="stub,"
        [[ ${texsig:1:1} == 1 ]] && sig+="thebib,"
        [[ ${texsig:2:1} == 1 ]] && sig+="bibcmd,"
        [[ ${texsig:3:1} == 1 ]] && sig+="printbib,"
        [[ ${texsig:4:1} == 1 ]] && sig+="bibitem,"
      else
        expect="no_tex"
      fi
      if [[ "$expect" == "no" ]]; then
        if [[ "$sig" == *stub,* ]]; then expect="auto_ignore"
        elif [[ "$sig" == *thebib,* || "$sig" == *bibcmd,* || "$sig" == *printbib,* || "$sig" == *bibitem,* || "$sig" == *bbl,* ]]; then expect="yes"
        fi
      fi
    fi
    # --- result-side signals ---
    local category="-" outbytes="-" firsterr="-"
    if [[ -f "$zip" ]]; then
      read -r category outbytes < <(unzip -p "$zip" telemetry.json 2>/dev/null | awk '
        { cat = "-"; ob = "-"
          if (match($0, /"category":"[^"]*"/))    cat = substr($0, RSTART+12, RLENGTH-13)
          if (match($0, /"output_bytes":[0-9]+/)) ob  = substr($0, RSTART+15, RLENGTH-15)
          print cat, ob }')
      [[ -n "${category:-}" ]] || category="-"
      [[ -n "${outbytes:-}" ]] || outbytes="-"
      firsterr=$(unzip -p "$zip" cortex.log 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -m1 -aE '^(Error|Fatal):' | cut -c1-160)
      [[ -n "$firsterr" ]] || firsterr="-"
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$expect" "${sig%,}" "$category" "$outbytes" "$firsterr"
  done
}
export -f pass2_batch

xargs -d '\n' -P "${SCAN_P:-32}" -n 20 bash -c 'pass2_batch "$@"' _ "$SUFFIX" "$ROOT" >> "$OUT"
