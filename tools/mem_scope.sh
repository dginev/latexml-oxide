#!/bin/bash
# mem_scope.sh — run a command (typically a whole `xargs -P N` / `parallel -j N`
# worker fan-out) inside a transient systemd --user cgroup scope with a HARD
# AGGREGATE memory cap. Reusable guard for any local batch sweep of cortex_worker
# / latexml_oxide conversions.
#
# WHY (2026-06-02 hardening): a per-worker `ulimit -v` (e.g. run_one.sh's 6 GB,
# benchmark_canvas.sh's 8 GB) bounds ONE worker but NOT the sum. With many
# parallel workers a cluster of heavy-math papers can collectively exceed
# physical RAM, and the kernel OOM-killer roams SYSTEM-WIDE — it can pick
# gnome-shell / the editor / containerd (observed: a cortex_worker reached 62 GB
# anon-rss and cascade-killed the desktop) → swap thrash → machine freeze, before
# any clean per-worker kill. This wrapper confines the whole tree to one cgroup:
# when the cgroup hits MemoryMax the kernel OOM-kills a process INSIDE the cgroup
# only (a worker, which the runner classifies as OOM — the correct fail-safe
# bias). The desktop/system are structurally untouchable; the machine can't freeze.
#
# Defaults sized for a 62 GiB / 20-core laptop, reserving ~16 GiB for OS+GUI+IDE.
# Override any limit via env: MEM_HIGH=48G MEM_MAX=52G MEM_SWAP=1G mem_scope.sh ...
#
# Usage:  tools/mem_scope.sh <command> [args...]
#   e.g.  tools/mem_scope.sh bash -c 'cat ids.txt | xargs -P 16 -I {} ./run_one.sh {}'
set -uo pipefail
MEM_HIGH=${MEM_HIGH:-40G}
MEM_MAX=${MEM_MAX:-46G}
MEM_SWAP=${MEM_SWAP:-2G}

if systemd-run --user --scope --collect -q \
     -p MemoryMax=64M -p MemorySwapMax=16M /bin/true >/dev/null 2>&1; then
  echo "mem_scope: cgroup cap active (MemoryHigh=$MEM_HIGH MemoryMax=$MEM_MAX MemorySwapMax=$MEM_SWAP)" >&2
  # --expand-environment=no: don't let systemd touch $1/$2/... — they belong to
  # the inner command (e.g. a `bash -c` we pass through).
  exec systemd-run --user --scope --collect --expand-environment=no \
       -p MemoryHigh="$MEM_HIGH" -p MemoryMax="$MEM_MAX" -p MemorySwapMax="$MEM_SWAP" \
       -- "$@"
else
  echo "mem_scope: WARNING — systemd --user scope unavailable; running WITHOUT aggregate memory cap" >&2
  exec "$@"
fi
