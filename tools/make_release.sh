#!/usr/bin/env bash
# Build prebuilt-binary release artifacts for latexml-oxide.
#
# Produces in `target/release-artifacts/`:
#
#   latexml-oxide-<version>-x86_64-unknown-linux-gnu.tar.gz       — portable archive
#   latexml-oxide-<version>-x86_64-unknown-linux-gnu.tar.gz.sha256 — SHA-256 sidecar (ripgrep format)
#   latexml-oxide_<version>-1_<arch>.deb                           — Debian package (arch = amd64 | arm64, from RELEASE_TARGET)
#   latexml-oxide_<version>-1_<arch>.deb.sha256                    — SHA-256 sidecar
#   RELEASE_BODY.md                                                — GitHub release notes (install + CHANGELOG slice)
#
# Single source of truth: `latexml_oxide/Cargo.toml`'s `version = "X.Y.Z"`.
# When invoked from CI with `$GITHUB_REF_NAME` set (tag push), we verify
# that the tag matches the Cargo.toml version and refuse to build on
# mismatch — this prevents shipping a tag that doesn't correspond to a
# bumped crate.
#
# Idempotent: blows away the staging dir at the start; safe to re-run.
#
# Local dry-run:  bash tools/make_release.sh
# CI invocation:  GITHUB_REF_NAME=$tag bash tools/make_release.sh

set -euo pipefail

# --- locate workspace root --------------------------------------------------
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

# --- resolve version --------------------------------------------------------
cargo_toml="latexml_oxide/Cargo.toml"
if [[ ! -f "${cargo_toml}" ]]; then
  echo "make_release: ${cargo_toml} missing — run from latexml-oxide checkout" >&2
  exit 1
fi
# First `version = "X"` line under [package]. The `latexml_oxide` crate has
# no nested tables before [features], so the first match is correct.
version="$(grep -m1 -E '^version = "' "${cargo_toml}" | sed -E 's/^version = "(.+)"$/\1/')"
if [[ -z "${version}" ]]; then
  echo "make_release: could not parse version from ${cargo_toml}" >&2
  exit 1
fi

if [[ -n "${GITHUB_REF_NAME:-}" && "${GITHUB_REF_NAME}" != "${version}" ]]; then
  echo "make_release: GITHUB_REF_NAME='${GITHUB_REF_NAME}' does not match" >&2
  echo "             Cargo.toml version '${version}'." >&2
  echo "             Bump latexml_oxide/Cargo.toml before tagging." >&2
  exit 1
fi

# Target triple. Defaults to the historical Linux target; the macOS release
# leg sets RELEASE_TARGET=aarch64-apple-darwin. We build NATIVELY on the
# matching runner (no `--target` cross-compile step), so RELEASE_TARGET is the
# *label* the host arch produces and MUST match the runner's architecture.
# Native binaries are never cross-OS (ELF vs Mach-O) — there is one artifact
# per (OS, arch); see docs/release/RELEASING.md "Release asset strategy".
target_triple="${RELEASE_TARGET:-x86_64-unknown-linux-gnu}"
case "${target_triple}" in
  *-linux-*)      os_family="linux" ;;
  *-apple-darwin) os_family="macos" ;;
  *-pc-windows-*) os_family="windows" ;;
  *) echo "make_release: unsupported RELEASE_TARGET='${target_triple}'" >&2; exit 1 ;;
esac

# Windows executables carry a `.exe` suffix. Empty elsewhere.
bin_suffix=""
[[ "${os_family}" == "windows" ]] && bin_suffix=".exe"

# Debian architecture label for the .deb FILENAME (Linux only). cargo-deb
# derives the control-file `Architecture:` from the native build host itself;
# we only need the filename to match. amd64 is the default, so the x86_64
# asset name is byte-identical to before — the aarch64 leg gets `arm64`.
case "${target_triple}" in
  aarch64-*-linux-*) deb_arch="arm64" ;;
  *)                 deb_arch="amd64" ;;
esac

