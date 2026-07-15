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

# --- stage the payload (identical on every platform) -------------------------
# The binary is self-contained everywhere; the archive around it exists so the
# binary always travels WITH its license notices. Windows used to publish a bare
# `.exe`, which made it the one download whose recipient got no notices at all
# (LICENSE_INVENTORY F7) — it now ships the same payload as a `.zip`.
if [[ "${os_family}" == "windows" ]]; then
  cp "${bin_path}" "${stage_dir}/latexml_oxide.exe"
else
  cp "${bin_path}" "${stage_dir}/latexml_oxide"
fi
cp README.md "${stage_dir}/README.md"
cp CHANGELOG.md "${stage_dir}/CHANGELOG.md"
cp LICENSE "${stage_dir}/LICENSE"
# THIRD-PARTY-NOTICES: prefer the release-time assembled file (hand-authored
# sections 1-4 + the cargo-about Rust-crate appendix + the section 6 copyleft
# texts, produced by tools/gen_notices.sh); fall back to the committed
# hand-authored file so a local `make_release.sh` without cargo-about still
# ships notices. In CI the `notices` job hands every leg the assembled file.
if [[ -f THIRD-PARTY-NOTICES.dist ]]; then
  cp THIRD-PARTY-NOTICES.dist "${stage_dir}/THIRD-PARTY-NOTICES"
else
  echo "make_release: WARNING — no THIRD-PARTY-NOTICES.dist; falling back to the" >&2
  echo "  committed hand-authored file (no Rust-crate appendix, no copyleft texts)." >&2
  cp THIRD-PARTY-NOTICES "${stage_dir}/THIRD-PARTY-NOTICES"
fi

# --- package -----------------------------------------------------------------
# Windows gets a `.zip` (the native format there, and Explorer opens it without
# extra tooling); everything else gets a `.tar.gz`.
tarball=""
zip_asset=""
if [[ "${os_family}" == "windows" ]]; then
  zip_asset="latexml-oxide-${version}-${target_triple}.zip"
  (
    cd "${artifacts_dir}"
    rm -f "${zip_asset}"
    # 7-Zip is present on GitHub's windows-latest image; `zip` covers Git-Bash
    # hosts that have it (and lets this path be exercised off-Windows);
    # Compress-Archive is the always-there PowerShell fallback. Fail loudly
    # rather than publish a Windows asset with no archive around it.
    if command -v 7z >/dev/null 2>&1; then
      7z a -tzip -mx=9 "${zip_asset}" "${stage_dir_name}" >/dev/null
    elif command -v zip >/dev/null 2>&1; then
      zip -q -9 -r "${zip_asset}" "${stage_dir_name}"
    elif command -v powershell >/dev/null 2>&1; then
      powershell -NoProfile -NonInteractive -Command \
        "Compress-Archive -Path '${stage_dir_name}' -DestinationPath '${zip_asset}' -Force"
    else
      echo "make_release: no zip tool found (tried 7z, zip, powershell)" >&2
      exit 1
    fi
  )
  ( cd "${artifacts_dir}" && sha256_sidecar "${zip_asset}" )
  echo "make_release: Windows zip asset ${zip_asset}"
else
  tarball="latexml-oxide-${version}-${target_triple}.tar.gz"
  ( cd "${artifacts_dir}" && tar -czf "${tarball}" "${stage_dir_name}" )
  ( cd "${artifacts_dir}" && sha256_sidecar "${tarball}" )
fi

