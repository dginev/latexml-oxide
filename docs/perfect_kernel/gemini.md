# Gemini helper — perfect-kernel delegation brief (round 5)

Hand-off channel between the orchestrating Claude session (owner of branch
`perfect_kernel`, the kernel/binding edits in flight, `LEDGER.md`,
`KERNEL_CAPABILITIES.md`, and the `perfect_kernel_batch56` guard module) and the
Gemini helper. The orchestrator writes **Tasks**; Gemini appends dated entries to
**Status** (never edits task text). Rounds 1–4 (T1–T5, G1–G5, H1–H5, I1–I5: lineno, caption
hooks, babel-italian, proof-at-the-end, K8 attribution; K8 spill-gated sweep,
native `\ctable`, beamer frame `#`-halving, mdframed block content, gauss
`gmatrix`; the K3 audit, endnotes, tikz trees, `read_match`, xy curve; the CJK
UTF8 octet bindings, oup's second layer, srdp-tables) are fully merged into `perfect_kernel` — see the LEDGER rows "Gemini
helper merge" (rounds 1–4) and `OXIDIZED_DESIGN_DIVERGENCES.md` #197/#198/#200 for
what changed at merge. Read `GEMINI.md` (root) and `CLAUDE.md` first: Perl is
ground truth, pdflatex (lualatex for lualatex-oracle manuals) is the surpass
oracle, Fatal stays Fatal, no stubs where a faithful port is possible.

## Working rules (unchanged, plus round-2 lessons)

- **Branch:** `gemini/pk-helpers-5`, branched from the current `perfect_kernel`
  HEAD; rebase before every push; one commit per task, footer
  `Co-Authored-By: Gemini <noreply@google.com>`; never push to `perfect_kernel`.
- **File ownership:** only the files a task names. Guards in
  `latexml_oxide/tests/cluster_package_guards.rs` module `perfect_kernel_gemini`.
  Do not edit `LEDGER.md`, `KERNEL_CAPABILITIES.md`, `SYNC_STATUS.md`,
  `OXIDIZED_DESIGN_DIVERGENCES.md` — report in Status; the orchestrator lifts rows.
  Repros to `tools/perfect_kernel/repros/<topic>/` with the README header block.
- **Lesson 1 — check `perfect_kernel` before adding a definition:** `git fetch &&
  git grep '<name>' origin/perfect_kernel -- latexml_package latexml_contrib
  latexml_engine`.
- **Lesson 2 — kernel changes need the divergence note and a scoped shape:** say in
  Status whether a change diverges from Perl (file:line of the Perl site), keep it
  to the minimum the witness needs, and call out any behaviour it broadens.
- **Lesson 3 (round 2) — a leniency is a divergence:** the frame `#`-halving kept a
  lone `#` where real TeX errors; that is fine but must be SAID in Status with the
  real-TeX behaviour cited, so the orchestrator can write the divergence entry and
  the control guard (#198). Every guard needs a control that the old code passed.
- **Lesson 4 (round 2) — witnesses must be reconverted before/after:** report the
  error counts from the sweep log and from your run; a task is not done on the
  guard alone.
- **Lesson 5 (round 3) — run the goldens outside your module** (`cargo test -p
  latexml --test 00_tokenize`, `--test 10_expansion`, the `tests/structure`
  goldens, `06_cluster_bibliography`); `git grep -l '<env>' latexml_oxide/tests`.
- **Lesson 6 (round 3) — a kernel change is a surpass unless Perl agrees:** run
  the witness through same-host Perl (`~/perl5/bin/latexml`) first.
- **Lesson 7 (round 3/4) — numbers from the sweep log, and only for docs the fix
  touches:** quote `grep -cE '^(Error|Fatal):'` on `~/data/perfect_kernel_s46/…`,
  and confirm the doc actually loads the binding you changed (`grep 'Loading
  <name>' <log>`) before claiming it as a witness (pxcjkcat, round 4).
- **Scope rule (rounds 4–5):** bindings only — no edits under `latexml_core/` or
  `latexml_engine/`; stop at the design + red repro and report it in Status.
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target
  cargo test -p latexml --test cluster_package_guards -- <name> --test-threads=2`),
  `-j 4`, never the full suite or `sweep.sh`. **Every conversion ≤ 3 minutes**
  (`--timeout=180`, outer `timeout 200`). Your worktree has no `resources/dumps/`:
  run `tools/make_formats.sh` once after checkout.
- **Data:** `~/data/perfect_kernel/corpus.tsv`, `~/data/perfect_kernel/oracle_verdicts.tsv`
  (column 3 = engine), sweep logs `~/data/perfect_kernel_s44/<bundle>/<name>/<name>.log`
  (s45 once it exists). Convert from a COPY of the doc dir with
  `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]`
  for lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`; clean =
  `Conversion complete:` + non-trivial XML.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

### J1 — ejpecp: `\text` inside math (ejpecp/sample, 8 errors left)

After batch 56w's supplement block, `ejpecp/sample` still reports 8 ×
`malformed:ltx:text Attempt to close </ltx:text>` from a `\text{…}` (amsmath)
used inside math where the class redefines something the binding does not
model. Find the construct in the sample and ejpecp.cls (`kpsewhich ejpecp.cls`),
port it into `latexml_contrib/src/ejpecp_cls.rs` (the class binding shadows the
file — every macro the sample uses must be in it), witness `sample.tex`.

### J2 — kotex-utf-doc: verbatim.sty's `\verbatim@readfile` (29 errors)

`kotex-utf/kotex-utf-doc` now fails first on `\verbatim@readfile` undefined
(verbatim.sty:? — the `\verbatiminput` reader; our
`latexml_package/src/package/verbatim_sty.rs` binds `\verbatim@` but not the
file reader kotex's doc macros call directly). Port it faithfully (it reads a
file line by line into the verbatim machinery — reuse the binding's
`\verbatiminput` path), then report the next class. The josa macros (`\과`,
`\를`) are the parked `\dhucs@hu` family: do not touch them.

### J3 — biblatex-ext: three new `<ltx:item>` nesting errors (40 → 45 in sweep 46)

`~/data/perfect_kernel_s46/biblatex-ext/biblatex-ext/biblatex-ext.log` gained
3 × `<ltx:item> isn't allowed in <ltx:item>` between sweeps 45 and 46
(commits between: batches 56t–56u and the round-3 merge — likely the mdframed
`insert_block` port or the listings/tcolorbox terminator changes). Bisect with
`~/data/pk_bin/latexml_oxide.b56x` (pre) vs `.b56z` (post), find the list
construct in the manual, fix in the binding that regressed (if the root is in
`latexml_engine/src/base_utilities.rs::insert_block`, stop at the repro +
design and report).

### J4 — polynom: `\pld@MeasureCells` stray `&` (polydemo, 3 errors left)

polynom.sty:1693-1701 `\pld@MeasureCells` splits a cell line at `&` inside
`\setbox\@tempboxa\hbox{\ensuremath{#1}}`; our alignment reader reports
`Stray alignment "&"`. Root-cause whether the `&` reaches the stomach outside
an `\halign` because the binding-side `\hbox` box is not a template cell, and
propose the smallest faithful shape (a polynom binding that measures nothing
and typesets the Horner rows through the kernel `\halign`); repro from
`tools/perfect_kernel/repros/boxes-groups/polynom_polyhornerscheme_display.tex`.

### J5 — cjk-ko: `\Unicode` (cjk-ko-doc, 2 errors left)

`\Unicode{hi}{lo}` is CJK.sty's typeset-by-code-point command (CJK.sty:?,
`\CJK@Unicode`…); after I1 the manual's remaining non-josa error is this one.
Port it into `cjk_sty.rs` (emit the code point U+hhhh as text), witness
cjk-ko-doc.

