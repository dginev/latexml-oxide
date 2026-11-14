#!/usr/bin/env bash
# next_entry_numbers.sh — the next free entry numbers of the numbered registries, read from the files at
# integration time (entry numbers were renumbered three times when payloads guessed them: KPE #568→576,
# OD #484→487, #493→#503+).
#
#   tools/perfect_kernel/next_entry_numbers.sh
#       prints the next free OXIDIZED_DESIGN number (max `### N.` over docs/parity/OXIDIZED_DESIGN_*.md and
#       docs/math/OXIDIZED_DESIGN_MATH.md, plus one), the next free KNOWN_PERL_ERRORS number (max `## N.`,
#       plus one), and each "Next free number" line of the OD index with whether it is stale.
#   tools/perfect_kernel/next_entry_numbers.sh --renumber <report.md> [--in-place]
#       maps a payload report's placeholders `OD #NEW1`, `OD #NEW2` … and `KPE #NEW1` … (numbered in order
#       of their NEW index) to the next free numbers, prints the mapping, and writes <report>.renumbered.md
#       (or rewrites the report with --in-place). A placeholder elsewhere in the text without its registry
#       prefix (`#NEW1` alone) is reported, not guessed.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
parity="$root/docs/parity"
od_files=("$parity"/OXIDIZED_DESIGN_*.md)
[ -f "$root/docs/math/OXIDIZED_DESIGN_MATH.md" ] && od_files+=("$root/docs/math/OXIDIZED_DESIGN_MATH.md")
kpe_file="$parity/KNOWN_PERL_ERRORS.md"

max_heading() { # <regex prefix> <files...>: the largest N of `<prefix>N.` headings
  local prefix=$1; shift
  grep -hoE "^${prefix}[0-9]+\." "$@" | grep -oE '[0-9]+' | sort -n | tail -1
}

od_max=$(max_heading '### ' "${od_files[@]}")
kpe_max=$(max_heading '## ' "$kpe_file")
od_next=$((od_max + 1))
kpe_next=$((kpe_max + 1))

# Duplicated numbers are a registry defect worth seeing at integration. The math file's #7-#18 collide by
# value with the divergences' on purpose (OXIDIZED_DESIGN.md numbering note), so it is left out here.
od_dups=$(grep -hoE '^### [0-9]+\.' "$parity"/OXIDIZED_DESIGN_*.md | grep -oE '[0-9]+' | sort -n | uniq -d | tr '\n' ' ')
kpe_dups=$(grep -hoE '^## [0-9]+\.' "$kpe_file" | grep -oE '[0-9]+' | sort -n | uniq -d | tr '\n' ' ')

echo "OD  next free: #$od_next  (max ### $od_max. over ${#od_files[@]} files)${od_dups:+  DUPLICATES: $od_dups}"
echo "KPE next free: #$kpe_next  (max ## $kpe_max. in KNOWN_PERL_ERRORS.md)${kpe_dups:+  DUPLICATES: $kpe_dups}"
grep -nH -E 'Next free[[:space:]]*$|Next free number' "$parity/OXIDIZED_DESIGN.md" "$parity/OXIDIZED_DESIGN_DIVERGENCES.md" 2>/dev/null |
  while IFS= read -r hit; do
    file=${hit%%:*}
    line=$(echo "$hit" | cut -d: -f2)
    # The number may sit on the next line (a wrapped "Next free\n> number: **#N**").
    stated=$(sed -n "${line},$((line + 1))p" "$file" | tr '\n' ' ' | grep -oE 'Next free[^#]*#[0-9]+' | grep -oE '#[0-9]+$' | head -1)
    state="stale (should be #$od_next)"
    [ "$stated" = "#$od_next" ] && state="current"
    echo "OD index ${file#"$root"/}:$line states ${stated:-?}: $state"
  done

[ "${1:-}" = "--renumber" ] || exit 0
report=${2:?usage: next_entry_numbers.sh --renumber <report.md> [--in-place]}
in_place=${3:-}
[ -f "$report" ] || { echo "no such report: $report" >&2; exit 2; }

script=""
map_registry() { # <prefix word> <first number>
  local word=$1 next=$2 k
  for k in $(grep -oE "${word} #NEW[0-9]+" "$report" | grep -oE '[0-9]+$' | sort -n | uniq); do
    echo "${word} #NEW${k} -> ${word} #${next}"
    # Word-bounded so NEW1 does not also rewrite NEW12.
    script+="s/${word} #NEW${k}\\b/${word} #${next}/g;"
    next=$((next + 1))
  done
}
map_registry OD "$od_next"
map_registry KPE "$kpe_next"
rewritten=$(mktemp)
trap 'rm -f "$rewritten"' EXIT
if [ -n "$script" ]; then sed -E "$script" "$report" > "$rewritten"; else cp "$report" "$rewritten"; fi
# Every placeholder must be gone: one the mapping did not cover (`OD #NEW1a`, a bare `#NEW2`) fails the
# run and nothing is written.
left=$(grep -noE '([A-Za-z]+ )?#NEW[0-9A-Za-z]*' "$rewritten" || true)
if [ -n "$left" ]; then
  echo "unmapped placeholders remain (nothing written):" >&2
  echo "$left" >&2
  exit 1
fi
if [ -z "$script" ]; then
  echo "no OD/KPE #NEW placeholders in $report"
  exit 0
fi
if [ "$in_place" = "--in-place" ]; then
  cp "$rewritten" "$report"
  echo "rewrote $report"
else
  out="${report%.md}.renumbered.md"
  cp "$rewritten" "$out"
  echo "wrote $out"
fi
