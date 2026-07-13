#!/usr/bin/env bash
# Portability probe — Phase 0 of the portability ladder (issue #217,
# docs/release/RELEASE_CRITERIA.md §3).
#
# Purpose: turn "may work on macOS" claims into measured facts. Reports
# the state of every native dependency latexml-oxide links against:
#
#   libxml2     (libxml 0.3.12   — pkg-config / LIBXML2 env, bindgen)
#   libxslt     (libxslt 0.1.3   + latexml_post/build.rs bare link line)
#   libkpathsea (kpathsea_sys    — pkg-config only, panics if missing)
#   libmarpa    (libmarpa-sys    — vendored tarball, ./configure && make + bindgen)
#
# then attempts crate-by-crate builds so a failure is attributed to the
# exact native binding, not "the workspace".
#
# Diagnostic by design: individual probes never abort the script, and a
# summary table prints at the end. The script exits 0 unless the final
# workspace build was attempted and failed.
#
# Env knobs:
#   PROBE_SKIP_BUILD=1   probe only, skip all cargo builds (fast local check)
#   PROBE_PROFILE=ci     cargo profile for the workspace build (default: ci)
#
# Must stay bash-3.2 compatible (macOS system bash) and BSD-find clean.

set -u

PROBE_PROFILE="${PROBE_PROFILE:-ci}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

SUMMARY=""
WORKSPACE_BUILD_FAILED=0

