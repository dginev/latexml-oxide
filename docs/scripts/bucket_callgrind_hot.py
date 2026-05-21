#!/usr/bin/env python3
"""Bucket callgrind hot frames by symbol category.

Use after a callgrind capture:

    cargo build --profile bench --bin cortex_worker --features cortex
    LATEXML_GRAPHICS_CACHE_OFF=1 valgrind --tool=callgrind \
        --callgrind-out-file=/tmp/callgrind.out --dump-instr=no \
        ./target/release/cortex_worker --standalone --input <zip> --output /tmp/cg.zip
    python3 docs/scripts/bucket_callgrind_hot.py /tmp/callgrind.out \
        ./target/release/cortex_worker

Useful when `perf record` is blocked by `perf_event_paranoid≥2` and
`callgrind_annotate` can't auto-resolve our Rust symbols (it shows
addresses as `???:0x…`). This script pipes the top frames through
`addr2line` and groups them into engine-level buckets.

The bucket rules below are tuned for latexml-oxide's typical hot
list (state lookups, token-vec allocation, latex_constructs
closures). Edit when audit needs to differentiate further.
"""

import os
import re
import shutil
import subprocess
import sys
from collections import defaultdict


def bucket_of(sym: str) -> str:
    if "load_definitions::{closure" in sym:
        return "engine:latex_constructs closures (raw-TeX macro bodies)"
    if "token::Token" in sym:
        return "Vec<Token> alloc/build"
    if "state::TableName" in sym:
        return "State map (TableName,Symbol,Stored) alloc"
    if "Option<alloc::borrow::Cow<str>>" in sym:
        return "Option<Cow<str>>::as_ref (state lookups)"
    if "Tokens" in sym and "fmt" in sym:
        return "Tokens fmt::{Debug,Display}"
    if "arena::" in sym:
        return "arena interner"
    if "mouth::Mouth" in sym:
        return "Mouth I/O"
    if "libxml" in sym:
        return "libxml ops"
    if "rewrite::" in sym:
        return "rewrite stage"
    if "roman_aux" in sym:
        return "common::cleaners::roman_aux"
    if "btree::" in sym:
        return "btree ops"
    if "copy_nonoverlapping" in sym:
        return "memcpy/copy_nonoverlapping"
    if sym == "??":
        return "[unresolved address]"
    return f"other: {sym[:80]}"


def main(argv: list[str]) -> int:
    if len(argv) != 3 or argv[1] in ("-h", "--help"):
        print(__doc__, file=sys.stderr)
        return 2
    callgrind_out, binary = argv[1:3]

    for tool in ("callgrind_annotate", "addr2line"):
        if shutil.which(tool) is None:
            print(f"error: {tool} not found on PATH", file=sys.stderr)
            return 1
    if not os.path.exists(callgrind_out):
        print(f"error: {callgrind_out} does not exist", file=sys.stderr)
        return 1
    if not os.path.exists(binary):
        print(f"error: {binary} does not exist", file=sys.stderr)
        return 1

    result = subprocess.run(
        ["callgrind_annotate", callgrind_out],
        capture_output=True, text=True, check=True,
    )
    acc: dict[str, float] = defaultdict(float)
    n = 0
    for line in result.stdout.splitlines():
        if "???:" not in line:
            continue
        m = re.search(r"\(\s*([0-9.]+)%\).*?(0x[0-9a-f]+)", line)
        if not m:
            continue
        pct, addr = float(m.group(1)), m.group(2)
        sym = subprocess.run(
            ["addr2line", "-e", binary, "-f", "-C", addr],
            capture_output=True, text=True,
        ).stdout.split("\n")[0]
        acc[bucket_of(sym)] += pct
        n += 1
        if n >= 60:
            break

    total = sum(acc.values())
    print(f"Top-{n} hot frames covering {total:.1f}% of total instructions:")
    for k, v in sorted(acc.items(), key=lambda kv: -kv[1]):
        print(f"  {v:5.2f}%  {k}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
