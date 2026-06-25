---
name: min-repro
description: >
  Reduce a confirmed failing arXiv paper to a minimal, self-contained reproducer
  (and optionally a regression-test fixture). Use after canvas-triage confirms a
  GENUINE-RUST-ONLY failure, or whenever you need the smallest .tex that still
  triggers a specific error/crash. Pairs with tools/bisect_repro.sh and
  first_error.sh. Invoke for "minimize this paper", "make a reproducer for X",
  "shrink to a failing case", "/min-repro".
---

Goal: the smallest `.tex` that still emits the **canary** (the exact error
line/class you are chasing). A reproducer that drops the canary is worthless, and
one that adds unrelated errors muddies the signal.

## Workflow

**1 — Pin the canary.** Run `tools/first_error.sh <paper.log>` to get the first
non-cascade error class with source context. That line (or a stable substring of
it) is your canary pattern — everything below preserves it.

**2 — Coarse bisection.** `tools/bisect_repro.sh <arxiv_id> [canary]` does
window-bisection from the first-error line. It narrows to the offending region
without you hand-editing. Respect the documented contract (reads the extracted
paper; `canary` optional and defaults to the first error).

**3 — Manual reduction** (when the script can't go further):
- Strip the preamble bottom-up; keep only `\usepackage`/`\def` lines the canary
  needs. Prefer `\documentclass{article}` unless the class itself is implicated.
- Replace `\input`/`\include` bodies with the minimal triggering snippet.
- Re-run after each cut: `cargo run --bin latexml_oxide -- --format=html5
  --log=r.log --dest=/tmp/r.html repro.tex` then ANSI-strip-grep for the canary
  (`sed 's/\x1b\[[0-9;]*m//g' r.log | grep -E '<canary>'`). Stop when any further
  cut loses the canary.

**4 — Confirm parity intent.** Re-run the *reduced* case through Perl
(`/usr/local/bin/latexml repro.tex`, verbose — never `--quiet`) on the same host.
A faithful reproducer should still show the Rust-only delta; if Perl now errors
too, the reduction changed the semantics — back off the last cut.

## Where the reproducer lands

- **`docs/reproducers/`** — a Rust-only bug we intend to fix (e.g.
  `pcolumn_block_content_in_p.tex`, `dcolumn_empty_todelim_display_math_leak.tex`).
- **`docs/out-of-scope/`** — confirmed out-of-scope (host pkg, DTD, etc.).
- **`docs/known_crashes/`** — a crash we are tracking but not yet fixing.

## Promoting to a regression test

When the fix lands, add a `[name].tex` / `[name].xml` pair under the relevant
crate's test tree (mirroring the Perl `t/` suite). **Run `cargo clean`** —
a compile-time plugin discovers test files, so a new pair is invisible until a
clean rebuild. Generate the expected `.xml` from the *fixed* binary, strip the
intentional-divergence artifacts before committing (no `%&#10;`; `--nocomments`
to drop `<!-- … -->` source-comment lines — see CLAUDE.md "Intentional
divergences"), and confirm `cargo test --tests --no-fail-fast` stays green.
