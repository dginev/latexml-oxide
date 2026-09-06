# Gemini helper — perfect-kernel delegation brief (round 7)

You are a helper on branch `perfect_kernel` of `~/git/latexml-oxide` (the perfect-
kernel program: `docs/PERFECT_KERNEL.md`). Work on a branch `gemini/pk-helpers-7`
cut from the current `perfect_kernel` tip (which now contains batch 56z AND your
round-6 merge, commit `02cd6e2c70`), push it, and append to the **Status** section at
the end of this file (never edit task text). Rounds 1–6 are merged into
`perfect_kernel`; the orchestrator generalised your L5 at merge (the kernel's
`\@currsize` is now a `\let`, sect13.rs — the five class copies are gone) and
recorded L1's `\XC@getcolor` simplification and L9's `\maketitle` unlock as
correctness items in `docs/perfect_kernel/KERNEL_CAPABILITIES.md`. The Perl source
under `LaTeXML/` is ground truth, pdflatex (lualatex for lualatex-oracle manuals)
is the surpass oracle; sweep #49 (batch 56z) and #50 (your round 6) logs land in
`~/data/perfect_kernel_s49/` and `~/data/perfect_kernel_s50/` as `<bundle>/<name>/<name>.log`.

## Working rules (unchanged, plus round-2 lessons)

- **This file lists only OPEN tasks.** At each merge the orchestrator lifts your
  Status entries into `LEDGER.md`/`KERNEL_CAPABILITIES.md` and deletes them here
  together with the solved task text; a task that is still open is carried over
  under a new number. Append your Status for THIS round below; nothing older
  belongs here.

- **Branch:** `gemini/pk-helpers-6`, branched from the current `perfect_kernel`
  HEAD (batch 56x lands there shortly — rebase onto it when it does; it changes
  `\begin`/`\end` hooks, `\iffontchar`, fontenc, `\parbox`); rebase before every push; one commit per task, footer
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
  (column 3 = engine), sweep logs `~/data/perfect_kernel_s47/<bundle>/<name>/<name>.log`
  (s47 is the current sweep: `~/data/perfect_kernel_s47/…`; residue table with first errors: `~/data/pk_agents/w16/residue47_first_errors.tsv`). Convert from a COPY of the doc dir with
  `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]`
  for lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`; clean =
  `Conversion complete:` + non-trivial XML.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

### M1 — the algpseudocodex binding (carried over from round 6, still open)
when done, NOT `perfect_kernel`): create `latexml_contrib/src/algpseudocodex_sty.rs`
(register in `latexml_contrib/src/lib.rs`, `pub mod` there) so the package stops
raw-loading. The full work-prep is in `scratchpad/ALGPSEUDOCODEX_HANDOFF.md` in the
main checkout (read it first: symptoms, source lines algpseudocodex.sty:139/447/460/
491/896-925, the witness `scratchpad/new_2511.21969/` main `main-ieee.tex`, the
tabto_sty.rs precedent #151). Scope: `RequirePackage!("algorithmicx")`, then override
`\Comment` (one-line right-flushed `⊳ … ⊲`, honouring `italicComments`, no
`\settowidth`/`\tabto`/varwidth minipage), `\LComment` (full-width line, no zero-height
leader), and `\BeginBox`/`\EndBox`/`\BoxedString`/`\algpx@drawCodeBox` (pass content
through inside an in-flow bordered wrapper carrying the `draw=` colour and dash style
as classes — the two-pass tikz overlay cannot run). Deliverables: the binding, a
guard (a `\Comment` line is single-height: no empty numbered line; the box wrapper
exists with its colour class), a DIVERGENCES entry drafted in Status (the
orchestrator lifts it), and the witness's Algorithm 2 converted with 0 errors.
Report pdflatex-vs-ours shape (the PDF is arxiv.org/pdf/2511.21969). When you
finish, say so in Status; the orchestrator deletes the handoff file.

### M2 — `\maketitle` honours the class's dropped body (generalises your L9): the lock
(`latexml_core/src/state.rs`, the `<cs>:locked` drop that already records
`<cs>:redefined` and, since 56z, `<cs>:redefined@nargs`) should also keep the dropped
definition's BODY as `<cs>:redefined@body` (Stored::Tokens of the Expandable's
expansion) for `\def`/`\gdef`/`\renewcommand`/`\newcommand` drops. Then the kernel
`\maketitle` (`latexml_engine/src/latex_constructs/sect05.rs` ~956, the macro ending
in `\lx@maketitle@cleanup`) runs that body AFTER `\lx@frontmatterhere` when
`\maketitle:redefined` is set, so a class that layers real work onto `\maketitle`
(uspatent.cls:188-191 `\patentTitlePage\patentStart`) gets it without a per-class
unlock. Retire `uspatent_cls.rs`'s unlock (keep the binding file if the raw class
needs anything else; otherwise delete it and the dispatch row). Watch: the dropped
body must not re-emit the title (our frontmatter already did) — reason through
uspatent's `\patentTitlePage` and report what it would typeset; if it duplicates
the title block, the body should run with `\@title`/`\@author` emptied (the
`\lx@maketitle@cleanup` shape). Guard: uspatent repro stays 0 errors with
`[0001]` numbering, and a control class whose `\maketitle` redefinition only adds
`\thispagestyle{empty}` converts unchanged. Witness: uspatent/PatentApplication.

### M3 — xcolor `\XC@getcolor` faithful normalisation: xcolor.sty:1373-1390
(`\XC@getcolor#1#2{\begingroup\toks@{#1}\XC@getc@lor#1\XC@@\aftergroupdef#2{\@@tmp}}`
+ `\XC@getc@lor`). Port it (RawTeX from the real file is fine — the helpers
`\XC@@`, `\@ifxempty`, `\aftergroupdef`, `\XC@edef` exist in the binding; check
each) so `\pst@getcolor{red!50}` yields what xcolor yields. Guard: `\XC@getcolor{red!50}\x`
→ `\x` equals xcolor's result (run pdflatex with `\typeout{\meaning\x}` to get
the oracle string). Witness: dsptricks/dspTricksManual (the psmatrix template
desync remains D12 — do not chase it).

### M4 — singles from the sweep-49 residue (one root each, same deliverable shape):
kksymbols/kksymbols-doc (7 errors), notebeamer/notebeamer-demo (6),
biblatex-cheatsheet/biblatex-cheatsheet (3), istgame/istgame-doc (2),
gentombow/gentombow (1), scanpages/scanpages-doc (1), xebaposter/poster (1). Take
the s49 log's FIRST error, classify against same-host Perl, fix in the binding
layer when the root is a binding gap (bindings outrank raw files), otherwise
stop at the engine seam with a repro.

### M5 — verification of round 6 on sweep #50 (when `~/data/perfect_kernel_s50/`
exists): compare s49 vs s50 for every oracle-clean doc (`~/data/perfect_kernel/
oracle_verdicts.tsv`, engine pdflatex|lualatex, exit 0, errors 0); list every doc
whose error count ROSE (a regression of L1–L9 — report first, with the two first
errors side by side) and every doc that reached 0.

## Status (Gemini → orchestrator; append-only, newest last; round 7 only)
