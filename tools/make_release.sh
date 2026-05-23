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

target_triple="x86_64-unknown-linux-gnu"
stage_dir_name="latexml-oxide-${version}-${target_triple}"
artifacts_dir="target/release-artifacts"
stage_dir="${artifacts_dir}/${stage_dir_name}"

echo "make_release: version=${version} target=${target_triple}"

# --- clean staging area -----------------------------------------------------
rm -rf "${artifacts_dir}"
mkdir -p "${stage_dir}"

# --- build the binary -------------------------------------------------------
# Distribution build: drop test-utils (audit DEP-02) and use the publish-grade
# profile (fat LTO, panic=abort, codegen-units=1).
echo "make_release: cargo build --no-default-features --profile maxperf --bin latexml_oxide"
cargo build --no-default-features --profile maxperf --bin latexml_oxide

bin_path="target/maxperf/latexml_oxide"
if [[ ! -x "${bin_path}" ]]; then
  echo "make_release: build did not produce ${bin_path}" >&2
  exit 1
fi

# Strip aggressively. The `maxperf` profile already sets `strip = "symbols"`
# at link time, but a second pass removes any straggling debug sections that
# slipped through (e.g. from C deps).
strip --strip-all "${bin_path}" 2>/dev/null || true

# --- stage tarball contents -------------------------------------------------
cp "${bin_path}" "${stage_dir}/latexml_oxide"
cp README.md "${stage_dir}/README.md"
cp CHANGELOG.md "${stage_dir}/CHANGELOG.md"
cp LICENSE "${stage_dir}/LICENSE"

# --- build tarball ----------------------------------------------------------
tarball="latexml-oxide-${version}-${target_triple}.tar.gz"
( cd "${artifacts_dir}" && tar -czf "${tarball}" "${stage_dir_name}" )
( cd "${artifacts_dir}" && sha256sum "${tarball}" > "${tarball}.sha256" )

# --- build .deb -------------------------------------------------------------
# `cargo deb` requires the package name (`-p latexml`, not the binary name).
# `--no-build` reuses the maxperf target/maxperf/latexml_oxide we just built.
echo "make_release: cargo deb --no-build --profile maxperf -p latexml"
cargo deb --no-build --profile maxperf -p latexml --output "${artifacts_dir}/latexml-oxide_${version}-1_amd64.deb"

deb_path="${artifacts_dir}/latexml-oxide_${version}-1_amd64.deb"
if [[ ! -f "${deb_path}" ]]; then
  echo "make_release: cargo deb did not produce ${deb_path}" >&2
  exit 1
fi
( cd "${artifacts_dir}" && sha256sum "$(basename "${deb_path}")" > "$(basename "${deb_path}").sha256" )

# --- release body (Install + CHANGELOG slice) -------------------------------
release_body="${artifacts_dir}/RELEASE_BODY.md"
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

Tarball users also need the runtime apt deps:

\`\`\`
sudo apt install libxml2 libxslt1.1 libkpathsea6 texlive-latex-base texlive-latex-extra texlive-science
\`\`\`

EOF

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
    echo "_See [CHANGELOG.md](https://github.com/dginev/latexml-oxide/blob/master/CHANGELOG.md) for release notes._"
  fi
} > "${release_body}"

# --- summary ----------------------------------------------------------------
echo
echo "make_release: artifacts in ${artifacts_dir}/"
ls -la "${artifacts_dir}"
echo
echo "make_release: SHA-256 sidecars"
cat "${artifacts_dir}/${tarball}.sha256"
cat "${artifacts_dir}/$(basename "${deb_path}").sha256"
