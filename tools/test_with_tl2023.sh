#!/bin/bash
# Run the Rust test suite (or a single -p/--test selector) against
# ~/data/texlive2023 instead of the system /usr/local/texlive/2025.
#
# Purpose: reproduce the CI Ubuntu-24.04 texlive-2023 environment locally
# so that texlive-version-sensitive tests (e.g. 50_structure::IEEE_test,
# which depends on IEEEtran.cls packaging differences across TL releases)
# can be validated without waiting on CI.
#
# Usage:
#   tools/test_with_tl2023.sh                          # full cargo test --profile release-light --tests --workspace
#   tools/test_with_tl2023.sh -p latexml --test 50_structure IEEE_test
#   INSTALL_CI_PACKAGES=1 tools/test_with_tl2023.sh    # run tlmgr install for CI-parity collections first
#   REBUILD_PERL_FORMATS=1 tools/test_with_tl2023.sh   # rebuild Perl dumps under TL2023 before tests
#
# See docs/SYNC_STATUS.md §"Upstream Perl sync audit" for the CI texlive
# evidence (texlive-binaries 2023.20230311.66589 on Ubuntu noble).
#
# CI-parity package set: Ubuntu-noble CI installs
#   texlive texlive-latex-extra texlive-science
#   texlive-lang-{german,french,greek,cyrillic,european}
# which map to TL2023 collections:
#   collection-latexextra     collection-mathscience
#   collection-lang{german,french,greek,cyrillic,european}
#   collection-fontsrecommended  collection-fontsextra  collection-pictures
# INSTALL_CI_PACKAGES=1 runs these via tlmgr before the probe banner.

set -euo pipefail

TL2023_ROOT="${HOME}/data/texlive2023"
TL2023_BIN="${TL2023_ROOT}/bin/x86_64-linux"

if [ ! -x "${TL2023_BIN}/kpsewhich" ]; then
  echo "error: TL2023 install not found at ${TL2023_BIN}" >&2
  echo "       run install-tl with profile at ~/data/tl2023-setup/profile.txt first" >&2
  exit 2
fi

# Front-load the TL2023 bin so kpsewhich/latex/pdflatex resolve to 2023
# binaries; core rust-libxml / system libs unchanged.
export PATH="${TL2023_BIN}:${PATH}"

# Optional CI-parity package bootstrap. Safe to re-run: tlmgr install is
# idempotent, skips already-installed packages, and is bounded by the
# TL2023 tlnet-final snapshot.
#
# Mirror note: the default install-tl config points at ftp.math.utah.edu,
# which was observed to be connection-unreachable (port 443 timeout) as
# of 2026-04-23. Chemnitz's TUG-historic mirror is an alternate with the
# identical frozen tlnet-final tree — pin to it when the default mirror
# hangs. Override via TLMGR_REPO if you want a different mirror.
TLMGR_REPO="${TLMGR_REPO:-https://ftp.tu-chemnitz.de/pub/tug/historic/systems/texlive/2023/tlnet-final}"
if [ "${INSTALL_CI_PACKAGES:-0}" = "1" ]; then
  echo "=== Installing CI-equivalent TL2023 collections ==="
  echo "    repo=${TLMGR_REPO}"
  current_repo=$("${TL2023_BIN}/tlmgr" option repository 2>/dev/null | awk -F': ' '{print $2}' | head -1)
  if [ "${current_repo}" != "${TLMGR_REPO}" ]; then
    "${TL2023_BIN}/tlmgr" option repository "${TLMGR_REPO}"
  fi
  "${TL2023_BIN}/tlmgr" install \
    collection-latexextra collection-mathscience \
    collection-langgerman collection-langfrench \
    collection-langgreek collection-langcyrillic collection-langeuropean \
    collection-fontsrecommended collection-fontsextra collection-pictures
  echo "=== tlmgr install done ==="
  echo
fi

# Announce what we're using so the test output is self-documenting.
# IEEEtran.cls (and similar probe targets) may be missing from a minimal
# local TL2023 install — the CI runner pulls `texlive-science` via apt which
# includes it, but `install-tl --profile minimal` does not. The probe is
# diagnostic only, so `|| true` each lookup — we want the wrapper to fall
# through to the actual build/test work, not die on a missing probe target.
echo "=== Running with TL2023 ==="
kpsewhich --version | head -1
echo "IEEEtran.cls: $(kpsewhich IEEEtran.cls 2>/dev/null || echo '<not installed>')"
echo "article.cls: $(kpsewhich article.cls 2>/dev/null || echo '<not installed>')"
echo "==========================="
echo

# If IEEEtran is missing, the 50_structure::IEEE_test cannot exercise the
# real CI package set — flag that clearly so the wrapper's report is honest.
if ! kpsewhich IEEEtran.cls >/dev/null 2>&1; then
  echo "WARNING: IEEEtran.cls not found in local TL2023. To install:"
  echo "  ${TL2023_BIN}/tlmgr install IEEEtran"
  echo "without it, IEEE-related tests won't reproduce CI behaviour locally."
  echo
fi

# Regenerate the kernel dump against TL2023 before the test, otherwise
# cargo test will use the stale dump built with TL2025.
cd "$(dirname "$0")/.."
./tools/make_formats.sh

# The Perl LaTeXML tree under LaTeXML/ has its own texlive-version-
# sensitive format dumps (LaTeXML/blib/.../plain_dump.pool.ltxml and
# latex_dump.pool.ltxml), produced by `make formats` in that tree.
# Several Rust tests compare Rust output against Perl output produced
# by those dumps, so a TL-version mismatch between the Perl dumps and
# the ambient kpsewhich can surface as spurious test failures (e.g.
# the false-alarm IEEE_test regression that traced to IEEEtran.cls
# moving between TL2023 and TL2025).
#
# Rebuilding the Perl dumps is multi-minute and requires the LaTeXML/
# tree to have been `perl Makefile.PL && make`-prepared (which is
# already the case for this checkout). Opt in with:
#     REBUILD_PERL_FORMATS=1 tools/test_with_tl2023.sh ...
if [ "${REBUILD_PERL_FORMATS:-0}" = "1" ]; then
  if [ ! -f LaTeXML/Makefile ]; then
    echo "error: LaTeXML/Makefile missing — run \`cd LaTeXML && perl Makefile.PL\` first" >&2
    exit 3
  fi
  echo "=== Rebuilding Perl LaTeXML formats against TL2023 ==="
  ( cd LaTeXML && make formats )
  echo
fi

exec cargo test --profile release-light --tests "$@" -- --test-threads=1