section() { printf '\n=== %s ===\n' "$*"; }
note()    { printf '  %s\n' "$*"; }
record()  { SUMMARY="${SUMMARY}
  $1"; }

# ---------------------------------------------------------------------------
section "1. System"
uname -a
if command -v sw_vers >/dev/null 2>&1; then sw_vers; fi
note "arch: $(uname -m)"
note "cores: $(getconf _NPROCESSORS_ONLN 2>/dev/null || echo '?')"
record "system: $(uname -s) $(uname -m)"

# ---------------------------------------------------------------------------
section "2. Toolchain"
for tool in rustc cargo cc clang make pkg-config autoconf libtool; do
  if command -v "$tool" >/dev/null 2>&1; then
    note "$tool: $(command -v "$tool") — $("$tool" --version 2>/dev/null | head -1)"
  else
    note "$tool: NOT FOUND"
  fi
done
note "PKG_CONFIG_PATH=${PKG_CONFIG_PATH:-<unset>}"
note "LIBXML2=${LIBXML2:-<unset>}"
if command -v brew >/dev/null 2>&1; then
  note "brew prefix: $(brew --prefix)"
fi

# ---------------------------------------------------------------------------
section "3. pkg-config probes (what the build scripts will see)"
probe_pc() {
  local mod="$1"
  if pkg-config --exists "$mod" 2>/dev/null; then
    note "FOUND   $mod $(pkg-config --modversion "$mod" 2>/dev/null)"
    note "        cflags: $(pkg-config --cflags "$mod" 2>/dev/null)"
    note "        libs:   $(pkg-config --libs "$mod" 2>/dev/null)"
    record "pkg-config $mod: FOUND ($(pkg-config --modversion "$mod" 2>/dev/null))"
  else
    note "MISSING $mod"
    record "pkg-config $mod: MISSING"
  fi
}
if command -v pkg-config >/dev/null 2>&1; then
  # The four modules our native bindings (would) probe:
  probe_pc libxml-2.0   # rust-libxml build.rs
  probe_pc libxslt      # what a fixed latexml_post/build.rs would probe
  probe_pc libexslt
  probe_pc kpathsea     # kpathsea_sys build.rs — panics today if MISSING
else
  note "pkg-config itself is missing — every probe-based binding will fail"
  record "pkg-config: NOT INSTALLED"
fi

# ---------------------------------------------------------------------------
section "4. TeX distribution"
TL_ROOT=""
if command -v kpsewhich >/dev/null 2>&1; then
  note "kpsewhich: $(command -v kpsewhich)"
  note "version: $(kpsewhich --version 2>/dev/null | head -1)"
  for var in SELFAUTOLOC SELFAUTODIR SELFAUTOPARENT TEXMFDIST TEXMFROOT; do
    note "$var = $(kpsewhich -var-value="$var" 2>/dev/null)"
  done
  TL_ROOT="$(kpsewhich -var-value=SELFAUTOPARENT 2>/dev/null)"
  record "kpsewhich: FOUND ($(command -v kpsewhich))"
else
  note "kpsewhich: NOT FOUND (no TeX distribution on PATH)"
  record "kpsewhich: MISSING"
fi
if command -v pdflatex >/dev/null 2>&1; then
  note "pdflatex: $(pdflatex --version 2>/dev/null | head -1)"
else
  note "pdflatex: NOT FOUND"
fi
if command -v tlmgr >/dev/null 2>&1; then
  note "tlmgr: $(command -v tlmgr)"
fi

# ---------------------------------------------------------------------------
section "5. kpathsea build artifacts (headers / libs / .pc) on disk"
# Where could libkpathsea dev artifacts live? TL tree, Homebrew, MacPorts,
# system paths. This answers the #217 question "does MacTeX bring them?".
KPSE_ROOTS="/usr/local/texlive /Library/TeX /opt/homebrew /opt/local /usr/local/lib /usr/local/include /usr/lib /usr/include"
if [ -n "$TL_ROOT" ] && [ -d "$TL_ROOT" ]; then
  KPSE_ROOTS="$TL_ROOT $KPSE_ROOTS"
fi
if command -v brew >/dev/null 2>&1; then
  KPSE_ROOTS="$KPSE_ROOTS $(brew --prefix)/lib $(brew --prefix)/include $(brew --prefix)/opt"
fi
# (Linux multiarch dirs are covered by the /usr/lib recursion.)

search_artifact() { # label, find-name-pattern
  local label="$1" pat="$2" found=0 root hit
  note "--- $label ($pat) ---"
  for root in $KPSE_ROOTS; do
    [ -d "$root" ] || continue
    while IFS= read -r hit; do
      [ -n "$hit" ] || continue
      note "  $hit"
      found=1
    done <<EOF
$(find "$root" \( -type f -o -type l \) -name "$pat" 2>/dev/null | head -10)
EOF
  done
  if [ "$found" -eq 0 ]; then note "  (none found)"; fi
  if [ "$found" -eq 1 ]; then record "$label: FOUND on disk"; else record "$label: NOT FOUND on disk"; fi
}
# Dedup roots not needed — duplicate hits are fine in a diagnostic.
search_artifact "kpathsea header"  "kpathsea.h"
search_artifact "kpathsea library" "libkpathsea*"
search_artifact "kpathsea.pc"      "kpathsea.pc"

# ---------------------------------------------------------------------------
section "6. libxslt/libexslt artifacts (latexml_post bare-link check)"
# latexml_post/build.rs emits `cargo:rustc-link-lib=xslt`/`exslt` with NO
# search path. That only links if libxslt is in the default linker path.
search_artifact "libxslt library"  "libxslt.*"
search_artifact "libexslt library" "libexslt.*"
if command -v brew >/dev/null 2>&1; then
  if brew list libxslt >/dev/null 2>&1; then
    note "brew libxslt keg: $(brew --prefix libxslt 2>/dev/null)"
    note "keg-only: $(brew info --json=v2 libxslt 2>/dev/null | grep -o '"keg_only": *[a-z]*' | head -1)"
  fi
fi

# ---------------------------------------------------------------------------
section "7. bindgen prerequisite: libclang (needed by libxml + libmarpa-sys)"
LIBCLANG_FOUND=0
if command -v llvm-config >/dev/null 2>&1; then
  note "llvm-config libdir: $(llvm-config --libdir 2>/dev/null)"
fi
for cand in \
  "${LIBCLANG_PATH:-}" \
  "$(command -v xcode-select >/dev/null 2>&1 && xcode-select -p 2>/dev/null)/Toolchains/XcodeDefault.xctoolchain/usr/lib" \
  /Library/Developer/CommandLineTools/usr/lib \
  "$(command -v brew >/dev/null 2>&1 && brew --prefix llvm 2>/dev/null)/lib" \
  /usr/lib/llvm-*/lib /usr/lib/x86_64-linux-gnu /usr/lib; do
  [ -n "$cand" ] && [ -d "$cand" ] || continue
  hits="$(find "$cand" -maxdepth 1 \( -name 'libclang.dylib' -o -name 'libclang.so*' -o -name 'libclang-*.so*' \) 2>/dev/null | head -3)"
  if [ -n "$hits" ]; then
    note "libclang in $cand:"
    printf '%s\n' "$hits" | sed 's/^/    /'
    LIBCLANG_FOUND=1
  fi
done
if [ "$LIBCLANG_FOUND" -eq 1 ]; then record "libclang (bindgen): FOUND"; else
  note "no libclang found in common locations (bindgen may still find one via clang-sys)"
  record "libclang (bindgen): NOT FOUND in common paths"
fi

# ---------------------------------------------------------------------------
if [ "${PROBE_SKIP_BUILD:-0}" = "1" ]; then
  section "8. Build attempts — SKIPPED (PROBE_SKIP_BUILD=1)"
  record "builds: skipped"
else
  section "8. Build attempts (crate-by-crate native-binding attribution)"
  # ANSI-free build logs — report files must be grep-able (the same
  # lesson as CLAUDE.md "Canvas signal integrity").
  export CARGO_TERM_COLOR=never
  # Extend PKG_CONFIG_PATH with every directory holding a discovered
  # kpathsea.pc, so a TL-tree-shipped .pc gets a chance even when the
  # distribution didn't register it. Report the extension explicitly:
  # whatever lands here is the incantation the README must document.
  EXTRA_PC_DIRS=""
  for root in $KPSE_ROOTS; do
    [ -d "$root" ] || continue
    for pc in $(find "$root" -name 'kpathsea.pc' 2>/dev/null | head -5); do
      EXTRA_PC_DIRS="$EXTRA_PC_DIRS:$(dirname "$pc")"
    done
  done
  if [ -n "$EXTRA_PC_DIRS" ]; then
    export PKG_CONFIG_PATH="${PKG_CONFIG_PATH:-}${EXTRA_PC_DIRS}"
    note "extended PKG_CONFIG_PATH=$PKG_CONFIG_PATH"
  fi

