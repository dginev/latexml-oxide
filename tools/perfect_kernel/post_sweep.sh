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
#
# POST_MODE=mono runs the whole pipeline in ONE process per doc — `.tex` →
# `.html` with the sweep's preload (run_doc.sh's raw-interpretation profile and
# its lualatex-oracle gate) — instead of post-processing the sweep's core XML
# with `--whatsin=xml`. The split form rebuilds the bibliography session from
# the preloads only, so preamble macros used inside `.bib` fields
# (`\mkbibemph`, `\onethird`, `\cmslink` — biblatex-chicago and friends) come
# back undefined there and only there; the monolithic form is what a user runs
# and is the audit's post pass of record (LEDGER 2026-09-17). `CORPUS`
# (default `~/data/perfect_kernel/corpus.tsv`) supplies each doc's `.tex`.
set -uo pipefail
XMLROOT="${1:?xmlroot}"
HTMLROOT="${2:-${XMLROOT}_html}"
JOBS="${JOBS:-4}"
TIMEOUT_S="${TIMEOUT_S:-180}"
BIN="${WORKER_BIN:-$HOME/data/pk_bin/latexml_oxide.sweep75}"
DOCROOT="${DOCROOT:-$(kpsewhich -var-value=TEXMFDIST)/doc/latex}"
POST_MODE="${POST_MODE:-split}"
CORPUS="${CORPUS:-$HOME/data/perfect_kernel/corpus.tsv}"
export XMLROOT HTMLROOT TIMEOUT_S BIN DOCROOT POST_MODE CORPUS
mkdir -p "$HTMLROOT"
awk -F'\t' '$3<=1 && $5==0 && $6==0' "$XMLROOT/sweep_verdicts.tsv" > "$HTMLROOT/sweep_verdicts.tsv"
touch "$HTMLROOT/post_verdicts.tsv"
post_one() {
  local bundle="$1" name="$2"
  local out="$HTMLROOT/$bundle/$name"
  grep -q "^$bundle	$name	" "$HTMLROOT/post_verdicts.tsv" && return
  mkdir -p "$out"
  local t0=$SECONDS
  if [[ "$POST_MODE" == mono ]]; then
    # One process, with EXACTLY the profile the sweep used for this doc: the
    # core XML records it in its `<?latexml package="latexml" options=…?>`
    # PI, so run_doc.sh's gate (oracle engine + its exceptions) is never
    # duplicated here — a re-derived gate ran polyglossia/hvarabic/musical
    # under pdfTeX and their engine checks halted the job (`\batchmode\read`).
    local tex preload opts
    tex=$(awk -F'\t' -v b="$bundle" -v n="$name" '$1==b {k=split($2,p,"/"); if (p[k]==n".tex") {print $2; exit}}' "$CORPUS")
    opts=$(grep -m1 -o '<?latexml package="latexml" options="[^"]*"' "$XMLROOT/$bundle/$name/$name.xml" | sed 's/.*options="//; s/"$//')
    preload="[${opts:-rawstyles,rawclasses}]latexml.sty"
    ( cd "$out" && ulimit -v 8912896 && timeout "$((TIMEOUT_S + 30))" "$BIN" \
        --preload="$preload" --max-memory=8192 --dest="$out/$name.html" \
        --nodefaultresources --timeout="$TIMEOUT_S" "$tex" \
        2>&1 | sed 's/\x1b\[[0-9;]*m//g' > "$out/$name.post.log" )
  else
    ( cd "$out" && ulimit -v 8912896 && timeout "$((TIMEOUT_S + 30))" "$BIN" --whatsin=xml \
        "$XMLROOT/$bundle/$name/$name.xml" --dest="$out/$name.html" \
        --sourcedirectory="$DOCROOT/$bundle" --nodefaultresources --timeout="$TIMEOUT_S" \
        2>&1 | sed 's/\x1b\[[0-9;]*m//g' > "$out/$name.post.log" )
  fi
  local rc=${PIPESTATUS[0]}
  [[ -s "$out/$name.html" ]] || rc=${rc:-1}; [[ -s "$out/$name.html" ]] || [[ $rc -ne 0 ]] || rc=99
  local errs warns
  errs=$(grep -cE '(Error|Fatal):[A-Za-z_]+:' "$out/$name.post.log")
  warns=$(grep -cE 'Warning:[A-Za-z_]+:' "$out/$name.post.log")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$bundle" "$name" "$rc" "$errs" "$warns" "$((SECONDS - t0))" >> "$HTMLROOT/post_verdicts.tsv"
}
export -f post_one
cut -f1,2 "$HTMLROOT/sweep_verdicts.tsv" | xargs -P "$JOBS" -n2 bash -c 'post_one "$@"' _
sort -o "$HTMLROOT/post_verdicts.tsv" "$HTMLROOT/post_verdicts.tsv"
awk -F'\t' '{n++; if($3==0)ok++; e+=$4; if($4>0)de++} END{printf "docs=%d exit0=%d docs-with-errors=%d total-errors=%d\n", n, ok, de, e}' "$HTMLROOT/post_verdicts.tsv"
