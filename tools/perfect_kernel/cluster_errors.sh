#!/usr/bin/env bash
# Cluster sweep failures by first-error signature.
#
# Usage: cluster_errors.sh [outroot]
# Reads <outroot>/sweep_verdicts.tsv; for every doc with status >= 2 extracts
# the FIRST `Error:`/`Fatal:` line from its log, normalizes it to a signature
# (drops the argument-specific tail), and prints a ranked cluster table:
#   count \t signature \t example-doc
set -uo pipefail

OUTROOT="${1:-$HOME/data/perfect_kernel}"
V="$OUTROOT/sweep_verdicts.tsv"
[[ -f "$V" ]] || { echo "no $V — run sweep.sh first" >&2; exit 1; }

awk -F'\t' '$3>=2' "$V" | while IFS=$'\t' read -r bundle name status rest; do
  log="$OUTROOT/$bundle/$name/$name.log"
  sig=""
  if [[ "$status" == 124 ]]; then
    sig="TIMEOUT"
  elif [[ "$status" == 137 ]]; then
    sig="KILLED"
  elif [[ -f "$log" ]]; then
    sig=$(grep -m1 '^\(Error\|Fatal\):[a-z]' "$log" |
      sed -e 's/\(^[A-Za-z]*:[a-z_]*:[^ ]*\).*/\1/')
  fi
  [[ -z "$sig" ]] && sig="NO-LOG-SIGNATURE"
  printf '%s\t%s/%s\n' "$sig" "$bundle" "$name"
done | sort | awk -F'\t' '
  $1!=prev { if (prev!="") printf "%d\t%s\t%s\n", n, prev, ex; prev=$1; n=0; ex=$2 }
  { n++ }
  END { if (prev!="") printf "%d\t%s\t%s\n", n, prev, ex }' |
  sort -rn