# --- build .deb (Debian-family targets only) --------------------------------
# macOS has no .deb equivalent in this pipeline (a Homebrew tap is the natural
# future analogue); the tarball above is the macOS deliverable.
deb_path=""
if [[ "${os_family}" == "linux" ]]; then
  # The .deb does NOT get its notices from ${stage_dir}: cargo-deb builds its
  # payload from the asset list in latexml_oxide/Cargo.toml, which points at
  # `../THIRD-PARTY-NOTICES` -- the COMMITTED repo-root file (hand-authored
  # sections 1-4 only). Staging the assembled file into ${stage_dir} above does
  # nothing for it, so the .deb -- the install path the README calls the easiest
  # way in -- would ship with no Rust-crate appendix (§5), none of the copyleft
  # texts the static libkpathsea/libmarpa links oblige (§6), and no relink
  # provenance (§7). Point the path cargo-deb actually reads at the assembled
  # file for the duration of the build, then put the committed file back so a
  # local run leaves no dirty tree.
  notices_backup=""
  restore_notices() {
    if [[ -n "${notices_backup}" && -f "${notices_backup}" ]]; then
      mv -f "${notices_backup}" THIRD-PARTY-NOTICES
      notices_backup=""
    fi
  }
  if [[ -f THIRD-PARTY-NOTICES.dist ]]; then
    notices_backup="$(mktemp)"
    cp THIRD-PARTY-NOTICES "${notices_backup}"
    trap restore_notices EXIT
    cp THIRD-PARTY-NOTICES.dist THIRD-PARTY-NOTICES
    echo "make_release: .deb notices <- THIRD-PARTY-NOTICES.dist ($(wc -l < THIRD-PARTY-NOTICES) lines)"
  fi

  # `cargo deb` requires the package name (`-p latexml`, not the binary name).
  # `--no-build` reuses the maxperf target/maxperf/latexml_oxide we just built.
  echo "make_release: cargo deb --no-build --profile maxperf -p latexml (${deb_arch})"
  cargo deb --no-build --profile maxperf -p latexml --output "${artifacts_dir}/latexml-oxide_${version}-1_${deb_arch}.deb"

  deb_path="${artifacts_dir}/latexml-oxide_${version}-1_${deb_arch}.deb"

  restore_notices
  trap - EXIT

  if [[ ! -f "${deb_path}" ]]; then
    echo "make_release: cargo deb did not produce ${deb_path}" >&2
    exit 1
  fi

  # Prove it rather than assume it: read the notices back OUT of the built .deb.
  # This is the artifact users install, and the failure mode it guards (shipping
  # the committed 1-4 file) is invisible from the outside -- the .deb builds and
  # installs fine either way.
  if [[ -f THIRD-PARTY-NOTICES.dist ]] && command -v dpkg-deb >/dev/null 2>&1; then
    deb_notices="$(dpkg-deb --fsys-tarfile "${deb_path}" 2>/dev/null \
      | tar -xO ./usr/share/doc/latexml-oxide/THIRD-PARTY-NOTICES 2>/dev/null || true)"
    for needle in "5. RUST DEPENDENCY LICENSES" "6. COPYLEFT LICENSE TEXTS" "7. SOURCE PROVENANCE"; do
      if ! grep -qF "${needle}" <<<"${deb_notices}"; then
        echo "make_release: the .deb's THIRD-PARTY-NOTICES is missing '${needle}'." >&2
        echo "  cargo-deb reads latexml_oxide/Cargo.toml's asset list (../THIRD-PARTY-NOTICES);" >&2
        echo "  it must see the assembled file, not the committed sections 1-4." >&2
        exit 1
      fi
    done
    echo "make_release: verified the .deb ships the assembled notices ($(wc -l <<<"${deb_notices}") lines)"
  fi

  ( cd "${artifacts_dir}" && sha256_sidecar "$(basename "${deb_path}")" )
fi

# --- release body (Install + CHANGELOG slice) -------------------------------
# The shared release body is emitted by the publishing (Linux) leg only. The
# macOS leg uploads its tarball as a CI artifact that the Linux `release` job
# collects before publishing; set RELEASE_MACOS_TARBALL to that filename to
# list it among the downloadable assets here. Step-by-step install commands
# live in the README (linked from the notes), not on the release page.
release_body="${artifacts_dir}/RELEASE_BODY.md"
if [[ "${os_family}" == "linux" ]]; then
{
  cat <<EOF
## Installation

These are **self-contained** binaries — libxml2, libxslt, and kpathsea are baked
in, so the only runtime requirement is a **TeX distribution** (TeX Live, MacTeX,
or MiKTeX) on your \`PATH\`. Graphics tools (ImageMagick, Ghostscript, MuPDF,
dvipng, dvisvgm) are optional — used for figure conversion when present.

**Step-by-step, per-platform install instructions live in the [README
Installation guide](https://github.com/dginev/latexml-oxide#install-prebuilt-binaries).**
This page lists only the assets, so the commands stay in one place.

Download the asset for your platform (each has a matching \`.sha256\` sidecar):

- **Linux (x86-64)** — \`latexml-oxide_${version}-1_amd64.deb\`, or the portable \`latexml-oxide-${version}-${target_triple}.tar.gz\`
EOF

  if [[ -n "${RELEASE_LINUX_ARM64_TARBALL:-}" ]]; then
    echo "- **Linux (aarch64 / arm64)** — \`latexml-oxide_${version}-1_arm64.deb\`, or the portable \`${RELEASE_LINUX_ARM64_TARBALL}\`"
  fi
  if [[ -n "${RELEASE_MACOS_TARBALL:-}" ]]; then
    echo "- **macOS (Apple Silicon)** — \`${RELEASE_MACOS_TARBALL}\`"
  fi
  if [[ -n "${RELEASE_MACOS_INTEL_TARBALL:-}" ]]; then
    echo "- **macOS (Intel)** — \`${RELEASE_MACOS_INTEL_TARBALL}\`"
  fi
  if [[ -n "${RELEASE_WINDOWS_ZIP:-}" ]]; then
    echo "- **Windows (x86-64)** — \`${RELEASE_WINDOWS_ZIP}\` (unzip; a single self-contained \`latexml_oxide.exe\`, no installer)"
  fi
  echo

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
if [[ -n "${zip_asset}" ]]; then
  cat "${artifacts_dir}/${zip_asset}.sha256"
fi
if [[ -n "${deb_path}" ]]; then
  cat "${artifacts_dir}/$(basename "${deb_path}").sha256"
fi
