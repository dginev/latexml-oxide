#!/bin/bash
# K13 stage 2 (KERNEL_CAPABILITIES): audit every compiled package binding against its real .sty,
# one `binding_audit` process per package (180 s, 8 GB), ranked by how many papers of an arXiv A/B
# sample load the binding.
#
# usage: binding_conformance.sh <outdir> [jobs] [ab-workdir]
#   BIN     the binding_audit binary (default target/release/binding_audit)
#   CORES   taskset core list (default 64-127 on a host with more than 64 cores, else all of them)
#   ab-workdir  an arxiv_ab.sh work dir (…/work_k<label>); each paper's log.B gives the weights
# Packages that need another loaded first: binding_conformance_preambles.tsv.
# output: <outdir>/conformance.tsv — weight, package, cs, severity, classes, real and binding
#   signatures, chains and notes, the two sessions' errors; ranked by weight then severity.
#   <outdir>/summary.tsv — one row per package; failures are printed.
set -uo pipefail
OUT=$1; JOBS=${2:-8}; SAMPLE=${3:-}
BIN=$(readlink -f "${BIN:-target/release/binding_audit}")
[[ -x $BIN ]] || { echo "not an executable: $BIN" >&2; exit 2; }
if [[ -z ${CORES:-} ]]; then
  if (( $(nproc --all) > 64 )); then CORES=64-127; else CORES=0-$(( $(nproc --all) - 1 )); fi
fi
mkdir -p "$OUT/rows"
: > "$OUT/status.tsv"
"$BIN" --list > "$OUT/packages.txt"
: > "$OUT/weights.tsv"
if [ -n "$SAMPLE" ]; then
  # One count per paper that loads the binding (a log may announce it more than once).
  grep -o '(Loading [^ )]*_sty\.rs' "$SAMPLE"/*/log.B | sort -u | sed 's/^.*(Loading //; s/_sty\.rs$//' \
    | sort | uniq -c | awk '{print $2 "\t" $1}' > "$OUT/weights.tsv"
fi
# A package that needs another loaded first gets its preamble (package<TAB>preamble).
PREAMBLES=$(readlink -f "$(dirname "$0")/binding_conformance_preambles.tsv")
export BIN OUT CORES PREAMBLES
xargs -a "$OUT/packages.txt" -P "$JOBS" -I{} bash -c '
  ulimit -v 8912896
  preamble=$(awk -F"\t" -v p="$1" "\$1==p {print \$2}" "$PREAMBLES")
  timeout 180 taskset -c "$CORES" "$BIN" "$1" "$preamble" > "$OUT/rows/$1.tsv" 2> "$OUT/rows/$1.err"
  printf "%s\t%s\n" "$1" "$?" >> "$OUT/status.tsv"' _ {}
python3 "$(dirname "$0")/binding_conformance_rank.py" "$OUT"
