#!/usr/bin/env bash
# next_entry_numbers.sh — the next free entry numbers of the numbered registries, read from the files at
# integration time (entry numbers were renumbered three times when payloads guessed them: KPE #568→576,
# OD #484→487, #493→#503+), and from every open PR's head (2026-11-14: four parallel branches each took
# the working tree's next free number, so OD #1026 was claimed three times and #1032/#1035 twice).
#
#   tools/perfect_kernel/next_entry_numbers.sh [--local-only]
#       prints the next free OXIDIZED_DESIGN number (max `### N.` over docs/parity/OXIDIZED_DESIGN_*.md and
#       docs/math/OXIDIZED_DESIGN_MATH.md, plus one), the next free KNOWN_PERL_ERRORS number (max `## N.`,
#       plus one), and each "Next free number" line of the OD index with whether it is stale. The max is
#       taken over the working tree AND every open PR's head (`gh pr list`, up to $NEXT_ENTRY_NUMBERS_PR_LIMIT,
#       default 1000, warning when reached; each `refs/pull/<N>/head` is fetched from $NEXT_ENTRY_NUMBERS_REMOTE,
#       default origin, into a per-run scratch ref deleted on exit; nothing is checked out), with the numbers each PR holds above the working tree's max (`OD #1037-#1038 held
#       by #845`) and every number two sources define with different headings (`CLASH: OD #1037 defined
#       differently by #1 #2`). Without gh, or when gh or a fetch fails, a WARNING goes to stderr and the
#       unscanned heads are left out. --local-only reads the working tree alone (no network).
#   tools/perfect_kernel/next_entry_numbers.sh --renumber <report.md> [--in-place] [--local-only]
#       maps a payload report's placeholders `OD #NEW1`, `OD #NEW2` … and `KPE #NEW1` … (numbered in order
#       of their NEW index) to the next free numbers (the same combined max), prints the mapping, and writes
#       <report>.renumbered.md (or rewrites the report with --in-place). A placeholder elsewhere in the text
#       without its registry prefix (`#NEW1` alone) is reported, not guessed.
#   tools/perfect_kernel/next_entry_numbers_test.sh is the offline self-test (a fake gh, a scratch repo).
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
# The repo is the script's own: a GIT_DIR the pre-push hook exports (a linked worktree's) must not redirect git.
unset $(git rev-parse --local-env-vars)
parity="$root/docs/parity"
od_files=("$parity"/OXIDIZED_DESIGN_*.md)
[ -f "$root/docs/math/OXIDIZED_DESIGN_MATH.md" ] && od_files+=("$root/docs/math/OXIDIZED_DESIGN_MATH.md")
kpe_file="$parity/KNOWN_PERL_ERRORS.md"

max_heading() { # <regex prefix> <files...>: the largest N of `<prefix>N.` headings
  local prefix=$1; shift
  grep -hoE "^${prefix}[0-9]+\." "$@" | grep -oE '[0-9]+' | sort -n | tail -1
}

local_only=0
renumber=""
in_place=""
while [ $# -gt 0 ]; do
  case $1 in
    --local-only) local_only=1 ;;
    --renumber) renumber=${2:?usage: next_entry_numbers.sh --renumber <report.md> [--in-place]}; shift ;;
    --in-place) in_place=--in-place ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

od_max=$(max_heading '### ' "${od_files[@]}")
kpe_max=$(max_heading '## ' "$kpe_file")
local_od_max=$od_max
local_kpe_max=$kpe_max

tmpdir=$(mktemp -d)
scratch_refs=()
cleanup() {
  local ref
  for ref in ${scratch_refs[@]+"${scratch_refs[@]}"}; do git -C "$root" update-ref -d "$ref" 2>/dev/null || true; done
  rm -rf -- "$tmpdir"
}
trap cleanup EXIT

