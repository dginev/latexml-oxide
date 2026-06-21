#!/bin/sh
# Turnkey entrypoint for the latexml-oxide cortex_worker fleet — the Rust counterpart of the legacy
# Perl `latexml_harness`. Brings up the self-supervising `--harness` fleet pointed at a dispatcher.
#
#   cortex-worker <dispatcher-host> [ventilator-port] [sink-port] [service]
#   defaults:                        51695            51696       oxidized-tex-to-html
#
#   env:  PROFILE=ar5iv (default) | generic
#         WORKERS=<n>  pin the fleet size (battle-hardened sweet spot = physical cores + 1/8;
#                      unset = the harness CPU-derived default).
#
# If the first argument is a flag (starts with `-`), it and the rest are passed straight to
# cortex_worker instead — e.g. a one-off `--standalone --input … --output …` conversion.
set -e
# Disk-backed scratch (NOT a ramdisk — /dev/shm exhaustion truncates inputs at scale, CorTeX D-18).
export TMPDIR="${TMPDIR:-/opt/cortex-scratch}"
mkdir -p "$TMPDIR"

# Advanced/standalone: hand flags through verbatim.
case "${1:-}" in
  -*) exec cortex_worker "$@" ;;
esac

HOST="${1:-127.0.0.1}"
VENT="${2:-51695}"
SINK="${3:-51696}"
SERVICE="${4:-oxidized-tex-to-html}"

# Optional explicit worker count (else the harness sizes itself); keep memory/timeout guards at the
# binary's validated defaults — see docs/CORTEX_WORKER_HARNESS.md.
WORKERS_FLAG=""
[ -n "${WORKERS:-}" ] && WORKERS_FLAG="--workers ${WORKERS}"

exec cortex_worker --harness \
  --source-address "tcp://${HOST}:${VENT}" \
  --sink-address   "tcp://${HOST}:${SINK}" \
  --service "${SERVICE}" \
  --profile "${PROFILE:-ar5iv}" \
  ${WORKERS_FLAG}