# SHA-256 sidecar helper: GNU coreutils `sha256sum` on Linux, BSD `shasum`
# on macOS. Both emit the same "<hash>  <file>" two-space format. Call from
# inside artifacts_dir so the sidecar records the bare filename.
sha256_sidecar() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${f}" > "${f}.sha256"
  else
    shasum -a 256 "${f}" > "${f}.sha256"
  fi
}

stage_dir_name="latexml-oxide-${version}-${target_triple}"
artifacts_dir="target/release-artifacts"
stage_dir="${artifacts_dir}/${stage_dir_name}"

echo "make_release: version=${version} target=${target_triple} os=${os_family}"

# --- clean staging area -----------------------------------------------------
rm -rf "${artifacts_dir}"
mkdir -p "${stage_dir}"

# --- build the binary -------------------------------------------------------
# Distribution build: drop test-utils (audit DEP-02) but KEEP runtime-bindings
# (ship the Rhai script-bindings capability so users customize contributed
# bindings without recompiling — runtime opt-in, so default conversions are
# unchanged), on the publish-grade profile (fat LTO, panic=abort,
# codegen-units=1).
# Per-target extra features (e.g. the Windows leg opts into
# `kpathsea-build-from-source` for a static in-process libkpathsea). Empty
# elsewhere, so the recipe is unchanged on Linux/macOS.
features="runtime-bindings${RELEASE_EXTRA_FEATURES:+,${RELEASE_EXTRA_FEATURES}}"
echo "make_release: cargo build --no-default-features --features ${features} --profile maxperf --bin latexml_oxide"
cargo build --no-default-features --features "${features}" --profile maxperf --bin latexml_oxide

bin_path="target/maxperf/latexml_oxide${bin_suffix}"
# -f not -x: Git Bash on Windows doesn't report a .exe as `-x`.
if [[ ! -f "${bin_path}" ]]; then
  echo "make_release: build did not produce ${bin_path}" >&2
  exit 1
fi

# Strip aggressively. The `maxperf` profile already sets `strip = "symbols"`
# at link time, but a second pass removes any straggling debug sections that
# slipped through (e.g. from C deps). GNU strip uses --strip-all; macOS
# (Mach-O) strip has no such flag, so use -x there (best-effort).
if [[ "${os_family}" == "macos" ]]; then
  strip -x "${bin_path}" 2>/dev/null || true
  # Re-apply a valid ad-hoc signature. Apple's linker ad-hoc-signs arm64
  # Mach-O at link time, but the `strip` above mutates the binary and can
  # invalidate that signature — an arm64 binary with a broken signature is
  # killed at exec (`Killed: 9`). `codesign --sign -` (dash = ad-hoc, no
  # Apple Developer cert needed) restores a valid signature. This is the
  # ripgrep-style posture: no notarization, but the binary always runs, and
  # `curl`-installed tarballs stay Gatekeeper-warning-free (no quarantine
  # xattr on terminal downloads). See docs/release/RELEASING.md "macOS
  # Gatekeeper & code signing".
  codesign --sign - --force "${bin_path}" 2>/dev/null || true
elif [[ "${os_family}" == "windows" ]]; then
  # No external `strip` on MSVC: debug info lives in a separate `.pdb`, so the
  # linked `.exe` is already lean (the maxperf profile's `strip = "symbols"`
  # covers what little remains). Nothing to do here.
  :
else
  strip --strip-all "${bin_path}" 2>/dev/null || true
fi

# Windows ships a SINGLE self-contained `.exe` (the user runs it directly), not a
# tarball — so its packaging diverges from the Linux/macOS tarball flow. Static
# libxml2/libxslt (vcpkg, x64-windows-static-md) + the ubiquitous dynamic MSVC
# runtime; kpathsea is resolved via subprocess `kpsewhich`, never linked.
tarball=""
exe_asset=""
if [[ "${os_family}" == "windows" ]]; then
  exe_asset="latexml-oxide-${version}-${target_triple}.exe"
  cp "${bin_path}" "${artifacts_dir}/${exe_asset}"
  ( cd "${artifacts_dir}" && sha256_sidecar "${exe_asset}" )
  echo "make_release: Windows single-exe asset ${exe_asset}"
