#!/usr/bin/env bash
# Build a self-contained, PIC static libkpathsea.a from the TeX Live source and
# point the kpathsea_sys build at it (KPATHSEA_LIB_DIR + KPATHSEA_STATIC, emitted
# to $GITHUB_ENV when running under Actions). Used by the Linux and macOS release
# legs in .github/workflows/release.yml.
#
# Why static, from source:
#   * Static link => the release binary does IN-PROCESS kpathsea lookups (fast)
#     with NO runtime libkpathsea dependency, so it launches on MacTeX / no-TeX
#     and degrades to empty lookups instead of failing to load.
#   * From source (not the system libkpathsea-dev .a) because that archive is
#     NON-PIC, and our proc-macro crate (latexml_codegen) is a cdylib that links
#     kpathsea transitively — a non-PIC archive is rejected there ("R_X86_64_PC32
#     relocation ... recompile with -fPIC"). `--with-pic` produces a PIC archive
#     that links into BOTH the cdylib and the final binary. macOS Mach-O is PIC
#     by default, so `--with-pic` is a harmless no-op there; Homebrew also ships
#     no static .a at all, so source-building is the only macOS option.
#
# Pinned to a commit whose kpathsea matches the kpathsea_sys bindgen bindings
# (6.4.1 / TL2025): the `format_info` struct walk in the `kpathsea` crate depends
# on the layout. Bump KPSE_REF in lockstep with a bindings regeneration.
set -euo pipefail

KPSE_REF="${KPSE_REF:-def12ffd4d6e46bae03b3e5c7ff6f5f14dced3ab}"
SRC_DIR="${KPSE_SRC_DIR:-/tmp/tlsrc}"

echo "[build_static_kpathsea] fetching kpathsea source @ ${KPSE_REF} (sparse, shallow)"
rm -rf "$SRC_DIR"
mkdir -p "$SRC_DIR"
cd "$SRC_DIR"
git init -q
git remote add origin https://github.com/TeX-Live/texlive-source.git
git sparse-checkout init --cone
git sparse-checkout set texk/kpathsea build-aux m4
git fetch --depth 1 --filter=blob:none origin "$KPSE_REF"
git checkout -q FETCH_HEAD

cd texk/kpathsea
echo "[build_static_kpathsea] configure --enable-static --with-pic"
./configure --enable-static --disable-shared --disable-dependency-tracking --with-pic >/dev/null
echo "[build_static_kpathsea] make libkpathsea.la"
make libkpathsea.la >/dev/null

libdir="${SRC_DIR}/texk/kpathsea/.libs"
if [[ ! -f "${libdir}/libkpathsea.a" ]]; then
  echo "[build_static_kpathsea] ERROR: libkpathsea.a not produced" >&2
  exit 1
fi
echo "[build_static_kpathsea] built ${libdir}/libkpathsea.a"

if [[ -n "${GITHUB_ENV:-}" ]]; then
  {
    echo "KPATHSEA_LIB_DIR=${libdir}"
    echo "KPATHSEA_STATIC=1"
  } >> "$GITHUB_ENV"
  echo "[build_static_kpathsea] exported KPATHSEA_LIB_DIR / KPATHSEA_STATIC=1"
fi
