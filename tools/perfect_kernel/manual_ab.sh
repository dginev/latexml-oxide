#!/bin/bash
# Manual-corpus A/B: run a corpus subset with two binaries, one after the other, on the sweep
# cores (0-63), through sweep.sh/run_doc.sh at sweep settings (180 s, 8 GB).
#   manual_ab.sh <corpus-subset.tsv> <binA> <outA> <binB> <outB> [jobs=12]
# Build the subset by grepping ~/data/perfect_kernel/corpus.tsv for the manuals that exercise
# the changed mechanism; compare with manual_ab_compare.py.
C=$1; JOBS_AB=${6:-12}
HERE="$(cd "$(dirname "$0")" && pwd)"
export PATH=/usr/local/texlive/2025/bin/x86_64-linux:$PATH TEXMFROOT=/usr/local/texlive/2025 TEXMFDIST=/usr/local/texlive/2025/texmf-dist TEXMFCNF=/usr/local/texlive/2025/texmf-dist/web2c LATEXML_DUMP_DIR=${LATEXML_DUMP_DIR:-$HOME/data/pk_agents/vendor_dumps56id/resources/dumps}
ulimit -v 8912896
JOBS=$JOBS_AB TIMEOUT_S=180 WORKER_BIN=$2 taskset -c 0-63 "$HERE/sweep.sh" "$C" "$3"
JOBS=$JOBS_AB TIMEOUT_S=180 WORKER_BIN=$4 taskset -c 0-63 "$HERE/sweep.sh" "$C" "$5"
