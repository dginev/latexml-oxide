# Gemini helper — perfect-kernel delegation brief

This file is the hand-off channel between the orchestrating Claude session (owner of
branch `perfect_kernel`, the kernel/binding edits in flight, `LEDGER.md`, and the
`perfect_kernel_batch56` guard module) and the Gemini helper agent. The orchestrator
writes the **Tasks** section; Gemini writes the **Status** section at the bottom
(append-only, one dated entry per task, never edit the task text). Read `GEMINI.md`
(root) and `CLAUDE.md` first: Perl is ground truth, pdflatex (lualatex for
lualatex-oracle manuals) is the surpass oracle, Fatal stays Fatal, no stubs where a
faithful port is possible.

## Working rules for this brief

- **Branch:** work on `gemini/pk-helpers`, branched from the current `perfect_kernel`
  HEAD; rebase onto `perfect_kernel` before every push (the orchestrator lands batches
  there several times a day). One commit per task, footer
  `Co-Authored-By: Gemini <noreply@google.com>`. Never push to `perfect_kernel` or
  `main`; the orchestrator cherry-picks or merges.
- **File ownership.** Edit only the files a task names. Guards go in
  `latexml_oxide/tests/cluster_package_guards.rs` in a NEW module
  `perfect_kernel_gemini` (create it at the end of the file; copy the `convert`,
  `convert_with`, `convert_args`, `error_count` helpers from
  `perfect_kernel_batch46`/`batch56` by `use super::...`, do not touch those modules).
  Do not edit `docs/perfect_kernel/LEDGER.md`, `KERNEL_CAPABILITIES.md`, or
  `SYNC_STATUS.md` — report in **Status** below and the orchestrator lifts the rows.
  Repros go to `tools/perfect_kernel/repros/<topic>/` with the header conventions of
  `tools/perfect_kernel/repros/README.md` (a `% witness:` / `% oracle:` / `% expect:` /
  `% status:` block).
- **Thermals (docs/THERMALS.md):** the orchestrator runs the full `cargo nextest`
  suite and corpus sweeps; you run only targeted guards
  (`CARGO_TARGET_DIR=$HOME/data/gemini_target cargo test -p latexml --test
  cluster_package_guards -- <name> --test-threads=2`) and single conversions, never
  the full suite, never `sweep.sh`. `-j 4` for cargo. **Every conversion you start is
  capped at 3 minutes** (`--timeout=180` plus an outer `timeout 200`); a probe that
  needs longer is the wrong probe.
- **Binaries and data.** Debug binary of your branch: build with
  `CARGO_TARGET_DIR=$HOME/data/gemini_target cargo build -p latexml --bin latexml_oxide`.
  Corpus sources: `~/data/perfect_kernel/corpus.tsv` (bundle, tex path, pdf path);
  oracle verdicts `~/data/perfect_kernel/oracle_verdicts.tsv` (column 3 = engine);
  sweep-43 per-doc logs `~/data/perfect_kernel_s43/<bundle>/<name>/<name>.log`.
  Convert a manual from a COPY of its directory (the doc dirs are read-only):
  `cd <copy> && latexml_oxide --timeout=180 --preload='[rawstyles,rawclasses]latexml.sty'
  --dest=out.xml <name>.tex 2> stderr.txt`; use
  `[rawstyles,rawclasses,luatex]latexml.sty` for lualatex-oracle manuals. Errors are
  the ANSI-stripped `^Error:|^Fatal:` lines (`sed 's/\x1b\[[0-9;]*m//g'`). A run is
  clean only with a `Conversion complete:` line and a non-trivial XML.
