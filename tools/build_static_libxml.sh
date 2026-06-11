#!/usr/bin/env bash
# Build self-contained, PIC static libxml2 + libxslt + libexslt from the GNOME
# release tarballs and point the rust-libxml / rust-libxslt build scripts at them
# (PKG_CONFIG_PATH + LIBXML2_STATIC + LIBXSLT_STATIC, emitted to $GITHUB_ENV under
# Actions). Used by the Linux and macOS release legs in
# .github/workflows/release.yml. Companion to build_static_kpathsea.sh.
#
# Why static, from source:
#   * libxml2 2.14 bumped its SONAME libxml2.so.2 -> libxml2.so.16 (a declared ABI
#     break). A binary that dynamically links the build host's libxml2 only loads
#     on hosts carrying that exact SONAME — .so.2 (Ubuntu 22.04/24.04 LTS,
#     Debian 12) OR .so.16 (Debian 13+, Arch, libxml2 >= 2.14), never both. Static
#     linking removes the runtime libxml2/libxslt dependency entirely, so the
#     binary runs against ANY (or no) host libxml2.
#   * From source (not the system libxml2-dev .a) because that archive is NON-PIC
#     (R_X86_64_PC32 relocations), and our proc-macro crate (latexml_codegen) is a
#     cdylib that links libxml transitively via latexml_core — a non-PIC archive is
#     rejected there ("recompile with -fPIC"). `--with-pic` produces a PIC archive
#     that links into BOTH the cdylib and the final binary. macOS Mach-O is PIC by
#     default, so `--with-pic` is a harmless no-op there.
#   * Into a NON-system prefix so pkg-config's static guard permits a static link
#     (it refuses `static=` for libs under /usr/lib); the crates then probe with
#     `.statik(true)`.
#
# Build-configuration policy — a DETERMINISTIC, MINIMAL dependency closure:
#   The purpose of static linking is host-independence. `configure` undermines that
#   by AUTO-DETECTING optional external libraries from whatever dev headers happen
#   to sit on the build host — so the same script yields a DIFFERENT dependency
#   surface per runner. (That is exactly how the macOS leg pulled libgcrypt/
#   libgpg-error via Homebrew while Linux did not, breaking the link with
#   unresolved __gpg_* symbols.) A point-patch for crypto would leave the next
#   environment-specific lib to bite. So the rule is: every OPTIONAL,
#   EXTERNAL-DEPENDENCY feature is disabled EXPLICITLY here; nothing is inherited
#   from the host. We keep exactly two things:
#     1. the core API LaTeXML exercises (XPath, RelaxNG/schemas, c14n, output,
#        XInclude, reader/writer, …) — these carry no external deps; and
#     2. iconv — the lone external dep that is a guaranteed, stable system library
#        on every target (glibc on Linux, libiconv on macOS), so it may stay
#        dynamic under the same rule that lets libc/libm/libgcc_s/CoreFoundation
#        stay dynamic. (Disabling it would only trade a stable dep for an encoding-
#        coverage risk.)
#   Everything else optional+external is OFF: python, zlib, lzma, icu (libxml2;
#   icu's SONAME churns), crypto (libxslt). Net closure: -lm (+ -liconv on macOS),
#   reproducibly, regardless of build host. Any NEW optional dep must be opted into
#   here CONSCIOUSLY, never auto-detected.
#
# Build parallelism is capped (JOBS, default 4): the dev box hard-freezes under a
# full -j load. CI runners can raise it (JOBS=$(nproc)).
set -euo pipefail

LIBXML2_VER="${LIBXML2_VER:-2.13.5}"
LIBXSLT_VER="${LIBXSLT_VER:-1.1.42}"
JOBS="${JOBS:-4}"

WORK="${LIBXML_STATIC_WORK:-/tmp/libxml-static}"
PREFIX="${WORK}/prefix"
rm -rf "$WORK"
mkdir -p "$WORK" "$PREFIX"
cd "$WORK"

xml_major="${LIBXML2_VER%.*}"   # 2.13.5 -> 2.13
xslt_major="${LIBXSLT_VER%.*}"  # 1.1.42 -> 1.1

fetch() { # url
  echo "[build_static_libxml] fetch $1"
  curl -fsSL "$1" -o "$(basename "$1")"
}

