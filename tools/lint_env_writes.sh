#!/usr/bin/env bash
# lint_env_writes.sh — the process environment is written only before the first thread exists.
#
# glibc's `getenv` is lock-free by contract. A `setenv`/`putenv` that adds a name reallocates `environ` and frees
# the old array, so a `getenv` walking it on another thread reads freed memory. Rust's `std::env` lock covers
# neither C writers (libkpathsea's `kpathsea_set_program_name` `putenv`s) nor C readers (libkpathsea, chrono's `TZ`
# in `localtime_r`). The only sound ordering is: every write before any second thread.
# Witnesses (docs/performance/STABILITY_WITNESSES.md, Cluster J): sweep #35's general-protection fault in `getenv`
# (pstool/pstool); 9 SIGSEGV in 540 conversions on 0.7.5 (PR 823); 7 of 200 `--init=latex.ltx` runs on aarch64
# (PR 812).
#
# This lint FAILS on a `set_var`/`remove_var` or a libc `setenv`/`putenv`/`unsetenv` call in the workspace's
# library and binary sources unless its (file, variable) pair is allowlisted below: a start-up write that runs
# before the first spawn (say where), or test-only code. A new site either passes its value in-process instead of
# through the environment, or joins the list with its ordering argument. Integration tests (`*/tests/`) are not
# scanned. A call whose variable is not a string literal on the same line matches only a `*` entry.

set -euo pipefail
cd "${LINT_ROOT:-$(dirname "$0")/..}"

# file|VARIABLE|why it is sound
ALLOW=(
  "latexml_core/src/util/pathname.rs|TEXMFROOT|select_kpaths, forced by init_kpathsea() on the converting thread before the kpathsea-prewarm spawn (both binaries)"
  "latexml_oxide/bin/latexml_oxide.rs|LATEXML_INI_MODE|--init, set before init_kpathsea() and the kpathsea-prewarm spawn"
  "latexml_oxide/src/util/test.rs|LATEXML_RSS_CAP_BYTES|test-utils: under a Once at the top of each generated test, before any conversion"
  "latexml_oxide/src/identity.rs|SOURCE_DATE_EPOCH|a #[test]"
  "latexml_post/src/graphics_cache.rs|LATEXML_GRAPHICS_CACHE_DIR|#[cfg(test)], under the binary-wide env_lock"
  "latexml_post/src/test_env.rs|*|#[cfg(test)] EnvGuard, under the binary-wide env_lock"
  "latexml_contrib/src/script_bindings/tests.rs|LATEXML_DISABLE_SHELL_ESCAPE|#[cfg(test)]"
)

ROOTS=()
for d in latexml_core/src latexml_codegen/src latexml_engine/src latexml_package/src latexml_math_parser/src \
  latexml_contrib/src latexml_oxide/src latexml_oxide/bin latexml_post/src; do
  [ -d "$d" ] && ROOTS+=("$d")
done

hits=$(grep -rnE --include='*.rs' '(\b(set_var|remove_var)[[:space:]]*\(|\blibc::(setenv|putenv|unsetenv)\b)' "${ROOTS[@]}" \
  | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)

violations=""
while IFS= read -r hit; do
  [ -z "$hit" ] && continue
  file=${hit%%:*}
  var=$(printf '%s\n' "$hit" | sed -nE 's/.*(set_var|remove_var)[[:space:]]*\([[:space:]]*"([^"]+)".*/\2/p')
  allowed=0
  for entry in "${ALLOW[@]}"; do
    afile=${entry%%|*}
    rest=${entry#*|}
    avar=${rest%%|*}
    if [ "$file" = "$afile" ] && { [ "$avar" = "*" ] || { [ -n "$var" ] && [ "$avar" = "$var" ]; }; }; then
      allowed=1
      break
    fi
  done
  [ "$allowed" = 0 ] && violations+="$hit"$'\n'
done <<< "$hits"

if [ -n "$violations" ]; then
  echo "ERROR: an environment write outside the start-up allowlist. A write after the first thread spawn is a"
  echo "use-after-free of \`environ\` under any concurrent getenv (STABILITY_WITNESSES Cluster J). Pass the value"
  echo "in-process, or add the (file, variable) pair to tools/lint_env_writes.sh with the reason it precedes every spawn."
  echo
  printf '%s' "$violations"
  exit 1
fi
echo "OK: every environment write is an allowlisted start-up or test-only site (${#ALLOW[@]} entries)."
