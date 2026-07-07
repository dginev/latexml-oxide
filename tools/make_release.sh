#!/usr/bin/env bash
# Build prebuilt-binary release artifacts for latexml-oxide.
#
# Produces in `target/release-artifacts/`:
#
#   latexml-oxide-<version>-x86_64-unknown-linux-gnu.tar.gz       — portable archive
#   latexml-oxide-<version>-x86_64-unknown-linux-gnu.tar.gz.sha256 — SHA-256 sidecar (ripgrep format)
#   latexml-oxide_<version>-1_amd64.deb                            — Debian package
#   latexml-oxide_<version>-1_amd64.deb.sha256                     — SHA-256 sidecar
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
# per (OS, arch); see docs/RELEASING.md "Release asset strategy".
target_triple="${RELEASE_TARGET:-x86_64-unknown-linux-gnu}"
case "${target_triple}" in
  *-linux-*)      os_family="linux" ;;
  *-apple-darwin) os_family="macos" ;;
  *) echo "make_release: unsupported RELEASE_TARGET='${target_triple}'" >&2; exit 1 ;;
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
echo "make_release: cargo build --no-default-features --features runtime-bindings --profile maxperf --bin latexml_oxide"
cargo build --no-default-features --features runtime-bindings --profile maxperf --bin latexml_oxide

bin_path="target/maxperf/latexml_oxide"
if [[ ! -x "${bin_path}" ]]; then
  echo "make_release: build did not produce ${bin_path}" >&2
  exit 1
fi

# Strip aggressively. The `maxperf` profile already sets `strip = "symbols"`
# at link time, but a second pass removes any straggling debug sections that
# slipped through (e.g. from C deps). GNU strip uses --strip-all; macOS
# (Mach-O) strip has no such flag, so use -x there (best-effort).
if [[ "${os_family}" == "macos" ]]; then
  strip -x "${bin_path}" 2>/dev/null || true
else
  strip --strip-all "${bin_path}" 2>/dev/null || true
fi

# --- stage tarball contents -------------------------------------------------
cp "${bin_path}" "${stage_dir}/latexml_oxide"
cp README.md "${stage_dir}/README.md"
cp CHANGELOG.md "${stage_dir}/CHANGELOG.md"
cp LICENSE "${stage_dir}/LICENSE"

# --- build tarball ----------------------------------------------------------
tarball="latexml-oxide-${version}-${target_triple}.tar.gz"
( cd "${artifacts_dir}" && tar -czf "${tarball}" "${stage_dir_name}" )
( cd "${artifacts_dir}" && sha256_sidecar "${tarball}" )

# --- build .deb (Debian-family targets only) --------------------------------
# macOS has no .deb equivalent in this pipeline (a Homebrew tap is the natural
# future analogue); the tarball above is the macOS deliverable.
deb_path=""
if [[ "${os_family}" == "linux" ]]; then
  # `cargo deb` requires the package name (`-p latexml`, not the binary name).
  # `--no-build` reuses the maxperf target/maxperf/latexml_oxide we just built.
  echo "make_release: cargo deb --no-build --profile maxperf -p latexml"
  cargo deb --no-build --profile maxperf -p latexml --output "${artifacts_dir}/latexml-oxide_${version}-1_amd64.deb"

  deb_path="${artifacts_dir}/latexml-oxide_${version}-1_amd64.deb"
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
sudo apt install texlive-latex-base texlive-latex-extra texlive-science imagemagick
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
cat "${artifacts_dir}/${tarball}.sha256"
if [[ -n "${deb_path}" ]]; then
  cat "${artifacts_dir}/$(basename "${deb_path}").sha256"
fi
