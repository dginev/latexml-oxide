#!/usr/bin/env bash
# ci_apt_install.sh — `apt-get install` on a GitHub-hosted Ubuntu runner that survives a slow mirror.
#
# The runner image's apt sources read a mirror list (/etc/apt/apt-mirrors.txt, a `mirror+file:` source: one
# URL per line, `priority:N` after a tab). apt moves to the next mirror only on a hard failure, and its http
# timeout fires only on silence, so a mirror that is alive but slow holds the job until the step's own
# timeout. Witness: PR #913's CI run 44384621418 (2026-11-18) — the three test shards whose runner listed
# us-east-2.ec2.archive.ubuntu.com first spent the step's 20 minutes on the 715 MB of archives; the eight
# that listed azure.archive.ubuntu.com fetched the same set in 8 s to 2 min 8 s.
#
# So the DOWNLOAD (`--download-only`, safe to interrupt; dpkg never runs under a timer) gets ATTEMPT_SECS.
# If it has not finished by then, the first mirror in the list is dropped and the download is retried for up
# to RETRY_SECS against the rest. Archives already fetched stay in apt's cache, so a retry continues rather
# than restarts. The install itself then runs from the cache.
#
# Usage: tools/ci_apt_install.sh <package>...
# Env:   ATTEMPT_SECS (default 300), RETRY_SECS (default 600), APT_MIRROR_LIST (default /etc/apt/apt-mirrors.txt)
set -euo pipefail

ATTEMPT_SECS="${ATTEMPT_SECS:-300}"
RETRY_SECS="${RETRY_SECS:-600}"
LIST="${APT_MIRROR_LIST:-/etc/apt/apt-mirrors.txt}"
APT=(apt-get -o Acquire::Retries=5 -o Acquire::http::Timeout=60 -o Acquire::https::Timeout=60)

(($#)) || { echo "usage: $0 <package>..." >&2; exit 2; }

show_mirrors() {
  if [[ -f $LIST ]]; then
    echo "apt mirror list ($LIST):"
    sed 's/^/  /' "$LIST"
  fi
}

# fetch <seconds> <package>...: refresh the indexes and download every archive the install needs, each
# within <seconds>.
fetch() {
  local secs=$1
  shift
  sudo timeout -k 10 "$secs" "${APT[@]}" update &&
    sudo timeout -k 10 "$secs" "${APT[@]}" install -y --no-install-recommends --download-only "$@"
}

show_mirrors
if ! fetch "$ATTEMPT_SECS" "$@"; then
  echo "::warning::apt download did not finish in ${ATTEMPT_SECS}s; retrying without the first mirror"
  if [[ -f $LIST ]] && (($(grep -cE '^[a-z]+://' "$LIST") > 1)); then
    first=$(grep -m1 -E '^[a-z]+://' "$LIST")
    grep -vxF "$first" "$LIST" | sudo tee "$LIST.retry" >/dev/null
    sudo mv "$LIST.retry" "$LIST"
    show_mirrors
  fi
  fetch "$RETRY_SECS" "$@"
fi

sudo "${APT[@]}" install -y --no-install-recommends "$@"
