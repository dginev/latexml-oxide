#!/usr/bin/env bash
# fetch_ar5iv_css.sh [--verify] [<dir>] — place the pinned ar5iv-css release's dist/ar5iv.min.css in <dir>
# (default resources/ar5iv-css/, gitignored), which latexml_post/build.rs embeds as the bundled `ar5iv.css`.
#
# The pin is tools/ar5iv-css.pin (tag, commit, SHA-256); the file is fetched at the commit, unauthenticated (the
# repository is public). Writes <dir>/ar5iv.min.css and <dir>/ar5iv-css.version ("<tag> <commit>", for the embedded
# banner). A file already in place is checked, not fetched again; a SHA-256 that differs from the pin fails, as does
# any download error. --verify only checks, writing nothing (no network): non-zero when the file is absent or differs,
# or when ar5iv-css.version is absent or not exactly "<tag> <commit>" of the pin.
#
# Release-only: release-prepare.yml fetches it once into the `ar5iv-css` artifact, and tools/make_release.sh and
# docker.yml verify it where they build. Dev and CI builds never fetch it (the build script reads only what is on disk).
# Runs under GNU bash, Git Bash on Windows and macOS's bash 3.2.
set -euo pipefail

verify_only=0
if [ "${1:-}" = "--verify" ]; then
  verify_only=1
  shift
fi
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dir="${1:-${repo_root}/resources/ar5iv-css}"
pin="${repo_root}/tools/ar5iv-css.pin"

tag="$(sed -n 's/^AR5IV_CSS_TAG=//p' "$pin" | head -n1)"
commit="$(sed -n 's/^AR5IV_CSS_COMMIT=//p' "$pin" | head -n1)"
want="$(sed -n 's/^AR5IV_CSS_SHA256=//p' "$pin" | head -n1)"
if ! printf '%s\n' "$tag" | grep -Eq '^[0-9A-Za-z._-]+$' || ! printf '%s\n' "$commit" | grep -Eq '^[0-9a-f]{40}$' ||
  ! printf '%s\n' "$want" | grep -Eq '^[0-9a-f]{64}$'; then
  echo "fetch_ar5iv_css: malformed pin ${pin} (tag '${tag}', commit '${commit}', sha256 '${want}')" >&2
  exit 1
fi

sha256() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

css="${dir}/ar5iv.min.css"
if [ -f "$css" ]; then
  got="$(sha256 "$css")"
  if [ "$got" != "$want" ]; then
    echo "fetch_ar5iv_css: ${css} has sha256 ${got}, the pin wants ${want} (ar5iv-css ${tag}); remove it to refetch" >&2
    exit 1
  fi
  stamp="${dir}/ar5iv-css.version"
  if [ "$verify_only" -eq 0 ]; then
    printf '%s %s\n' "$tag" "$commit" > "$stamp"
  elif [ "$(cat "$stamp" 2> /dev/null)" != "${tag} ${commit}" ]; then
    echo "fetch_ar5iv_css: ${stamp} is absent or not '${tag} ${commit}' (run tools/fetch_ar5iv_css.sh to rewrite it)" >&2
    exit 1
  fi
  echo "fetch_ar5iv_css: ${css} is ar5iv-css ${tag} (${commit}, sha256 ${got})"
  exit 0
fi
if [ "$verify_only" -eq 1 ]; then
  echo "fetch_ar5iv_css: ${css} is missing (run tools/fetch_ar5iv_css.sh to fetch ar5iv-css ${tag})" >&2
  exit 1
fi

url="${AR5IV_CSS_URL_BASE:-https://raw.githubusercontent.com/dginev/ar5iv-css}/${commit}/dist/ar5iv.min.css"
mkdir -p "$dir"
tmp="${css}.part"
rm -f "$tmp"
if ! curl -fsSL --retry 3 --retry-delay 2 -o "$tmp" "$url"; then
  rm -f "$tmp"
  echo "fetch_ar5iv_css: download failed: ${url}" >&2
  exit 1
fi
got="$(sha256 "$tmp")"
if [ "$got" != "$want" ]; then
  rm -f "$tmp"
  echo "fetch_ar5iv_css: ${url} has sha256 ${got}, the pin wants ${want}" >&2
  exit 1
fi
mv "$tmp" "$css"
printf '%s %s\n' "$tag" "$commit" > "${dir}/ar5iv-css.version"
echo "fetch_ar5iv_css: fetched ar5iv-css ${tag} (${commit}) into ${css} (sha256 ${got})"
