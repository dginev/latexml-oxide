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
#   tools/test_with_tl2023.sh                        # full cargo test --release --tests --workspace
#   tools/test_with_tl2023.sh -p latexml --test 50_structure IEEE_test
#
# See docs/SYNC_STATUS.md §"Upstream Perl sync audit" for the CI texlive
# evidence (texlive-binaries 2023.20230311.66589 on Ubuntu noble).

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

exec cargo test --release --tests "$@" -- --test-threads=1
