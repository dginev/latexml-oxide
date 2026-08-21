#!/usr/bin/env bash
# tools/lint.sh — the single source of truth for the quality gate.
#
# Called by BOTH `.github/workflows/CI.yml` (the `lint` job) and
# `.githooks/pre-push`, so the two cannot drift. They had: the hook ran the
# xml:id ratchet, fmt and clippy; CI ran fmt, clippy, rustdoc, cargo-deny, the
# vendored-native audit and cargo-machete. Neither was a superset, so a push
# could pass the hook and still fail CI on a check the hook never ran (and the
# ratchet was never enforced in CI at all). This script runs the union.
#
# Unlike CI, it does NOT stop at the first failure: it runs every check and
# reports all of them at the end. CI's fail-fast hides check N+1 behind check
# N, which turns one bad push into a sequence of one-fix-per-run cycles.
#
# Usage:
#   tools/lint.sh                # run everything, summarize failures
#   tools/lint.sh --fail-fast    # stop at the first failure (CI-like)
#
# Env:
#   LINT_PROFILE  cargo profile for clippy/rustdoc. CI sets `ci` (tuned for the
#                 RAM-bounded runner). Local default is the dev profile — per
#                 CLAUDE.md, local dev should not mimic CI's stripped profile.
#   LINT_STRICT   1 = a missing required tool is an error (CI sets this).
#                 unset/0 = warn loudly and continue (local convenience).
#
# Bypass in an emergency: `git push --no-verify`.

set -uo pipefail

cd "$(dirname "$0")/.."

fail_fast=0
[ "${1:-}" = "--fail-fast" ] && fail_fast=1

profile_args=()
[ -n "${LINT_PROFILE:-}" ] && profile_args=(--profile "$LINT_PROFILE")
# NOTE: expand this as ${profile_args[@]+"${profile_args[@]}"}, never as a bare
# "${profile_args[@]}". Under `set -u`, bash before 4.4 treats an EMPTY array's
# expansion as an unbound variable and exits. profile_args is empty on every
# local run (LINT_PROFILE unset — the pre-push path) and macOS ships bash 3.2 as
# /bin/bash, so the bare form would kill the hook there while passing on this
# ubuntu CI and on any bash 5 dev box.

if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
  BOLD=$'\033[1m'; RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; OFF=$'\033[0m'
else
  BOLD=''; RED=''; GREEN=''; YELLOW=''; OFF=''
fi

failed=()   # names of checks that failed
skipped=()  # names of checks skipped for a missing tool

step=0
# run <name> <fix-hint> -- <command...>
run() {
  local name="$1" hint="$2"; shift 3   # drop the literal `--`
  step=$((step + 1))
  printf '%s\n' "${BOLD}[$step] $name${OFF}"
  if "$@"; then
    printf '%s\n\n' "    ${GREEN}ok${OFF}"
  else
    printf '%s\n\n' "    ${RED}FAILED${OFF} — $hint"
    failed+=("$name")
    if [ "$fail_fast" = 1 ]; then
      summarize
      exit 1
    fi
  fi
}

# skip <name> <why> — record loudly. A skip must never read as a pass.
skip() {
  step=$((step + 1))
  printf '%s\n' "${BOLD}[$step] $1${OFF}"
  printf '%s\n\n' "    ${YELLOW}SKIPPED${OFF} — $2"
  skipped+=("$1")
}

