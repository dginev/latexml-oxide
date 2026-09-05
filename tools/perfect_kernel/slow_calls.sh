#!/bin/bash
# slow_calls.sh <sweep_dir> [threshold_secs=60] — the slow-call audit (user directive
# 2026-09-05: a conversion over a minute must be justified by a large, highly
# structured, content-preserving output). One row per doc over the threshold:
#   doc  status  errors  secs  tex_KB  xml_KB  sections  Math  tabular  figure  picture/svg  KB/s  verdict
# verdict: JUSTIFIED when the output is large and structured relative to the time
# (>= 25 KB of XML per second, or >= 2 MB of XML), TIMEOUT when the run was killed,
# else SUSPECT — a perf root to chase (docs/performance/PERFORMANCE.md lane).
dir=${1:?sweep dir}; thr=${2:-60}
corpus=~/data/perfect_kernel/corpus.tsv
printf 'doc\tstatus\terrors\tsecs\ttex_KB\txml_KB\tsections\tMath\ttabular\tfigure\tpicture\tKB_per_s\tverdict\n'
awk -F'\t' -v t="$thr" '$8>t {print $1"\t"$2"\t"$3"\t"$5"\t"$8}' "$dir/sweep_verdicts.tsv" | sort -t$'\t' -k5 -rn |
while IFS=$'\t' read b n st err secs; do
  x="$dir/$b/$n/$n.xml"; xb=0; [ -f "$x" ] && xb=$(stat -c %s "$x")
  src=$(awk -F'\t' -v b="$b" -v n="$n" '$1==b && $2==n {print $2; exit}' "$corpus"); src=$(awk -F'\t' -v n="$n" '$2 ~ ("/" n "\\.tex$") {print $2; exit}' "$corpus")
  tb=0; [ -n "$src" ] && [ -f "$src" ] && tb=$(stat -c %s "$src")
  sec=0; math=0; tab=0; fig=0; pic=0
  if [ -f "$x" ]; then sec=$(grep -c '<section' "$x"); math=$(grep -o '<Math ' "$x" | wc -l); tab=$(grep -o '<tabular' "$x" | wc -l); fig=$(grep -o '<figure' "$x" | wc -l); pic=$(grep -o '<picture\|<svg' "$x" | wc -l); fi
  xkb=$((xb/1024)); tkb=$((tb/1024)); rate=$(awk -v k="$xkb" -v s="$secs" 'BEGIN{ if (s>0) printf "%.1f", k/s; else print 0 }')
  v=SUSPECT; [ "$st" = 124 ] && v=TIMEOUT
  awk -v r="$rate" -v k="$xkb" 'BEGIN{exit !(r>=25 || k>=2048)}' && [ "$v" != TIMEOUT ] && v=JUSTIFIED
  printf '%s/%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$b" "$n" "$st" "$err" "$secs" "$tkb" "$xkb" "$sec" "$math" "$tab" "$fig" "$pic" "$rate" "$v"
done
