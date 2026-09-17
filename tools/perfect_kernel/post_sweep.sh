#!/usr/bin/env bash
# Post-process every S0∧S1 core XML of a sweep outroot into HTML5, so the S3
# content audit and the semantic-markup audit can run over the HTML that a
# user actually gets (bibliographies, cross-references, MathML, split pages
# all come from the post stage).
#
#   post_sweep.sh <xmlroot> [htmlroot]      (default htmlroot = <xmlroot>_html)
#
# Each doc runs `latexml_oxide --whatsin=xml <xml> --dest=<htmlroot>/<b>/<n>/<n>.html
# --sourcedirectory=$DOCROOT/<b>` (the bundle dir supplies .bib/.bbl/graphics),
# ANSI-free stdout+stderr in <n>.post.log. Verdicts in <htmlroot>/post_verdicts.tsv:
#   bundle name exit errors warnings seconds
# and <htmlroot>/sweep_verdicts.tsv carries the S0∧S1 rows so s3_sweep.sh can
# run over the html root unchanged (S3_EXT=html). Resumable: docs with a
# verdict row are skipped. Env: JOBS (4), TIMEOUT_S (180), WORKER_BIN.
set -uo pipefail
XMLROOT="${1:?xmlroot}"
HTMLROOT="${2:-${XMLROOT}_html}"
JOBS="${JOBS:-4}"
TIMEOUT_S="${TIMEOUT_S:-180}"
BIN="${WORKER_BIN:-$HOME/data/pk_bin/latexml_oxide.sweep75}"
DOCROOT="${DOCROOT:-$(kpsewhich -var-value=TEXMFDIST)/doc/latex}"
export XMLROOT HTMLROOT TIMEOUT_S BIN DOCROOT
mkdir -p "$HTMLROOT"
awk -F'\t' '$3<=1 && $5==0 && $6==0' "$XMLROOT/sweep_verdicts.tsv" > "$HTMLROOT/sweep_verdicts.tsv"
touch "$HTMLROOT/post_verdicts.tsv"
post_one() {
  local bundle="$1" name="$2"
  local out="$HTMLROOT/$bundle/$name"
  grep -q "^$bundle	$name	" "$HTMLROOT/post_verdicts.tsv" && return
  mkdir -p "$out"
  local t0=$SECONDS
  ( cd "$out" && ulimit -v 6291456 && timeout "$((TIMEOUT_S + 30))" "$BIN" --whatsin=xml \
      "$XMLROOT/$bundle/$name/$name.xml" --dest="$out/$name.html" \
      --sourcedirectory="$DOCROOT/$bundle" --nodefaultresources --timeout="$TIMEOUT_S" \
      2>&1 | sed 's/\x1b\[[0-9;]*m//g' > "$out/$name.post.log" )
  local rc=${PIPESTATUS[0]}
  [[ -s "$out/$name.html" ]] || rc=${rc:-1}; [[ -s "$out/$name.html" ]] || [[ $rc -ne 0 ]] || rc=99
  local errs warns
  errs=$(grep -c '^Error:' "$out/$name.post.log")
  warns=$(grep -c '^Warning:' "$out/$name.post.log")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$bundle" "$name" "$rc" "$errs" "$warns" "$((SECONDS - t0))" >> "$HTMLROOT/post_verdicts.tsv"
}
export -f post_one
cut -f1,2 "$HTMLROOT/sweep_verdicts.tsv" | xargs -P "$JOBS" -n2 bash -c 'post_one "$@"' _
sort -o "$HTMLROOT/post_verdicts.tsv" "$HTMLROOT/post_verdicts.tsv"
awk -F'\t' '{n++; if($3==0)ok++; e+=$4; if($4>0)de++} END{printf "docs=%d exit0=%d docs-with-errors=%d total-errors=%d\n", n, ok, de, e}' "$HTMLROOT/post_verdicts.tsv"
