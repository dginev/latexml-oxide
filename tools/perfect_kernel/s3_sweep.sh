#!/usr/bin/env bash
# S3 content-recall sweep: run s3_audit.sh over every S0∧S1 document of a
# sweep outroot and write <outroot>/s3_verdicts.tsv:
#   bundle  name  recall_pct  found  total  missing  missing_sample
# (S0∧S1 = status ≤ 1, errors 0, fatals 0 in sweep_verdicts.tsv.) Docs whose
# golden PDF is absent or empty are listed with recall "-" so they are not
# mistaken for content loss. Env: JOBS (default 4); S3_EXT=html audits a
# post_sweep.sh html root (which carries its own sweep_verdicts.tsv).
set -uo pipefail
OUTROOT="${1:-$HOME/data/perfect_kernel}"
JOBS="${JOBS:-4}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export HERE OUTROOT S3_EXT="${S3_EXT:-xml}"
audit_one() {
  local bundle="$1" name="$2"
  local out
  out=$("$HERE/s3_audit.sh" "$bundle/$name" "$OUTROOT" 2>&1)
  local rc=$?
  if [[ $rc -ne 0 ]] || ! echo "$out" | grep -q "recall=" || echo "$out" | grep -q "(0/0 distinct"; then
    printf '%s\t%s\t-\t0\t0\t0\t%s\n' "$bundle" "$name" "$(echo "$out" | head -1 | tr '\t' ' ')"
    return
  fi
  # line 1: "<doc>\trecall=NN.N%\t(F/T distinct pdf words; M missing)"; line 2: "  missing sample: w1 w2 …"
  local pct found total missing sample
  pct=$(echo "$out" | sed -n '1s/.*recall=\([0-9.]*\)%.*/\1/p')
  found=$(echo "$out" | sed -n '1s/.*(\([0-9]*\)\/\([0-9]*\) distinct.*/\1/p')
  total=$(echo "$out" | sed -n '1s/.*(\([0-9]*\)\/\([0-9]*\) distinct.*/\2/p')
  missing=$(echo "$out" | sed -n '1s/.*; \([0-9]*\) missing.*/\1/p')
  sample=$(echo "$out" | sed -n '2s/^ *missing sample: *//p' | tr '\t' ' ')
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$bundle" "$name" "$pct" "$found" "$total" "$missing" "$sample"
}
export -f audit_one
awk -F'\t' '$3<=1 && $5==0 && $6==0 {print $1"\t"$2}' "$OUTROOT/sweep_verdicts.tsv" |
  xargs -P "$JOBS" -n2 bash -c 'audit_one "$@"' _ | sort > "$OUTROOT/s3_verdicts.tsv"
n=$(wc -l < "$OUTROOT/s3_verdicts.tsv")
awk -F'\t' -v n="$n" '$3!="-"{c++; s[c]=$3+0; if($3>=95)a++; else if($3>=90)b++; else if($3>=80)d++; else if($3>=60)e++; else f++}
  $3=="-"{np++}
  END{asort(s); printf "docs=%d audited=%d no-pdf=%d median=%.1f  >=95:%d 90-95:%d 80-90:%d 60-80:%d <60:%d\n", n, c, np, s[int((c+1)/2)], a,b,d,e,f}' "$OUTROOT/s3_verdicts.tsv"