else
  # --- stage tarball contents -----------------------------------------------
  cp "${bin_path}" "${stage_dir}/latexml_oxide"
  cp README.md "${stage_dir}/README.md"
  cp CHANGELOG.md "${stage_dir}/CHANGELOG.md"
  cp LICENSE "${stage_dir}/LICENSE"
  # THIRD-PARTY-NOTICES: prefer the release-time assembled file (hand-authored
  # sections 1-4 + the cargo-about Rust-crate appendix produced by
  # tools/gen_notices.sh); fall back to the committed hand-authored file so a
  # local `make_release.sh` without cargo-about still ships notices.
  if [[ -f THIRD-PARTY-NOTICES.dist ]]; then
    cp THIRD-PARTY-NOTICES.dist "${stage_dir}/THIRD-PARTY-NOTICES"
  else
    cp THIRD-PARTY-NOTICES "${stage_dir}/THIRD-PARTY-NOTICES"
  fi

  # --- build tarball --------------------------------------------------------
  tarball="latexml-oxide-${version}-${target_triple}.tar.gz"
  ( cd "${artifacts_dir}" && tar -czf "${tarball}" "${stage_dir_name}" )
  ( cd "${artifacts_dir}" && sha256_sidecar "${tarball}" )
fi

# --- build .deb (Debian-family targets only) --------------------------------
# macOS has no .deb equivalent in this pipeline (a Homebrew tap is the natural
# future analogue); the tarball above is the macOS deliverable.
deb_path=""
if [[ "${os_family}" == "linux" ]]; then
  # `cargo deb` requires the package name (`-p latexml`, not the binary name).
  # `--no-build` reuses the maxperf target/maxperf/latexml_oxide we just built.
  echo "make_release: cargo deb --no-build --profile maxperf -p latexml (${deb_arch})"
  cargo deb --no-build --profile maxperf -p latexml --output "${artifacts_dir}/latexml-oxide_${version}-1_${deb_arch}.deb"

  deb_path="${artifacts_dir}/latexml-oxide_${version}-1_${deb_arch}.deb"
  if [[ ! -f "${deb_path}" ]]; then
    echo "make_release: cargo deb did not produce ${deb_path}" >&2
    exit 1
  fi
  ( cd "${artifacts_dir}" && sha256_sidecar "$(basename "${deb_path}")" )
fi

