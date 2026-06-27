#!/usr/bin/env bash
# miri_check.sh — run Miri (the MIR interpreter, UB detector) over the
# workspace's FFI-FREE pure-Rust `unsafe`.
#
# WHY SCOPED, NOT WHOLE-WORKSPACE: most of this project's `unsafe` is C FFI
# (libc waitpid/poll/fork in the LSP server, kpathsea, libxml2/libxslt via
# dlsym, getrusage, signal handlers) — Miri CANNOT execute extern "C" calls,
# so a blanket `cargo miri test` aborts on the first FFI call. See
# docs/SAFETY.md for the full `unsafe` inventory and which category each site
# is in. The categories with genuine UB potential that Miri *can* check are:
#  (1) the string-interner / arena (`latexml_core/src/common/arena.rs`): unchecked
#      `resolve_unchecked` bounds-skips (cat. B) and the re-entrant `&mut *ptr`
#      (cat. C);
#  (2) the `runtime-bindings` (Rhai) trampoline's re-entrant `&mut Document`
#      round-trip (PR #248 B1), modeled libxml2-free in
#      `latexml_core::runtime_bindings_reentrancy_model` so Miri can adjudicate the
#      reborrow aliasing under BOTH Stacked and Tree Borrows.
# That is what this script exercises.
#
# FLAGS:
#  -Zmiri-disable-isolation : allow time/getrandom (harmless for these tests)
#  -Zmiri-ignore-leaks      : the engine's interner is a process-lifetime
#                             `once_cell::Lazy<RefCell<StringInterner>>` that
#                             never drops BY DESIGN; without this flag Miri
#                             reports the singleton as a "leak" (not UB).
#
# Usage: tools/miri_check.sh            # run arena (SB) + reentrancy model (SB+TB)
#        tools/miri_check.sh <filter>   # run ONLY this test-name filter (SB)
set -euo pipefail

export MIRIFLAGS="${MIRIFLAGS:-} -Zmiri-disable-isolation -Zmiri-ignore-leaks"

if ! rustup component list --toolchain nightly 2>/dev/null | grep -q 'miri.*(installed)'; then
  echo "miri not installed; installing for the nightly toolchain…" >&2
  rustup component add --toolchain nightly miri
fi

run() { # <label> <extra-miriflags> <test-filter>
  echo "==> [$1] cargo +nightly miri test -p latexml_core --lib $3" >&2
  echo "    MIRIFLAGS=${MIRIFLAGS} $2" >&2
  MIRIFLAGS="${MIRIFLAGS} $2" cargo +nightly miri test -p latexml_core --lib "$3"
}

if [ "$#" -ge 1 ]; then
  # Explicit filter: single Stacked-Borrows run (ad-hoc / back-compat).
  run "stacked" "" "$1"
else
  # Default (CI): arena under Stacked Borrows, plus the runtime-bindings
  # re-entrancy model under BOTH Stacked and Tree Borrows (PR #248 B1 — the
  # whole point of the model is the aliasing soundness, which Tree Borrows
  # checks more strictly).
  run "stacked" "" "arena"
  run "stacked" "" "reentrancy_model"
  run "tree-borrows" "-Zmiri-tree-borrows" "reentrancy_model"
fi
