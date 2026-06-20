#!/usr/bin/env bash
# Profile-Guided Optimization (PGO) profile generation for the release binary.
#
# This is PASS 1-3 of the two-pass PGO pipeline documented in
# docs/PERFORMANCE.md ("Build-pipeline optimization roadmap"):
#
#   1. Instrument  — build latexml_oxide with `-Cprofile-generate`.
#   2. Train       — run the instrumented binary over a DIVERSE arXiv/TeX
#                    slice so LLVM records which catcode arms, macro bodies,
#                    grammar rules, and digestion paths are actually hot.
#   3. Merge       — `llvm-profdata merge` the raw `.profraw` files into a
#                    single `target/pgo/merged.profdata`.
#
# PASS 4 (Optimize) is `make_release.sh` itself: re-run it with
# `PGO_PROFILE=target/pgo/merged.profdata` and it feeds `-Cprofile-use` into
# the existing maxperf (fat-LTO / CGU-1) build — PGO stacks with LTO because
# the profile informs LTO inlining. Keeping the optimize+package step in the
# single release path (rather than duplicating it here) means there is exactly
# ONE place that produces a shippable artifact.
#
# Why PGO here: latexml-oxide is CPU-bound and intensely branch-heavy
# (mouth catcode dispatch, gullet macro lookup/expansion, stomach digestion,
# the Marpa-style math grammar). The compiler cannot statically predict the
# hot arm of any of those; a runtime profile can. Interpreter/parser workloads
# typically gain ~10-20%, a proportional fleet tasks/s increase.
#
# Usage:
#   # Generate the profile from the in-repo diverse test corpus (portable; CI):
#   bash tools/make_release_pgo.sh
#
#   # Best results — train on a real diverse arXiv slice (the named training
#   # set; math-heavy / TikZ / plain / expl3, NOT one paper):
#   PGO_TRAIN_DIR=/data/arxiv/2106 bash tools/make_release_pgo.sh
#
#   # Then build + package the PGO-optimized release:
#   PGO_PROFILE=target/pgo/merged.profdata bash tools/make_release.sh
#
# Tunables (env):
#   PGO_TRAIN_DIR       — directory tree to glob `*.tex` from for training.
#                         Default: a curated diverse slice of latexml_oxide/tests.
#   PGO_TRAIN_LIMIT     — cap the number of training documents (default 60).
#   PGO_INSTRUMENT_PROFILE — cargo profile for the instrument build
#                         (default `release`; the profile shape need not match
#                         the optimize profile — profiles are function-keyed).
#                         Set `dev` for a fast mechanics check.
#   PGO_TIMEOUT_SECS    — per-document training timeout (default 60).
#
# The profile is workload+code-specific: re-run this at release time whenever
# the engine changes meaningfully (not set-and-forget). It produces NO runtime
# artifacts — the self-contained-binary guarantee is intact.

set -euo pipefail

# --- locate workspace root --------------------------------------------------
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

# --- locate llvm-profdata from the active toolchain sysroot -----------------
# Shipped by the `llvm-tools-preview` component (rustup component add
# llvm-tools-preview). Resolving it from the sysroot guarantees the version
# matches the rustc that emitted the instrumentation (mismatched llvm-profdata
# silently produces an unusable profile).
sysroot="$(rustc --print sysroot)"
host_triple="$(rustc -vV | sed -n 's/^host: //p')"
llvm_profdata="${sysroot}/lib/rustlib/${host_triple}/bin/llvm-profdata"
if [[ ! -x "${llvm_profdata}" ]]; then
  echo "make_release_pgo: llvm-profdata not found at ${llvm_profdata}" >&2
  echo "  Install it for the pinned toolchain:" >&2
  echo "    rustup component add llvm-tools-preview" >&2
  exit 1
fi

# --- tunables ---------------------------------------------------------------
instrument_profile="${PGO_INSTRUMENT_PROFILE:-release}"
train_limit="${PGO_TRAIN_LIMIT:-60}"
train_timeout="${PGO_TIMEOUT_SECS:-60}"

pgo_dir="${repo_root}/target/pgo"
raw_dir="${pgo_dir}/raw"
merged="${pgo_dir}/merged.profdata"

echo "make_release_pgo: instrument_profile=${instrument_profile} train_limit=${train_limit}"
echo "make_release_pgo: llvm-profdata=${llvm_profdata}"

# --- assemble the training corpus -------------------------------------------
# Default: a curated, DIVERSE slice of the in-repo test corpus so the script
# is portable (works in any checkout, including CI, with no external data).
# Operators should override PGO_TRAIN_DIR with a real arXiv sandbox slice for a
# production-grade profile — broader inputs exercise more of the hot path.
declare -a train_files=()
if [[ -n "${PGO_TRAIN_DIR:-}" ]]; then
  if [[ ! -d "${PGO_TRAIN_DIR}" ]]; then
    echo "make_release_pgo: PGO_TRAIN_DIR='${PGO_TRAIN_DIR}' is not a directory" >&2
    exit 1
  fi
  echo "make_release_pgo: training corpus = ${PGO_TRAIN_DIR} (*.tex)"
  while IFS= read -r f; do train_files+=("${f}"); done \
    < <(find "${PGO_TRAIN_DIR}" -name '*.tex' -type f | sort | head -n "${train_limit}")
