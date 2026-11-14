#!/bin/bash
# The per-batch manual regression net (KERNEL_CAPABILITIES K17): the fixed manual set
# (manual_net.tsv, chosen by manual_net_select.py for coverage of what the corpus loads and
# produces) with both binaries, each with its own dumps; then both sides' core XML through the
# RelaxNG validator (validate.sh) and every repro topic with both binaries.
#   manual_net.sh <binA> <dumpsA> <binB> <dumpsB> <outdir> [jobs=12]
# Runs on cores 64-127 (CORES=… to change), as the arXiv A/B does (arxiv_ab_one.sh); a sweep
# keeps 0-63. Compare with manual_net_compare.py <outdir>. <outdir> must not exist: sweep.sh
# skips any document that already has a verdict, so a reused outdir would compare stale runs.
set -uo pipefail
[[ $# -ge 5 ]] || { sed -n '2,9p' "$0"; exit 2; }
A=$(readlink -f "$1") DA=$(readlink -f "$2") B=$(readlink -f "$3") DB=$(readlink -f "$4")
OUT=$(readlink -m "$5") J=${6:-12}
HERE="$(cd "$(dirname "$0")" && pwd)"
[[ -e $OUT ]] && { echo "outdir exists: $OUT" >&2; exit 2; }
for bin in "$A" "$B"; do [[ -x $bin ]] || { echo "not an executable: $bin" >&2; exit 2; }; done
for d in "$DA" "$DB"; do
  ls "$d"/latex.*.dump.txt "$d"/plain.*.dump.txt >/dev/null 2>&1 \
    || { echo "no latex/plain dumps in $d" >&2; exit 2; }
done
# The dumps are chosen per side through LATEXML_DUMP_DIR only; an inherited override would load
# the same dump for both sides (latex.rs) and hide a dump change.
unset LATEXML_DUMP_PATH LATEXML_PLAIN_DUMP_PATH LATEXML_NODUMP
export PATH=/usr/local/texlive/2025/bin/x86_64-linux:$PATH TEXMFROOT=/usr/local/texlive/2025 \
  TEXMFDIST=/usr/local/texlive/2025/texmf-dist TEXMFCNF=/usr/local/texlive/2025/texmf-dist/web2c
# One fixed date for both sides: `\today`/`\time` (memoir/memman, awesomebox,
# datetime2-romanian-test-pdftex, kaytannollista-latexia) and shuffles seeded from them
# (examdesign/examplec, expkv-bundle, jeuxcartes/JeuxCartes-doc, tikzbrickfigurines-doc) would
# otherwise differ between the runs.
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-1767225600}
export CORES=${CORES:-64-127}
mkdir -p "$OUT"
cd "$OUT" || exit 2  # jing's JVM writes crash reports (hs_err_pid*.log) to its cwd
# The 8 GB address-space cap is for the conversions only: under it, jing's JVMs fail to start
# ("insufficient memory"), which validate.sh would count as an invalid document.
for side in A B; do
  if [[ $side == A ]]; then bin=$A dumps=$DA; else bin=$B dumps=$DB; fi
  ( ulimit -v 8912896
    LATEXML_DUMP_DIR=$dumps JOBS=$J TIMEOUT_S=180 WORKER_BIN=$bin taskset -c "$CORES" \
      "$HERE/sweep.sh" "$HERE/manual_net.tsv" "$OUT/$side" >"$OUT/$side.log" 2>&1 )
  JOBS=8 taskset -c "$CORES" "$HERE/validate.sh" "$OUT/$side" >"$OUT/$side.validate.log" 2>&1
done
# The repro catalog, one topic per job and side. repros.sh exits 1 when a GREEN repro errors more
# than its `% expect:` count, a CONTROL one converts clean, or any repro writes no XML; 2 on a
# setup error. The comparer reads the per-repro rows, so a new failure inside an already failing
# topic still shows.
for side in A B; do
  if [[ $side == A ]]; then bin=$A dumps=$DA; else bin=$B dumps=$DB; fi
  mkdir -p "$OUT/repros.$side"
  ls -d "$HERE"/repros/*/ | xargs -P "$J" -I{} bash -c '
    ulimit -v 8912896
    topic=$(basename "$1")
    LATEXML_DUMP_DIR=$3 taskset -c "$CORES" "$4/repros.sh" "$topic" --bin "$2" \
      --out "$5/$topic" >"$5/$topic.txt" 2>&1
    printf "%s\t%s\n" "$topic" "$?" >"$5/$topic.status"' _ {} "$bin" "$dumps" "$HERE" "$OUT/repros.$side"
  cat "$OUT/repros.$side"/*.status | sort >"$OUT/repros.$side/status.tsv"
done
echo "NET_DONE $OUT"
