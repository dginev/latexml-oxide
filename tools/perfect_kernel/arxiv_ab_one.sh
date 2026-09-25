#!/bin/bash
# One arXiv zip, two binaries: arxiv_ab_one.sh <zip> <binA> <binB> <run> (driven by arxiv_ab.sh)
# Outputs (out.{A,B}.xml, log.{A,B}) stay in work_<run>/<id>/; one TSV row per binary on stdout.
# Env: DUMP_A / DUMP_B give each side its own LATEXML_DUMP_DIR (default: the shared one).
export PATH=/usr/local/texlive/2025/bin/x86_64-linux:$PATH TEXMFROOT=/usr/local/texlive/2025 TEXMFDIST=/usr/local/texlive/2025/texmf-dist TEXMFCNF=/usr/local/texlive/2025/texmf-dist/web2c LATEXML_DUMP_DIR=${LATEXML_DUMP_DIR:-$HOME/data/pk_agents/vendor_dumps56id/resources/dumps}
ulimit -v 8912896
zip=$1; A=$2; B=$3; run=$4
id=$(basename $zip .zip)
w=${AB_HOME:-$HOME/data/pk_agents/ab56il}/work_$run/$id; rm -rf $w; mkdir -p $w; cd $w || exit 1
unzip -qo $zip 2>/dev/null
main=$(python3 -c "import json;d=json.load(open('00README.json'));print(next((s['filename'] for s in d.get('sources',[]) if s.get('usage')=='toplevel'),''))" 2>/dev/null)
[ -z "$main" ] && main=$(grep -l '\\documentclass' *.tex 2>/dev/null | head -1)
[ -z "$main" ] && { echo -e "$id\tNOMAIN"; exit 0; }
for tag in A B; do
  bin=$A; [ $tag = B ] && bin=$B
  # Per-side dumps (DUMP_A / DUMP_B), for a change that alters the dump itself;
  # both default to LATEXML_DUMP_DIR.
  dumps=${DUMP_A:-$LATEXML_DUMP_DIR}; [ $tag = B ] && dumps=${DUMP_B:-$LATEXML_DUMP_DIR}
  s0=$(date +%s)
  LATEXML_DUMP_DIR=$dumps timeout 330 taskset -c 64-127 $bin --preload=ar5iv.sty --timeout=300 --nocomments --dest=out.$tag.xml "$main" > log.$tag 2>&1
  rc=$?; s1=$(date +%s)
  sed -i 's/\x1b\[[0-9;]*m//g' log.$tag
  e=$(grep -ac '^Error:' log.$tag); f=$(grep -ac '^Fatal:' log.$tag); wn=$(grep -ac '^Warning:' log.$tag)
  bi=$(grep -c '<bibitem' out.$tag.xml 2>/dev/null); tb=$(grep -c '<tabular' out.$tag.xml 2>/dev/null)
  words=$(python3 -c "import re,sys;s=open('out.$tag.xml',errors='replace').read();print(len(re.sub(r'<[^>]+>',' ',s).split()))" 2>/dev/null)
  echo -e "$id\t$tag\t$rc\t$e\t$f\t$wn\t${bi:-0}\t${tb:-0}\t${words:-0}\t$((s1-s0))"
done
find $w -mindepth 1 -maxdepth 1 ! -name "out.*.xml" ! -name "log.*" -exec rm -rf {} +