else
  # Curated diverse default: one representative per major engine subsystem.
  # math grammar, alignments, tikz, expl3, ams, complex tables, fonts, structure.
  echo "make_release_pgo: training corpus = curated in-repo slice (set PGO_TRAIN_DIR for arXiv)"
  for sub in parse math complex tikz expl3 ams trip structure fonts encoding; do
    while IFS= read -r f; do train_files+=("${f}"); done \
      < <(find "latexml_oxide/tests/${sub}" -name '*.tex' -type f 2>/dev/null | sort | head -8)
  done
  # Trim to the limit deterministically.
  if (( ${#train_files[@]} > train_limit )); then
    train_files=("${train_files[@]:0:${train_limit}}")
  fi
fi

if (( ${#train_files[@]} == 0 )); then
  echo "make_release_pgo: no training *.tex found — nothing to profile" >&2
  exit 1
fi
echo "make_release_pgo: ${#train_files[@]} training documents"

# --- PASS 1: instrument build -----------------------------------------------
# `-Cprofile-generate` writes per-run `.profraw` files at execution time. We
# build the SAME feature set as the release (`--no-default-features
# --features runtime-bindings`) so the instrumented code shape matches the
# shipped one. A clean raw dir avoids merging stale profiles from a prior run.
rm -rf "${raw_dir}" "${merged}"
mkdir -p "${raw_dir}"

echo "make_release_pgo: PASS 1 — instrument build (profile=${instrument_profile})"
RUSTFLAGS="${RUSTFLAGS:-} -Cprofile-generate=${raw_dir}" \
  cargo build --no-default-features --features runtime-bindings \
    --profile "${instrument_profile}" --bin latexml_oxide

# Resolve the instrumented binary path (dev profile lands in target/debug).
case "${instrument_profile}" in
  dev|test) bin_subdir="debug" ;;
  *)        bin_subdir="${instrument_profile}" ;;
esac
instr_bin="${repo_root}/target/${bin_subdir}/latexml_oxide"
if [[ ! -x "${instr_bin}" ]]; then
  echo "make_release_pgo: instrumented binary missing at ${instr_bin}" >&2
  exit 1
fi

# Discard the .profraw emitted DURING the build. `-Cprofile-generate` via
# RUSTFLAGS instruments every crate, including proc-macros and build.rs, which
# EXECUTE while cargo compiles and dump profiles for those (entirely different)
# executables. Their function profiles do not correspond to latexml_oxide's and
# are pure noise in the merge — clear them so only training (runtime) data
# feeds the optimized build.
rm -f "${raw_dir}"/*.profraw

# --- PASS 2: train ----------------------------------------------------------
# Convert each training document with the instrumented binary. `%m` enables
# LLVM's online-merge pool (bounded raw-file count); `%p` keeps per-pid files
# distinct. Conversions that error/timeout still emit a profile for the paths
# they did exercise, so failures are tolerated (`|| true`) — we are profiling
# code execution, not checking correctness here.
train_out="$(mktemp -d)"
trap 'rm -rf "${train_out}"' EXIT
export LLVM_PROFILE_FILE="${raw_dir}/pgo-%m-%p.profraw"

echo "make_release_pgo: PASS 2 — training (${#train_files[@]} docs, ${train_timeout}s each)"
n=0
for f in "${train_files[@]}"; do
  n=$((n + 1))
  printf '  [%d/%d] %s\n' "${n}" "${#train_files[@]}" "${f#${repo_root}/}"
  timeout "${train_timeout}" "${instr_bin}" \
    --destination "${train_out}/out.xml" --quiet "${f}" >/dev/null 2>&1 || true
  rm -f "${train_out}/out.xml"
done
unset LLVM_PROFILE_FILE

# --- PASS 3: merge ----------------------------------------------------------
shopt -s nullglob
raw_files=("${raw_dir}"/*.profraw)
shopt -u nullglob
if (( ${#raw_files[@]} == 0 )); then
  echo "make_release_pgo: no .profraw produced — training exercised no instrumented code" >&2
  exit 1
fi
echo "make_release_pgo: PASS 3 — merge ${#raw_files[@]} raw profiles"
"${llvm_profdata}" merge -o "${merged}" "${raw_files[@]}"

if [[ ! -s "${merged}" ]]; then
  echo "make_release_pgo: merged profile ${merged} is empty" >&2
  exit 1
fi

profile_kb="$(du -k "${merged}" | cut -f1)"
echo
echo "make_release_pgo: ✓ merged PGO profile → ${merged} (${profile_kb} KiB)"
echo "make_release_pgo: next — build + package the optimized release:"
echo
echo "    PGO_PROFILE=${merged} bash tools/make_release.sh"
echo