# --- release body (Install + CHANGELOG slice) -------------------------------
# The shared release body is emitted by the publishing (Linux) leg only. The
# macOS leg uploads its tarball as a CI artifact that the Linux `release` job
# collects before publishing; set RELEASE_MACOS_TARBALL to that filename to
# include its install section here.
release_body="${artifacts_dir}/RELEASE_BODY.md"
if [[ "${os_family}" == "linux" ]]; then
{
  cat <<EOF
## Install

### Debian / Ubuntu (.deb)

\`\`\`
curl -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/latexml-oxide_${version}-1_amd64.deb
sudo apt install ./latexml-oxide_${version}-1_amd64.deb
\`\`\`

### Portable tarball (x86_64 Linux)

\`\`\`
curl -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/latexml-oxide-${version}-${target_triple}.tar.gz
tar xzf latexml-oxide-${version}-${target_triple}.tar.gz
sudo cp latexml-oxide-${version}-${target_triple}/latexml_oxide /usr/local/bin/
\`\`\`

The binary bundles libxml2/libxslt/kpathsea, so tarball users only need a TeX
Live installation (read from your texmf tree) plus graphics tools:

\`\`\`
sudo apt install imagemagick mupdf-tools poppler-utils ghostscript dvipng dvisvgm \
                 texlive-latex-base texlive-latex-extra texlive-science
\`\`\`

EOF

  if [[ -n "${RELEASE_MACOS_TARBALL:-}" ]]; then
    cat <<EOF
### macOS (Apple Silicon)

\`\`\`
curl -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/${RELEASE_MACOS_TARBALL}
tar xzf ${RELEASE_MACOS_TARBALL}
sudo cp latexml-oxide-${version}-aarch64-apple-darwin/latexml_oxide /usr/local/bin/
\`\`\`

The binary bundles libxml2/libxslt, so macOS users only need a TeX distribution:

\`\`\`
# TeX Live — either Homebrew's:
brew install texlive
# …or MacTeX / BasicTeX, served via the subprocess-kpsewhich fallback.
\`\`\`

EOF
  fi

  if [[ -n "${RELEASE_MACOS_INTEL_TARBALL:-}" ]]; then
    cat <<EOF
### macOS (Intel)

For Intel Macs (built with a macOS 10.13 deployment target, so it runs on
older Intel machines up to the latest Sonoma):

\`\`\`
curl -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/${RELEASE_MACOS_INTEL_TARBALL}
tar xzf ${RELEASE_MACOS_INTEL_TARBALL}
sudo cp latexml-oxide-${version}-x86_64-apple-darwin/latexml_oxide /usr/local/bin/
\`\`\`

> Pick the tarball matching your Mac: \`aarch64\` for Apple Silicon (M1/M2/M3…),
> \`x86_64\` for Intel. \`uname -m\` prints \`arm64\` or \`x86_64\`.

EOF
  fi

  if [[ -n "${RELEASE_LINUX_ARM64_TARBALL:-}" ]]; then
    cat <<EOF
### Linux (aarch64 / arm64)

For 64-bit ARM Linux — AWS Graviton, Ampere, Raspberry Pi OS (64-bit), Apple
Silicon Linux VMs. Same self-contained binary as the x86_64 Linux assets.

\`\`\`
# Debian / Ubuntu (.deb)
curl -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/latexml-oxide_${version}-1_arm64.deb
sudo apt install ./latexml-oxide_${version}-1_arm64.deb

# …or the portable tarball
curl -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/${RELEASE_LINUX_ARM64_TARBALL}
tar xzf ${RELEASE_LINUX_ARM64_TARBALL}
sudo cp latexml-oxide-${version}-aarch64-unknown-linux-gnu/latexml_oxide /usr/local/bin/
\`\`\`

> Pick the asset matching your machine: \`x86_64\` for Intel/AMD, \`aarch64\`
> for ARM. \`uname -m\` prints \`x86_64\` or \`aarch64\`.

EOF
  fi

  if [[ -n "${RELEASE_WINDOWS_EXE:-}" ]]; then
    cat <<EOF
### Windows (x86_64)

A single self-contained \`.exe\` — download it and run it directly (no installer,
no tarball). Static libxml2/libxslt are baked in; it needs only the standard
Microsoft Visual C++ runtime (present on modern Windows / the VC++ redistributable)
and a TeX distribution (MiKTeX or TeX Live) on \`PATH\`, resolved via \`kpsewhich\`.

\`\`\`powershell
curl.exe -LO https://github.com/dginev/latexml-oxide/releases/download/${version}/${RELEASE_WINDOWS_EXE}
.\\${RELEASE_WINDOWS_EXE} --version
\`\`\`

EOF
  fi

  # Slice the CHANGELOG section for this version. CHANGELOG entries use
  # `## [VERSION]` headers (CommonMark task-list style — see CHANGELOG.md
  # for shape). Extract from the matching header up to the next `## `.
  if grep -qE "^## \[${version}\]" CHANGELOG.md; then
    echo "## Changelog"
    echo
    awk -v ver="${version}" '
      /^## \[/ {
        if (in_block) exit
        if ($0 ~ "^## \\[" ver "\\]") { in_block = 1; print; next }
      }
      in_block { print }
    ' CHANGELOG.md
  else
    echo "_See [CHANGELOG.md](https://github.com/dginev/latexml-oxide/blob/main/CHANGELOG.md) for release notes._"
  fi
} > "${release_body}"
fi

# --- summary ----------------------------------------------------------------
echo
echo "make_release: artifacts in ${artifacts_dir}/"
ls -la "${artifacts_dir}"
echo
echo "make_release: SHA-256 sidecars"
if [[ -n "${tarball}" ]]; then
  cat "${artifacts_dir}/${tarball}.sha256"
fi
if [[ -n "${exe_asset}" ]]; then
  cat "${artifacts_dir}/${exe_asset}.sha256"
fi
if [[ -n "${deb_path}" ]]; then
  cat "${artifacts_dir}/$(basename "${deb_path}").sha256"
fi
