#!/usr/bin/env bash
# Convert one TeX Live documentation manual to core XML under the
# perfect-kernel protocol: RAW interpretation of every .sty and .cls via
# `--preload=[rawstyles,rawclasses]latexml.sty` (no OmniBus, no binding
# shortcuts for the document's own packages beyond what the kernel provides).
#
# Usage: run_doc.sh <manual.tex> [outroot]
#   outroot defaults to ~/data/perfect_kernel
#
# Produces under <outroot>/<bundle>/<name>/:
#   <name>.xml          core XML output
#   <name>.log          full stderr (ANSI-stripped)
#   verdict.tsv         one line: bundle name status exit errors fatals seconds
#
# Status codes follow cortex: 3 = fatal, 2 = error, 1 = warning, 0 = clean.
# Exit code of this script is the status (0 also for warnings) — 124 = timeout.
set -uo pipefail

TEX="$1"
OUTROOT="${2:-$HOME/data/perfect_kernel}"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${WORKER_BIN:-$REPO/target/debug/latexml_oxide}"
TIMEOUT_S="${TIMEOUT_S:-300}"

name=$(basename "$TEX" .tex)
bundle=$(basename "$(dirname "$TEX")")
out="$OUTROOT/$bundle/$name"
mkdir -p "$out"

# RAM guard (see memory feedback_sandbox_run_discipline § feedback_sandbox_ram_guard): 6 GiB virtual.
ulimit -v 6291456

# Pin kpathsea to the SAME TeX Live the corpus comes from. The binary's linked
# (in-process) libkpathsea anchors on its compile-time distro tree — on a host
# with both a distro TL (/usr/share/texlive) and a vendor TL, raw styles would
# silently resolve against the WRONG (older) tree while the manuals under test
# ship with the vendor one (caught 2026-08-31: keyval.sty loading from
# /usr/share/texlive during a /usr/local/texlive/2025 corpus sweep). kpathsea
# honors TEXMF* env overrides in-process, so derive them from the ambient
# kpsewhich on PATH.
TL_ROOT="${TL_ROOT:-$(dirname "$(dirname "$(dirname "$(command -v kpsewhich)")")")}"
if [[ -d "$TL_ROOT/texmf-dist" ]]; then
  export TEXMFROOT="$TL_ROOT"
  export TEXMFDIST="$TL_ROOT/texmf-dist"
  export TEXMFCNF="$TL_ROOT/texmf-dist/web2c"
fi

# LuaLaTeX-authored docs (oracle needed lualatex) opt into the `luatex`
# latexml.sty profile (user decision 2026-08-31): engine probes read LuaTeX
# and \directlua runs through the texlua bridge.
PRELOAD='[rawstyles,rawclasses]latexml.sty'
# The oracle lives with the corpus (~/data/perfect_kernel), not with each
# sweep's outroot: a fresh outroot (perfect_kernel_s14) silently ran the 264
# clean-lualatex docs as pdfTeX (cartonaugh/ukbill/elpres/fontsetup 0→N
# "regressions", sweep 28) because the gate looked for the oracle next to it.
ORACLE="${ORACLE:-$HOME/data/perfect_kernel/oracle_verdicts.tsv}"
[[ -f "$ORACLE" ]] || ORACLE="$OUTROOT/oracle_verdicts.tsv"
# Gate on a CLEAN lualatex oracle (exit 0, zero errors): the oracle records
# engine=lualatex for every pdflatex-failure FALLBACK too, and profiling
# those (mostly pdfLaTeX-authored stale docs) under a LuaTeX identity
# regressed the whole corpus (+78k error mass, sweep 9 first run).
if [[ -f "$ORACLE" ]] && grep -qP "^$bundle\t$name\tlualatex\t0\t0$" "$ORACLE"; then
  PRELOAD='[rawstyles,rawclasses,luatex]latexml.sty'
fi

run_once() {
  timeout "$TIMEOUT_S" "$BIN" \
    --preload="$PRELOAD" \
    --xml \
    --timeout="$TIMEOUT_S" \
    --max-memory=6144 \
    --dest="$out/$name.xml" \
    "$TEX" >"$out/$name.stdout" 2>"$out/$name.raw.log"
  exit_code=$?
  # ANSI-strip the log (older/current binaries may color when not TTY-gated).
  sed 's/\x1b\[[0-9;]*m//g' "$out/$name.raw.log" >"$out/$name.log"
  rm -f "$out/$name.raw.log"
}

