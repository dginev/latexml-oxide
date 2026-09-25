# Gemini helper — perfect-kernel delegation brief (round 12)

You are a helper on branch `perfect_kernel` of `~/git/latexml-oxide` (the perfect-kernel
program: `docs/PERFECT_KERNEL.md`, whose **Roadmap** section is the ranked plan). Work on a
branch `gemini/pk-helpers-12` cut from the current `perfect_kernel` tip. Push it, and append to
the **Status** section at the end of this file; never edit task text. Round 11's tasks were not
started; they are carried over as P6-P8. The tasks below come from the roadmap's analysis
drivers, and each was spot-checked by the orchestrator. The Perl source under `LaTeXML/` is
ground truth. pdflatex (lualatex for lualatex-oracle manuals) is the surpass oracle.

## Working rules (unchanged, plus round-2 lessons)

- **No pull requests.** All work lands on `perfect_kernel` through your
  `gemini/pk-helpers-N` branches; the single PR to `main` is opened by the user
  when the branch is ready. Never open a PR (PR #798 was closed for this reason).
- **This file lists only OPEN tasks.** At each merge the orchestrator lifts your
  Status entries into `LEDGER.md`/`KERNEL_CAPABILITIES.md` and deletes them here
  together with the solved task text; a task that is still open is carried over
  under a new number. Append your Status for THIS round below; nothing older
  belongs here.

- **Branch:** `gemini/pk-helpers-12`, branched from the current `perfect_kernel`
  HEAD; rebase before every push; one commit per task, footer
  `Co-Authored-By: Gemini <noreply@google.com>`; never push to `perfect_kernel`.
- **File ownership:** only the files a task names. Guards in
  `latexml_oxide/tests/cluster_package_guards/perfect_kernel_gemini.rs` (the guard binary is
  split into one file per module since 6b20cd6f6d; the module name is unchanged).
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
- **Lesson 8 (round 5) — an engine seam report is a deliverable:** J3's bisect + red repro
  went straight into a kernel fix; keep doing that (file:line of the seam, the commit that
  introduced it, a ≤30-line repro, and the proposed rule).
- **Scope rule (rounds 4–6):** bindings only — no edits under `latexml_core/` or
  `latexml_engine/`; stop at the design + red repro and report it in Status.
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target
  cargo test -p latexml --test cluster_package_guards -- <name> --test-threads=2`),
  `-j 4`, never the full suite or `sweep.sh`. **Every conversion ≤ 3 minutes**
  (`--timeout=180`, outer `timeout 200`). Your worktree has no `resources/dumps/`:
  run `tools/make_formats.sh` once after checkout.
- **Data:** `~/data/perfect_kernel/corpus.tsv`, `~/data/perfect_kernel/oracle_verdicts.tsv`
  (column 3 = engine). The current sweep is **#123**: per-doc logs and XML in
  `~/data/perfect_kernel_s123/<bundle>/<name>/`, HTML recall in `~/data/perfect_kernel_s123_html/s3_verdicts.tsv`.
  The driver investigations behind these tasks, with repros and Perl/pdflatex comparisons, are
  under `~/data/pk_agents/w69/drivers/`: `streamA_probes/`, `streamB_rep/`, `streamE/`. Convert
  from a COPY of the doc dir with `--preload='[rawstyles,rawclasses]latexml.sty'`
  (`[rawstyles,rawclasses,luatex]` for lualatex manuals); errors are ANSI-stripped
  `^Error:|^Fatal:`; clean = `Conversion complete:` + non-trivial XML.
  Every guard asserts WHOLE elements (`latexml::util::test::assert_element`), never substrings.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

Scope for this round: bindings (`latexml_package/`, `latexml_contrib/`) and the files a task
names. The orchestrator is working in `latexml_core/`, `latexml_engine/` (except the one file P3
names) and `latexml_math_parser/`; do not edit them.

### P1 — pstricks drawing commands produce nothing (Rust-only)
`latexml_package/src/package/pstricks_sty.rs:276-298` and `pstricks_support_sty.rs:92+` swallow
the arguments of `\psline`, `\psframe`, `\pscircle`, `\psellipse`, `\psarc`, `\pspolygon`,
`\psdot(s)`, `\psbezier`, `\pscurve`, `\psgrid` and similar. The header comment says "DVI-only".
Perl builds the SVG primitives (LaTeXML/lib/LaTeXML/Package/pstricks_support.sty.ltxml:661-812:
`<ltx:rect>`, `<ltx:line>`, `<ltx:circle>`, …) through its PSCoord / PSDimension / PSAngle
parameter types and `DefPSConstructor`.
- Repro: `\documentclass{article}\usepackage{pstricks}\begin{document}\begin{pspicture}(0,0)(2,2)\psframe(0,0)(1,1)\end{pspicture}\end{document}`.
  Rust gives an empty `<picture … width="56.9pt"/>`. Perl gives
  `<g><rect fill="none" height="39.37" stroke="black" stroke-width="0.8" width="39.37" x="0" y="0"/></g>`.
- Also: `\pscircle[linewidth=1pt]{0.5}` leaks `.5` as text.
- Port the parameter types and the constructors, following Perl's line order.
- Batch 56ir (e79e23ec0d) just changed the framed boxes in the same file (`\psframebox` family,
  `lx_in_pspicture`, DIVERGENCES #297): keep them.
- Witnesses: the pstricks manuals under `~/data/perfect_kernel_s123` (grep `pstricks` in
  corpus sources), e.g. pst-poker-doc, egameps, lsc, pst-vowel, dsptricks.
- Guard: whole `<g>`/`<rect>`/`<line>` elements for a 4-primitive pspicture, 0 errors.

### P6 — forest: node keys become structure (carried over from round 11, N1)
`parse_forest_tokens` (latexml_contrib/src/forest_sty.rs) keeps a node's key list only as an
opaque string, and `\forestset`/`\bracketset` are `\relax`.
- Parse the comma list with pgfkeys' `key=value`/`{…}` rules (forest.sty
  `\forest@node@parseoptions`; cite the line).
- Map the label-affecting keys: `edge label` becomes a child
  `<ltx:text class="ltx_forest_edge_label">`; `tier` and `phantom` become classes (a phantom
  node emits no label); `name=` becomes `xml:id="forest.<name>"`.
- Record `\forestset{…}`/`\bracketset{…}`/`for tree={…}` as tree-wide defaults, applied before
  each node's own keys.
- Witnesses: forest-quickstart, fragoli_doc, milsymb, forest-doc; their errors must not rise.

### P7 — forest: `\Forest*`/`{forest*}` grouping and the library preambles (carried over, N2)
- The starred forms (forest.sty:8511 `\forest@group@env`) emit an inline
  `<ltx:inline-block class="ltx_forest_tree">`; the plain forms emit a block.
- `\useforestlibrary{…}` and `\forestapplylibrarydefaults{…}` record the library name in a
  document property (today `\relax`, forest_sty.rs:424).
- The linguistics library's `sn edges`/`nice empty nodes` keys must parse, through P6's parser.

### P8 — chemnum: the compound declaration's id resolves (carried over, N3)
`\refcmpd` emits `<ltx:ref>` to `cmpd.<label>` (chemnum_sty.rs:171-216). Make the DECLARATION
carry `xml:id="cmpd.<label>"` on an element the post stage indexes. Check this with an HTML
conversion of the round-10 guard doc: there must be a resolved `<a href="#cmpd.first">`.

## Status (Gemini → orchestrator; append-only, newest last; round 12 only)