- **Definition of done per task:** red repro → fix → green guard → the named witness
  manuals reconvert with 0 errors (or the task's stated residual) → `cargo +nightly fmt
  --all`, `cargo clippy --workspace --all-targets -- -D warnings` clean → commit →
  Status entry naming the guard, the witnesses' before/after error counts, and any
  settled dead end (one line each).

## Tasks (orchestrator → Gemini), in priority order

### T1 — K8 memory lever: attribute the stale-`node_boxes` sweep slowness (perf)

Context: `docs/perfect_kernel/KERNEL_CAPABILITIES.md` K8 log entries dated
2026-09-05. `Document::node_boxes` (latexml_core/src/document.rs) pins a `Digested`
per constructed node; streaming pass 1 sweeps entries whose node is no longer in the
live tree (`sweep_stale_node_boxes`), gated by a COUNT threshold
(`latexml_oxide/src/core_interface.rs::node_boxes_sweep_threshold`, default 1,000,000;
env `LXML_NODE_BOXES_SWEEP=<n>` overrides, `LXML_TRACE_NODE_BOXES=1` logs each sweep).
Measured on glossaries-user.tex (1.5 MB source, ~1,500 tcolorbox pictures, each
picture pinning ~0.6 MB): sweep every fragment → peak RSS 4.76 GB → 1.03–1.72 GB,
but wall time ~10× (killed at 12 min; 250 s without the sweep), in both the
big-fragment regime (`--max-memory=6144`, 2 fragments) and the frequent-yield regime
(`--max-memory=2048`, 1,024 fragments). The sweep itself is a mark over the post-spill
spine plus a `retain`, and the mark is cheap, so the cost is unexplained: candidates
are the mass drop of the swept `Digested` trees, a hot path that re-derives something a
swept entry used to answer, or the mark walking a large OPEN subtree.

Deliverable: (1) a synthetic document that reproduces the slowdown in **under 3
minutes** (start from 300 `\begin{tcolorbox}` boxes, scale until the sweep-on run is
measurably slower than sweep-off; `~/data/pk_probe/nb/t.tex` is the 300-box seed);
(2) `perf record -g` (or `LXML_TRACE_*`-style timing around the sweep and around
`spill_closed_subtrees`) attributing the time; (3) a fix proposal — likely sweeping by
WEIGHT or after every spill with the walk bounded to closed subtrees — with the
measured before/after (RSS peak and wall) on the synthetic doc. Do not change the
default threshold in the same commit as the attribution; land the attribution first.
Files: `latexml_oxide/src/core_interface.rs`, `latexml_core/src/document.rs` (the
sweep and spill functions only).

### T2 — lineno.sty binding gaps (witness lineno/ulineno, 18 errors)

`latexml_package/src/package/lineno_sty.rs` reimplements lineno with
`DefEnvironment`s; ulineno.tex (the user manual, pdflatex-clean) reaches
`\linenumberwidth` first and 17 more names after it (sweep-43 log
`~/data/perfect_kernel_s43/lineno/ulineno/ulineno.log`). Port the missing user-level
surface faithfully from lineno.sty (`kpsewhich lineno.sty`; Perl reference
`LaTeXML/lib/LaTeXML/Package/lineno.sty.ltxml` — note it omits `bframe`, which batch
56o added). Line numbers are presentation: what must survive is the TEXT and the
structure, so `\linenumberwidth` and friends are registers/macros, `\linelabel` is a
`\label`, `\lineref`/`\linerefp` are `\ref`s. Deliverable: guard
`lineno_manual_surface`, ulineno 18 → 0 (or list the residual with a reason).

### T3 — proof-at-the-end: a file written and input in the same run

Witness proof-at-the-end/proof-at-the-end_demo (pdflatex-clean after two runs):
`Error:missing_file:proof-at-the-end_demo-pratenddefaultcategory.tex`. The package
`\openout`s `\jobname-pratend<category>.tex` during the document and `\input`s it at
`\pratend` / at the end. Establish from proof-at-the-end.sty (`kpsewhich`) exactly when
the write happens and when the read happens, and what latexml-oxide's virtual file
store does for `\openout`/`\write`/`\closeout` followed by `\input` of the same name
in one run (`latexml_core/src/binding/virtual_files.rs`, the `\openout`/`\write`
primitives: `grep -rln '\\openout' latexml_engine/src latexml_core/src` → latexml_engine/src/tex_file_io.rs ). Perl LaTeXML's behavior (`LaTeXML/lib/LaTeXML/Core/…` `\openout`) is the
parity reference; pdflatex needs two runs, so the surpass target is "the second-run
output": content written earlier in the same run is readable. If the read precedes
the write (end-of-document flush), the faithful answer is the second-run shape: input
the file from the previous run's on-disk copy when present, else empty with a Warn.
Deliverable: guard `openout_then_input_same_run`, the witness 2 → 0.

### T4 — shtthesis (lualatex): `\caption@beginhook` and the caption internals

Witness shtthesis/shtthesis-user-guide (17 errors after batch 56o; first
`Error:undefined:\caption@beginhook`). shtthesis.cls patches caption.sty internals
(`caption3.sty`: `\caption@beginhook`, `\caption@endhook`, …). Our caption binding is
`latexml_package/src/package/caption_sty.rs` (Perl `caption.sty.ltxml`); bindings
outrank raw, so the binding must expose the hook surface classes patch
(`\caption@beginhook`/`\caption@endhook` as `\@empty`-initialised macros that
`\g@addto@macro` can extend, and whatever else the log lists). Deliverable: guard
`caption_hook_surface_for_class_patches`, shtthesis 17 → as low as the caption family
allows; list the non-caption residual by first error.

### T5 — babel-italian `\unit`/`\ap`/`\ped` under a class-set ISO compliance

Witnesses verifica/example4, example5 (pdflatex-clean). italian.ldf:156-164 sets
`\unit=\bbl@it@unit` at `\AtBeginDocument`, gated on `\it@ISOcompliance≠0`, which
verifica.cls:65 sets in ITS `\AtBeginDocument{\@ifpackagewith{babel}{italian}
{\setISOcompliance}…}`. In pdflatex the class hook runs BEFORE the ldf hook (the class
registers first: `\documentclass` precedes `\usepackage{babel}`); in latexml-oxide the
counter is still 0 when the ldf hook fires. Root-cause the ORDER: how
`latexml_engine/src/latex_constructs/sect02.rs::at_document_hook` stores and fires
`\AtBeginDocument` bodies (there are three stores: the raw `#`-bearing chunks, the
L3 `begindocument` hook, and the bindings' private store) versus latex.ltx's single
`\@begindocumenthook` list (latex.ltx:`\AtBeginDocument`), and where babel's
`.ldf` load registers relative to the class. **Read-only for sect02.rs** — the
orchestrator owns that file; deliver the mechanism with file:line, a minimal
pdflatex-clean repro (two `\AtBeginDocument` registrations whose order matters), and
the proposed ordering rule, in Status. If the fix is confined to the babel binding
(`latexml_package/src/package/babel_sty.rs` or the italian ldf handling), land it.

## Status (Gemini → orchestrator; append-only, newest last)

_(empty)_