  BUILD_LOG="$(mktemp)"
  build_one() { # package name
    local pkg="$1"
    printf '  cargo build -p %s ... ' "$pkg"
    if cargo build -p "$pkg" >"$BUILD_LOG" 2>&1; then
      printf 'OK\n'
      record "build $pkg: OK"
    else
      printf 'FAIL\n'
      record "build $pkg: FAIL"
      note "--- last 60 lines of $pkg failure ---"
      tail -60 "$BUILD_LOG" | sed 's/^/    /'
    fi
  }
  # Dependency-graph leaves first — each one isolates ONE native library:
  build_one libxml     # libxml2 via pkg-config (+ bindgen/libclang)
  build_one libxslt    # libxslt binding crate
  build_one kpathsea   # libkpathsea via pkg-config (panics if unfound)
  build_one marpa      # vendored libmarpa: ./configure && make + bindgen

  note ""
  note "workspace build (--profile $PROBE_PROFILE) ..."
  if cargo build --profile "$PROBE_PROFILE" --workspace >"$BUILD_LOG" 2>&1; then
    note "workspace build: OK"
    record "workspace build ($PROBE_PROFILE): OK"
  else
    WORKSPACE_BUILD_FAILED=1
    note "workspace build: FAIL"
    record "workspace build ($PROBE_PROFILE): FAIL"
    note "--- last 80 lines of workspace failure ---"
    tail -80 "$BUILD_LOG" | sed 's/^/    /'
  fi

  # -------------------------------------------------------------------------
  section "9. Smoke runs (only meaningful if the workspace built)"
  if [ "$WORKSPACE_BUILD_FAILED" -eq 0 ]; then
    if cargo run --profile "$PROBE_PROFILE" --bin latexmlmath_oxide -- '1+1=2' >"$BUILD_LOG" 2>&1; then
      note "latexmlmath '1+1=2': OK"
      record "smoke latexmlmath: OK"
    else
      note "latexmlmath '1+1=2': FAIL"
      record "smoke latexmlmath: FAIL"
      tail -40 "$BUILD_LOG" | sed 's/^/    /'
    fi
    SMOKE_DEST="$(mktemp -d)/hello.xml"
    if cargo run --profile "$PROBE_PROFILE" --bin latexml_oxide -- \
        latexml_oxide/tests/hello/hello.tex --destination="$SMOKE_DEST" >"$BUILD_LOG" 2>&1; then
      note "hello.tex conversion: OK ($SMOKE_DEST)"
      record "smoke hello.tex: OK"
    else
      note "hello.tex conversion: FAIL"
      record "smoke hello.tex: FAIL"
      tail -40 "$BUILD_LOG" | sed 's/^/    /'
    fi
  else
    note "skipped — workspace build failed"
    record "smoke runs: skipped (workspace build failed)"
  fi
  rm -f "$BUILD_LOG"
fi

# ---------------------------------------------------------------------------
section "SUMMARY"
printf '%s\n' "$SUMMARY"
echo
exit "$WORKSPACE_BUILD_FAILED"