fetch "https://download.gnome.org/sources/libxml2/${xml_major}/libxml2-${LIBXML2_VER}.tar.xz"
fetch "https://download.gnome.org/sources/libxslt/${xslt_major}/libxslt-${LIBXSLT_VER}.tar.xz"

tar xf "libxml2-${LIBXML2_VER}.tar.xz"
tar xf "libxslt-${LIBXSLT_VER}.tar.xz"

# --- libxml2 (built first; libxslt links against it) ----------------------------
echo "[build_static_libxml] configure libxml2 ${LIBXML2_VER} (static, PIC)"
cd "${WORK}/libxml2-${LIBXML2_VER}"
# --without-{python,zlib,lzma,icu}: disable every optional external-dependency
# feature per the configuration policy above (icu in particular — libicu's SONAME
# churns hard). The core API LaTeXML uses carries no external deps and stays on.
# Explicit CFLAGS=-fPIC: libtool's --with-pic alone does NOT force PIC objects for
# a --disable-shared (static-only) build here — it yields absolute R_X86_64_32
# relocations that a shared object (our proc-macro cdylib) rejects. -fPIC on every
# compile guarantees a PIC archive that links into both the cdylib and the binary.
./configure --prefix="$PREFIX" \
  --enable-static --disable-shared --with-pic \
  --without-python --without-zlib --without-lzma --without-icu \
  --disable-dependency-tracking \
  CFLAGS="-fPIC -O2" >/dev/null
echo "[build_static_libxml] make libxml2 -j${JOBS}"
make -j"${JOBS}" >/dev/null
make install >/dev/null

# --- libxslt + libexslt (against our static libxml2) ----------------------------
echo "[build_static_libxml] configure libxslt ${LIBXSLT_VER} (static, PIC)"
cd "${WORK}/libxslt-${LIBXSLT_VER}"
# --without-crypto: the libxslt arm of the same policy. The EXSLT crypto module
# (md5/sha1/...) pulls in libgcrypt + libgpg-error, which configure auto-detected
# on the macOS runner (Homebrew) but not on Linux, leaving libexslt.a with
# unresolved __gpg_* symbols at the final link. LaTeXML never uses crypto:*, so the
# module is off on every platform.
PKG_CONFIG_PATH="${PREFIX}/lib/pkgconfig" ./configure --prefix="$PREFIX" \
  --enable-static --disable-shared --with-pic \
  --without-python \
  --without-crypto \
  --with-libxml-prefix="$PREFIX" \
  --disable-dependency-tracking \
  CFLAGS="-fPIC -O2" >/dev/null
echo "[build_static_libxml] make libxslt -j${JOBS}"
make -j"${JOBS}" >/dev/null
make install >/dev/null

# --- verify the archives exist --------------------------------------------------
missing=0
for a in libxml2.a libxslt.a libexslt.a; do
  if [[ ! -f "${PREFIX}/lib/${a}" ]]; then
    echo "[build_static_libxml] ERROR: ${PREFIX}/lib/${a} not produced" >&2
    missing=1
  fi
done
[[ $missing -eq 0 ]] || exit 1
echo "[build_static_libxml] built static archives in ${PREFIX}/lib:"
ls -la "${PREFIX}/lib/"libxml2.a "${PREFIX}/lib/"libxslt.a "${PREFIX}/lib/"libexslt.a

# --- export for the rust-libxml / rust-libxslt build scripts ---------------------
if [[ -n "${GITHUB_ENV:-}" ]]; then
  {
    # Prepend our prefix so its static .pc files win over any host libxml2/libxslt
    # already on PKG_CONFIG_PATH (e.g. the macOS leg's brew keg paths), while
    # leaving those in place as a fallback.
    echo "PKG_CONFIG_PATH=${PREFIX}/lib/pkgconfig${PKG_CONFIG_PATH:+:${PKG_CONFIG_PATH}}"
    echo "LIBXML2_STATIC=1"
    echo "LIBXSLT_STATIC=1"
  } >> "$GITHUB_ENV"
  echo "[build_static_libxml] exported PKG_CONFIG_PATH / LIBXML2_STATIC / LIBXSLT_STATIC"
else
  echo "[build_static_libxml] (not under Actions) export these to build statically:"
  echo "  export PKG_CONFIG_PATH=${PREFIX}/lib/pkgconfig"
  echo "  export LIBXML2_STATIC=1 LIBXSLT_STATIC=1"
fi
