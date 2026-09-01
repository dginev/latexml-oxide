#!/usr/bin/env bash
# Tally a perfect-kernel sweep against the oracle and a previous sweep.
#
# Usage: tally.sh <current_verdicts.tsv> [previous_verdicts.tsv] [oracle_verdicts.tsv]
#   verdicts: bundle name status exit errors fatals warnings seconds
#             (cat ~/data/perfect_kernel/*/*/verdict.tsv)
#   oracle:   bundle name engine exit errors   (tools/perfect_kernel/oracle.sh)
#
# Reports the honest S0∧S1 bar (COMPLETED with zero errors — a 0-error
# timeout is NOT a win: it truncated content), the legacy zero-error count,
# timeouts split by errors-before-timeout, fatals, error mass, the exemplar
# rows, and — against the previous sweep — every per-document REGRESSION by
# error-count delta (status worsening to timeout/fatal, or errors rising by
# ≥ REG_DELTA, default 5), not merely zero-error flips: the nicematrix
# exemplar sat at 108→1001 for four sweeps undetected by a flip-only diff.
set -uo pipefail

CUR="$1"
PREV="${2:-}"
ORACLE="${3:-$HOME/data/perfect_kernel/oracle_verdicts.tsv}"
REG_DELTA="${REG_DELTA:-5}"
EXEMPLARS="${EXEMPLARS:-nicematrix/nicematrix nicematrix/nicematrix-french}"

export LC_ALL=C
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

# key status errors fatals seconds
awk -F'\t' -v OFS='\t' '{print $1"/"$2, $3, $5, $6, $8}' "$CUR" | sort -u >"$tmp/cur"
awk -F'\t' -v OFS='\t' '($3=="pdflatex"||$3=="lualatex") && $4==0 && $5==0 {print $1"/"$2}' \
  "$ORACLE" | sort -u >"$tmp/oracle"
join -t$'\t' "$tmp/oracle" "$tmp/cur" >"$tmp/slice"

summarize() {
  awk -F'\t' -v label="$1" '
    { n++; mass += $3
      if ($2 == 124) { to++; if ($3 == 0) to0++ }
      else if ($2 == 3 || $2 == 137) fat++
      if ($3 == 0) zero++
      if ($3 == 0 && $2 != 124 && $2 != 3 && $2 != 137) s01++ }
    END { printf "%-14s docs=%d  S0∧S1=%d (%.1f%%)  legacy-zero-error=%d  timeouts=%d (%d with 0 errs)  fatal/killed=%d  mass=%d\n",
          label, n, s01, 100*s01/n, zero, to, to0, fat, mass }' "$2"
}
summarize "ALL" "$tmp/cur"
summarize "ORACLE-CLEAN" "$tmp/slice"

echo "exemplars:"
for e in $EXEMPLARS; do grep -P "^\Q$e\E\t" "$tmp/cur" || echo "  $e: (no verdict)"; done

[[ -n "$PREV" ]] || exit 0
awk -F'\t' -v OFS='\t' '{print $1"/"$2, $3, $5, $6, $8}' "$PREV" | sort -u >"$tmp/prev"
join -t$'\t' "$tmp/prev" "$tmp/cur" >"$tmp/both"   # 1 key, 2-5 prev status/err/fat/sec, 6-9 cur status/err/fat/sec
echo
echo "vs previous ($(wc -l <"$tmp/both") docs in both):"
awk -F'\t' -v d="$REG_DELTA" '
  function bad(s) { return s == 124 || s == 3 || s == 137 }
  { if (($3 > 0 || bad($2)) && $7 == 0 && !bad($6)) nc++
    if (($7 - $3) >= d || (bad($6) && !bad($2))) { reg++; regs = regs sprintf("  %s: %s/%d -> %s/%d\n", $1, $2, $3, $6, $7) } }
  END { printf "  newly clean: %d\n  regressions (Δerr ≥ %d or status worsened): %d\n%s", nc, d, reg, regs }' "$tmp/both"
