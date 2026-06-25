#!/usr/bin/env bash
# miri_check.sh — run Miri (the MIR interpreter, UB detector) over the
# workspace's FFI-FREE pure-Rust `unsafe`.
#
# WHY SCOPED, NOT WHOLE-WORKSPACE: most of this project's `unsafe` is C FFI
# (libc waitpid/poll/fork in the LSP server, kpathsea, libxml2/libxslt via
# dlsym, getrusage, signal handlers) — Miri CANNOT execute extern "C" calls,
# so a blanket `cargo miri test` aborts on the first FFI call. See
# docs/SAFETY.md for the full `unsafe` inventory and which category each site
# is in. The category with genuine UB potential that Miri *can* check is the
# string-interner / arena (`latexml_core/src/common/arena.rs`): unchecked
# `resolve_unchecked` bounds-skips (cat. B) and the re-entrant `&mut *ptr`
# (cat. C). That is what this script exercises.
#
# FLAGS:
#  -Zmiri-disable-isolation : allow time/getrandom (harmless for these tests)
#  -Zmiri-ignore-leaks      : the engine's interner is a process-lifetime
#                             `once_cell::Lazy<RefCell<StringInterner>>` that
#                             never drops BY DESIGN; without this flag Miri
#                             reports the singleton as a "leak" (not UB).
#
# Usage: tools/miri_check.sh            # run the scoped arena UB check
#        tools/miri_check.sh <filter>   # override the test-name filter
set -euo pipefail

FILTER="${1:-arena}"
export MIRIFLAGS="${MIRIFLAGS:-} -Zmiri-disable-isolation -Zmiri-ignore-leaks"

if ! rustup component list --toolchain nightly 2>/dev/null | grep -q 'miri.*(installed)'; then
  echo "miri not installed; installing for the nightly toolchain…" >&2
  rustup component add --toolchain nightly miri
fi

echo "==> cargo +nightly miri test -p latexml_core --lib ${FILTER}" >&2
echo "    MIRIFLAGS=${MIRIFLAGS}" >&2
exec cargo +nightly miri test -p latexml_core --lib "${FILTER}"
