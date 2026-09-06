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

- **No pull requests.** All work lands on `perfect_kernel` through your
  `gemini/pk-helpers-N` branches; the single PR to `main` is opened by the user
  when the branch is ready. Never open a PR (PR #798 was closed for this reason).
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

### M1 — the algpseudocodex binding: finish it on `gemini/pk-helpers-7` (PR #798 is
CLOSED; cherry-pick your commit cda1297be2 from `feat-algpseudocodex-binding` onto the
helper branch, then apply the review findings). The orchestrator's read-only review
found, BLOCKING: (1) the DIVERGENCES entry must be **#210** (perfect_kernel runs through
#209; `main`'s numbering does not apply) and its premise must be current — on
perfect_kernel the raw load already renders `\Comment` on one line (`float:right`, no
minipage) and is 0 Fatal; the binding's job is `\LComment`/`\BeginBox`/`\EndBox`/
`\BoxedString` (3 undefined-CS errors today) and the in-flow box wrapper; (2) the
existing guard `statex_continues_the_open_line_box` (cluster_package_guards.rs ~12053)
raw-loads algpseudocodex and asserts `\State/\Statex/\State` = exactly 2
`<listingline>` + one `<break>` (the package's varwidth `\Statex`-in-open-box
semantics); your binding stubs that machinery, so the guard will flip — preserve the
`\Statex` → in-line break semantics in the binding (do not just edit the guard) and run
it; (3) multi-line boxes double-open: `\algpx@check@box`'s "already open" arm calls
`document.open_element("ltx:text", …)` on every `\item` while a box is open but
`\EndBox` closes one — open only on the pending→open transition (witness 2511.21969
Alg 2 lines 9 and 17 are multi-line boxes). MINOR: `noEnd` default is `[true]`
(algpseudocodex.sty:43 — End-lines suppressed by default), `commentColor` default is
`gray` (:48); say in the entry which options are honoured and which are dropped
(`indLines`, `spaceRequire`); the guard must assert `error_count == 0` and that the
comment sits in the `\State`'s own `<listingline>`, plus an `\LComment` structural
fact; when old `algorithmic` was loaded first, `algorithmicx_sty.rs:15-38` bails and
`\algrenewcomment` is undefined — guard against that path (witness class 2410.03000).
Deliverables: the corrected binding + guard on the helper branch, the #209 entry,
the witness's Algorithm 2 at 0 errors, Status entry. The orchestrator deletes the
handoff file after the merge.

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

### M3 — DONE by the orchestrator in batch 56aa (xcolor.sty:1373-1396 contract ported:
`\XC@getcolor` via `\extractcolorspec`, `\XC@undeclaredcolor` = `\color[model]{spec}`;
guard `xcolor_internal_api_matches_the_real_contract`; it also fixed the two gckanbun
flips lua-ul caused under your L1). Nothing to do; if M4/M5 show an xcolor-internal
error, report it under M5.

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
