#!/usr/bin/env bash
# cargo-serialized.sh — run cargo under a shared host-wide flock so
# concurrent agents/worktrees never run rustc at the same time.
#
# Why: when multiple agents run `cargo check`/`cargo build`/`cargo test`
# from parallel worktrees, each spawns its own rustc worker pool.
# On a 32 GB laptop with 6 worktrees that easily totals 20-30 GB RSS +
# swap thrash, and the kernel OOM-killer can reap the wrong process
# mid-compile, leaving a rustc in an unrecoverable state. We've seen it.
#
# This wrapper serialises at the `cargo` invocation level. Inside a
# single invocation rustc parallelism still applies (cargo -j N), but
# only ONE cargo may hold the lock at a time across the whole host.
#
# Usage:
#   tools/cargo-serialized.sh check --workspace
#   tools/cargo-serialized.sh build --release --bin latexml_oxide
#   tools/cargo-serialized.sh test --release --tests --workspace
#
# Environment:
#   LATEXML_CARGO_LOCK    path to the shared lockfile
#                         (default: /tmp/latexml-oxide-cargo.lock)
#   LATEXML_CARGO_LOCK_TIMEOUT    seconds to wait before giving up
#                         (default: 3600). Unset/<=0 = wait forever.
#
# Prints a short "waiting for lock" notice if the lock isn't
# immediately available, so the caller can see the serialisation
# kicking in rather than silently hanging.

set -euo pipefail

LOCKFILE="${LATEXML_CARGO_LOCK:-/tmp/latexml-oxide-cargo.lock}"
TIMEOUT="${LATEXML_CARGO_LOCK_TIMEOUT:-3600}"

# Create the lockfile if absent (shared; world-writable so any agent
# running under the same user account can hold it).
touch "$LOCKFILE"

# Probe the lock non-blocking first — purely so we can log a clear
# "queued behind another cargo" notice when we're about to wait.
if ! flock --nonblock --exclusive "$LOCKFILE" true 2>/dev/null; then
  holder=""
  # `fuser` is the most reliable way to name the current lock holder;
  # fall back gracefully if it's not installed.
  if command -v fuser >/dev/null 2>&1; then
    holder="$(fuser "$LOCKFILE" 2>&1 | awk 'NR==1 {print $0}')"
  fi
  echo "[cargo-serialized] lock busy (${LOCKFILE})${holder:+ — held by: $holder}; waiting up to ${TIMEOUT}s..." >&2
fi

# Acquire the lock and exec cargo. -E keeps environment, -w waits with
# timeout. Using `exec` hands the PID over to cargo so Ctrl-C, signals,
# and exit codes propagate transparently to the caller.
if [ "$TIMEOUT" -gt 0 ] 2>/dev/null; then
  exec flock --exclusive --wait "$TIMEOUT" "$LOCKFILE" cargo "$@"
else
  exec flock --exclusive "$LOCKFILE" cargo "$@"
fi