# One row per registry heading: registry, number, source (`#<pr>` or working-tree), heading line.
headings() { # <registry> <source> <heading prefix>: reads a registry file on stdin
  local reg=$1 src=$2 prefix=$3
  grep -E "^${prefix}[0-9]+\." | sed -E 's/[[:space:]]+$//' |
    awk -v reg="$reg" -v src="$src" -v plen=${#prefix} '{ n = substr($0, plen + 1); sub(/\..*/, "", n); print reg "\t" n "\t" src "\t" $0 }' || true
}
table="$tmpdir/headings.tsv"
: > "$table"
scanned=""
gh_ok=0
pr_count=0
if [ "$local_only" = 0 ]; then
  remote=${NEXT_ENTRY_NUMBERS_REMOTE:-origin}
  pr_limit=${NEXT_ENTRY_NUMBERS_PR_LIMIT:-1000}
  if ! command -v gh >/dev/null 2>&1; then
    echo "WARNING: open PR heads not scanned (gh not installed); numbers are the working tree's alone and may collide with an open PR's" >&2
  elif ! prs_json=$(cd "$root" && gh pr list --state open --limit "$pr_limit" --json number,headRefName,headRepositoryOwner,isCrossRepository 2>"$tmpdir/gh.err"); then
    echo "WARNING: open PR heads not scanned (gh pr list failed: $(head -1 "$tmpdir/gh.err")); numbers are the working tree's alone and may collide with an open PR's" >&2
  else
    gh_ok=1
    prs=$(printf '%s' "$prs_json" | grep -oE '"number":[[:space:]]*[0-9]+' | grep -oE '[0-9]+$' | sort -n || true)
    pr_count=$(printf '%s' "$prs" | grep -c . || true)
    [ "$pr_count" -ge "$pr_limit" ] &&
      echo "WARNING: gh pr list returned $pr_count open PRs, its --limit; heads past it are not scanned (raise NEXT_ENTRY_NUMBERS_PR_LIMIT)" >&2
    for pr in $prs; do
      # Per run ($$): concurrent runs in other worktrees of the same repo share refs/.
      ref="refs/next-entry-numbers/$$/pr/$pr"
      scratch_refs+=("$ref")
      # refs/pull/<N>/head, not the branch name, so a fork's PR is read too.
      if ! git -C "$root" fetch -q --no-tags "$remote" "+refs/pull/$pr/head:$ref" 2>"$tmpdir/fetch.err"; then
        echo "WARNING: open PR #$pr not scanned (git fetch $remote refs/pull/$pr/head failed: $(head -1 "$tmpdir/fetch.err")); its numbers may collide" >&2
        continue
      fi
      scanned+=" #$pr"
      while IFS= read -r path; do
        case $path in
          docs/parity/OXIDIZED_DESIGN_*.md | docs/math/OXIDIZED_DESIGN_MATH.md)
            git -C "$root" show "$ref:$path" | headings OD "#$pr" '### ' >> "$table" ;;
          docs/parity/KNOWN_PERL_ERRORS.md)
            git -C "$root" show "$ref:$path" | headings KPE "#$pr" '## ' >> "$table" ;;
        esac
      done < <(git -C "$root" ls-tree -r --name-only "$ref" -- docs/parity docs/math)
    done
  fi
fi
for f in "${od_files[@]}"; do headings OD working-tree '### ' < "$f" >> "$table"; done
headings KPE working-tree '## ' < "$kpe_file" >> "$table"

