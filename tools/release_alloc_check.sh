#!/usr/bin/env bash
# release_alloc_check.sh <latexml_oxide binary> — fail unless the shipped Linux binary's allocator initialises.
#
# mimalloc starts through an ELF constructor in `.init_array`. mold 1.0.3 (Ubuntu 22.04's apt package) leaves Rust
# std's `#[used]` init entry in an `.init_array` output section of its own and points DT_INIT_ARRAY at that one only,
# so mimalloc's `_mi_auto_process_init` never runs: the process stays in mimalloc's "preloading" mode, takes every
# 64 KiB page straight from the OS (10,094 mmap / 19,395 munmap on a small document against 96 / 22), and ignores
# `MIMALLOC_*`. 0.7.6 and 0.8.0-rc1's x86_64 Linux binaries shipped it. Two checks:
#   1. structural — DT_INIT_ARRAYSZ covers every `.init_array` section (the loader runs only that range);
#   2. behavioural — `MIMALLOC_SHOW_STATS=1` prints mimalloc's `reserved` line, which only an initialised mimalloc does.
set -uo pipefail

BIN="${1:?usage: $0 <latexml_oxide binary>}"
[ -x "$BIN" ] || { echo "::error::not an executable: $BIN"; exit 2; }

sections=$(readelf -SW "$BIN" | grep -c ' \.init_array ')
total=0
while read -r size; do total=$((total + 16#${size})); done < <(readelf -SW "$BIN" | grep ' \.init_array ' |
  sed -E 's/.*\.init_array +INIT_ARRAY +[0-9a-f]+ +[0-9a-f]+ +([0-9a-f]+) .*/\1/')
dt=$(readelf -dW "$BIN" | awk '/\(INIT_ARRAYSZ\)/ { print $3 }')
status=0
if [ -z "$dt" ] || [ "$dt" -ne "$total" ]; then
  echo "::error::DT_INIT_ARRAYSZ is ${dt:-missing} bytes but the binary's ${sections} .init_array section(s) hold ${total}: some constructors (mimalloc's) never run — a linker that splits .init_array (mold < 2)"
  status=1
fi
if ! MIMALLOC_SHOW_STATS=1 "$BIN" --version 2>&1 | grep -qE '^ *reserved *:'; then
  echo "::error::MIMALLOC_SHOW_STATS=1 printed no mimalloc statistics: the allocator never initialised"
  status=1
fi
[ "$status" -eq 0 ] && echo "OK: allocator initialises (${sections} .init_array section(s), DT_INIT_ARRAYSZ=${dt}, mimalloc stats printed)"
exit "$status"
