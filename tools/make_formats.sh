#!/usr/bin/env bash
# make_formats.sh — Rust-port equivalent of Perl LaTeXML's `make formats`.
#
# Builds latexml_oxide (if needed) and runs TWO --init invocations
# (mirroring Perl LaTeXML/Makefile.PL `formats` target which has two
# distinct dumps). Dumps are written with versioned filenames tagged by
# the ambient TeX Live release year (auto-detected via
# `kpsewhich -var-value=SELFAUTOPARENT`):
#   --init=plain.tex → resources/dumps/plain.YYYY.dump.txt
#   --init=latex.ltx → resources/dumps/latex.YYYY.dump.txt
# Multiple TL-year dumps coexist in resources/dumps/; the runtime picks
# the one that matches the ambient TL (with most-recent-year fallback).
#
# Call this once after checkout, after any TeX Live upgrade, or before
# running the test suite when matching a specific texlive is required
# (e.g. in CI). The Rust build does NOT embed the dump — it is read
# from disk at runtime (see latexml_package/build.rs), so regenerating
# requires no rebuild.
#
# Usage:
#   tools/make_formats.sh           # release build, latex.ltx
#   PROFILE=debug tools/make_formats.sh   # debug build (faster compile)
#
# Env:
#   PROFILE              release (default) | debug
#   LATEXML_DUMP_DIR     optional override — where to write the dump
#                        (default: resources/dumps/)

set -euo pipefail

cd "$(dirname "$0")/.."
PROFILE="${PROFILE:-debug}"

case "$PROFILE" in
  release) CARGO_PROFILE_FLAG="--release" ;;
  debug)   CARGO_PROFILE_FLAG=""          ;;
  ci)      CARGO_PROFILE_FLAG="--profile ci" ;;
  *) echo "PROFILE must be 'release', 'debug', or 'ci' (got: $PROFILE)" >&2; exit 2 ;;
esac

# Build the binary if it doesn't exist. We build only the latexml_oxide bin
# target to avoid pulling in unrelated workspace members / test deps.
BIN="target/$PROFILE/latexml_oxide"
if [ ! -x "$BIN" ]; then
  echo "[make_formats] building latexml_oxide ($PROFILE)..."
  cargo build $CARGO_PROFILE_FLAG --bin latexml_oxide
fi

# Generate BOTH dumps — strict mirror of Perl Makefile.PL `formats`
# target (LaTeXML/Makefile.PL):
#
#   $(INST_FMTDIR)/plain_dump.pool.ltxml: latexml --init=plain.tex
#   $(INST_FMTDIR)/latex_dump.pool.ltxml: latexml --init=latex.ltx
#
# Each captures the kernel-state delta from raw-loading its respective
# format file. Plain.tex contributes core TeX bindings (the
# \settabs/\sett@b chain, \matrix, \cases, etc.) that latex.ltx
# doesn't redefine — without the plain dump those CSes are missing
# at runtime. The binary writes each to resources/dumps/<basename>.dump.txt
# relative to CWD.
TL_YEAR="$(kpsewhich -var-value=SELFAUTOPARENT 2>/dev/null | sed -n 's:.*/\([0-9]\{4\}\)$:\1:p')"
if [ -z "$TL_YEAR" ]; then
  echo "[make_formats] could not detect TeXLive year via kpsewhich SELFAUTOPARENT" >&2
  exit 3
fi

echo "[make_formats] generating plain.${TL_YEAR}.dump.txt (--init=plain.tex)..."
"$BIN" --init=plain.tex

echo "[make_formats] generating latex.${TL_YEAR}.dump.txt (--init=latex.ltx)..."
"$BIN" --init=latex.ltx

if [ -n "${LATEXML_DUMP_DIR:-}" ]; then
  mkdir -p "$LATEXML_DUMP_DIR"
  cp "resources/dumps/plain.${TL_YEAR}.dump.txt" "$LATEXML_DUMP_DIR/" 2>/dev/null || true
  cp "resources/dumps/latex.${TL_YEAR}.dump.txt" "$LATEXML_DUMP_DIR/"
  cp "resources/dumps/texlive.${TL_YEAR}.version" "$LATEXML_DUMP_DIR/" 2>/dev/null || true
  echo "[make_formats] dumps copied to $LATEXML_DUMP_DIR"
fi

echo "[make_formats] done."
echo "[make_formats]   plain dump: resources/dumps/plain.${TL_YEAR}.dump.txt"
echo "[make_formats]   latex dump: resources/dumps/latex.${TL_YEAR}.dump.txt"
kpsewhich --version | head -1 | sed 's/^/[make_formats] texlive: /'
