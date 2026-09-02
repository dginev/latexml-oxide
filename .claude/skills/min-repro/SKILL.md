---
name: min-repro
description: >
  Reduce a confirmed failing arXiv paper to a minimal, self-contained reproducer
  (and optionally a regression-test fixture), AND isolate which single construct
  or token causes it via a known-good control twin (e.g. `\vspace*` vs `\par`).
  Use after canvas-triage confirms a GENUINE-RUST-ONLY failure, whenever you need
  the smallest .tex that still triggers a specific error/crash, or to pinpoint the
  culprit token behind a Rust-vs-Perl divergence and turn it into the red test.
  Pairs with tools/bisect_repro.sh and first_error.sh.
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
(`latexml repro.tex`, verbose — never `--quiet`) on the same host.
A faithful reproducer should still show the Rust-only delta; if Perl now errors
too, the reduction changed the semantics — back off the last cut.

**5 — Isolate the CAUSE with a control variable** (turns a minimal failure into a
*diagnosis*, and often straight into the red test). Once reduced, produce a
near-identical twin that differs by **one token** and is expected to *work* — a
known-good control for the same operation. The delta between the failing case and
its control pinpoints the culprit and rules out everything they share (schema,
surrounding bindings, the harness). Witness: `\hrulefill\vspace*{4pt}` (fails,
paragraph not closed) vs `\hrulefill\par` (works) proved the bug was the
paragraph-terminator's *arrival*, not `\vskip`/the schema — and the pair became
the guard (`50_structure::vspace_closes_leader_para`). Good controls: the
explicit form of an implicit action (`\par` for `\vspace*`), a sibling macro that
shares the machinery, or the same input one nesting level out. When the failure is
"X doesn't happen," the control is the case where X *does* — compare their runtime
state at the decision point (see `perl-port` §1b, "instrument the gate").

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
divergences"), and confirm `cargo nextest run --workspace` stays green.
