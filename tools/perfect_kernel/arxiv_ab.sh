#!/bin/bash
# arXiv A/B over a fixed paper sample (default: the 3,003-paper 2605 sample).
#   arxiv_ab.sh <run> <binA> <binB> [parallel=24]
# Per paper and binary: --preload=ar5iv.sty, 300 s, 8 GB, cores 64-127 (arxiv_ab_one.sh).
# Rows: $AB_HOME/results/<run>.tsv (id tag rc errors fatals warnings bibitems tabulars words secs).
# Outputs: $AB_HOME/work_<run>/<id>/{out,log}.{A,B}, ~7 GB per run: delete once triaged.
# Env: AB_HOME (default ~/data/pk_agents/ab56il), AB_INPUT (default ~/data/pk_agents/w67/k6ab/input).
run=$1; A=$2; B=$3; P=${4:-24}
HERE="$(cd "$(dirname "$0")" && pwd)"
export AB_HOME=${AB_HOME:-$HOME/data/pk_agents/ab56il}
IN=${AB_INPUT:-$HOME/data/pk_agents/w67/k6ab/input}
mkdir -p "$AB_HOME/results"
: > "$AB_HOME/results/$run.tsv"
ls "$IN"/*.zip | xargs -P "$P" -I{} "$HERE/arxiv_ab_one.sh" {} "$A" "$B" "$run" >> "$AB_HOME/results/$run.tsv"
python3 "$HERE/arxiv_ab_compare.py" "$AB_HOME/results/$run.tsv"