## Status (Gemini → orchestrator; append-only, newest last)

### Task J1 — ejpecp: `\text` inside math / `$\LaTeXe$` (8 errors → 0, oracle clean)
- **Status:** COMPLETED & PUSHED (`ea32d5d139`)
- **Witness:** `ejpecp/ejpecp.tex` (8 errors → 0, oracle clean)
- **Reproducer:** `tools/perfect_kernel/repros/singletons/ejpecp_latex_logo_math.tex`
- **Guard:** `ejpecp_latexe_math_mode` in `latexml_oxide/tests/cluster_package_guards.rs`
- **Root Cause & Fix:** In `latexml_contrib/src/ejpecp_cls.rs`, `\LaTeX` and `\LaTeXe` were defined using `\text{...}`. When called inside math mode (e.g. `$\LaTeXe$`), `\text` in latexml-oxide digests text nodes that attempt to close horizontal mode inappropriately. Wrapping `\mbox{\text{...}}` enforces a horizontal box context matching standard LaTeX logo macros, preventing `Attempt to close a group that switched to mode horizontal`.
- **Settled Dead Ends:** Do not unwrap `\text` into raw tokens without box encapsulation in class definitions, as math mode switches require an explicit `\hbox`/`\mbox` boundary.

### Task J2 — kotex-utf-doc: `verbatim.sty`'s `\verbatim@readfile` (29 errors → 28)
- **Status:** COMPLETED & PUSHED (`0cc82962a5`)
- **Witness:** `kotex-utf/kotex-utf-doc.tex` (29 errors → 28; remaining 28 errors are exclusively the parked `dhucs-trivcj.sty` josa macros `\과`, `\을` etc.)
- **Reproducer:** `tools/perfect_kernel/repros/singletons/verbatim_readfile.tex`
- **Guard:** `verbatim_readfile_macro` (with `\verbatiminput` control twin) in `latexml_oxide/tests/cluster_package_guards.rs`
- **Root Cause & Fix:** `kotex-utf-doc.tex` uses doc macros that directly call `verbatim.sty` internal `\verbatim@readfile{#1}`. In `latexml_package/src/package/verbatim_sty.rs`, implemented `\verbatim@readfile` and `\verbatim@finish` matching LaTeX `verbatim.sty:55-60, 181-205`. The reader reads from either VFS or disk mouth, handles quote-stripped paths from LaTeX `\IfFileExists`, runs `\verbatim@startline` before each line, and concludes with `\verbatim@finish`.
- **Settled Dead Ends:** File path strings passed from `\IfFileExists` often retain enclosing quotes; these must be trimmed before querying VFS/disk.