summarize() {
  printf '%s\n' "${BOLD}── summary ──${OFF}"
  if [ ${#skipped[@]} -gt 0 ]; then
    printf '%s\n' "${YELLOW}skipped:${OFF} ${skipped[*]}"
  fi
  if [ ${#failed[@]} -gt 0 ]; then
    printf '%s\n' "${RED}failed:${OFF} ${failed[*]}"
    printf '%s\n' "${RED}✗ lint gate failed (${#failed[@]} check(s))${OFF}"
  else
    printf '%s\n' "${GREEN}✓ all checks passed${OFF}"
  fi
}

# A required tool is missing: an error under LINT_STRICT (CI), else a loud skip.
require_or_skip() {
  local tool="$1" name="$2" hint="$3"
  if command -v "$tool" >/dev/null 2>&1; then
    return 0
  fi
  if [ "${LINT_STRICT:-0}" = 1 ]; then
    step=$((step + 1))
    printf '%s\n' "${BOLD}[$step] $name${OFF}"
    printf '%s\n\n' "    ${RED}FAILED${OFF} — $tool not installed (LINT_STRICT=1)"
    failed+=("$name")
  else
    skip "$name" "$tool not installed — $hint"
  fi
  return 1
}

# 1. Cheap source ratchet first (no compile): ban new string-keyed xml: attribute
#    accessors. See docs/archive/XMLID_ACCESSOR_AUDIT_2026-06-08.md.
run "xml:id accessor ratchet" \
  "a new string-keyed xml: accessor landed; use the *_ns form" -- \
  tools/lint_xmlid_accessor.sh

# 1b. Single diagnostic vehicle: no raw log::{info,warn,error}! in workspace
#     crates (bypasses tally, caps, suppression, taxonomy).
run "raw log-diagnostic ban" \
  "use Info!/Warn!/Error!/Fatal! or emit_{info,warn,error,fatal}" -- \
  tools/lint_raw_log_diag.sh

# 2. Formatting must already be applied (check, don't rewrite).
run "rustfmt (check)" \
  "run: cargo fmt --all" -- \
  cargo fmt --all --check

# 3. Clippy across the whole workspace and all targets (tests/benches/bins).
run "clippy (deny warnings, workspace + all targets)" \
  "fix them (try: cargo clippy --fix)" -- \
  cargo clippy --workspace --all-targets ${profile_args[@]+"${profile_args[@]}"} -- -D warnings

# 4. Rustdoc. Gated HERE, on every push/PR, not only in rustdoc.yml — that
#    workflow runs on push to main, so a doc warning introduced by a PR would
#    only surface once it had already landed. A broken intra-doc link is
#    invisible on the deployed site (it renders as dead text, not an error), so
#    the warning is the only signal there is.
run "rustdoc (deny warnings)" \
  "fix the doc warning; do not #[allow] it — the link still dies on the site" -- \
  env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps ${profile_args[@]+"${profile_args[@]}"}

# 5. The complement to cargo-deny, which reasons over crate MANIFESTS and so is
#    blind to vendored C: a `-sys` crate reports its Rust wrapper's license, not
#    the native library's. That gap is why libmarpa statically linked
#    MIT + LGPL-3.0/LGPL-2.1 code into every binary we ever shipped, attributed
#    nowhere, while `cargo deny check licenses` said "licenses ok" throughout.
#    This fails when a crate compiling native code shows up unaudited.
#    See docs/release/LICENSE_INVENTORY.md §D.2 / §A "Scope limit".
#    Note it can also fail with no local change at all: Cargo.lock is gitignored,
#    so a fresh resolution picks up new upstream versions of audited crates
#    (cc 1.3.0 -> 1.4.0 broke main and every open PR on 2026-07-24).
run "vendored native code audit" \
  "re-check what the new version compiles, then update AUDITED + LICENSE_INVENTORY §D.2" -- \
  python3 tools/audit_vendored_natives.py --verbose

# 6. cargo-deny. Best-effort by design: CI covers this with the PINNED
#    EmbarkStudios/cargo-deny-action (see the rationale in CI.yml — the floating
#    @v2 tag broke main once), so the action, not this line, is CI's gate. Here
#    it just lets a local run catch a license/advisory problem before pushing.
if command -v cargo-deny >/dev/null 2>&1; then
  run "cargo-deny (advisories + licenses + bans + sources)" \
    "see deny.toml and docs/release/LICENSE_INVENTORY.md" -- \
    cargo deny --all-features check
else
  skip "cargo-deny" "not installed (CI covers it via the pinned action; install: cargo install --locked cargo-deny)"
fi

# 7. Unused dependencies.
if require_or_skip cargo-machete "cargo-machete (unused dependencies)" \
     "install: cargo install --locked cargo-machete"; then
  run "cargo-machete (unused dependencies)" \
    "drop the unused dep, or allow it in Cargo.toml [package.metadata.cargo-machete]" -- \
    cargo machete
fi

# 8. Font-encoding fontmap drift vs pdftex's own golden (<enc>.enc ∘
#    glyphtounicode.tex). Needs the host TeX tree (kpsewhich + the .enc /
#    glyphtounicode.tex files), so it skips where those are absent — e.g. the
#    CI `lint` job, which installs no texlive. It runs for developers with a full
#    TL and on the pre-push hook; the per-slot conversion guard
#    (cluster_t1_ascii_tilde_circumflex_723) is the CI-side regression net.
if command -v kpsewhich >/dev/null 2>&1 && [ -n "$(kpsewhich glyphtounicode.tex 2>/dev/null)" ]; then
  run "fontmap drift (vs pdftex glyphtounicode golden)" \
    "a text-encoding fontmap slot drifted from <enc>.enc ∘ glyphtounicode.tex; fix the value or allowlist it with a reason in tools/fontmap_drift.py" -- \
    python3 tools/fontmap_drift.py
else
  skip "fontmap drift" "no TeX tree (kpsewhich/glyphtounicode.tex absent); the T1 conversion guard covers CI"
fi

summarize
[ ${#failed[@]} -eq 0 ]
