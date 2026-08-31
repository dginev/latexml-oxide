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

# RAM guard (see feedback_sandbox_ram_guard): 6 GiB virtual.
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
ORACLE="$OUTROOT/oracle_verdicts.tsv"
if [[ -f "$ORACLE" ]] && grep -qP "^$bundle\t$name\tlualatex\t" "$ORACLE"; then
  PRELOAD='[rawstyles,rawclasses,luatex]latexml.sty'
fi

start=$(date +%s.%N)
timeout "$TIMEOUT_S" "$BIN" \
  --preload="$PRELOAD" \
  --xml \
  --timeout="$TIMEOUT_S" \
  --max-memory=6144 \
  --dest="$out/$name.xml" \
  "$TEX" >"$out/$name.stdout" 2>"$out/$name.raw.log"
exit_code=$?
end=$(date +%s.%N)
secs=$(printf '%.1f' "$(echo "$end $start" | awk '{print $1-$2}')")

# ANSI-strip the log (older/current binaries may color when not TTY-gated).
sed 's/\x1b\[[0-9;]*m//g' "$out/$name.raw.log" >"$out/$name.log"
rm -f "$out/$name.raw.log"

# Strict error grep (feedback_strict_vs_lax_error_grep).
errors=$(grep -c '^Error:[a-z]' "$out/$name.log" || true)
fatals=$(grep -c '^Fatal:[a-z]' "$out/$name.log" || true)
warnings=$(grep -c '^Warning:[a-z]' "$out/$name.log" || true)

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

[[ $status -le 1 ]] && exit 0
exit "$status"