start=$(date +%s.%N)
run_once
# Wrong-engine retry (batch 56am, 2026-09-07). Since batch 56ak a `\batchmode
# \read -1` (iftex `\Require<engine>`, expl3 `\msg_fatal`) is the Fatal it is
# in TeX (tex.web §484), so a LuaLaTeX/XeLaTeX-authored manual whose oracle is
# NOT clean (and therefore ran under the pdfTeX identity above) now halts in
# seconds instead of limping on — sweep 59: 46 docs, 23 of them formerly partial
# output, two formerly 0-error (musical, dithesis/sampleNoArial). The kernel
# stays faithful; the harness gives such a doc the engine it was written for:
# one retry under the `luatex` identity, recorded next to the verdict.
# A second unambiguous LuaTeX-required signal: pgf's graph-drawing library
# (pgflibrarygraphdrawing.code.tex:16-24) errors "You need to run LuaTeX to
# use the graph drawing library" and `\endinput`s under any other engine —
# tikz-ext-manual (oracle lualatex, not clean, so the gate above excluded it)
# lost 14 errors to that gate's cascade (2026-09-09). Only docs that emit the
# message are retried, so the clean-oracle gate's guard against pdfLaTeX-
# authored stale docs is untouched.
rm -f "$out/retried_luatex"
if [[ "$PRELOAD" != *luatex* ]] && grep -q -e 'cannot \\read from terminal in nonstop modes' -e 'You need to run LuaTeX to use the graph drawing library' -e 'LuaTeX is required for this package' "$out/$name.log"; then
  PRELOAD='[rawstyles,rawclasses,luatex]latexml.sty'
  printf 'first run (pdfTeX identity) hit a wrong-engine halt or a LuaTeX-required library gate; retried under luatex\n' >"$out/retried_luatex"
  run_once
fi
# Third signal (batch 56eu, 2026-09-20): a Unicode-engine manual whose lualatex
# oracle did NOT compile cleanly on this host (missing fonts) fails the clean-
# oracle gate above and runs under the pdfTeX identity, where its font setup
# takes the pdfTeX branch — libertinus.sty:15-22 / neoschool.cls branch on
# \iftutex (honestly false) to a Type1 path that never defines \setmonofont — so
# the leaked argument becomes a leading <para> and every frontmatter element
# after it is schema-invalid (hvfloat/wide2s2c + 10 siblings, beamerthemeCelestia;
# identical in Perl and real pdflatex). The doc's REQUIRED engine is recorded by
# the oracle regardless of cleanliness; a demonstrated Unicode-font dependency in
# the pdfTeX run is the discriminator that keeps the +78k-error trap of trusting
# engine=lualatex alone (sweep 9) shut: BOTH must hold.
# Signal widened 2026-09-22 (user ruling: engine-primitive ink is out of scope by
# the intended-engine gate, but the speculative retry is the one safe lever): the
# luatexja/jlreq/xeCJK/kotex/emoji undefined-CS set (\kanjiskip, \setCJKmainfont,
# \fontid, …). Under the pdfTeX identity these manuals take their pTeX branch and
# leak the primitives' arguments as a leading <para>; under [luatex] jlreq/
# luatexja route around it (chemobabel-en 101 err/1 fatal → 25/0, asternote
# 25/1 → 13/0, emoji-doc 3 → 0). Keep-the-better-run below discards the ones
# that regress (jpnedumathsymbols-doc, pmhanguljamo-doc). Raw pTeX/encTeX/XeTeX
# primitives are never defined (DIFFICULT_CASES D9).
LEAK_RE='undefined:\\(setmonofont|setmainfont|setsansfont|setmathfont|IfFontExistsTF|directlua|setCJKmainfont|newCJKfontfamily|setemojifont|kanjiskip|xkanjiskip|ltjsetparameter|reDeclareMathAlphabet|fontid|pagedir|bodydir) '
if [[ "$PRELOAD" != *luatex* ]] && [[ -f "$ORACLE" ]] \
   && grep -qP "^$bundle\t$name\t(lualatex|xelatex)\t" "$ORACLE" \
   && grep -qE "$LEAK_RE" "$out/$name.log"; then
  # This retry is SPECULATIVE (the oracle was not clean), so keep whichever run is
  # better: s107 turned latex-via-exemplos (pdfTeX: 2 errors, 10 s) into a 300 s
  # TokenLimit Fatal under luatex, and pmhanguljamo-kdoc (PARKED luatexko) from
  # 3 errors into 5. A retry that Fatals/times out where the first run did not,
  # or that logs MORE errors, is discarded and the pdfTeX artifacts restored.
  for ext in xml log stdout; do cp -f "$out/$name.$ext" "$out/$name.pdftex.$ext" 2>/dev/null || true; done
  first_exit=$exit_code
  first_err=$(grep -cE 'Error:[a-z_]+:' "$out/$name.log" || true)
  first_fatal=$(grep -cE 'Fatal:[A-Za-z_]+:' "$out/$name.log" || true)
  first_leak=$(grep -cE "$LEAK_RE" "$out/$name.log" || true)
  PRELOAD='[rawstyles,rawclasses,luatex]latexml.sty'
  printf 'first run (pdfTeX identity) leaked a Unicode-engine font command and the oracle engine is lualatex/xelatex; retried under luatex\n' >"$out/retried_luatex"
  run_once
  new_exit=$exit_code
  new_err=$(grep -cE 'Error:[a-z_]+:' "$out/$name.log" || true)
  new_fatal=$(grep -cE 'Fatal:[A-Za-z_]+:' "$out/$name.log" || true)
  new_leak=$(grep -cE "$LEAK_RE" "$out/$name.log" || true)
  # The error-line count is anti-correlated with schema validity here (batch
  # 56fq, 2026-09-22): the pdfTeX run's font leak is TWO Error lines but the
  # leaked arguments form a leading <para> that makes every frontmatter element
  # after it invalid (4 jing errors), while the luatex run's extra Error lines
  # are inline <ERROR> ink that the schema admits. pgfornament-han-doc went
  # newly invalid in s108 because a pdfTeX-path improvement (16 < 19 errors)
  # flipped this choice to the invalid run. A retry that RESOLVED the leak
  # (no leaked font command left) and did not Fatal or time out is kept
  # regardless of the error-line count; the error-count tie-break stays for
  # retries that did not resolve it (pmhanguljamo-kdoc: luatexko primitives
  # never defined → still leaking → falls through → pdfTeX kept).
  leak_resolved=0
  if (( first_leak > 0 && new_leak == 0 )); then leak_resolved=1; fi
  if (( new_exit == 124 && first_exit != 124 )) || (( new_fatal > first_fatal )) \
     || (( new_fatal == first_fatal && new_err > first_err && leak_resolved == 0 )); then
    for ext in xml log stdout; do mv -f "$out/$name.pdftex.$ext" "$out/$name.$ext" 2>/dev/null || true; done
    exit_code=$first_exit
    printf 'luatex retry was WORSE (exit %s, %s fatals, %s errors vs pdfTeX exit %s, %s fatals, %s errors); kept the pdfTeX run\n' \
      "$new_exit" "$new_fatal" "$new_err" "$first_exit" "$first_fatal" "$first_err" >>"$out/retried_luatex"
  else
    if (( leak_resolved == 1 && new_err > first_err )); then
      printf 'luatex retry resolved the Unicode-font leak (%s → 0 leaked commands) and is kept although it logs more errors (%s vs %s): the leak is a leading <para> that invalidates the frontmatter, the extra errors are inline ink\n' \
        "$first_leak" "$new_err" "$first_err" >>"$out/retried_luatex"
    fi
    rm -f "$out/$name.pdftex."{xml,log,stdout}
  fi