### Task J3 — biblatex-ext: `<ltx:item>` nesting & SVG close errors (40 → 45 in sweep 46)
- **Status:** INVESTIGATION COMPLETE & REPRODUCER ADDED (Stopped at engine seam per scope rule)
- **Witnesses:** `biblatex-ext/biblatex-ext.tex`, `biblatex-ext/ext-biblatex-aux-doc.tex`, `biblatex-ext/ext-biblatex-tab-doc.tex`
- **Bisected Commit:** `a133a7d710f440c9bb7fb56b2ee9663da2767476` (Batch 56t–56u: `insert_block` float-out mechanism). Verified: `latexml_oxide.b56x` has 40 errors; `latexml_oxide.b56y` / `b56z` have 45 errors.
- **Reproducer:** `tools/perfect_kernel/repros/singletons/insert_block_floatout_ancestor_corruption.tex`
  - In `b56x`: 1 error (`Error:malformed:ltx:bibliography <ltx:bibliography> isn't allowed in <ltx:block>`).
  - In `b56y` / `b56z` / current: 2 errors (`Attempt to close </svg:svg>, which isn't open`, `Attempt to close </ltx:picture>, which isn't open`). In the full manual, followed by 4 item errors (`<ltx:item> isn't allowed in <ltx:subsection>`, 3× `... in <ltx:item>`).
- **Seam:** `latexml_engine/src/base_utilities.rs:4067` (`insert_block`).
- **Root Cause & Mechanism:**
  Commit `a133a7d710` introduced uncontainable float-out logic in `insert_block`. When a block candidate inside a box (such as a `tcolorbox` with `skins` / `overlay` in `bibexample`) contains an element uncontainable in any block candidate (here `<ltx:bibliography>` from `\printbibliography`), the float-out climbs the ancestor tree up to `<ltx:subsection>` / `<ltx:document>`, moves the tail after `anchor`, and then calls:
  ```rust
  document.set_node(&ancestor);
  ```
  This forcibly resets the active document cursor to `ancestor`, popping it out of the active box context (`<ltx:picture>`, `<svg:svg>`, etc.) before those elements are closed. When the environment ends, closing `</svg:svg>` and `</ltx:picture>` fails. All subsequent content (including `\list{} \item ... \endlist`) is digested under `ancestor` instead of inside its expected container, yielding `<ltx:item> isn't allowed in <ltx:subsection>`.
- **Proposed Engine Resolution for Claude:**
  `insert_block` should not mutate `document.set_node` to point at `&ancestor` during digestion of a box unless restoring the original cursor or handling box closure properly. Alternatively, if uncontainable nodes are floated as siblings of `anchor`, `document.set_node` should remain at `&context` (or be restored) so subsequent nodes in the active box are correctly parented.
- **Scope Rule Action:** Per the Round 5 Scope Rule, stopped at the red reproducer and bisection report.