# Per registry: the max over every source; the ranges each PR holds above the working tree's max; and every
# number whose heading set differs between two sources that define it (equal sets are one inherited entry,
# so main's entries and the math file's by-value #7-#18 are not clashes).
report_heads() {
  awk -F'\t' -v od_local="$local_od_max" -v kpe_local="$local_kpe_max" '
    { key = $1 "\t" $2; if (!((key, $3) in set)) { srcs[key] = srcs[key] " " $3; order[++nk] = key } set[key, $3] = set[key, $3] "\n" $4
      if ($2 + 0 > max[$1]) max[$1] = $2 + 0
      lim = ($1 == "OD") ? od_local : kpe_local
      if ($3 != "working-tree" && $2 + 0 > lim && !seen_held[$1, $3, $2]++) held[$1 "\t" $3] = held[$1 "\t" $3] " " $2 }
    END {
      for (r in max) print "MAX\t" r "\t" max[r]
      for (k in held) {
        split(k, rs, "\t"); m = split(held[k], ns, " "); line = ""; i = 1
        while (i <= m) { j = i; while (j < m && ns[j + 1] == ns[j] + 1) j++
          line = line (line ? ", " : "") rs[1] " #" ns[i] (j > i ? "-#" ns[j] : ""); i = j + 1 }
        print "HELD\t" rs[2] "\t" line " held by " rs[2] }
      for (q = 1; q <= nk; q++) { key = order[q]; if (done[key]++) continue
        n = split(substr(srcs[key], 2), ss, " "); first = ""; differ = 0
        for (i = 1; i <= n; i++) { s = set[key, ss[i]]; c = split(substr(s, 2), hs, "\n"); asort_lines(hs, c); v = ""
          for (h = 1; h <= c; h++) v = v "\n" hs[h]
          if (i == 1) first = v; else if (v != first) differ = 1 }
        if (n > 1 && differ) { split(key, kk, "\t"); print "CLASH\t" kk[2] "\tCLASH: " kk[1] " #" kk[2] " defined differently by " substr(srcs[key], 2) } }
    }
    function asort_lines(a, c,   i, j, t) { for (i = 2; i <= c; i++) for (j = i; j > 1 && a[j - 1] > a[j]; j--) { t = a[j]; a[j] = a[j - 1]; a[j - 1] = t } }
  ' "$table"
}
summary=$(report_heads)
od_max=$(printf '%s\n' "$summary" | awk -F'\t' '$1 == "MAX" && $2 == "OD" { print $3 }')
kpe_max=$(printf '%s\n' "$summary" | awk -F'\t' '$1 == "MAX" && $2 == "KPE" { print $3 }')
od_next=$((od_max + 1))
kpe_next=$((kpe_max + 1))

# Duplicated numbers are a registry defect worth seeing at integration. The math file's #7-#18 collide by
# value with the divergences' on purpose (OXIDIZED_DESIGN.md numbering note), so it is left out here.
od_dups=$(grep -hoE '^### [0-9]+\.' "$parity"/OXIDIZED_DESIGN_*.md | grep -oE '[0-9]+' | sort -n | uniq -d | tr '\n' ' ')
kpe_dups=$(grep -hoE '^## [0-9]+\.' "$kpe_file" | grep -oE '[0-9]+' | sort -n | uniq -d | tr '\n' ' ')

heads_note() { [ -n "$scanned" ] && [ "$1" != "$2" ] && echo "; $3$2. on an open PR head"; true; }
echo "OD  next free: #$od_next  (max ### $local_od_max. over ${#od_files[@]} files$(heads_note "$local_od_max" "$od_max" '### '))${od_dups:+  DUPLICATES: $od_dups}"
echo "KPE next free: #$kpe_next  (max ## $local_kpe_max. in KNOWN_PERL_ERRORS.md$(heads_note "$local_kpe_max" "$kpe_max" '## '))${kpe_dups:+  DUPLICATES: $kpe_dups}"
if [ "$local_only" = 1 ]; then
  echo "open PR heads: not scanned (--local-only)"
elif [ -n "$scanned" ]; then
  echo "open PR heads scanned:$scanned"
elif [ "$gh_ok" = 1 ] && [ "$pr_count" -gt 0 ]; then
  echo "open PR heads: none scanned (every fetch failed)"
elif [ "$gh_ok" = 1 ]; then
  echo "open PR heads: none open"
fi
printf '%s\n' "$summary" | awk -F'\t' '$1 == "HELD"' | sort -s -t'#' -k2,2n | cut -f3
printf '%s\n' "$summary" | awk -F'\t' '$1 == "CLASH"' | sort -k2,2n | cut -f3
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

[ -n "$renumber" ] || exit 0
report=$renumber
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
rewritten="$tmpdir/rewritten"
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