fi
end=$(date +%s.%N)
secs=$(printf '%.1f' "$(echo "$end $start" | awk '{print $1-$2}')")

# A fused eager conversion restarts under --streaming (batch 56dw) and the
# stderr log then holds BOTH attempts: the first one's memory Fatal and errors
# belong to an attempt that produced no output. Count from the last restart
# marker on, so the verdict describes the attempt whose document was written
# (sweep 102 doubled the error counts of every fuse document before this).
counted="$out/$name.counted.log"
if grep -q '^Info:streaming:restart' "$out/$name.log"; then
  awk '/^Info:streaming:restart/ {buf=""; next} {buf=buf $0 "\n"} END {printf "%s", buf}' \
    "$out/$name.log" >"$counted"
else
  counted="$out/$name.log"
fi
# Strict error grep (feedback_strict_vs_lax_error_grep).
errors=$(grep -cE 'Error:[a-z_]+:' "$counted" || true)
# Fatal TARGETS are capitalized (`Fatal:Timeout:TokenLimit`,
# `Fatal:TooManyErrors:MaxLimit`, `Fatal:Mouth:EoF`); only `Fatal:oom:` is
# lowercase, so a `[a-z]` class here counted 25 of sweep 28's ~290 fatals
# (status stayed right only via the exit code). Match any target letter.
fatals=$(grep -cE 'Fatal:[A-Za-z_]+:' "$counted" || true)
warnings=$(grep -cE 'Warning:[a-z_]+:' "$counted" || true)
[[ "$counted" != "$out/$name.log" ]] && rm -f "$counted"

if [[ $exit_code == 124 ]]; then
  status=124 # timeout
elif [[ $fatals -gt 0 || $exit_code -gt 1 ]]; then
  status=3
elif [[ $errors -gt 0 ]]; then
  status=2
elif [[ $warnings -gt 0 ]]; then
  status=1
else
  status=0
fi

printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
  "$bundle" "$name" "$status" "$exit_code" "$errors" "$fatals" "$warnings" "$secs" \
  | tee "$out/verdict.tsv"

# Error-storm logs run to 1.4 GB (quran/texnegar under a 1001-error cascade):
# 2026-09-02 a sweep filled /home mid-run and its verdicts were lost. The
# counts above are already taken; keep the head and tail for triage.
if [[ $(stat -c %s "$out/$name.log") -gt 52428800 ]]; then
  { head -c 20971520 "$out/$name.log"; printf '\n[... log truncated by run_doc.sh (>50 MB) ...]\n'; tail -c 1048576 "$out/$name.log"; } >"$out/$name.log.trunc"
  mv "$out/$name.log.trunc" "$out/$name.log"
fi

[[ $status -le 1 ]] && exit 0
exit "$status"
