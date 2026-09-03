---
name: latexml-min-repro
description: Reduce a confirmed latexml-oxide failure to the smallest self-contained TeX reproducer while preserving the exact Rust-only canary and a known-good control. Use after triage, for crashes or diagnostic divergences, or when creating a focused regression fixture.
---

# Reduce a confirmed failure

The target is the smallest self-contained input that still emits the exact
diagnostic, crash, or structural canary. A smaller file that loses the canary is
not a reproducer.

## Reduction sequence

1. Pin the first non-cascade signal with `tools/first_error.sh <log>`. Choose a
   stable, specific substring or structural assertion as the canary.
2. Run `tools/bisect_repro.sh <arxiv-id> [canary]` for coarse source-window
   reduction before hand-editing a large paper.
3. Reduce manually one dependency at a time: simplify to `article` unless the
   class is implicated, remove preamble entries bottom-up, inline required
   inputs, and delete unrelated body regions. Re-run after every cut.
4. Run the reduced input through verbose same-host Perl. The Rust-only delta must
   remain; if Perl now fails in the same way, restore the last semantic cut.
5. Make a known-good twin differing by one token or state transition. Compare
   runtime state at the failing gate. This control often identifies whether the
   observed primitive is the cause or merely the first victim.

Example rerun shape:

```bash
cargo run --bin latexml_oxide -- --format=html5 --log=repro.log \
  --dest=/tmp/repro.html repro.tex
sed 's/\x1b\[[0-9;]*m//g' repro.log | grep -E '<canary>'
```

## Place and promote

- Intended Rust fix: `docs/reproducers/` until promoted to a test.
- Confirmed external/out-of-scope case: `docs/out-of-scope/`.
- Unresolved crash: `docs/known_crashes/`.

Put the eventual test at the layer that owns the bug. Core conversion commonly
uses a `.tex`/`.xml` pair; post-processing or CLI behavior needs its corresponding
Rust harness. Hand-author the red assertion before the fix. Do not bless output
from the buggy binary.

A new `.tex`/`.xml` pair is discovered at compile time, so run `cargo clean`
once, prove the focused test is red for the intended reason, then implement and
prove it green. Re-run the original paper after the reduced guard passes.

