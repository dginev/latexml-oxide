#!/usr/bin/env bash
# make_formats.sh — Rust-port equivalent of Perl LaTeXML's `make formats`.
#
# Builds latexml_oxide (if needed) and runs --init=latex.ltx to produce
# resources/dumps/latex.dump.txt for the ambient TeX Live install.
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
PROFILE="${PROFILE:-release}"

case "$PROFILE" in
  release) CARGO_PROFILE_FLAG="--release" ;;
  debug)   CARGO_PROFILE_FLAG=""         ;;
  *) echo "PROFILE must be 'release' or 'debug' (got: $PROFILE)" >&2; exit 2 ;;
esac

# Build the binary if it doesn't exist. We build only the latexml_oxide bin
# target to avoid pulling in unrelated workspace members / test deps.
BIN="target/$PROFILE/latexml_oxide"
if [ ! -x "$BIN" ]; then
  echo "[make_formats] building latexml_oxide ($PROFILE)..."
  cargo build $CARGO_PROFILE_FLAG --bin latexml_oxide
fi

# Generate the dump. The binary writes to resources/dumps/latex.dump.txt
# relative to CWD (the repo root, via cd above). Honor LATEXML_DUMP_DIR
# if the user wants a custom location.
if [ -n "${LATEXML_DUMP_DIR:-}" ]; then
  mkdir -p "$LATEXML_DUMP_DIR"
  # Binary defaults to resources/dumps; we symlink or copy afterward.
  "$BIN" --init=latex.ltx
  cp resources/dumps/latex.dump.txt "$LATEXML_DUMP_DIR/latex.dump.txt"
  cp resources/dumps/texlive.version "$LATEXML_DUMP_DIR/texlive.version" 2>/dev/null || true
  echo "[make_formats] dump copied to $LATEXML_DUMP_DIR"
else
  "$BIN" --init=latex.ltx
fi

echo "[make_formats] done. dump: resources/dumps/latex.dump.txt"
kpsewhich --version | head -1 | sed 's/^/[make_formats] texlive: /'
