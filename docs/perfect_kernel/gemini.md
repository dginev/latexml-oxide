# Gemini helper — perfect-kernel delegation brief (round 8)

You are a helper on branch `perfect_kernel` of `~/git/latexml-oxide` (the perfect-
kernel program: `docs/PERFECT_KERNEL.md`). Work on a branch `gemini/pk-helpers-8`
cut from the current `perfect_kernel` tip (which contains batch 56ab AND your
round-7 merge, commit `66a7bf6144`, plus the orchestrator's fixup commit right after
it), push it, and append to the **Status** section at the end of this file (never
edit task text). Rounds 1–7 are merged. Round-7 review outcome, for your calibration:
the algpseudocodex entry is DIVERGENCES **#214** (numbers continue from the file's
last entry on `perfect_kernel`, never from `main`); the `DefKeyVal!` registrations
shadowed your raw `\KV@…` handlers (every explicit option was a no-op — handlers now
come after the registrations); your M2 (replay any dropped `\maketitle` body) was REVERTED —
the full suite showed resphilosophica.cls:331's body running against the amsart
binding, which lacks `\@setcopyright`/`\andify`/`\@maketitle@hook` (a generic
replay is the unseen-class backfire the orchestrator warns about; the uspatent
binding's explicit unlock is back); a hard-coded page count of 10 became l3's own fallback of 1; the xcolor guard
asserted a driver literal no code produces (M3 had already landed in batch 56aa —
always `git grep` `perfect_kernel` before re-implementing); three guards had no
structural assertion; a `\ExplSyntaxOn … \ExplSyntaxOff` block at latexml.sty top level
(`latexml_sty/mod.rs`) raw-loaded expl3 while the format was still loading and
pinned the dvips backend before the document could choose pdftex — top-level code
there names expl3 macros through `\csname`, never `\ExplSyntaxOn` (rewritten at merge). The istgame seam you reported (`\LoadClassWithOptions` dropping
the caller's options) is fixed in the kernel — an engine-seam report with a repro is
exactly the deliverable we want. The Perl source under `LaTeXML/` is ground truth,
pdflatex (lualatex for lualatex-oracle manuals) is the surpass oracle.

## Working rules (unchanged, plus round-2 lessons)

- **No pull requests.** All work lands on `perfect_kernel` through your
  `gemini/pk-helpers-N` branches; the single PR to `main` is opened by the user
  when the branch is ready. Never open a PR (PR #798 was closed for this reason).
- **This file lists only OPEN tasks.** At each merge the orchestrator lifts your
  Status entries into `LEDGER.md`/`KERNEL_CAPABILITIES.md` and deletes them here
  together with the solved task text; a task that is still open is carried over
  under a new number. Append your Status for THIS round below; nothing older
  belongs here.

- **Branch:** `gemini/pk-helpers-8`, branched from the current `perfect_kernel`
  HEAD (the round-7 merge + its fixup commit); rebase before every push; one commit per task, footer
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
  (column 3 = engine), sweep logs `~/data/perfect_kernel_s51/<bundle>/<name>/<name>.log`
  (s53 = batch 56ac, s54 = 56ad are the current sweeps; s52 = 56ab + your round 7). Convert from a COPY of the doc dir with
  `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]`
  for lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`; clean =
  `Conversion complete:` + non-trivial XML.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

### N1 — the remaining oracle-clean singles (sweep 51, `~/data/perfect_kernel_s51/<bundle>/<name>/<name>.log`)

Same protocol as round 7's M4: classify vs same-host Perl, fix in a binding when
RUST-ONLY or a surpass is cheap and faithful, report SHARED-with-a-Perl-count
otherwise, and file an engine-seam report (file:line, ≤30-line repro under
`tools/perfect_kernel/repros/<topic>/`) whenever the root is under `latexml_core`/
`latexml_engine`.

- `updatemarks/updatemarks` (2): first `Error:undefined:\ERROR` — the doc's own
  test macro? Check `updatemarks.tex` for `\ERROR` and whether it is defined by a
  package the doc loads; your `test_updatemarks*` probes in the repo root suggest you
  started this — finish it there and delete the probes (they are untracked).
- `latex-doc-ptr/latex-doc-ptr` (1): `Error:missing_file:latex-doc-ptr.ent` —
  find where `latex-doc-ptr.ent` lives in the TL tree (`kpsewhich`/`find`), and
  whether the doc reads it via `\input`/`\InputIfFileExists`; if the file is only in
  `source/`, that is an oracle-corpus artifact: say so with the path.
- `tagpdf/tagpdf` (1): `Error:undefined:\lst@TestEOLChar` — listings internal
  used by tagpdf's own code; check what defines it in listings.sty (`lstmisc.sty`?)
  and why our listings binding lacks it.
- `newpax/doc-use-pax` (1): `Error:undefined:\fail` — likely a deliberate
  `\fail` in an example; oracle-clean says pdflatex passes, so find how.
- `manyind/mindsample` (3): `Error:undefined:\nwletre` — manyind.sty internals;
  check whether the doc raw-loads manyind.sty and what defines `\nwletre`.

### N2 — algpseudocodex completeness (binding, `latexml_contrib/src/algpseudocodex_sty.rs`) — REGRESSION FIRST

Sweep 52 (your round 7 merged) FLIPPED the package's own manual
`algpseudocodex/algpseudocodex` from 0 (raw load of the .sty) to 13 errors under the
binding: `\Output`/`\Structure`/`\Properties`/`\Methods` undefined (the manual defines
them with `\algnewcommand`/`\algrenewcommand`/`\algdef` — the algorithmicx
definition API must work through the binding exactly as through the raw package),
`\tikzset` undefined (algpseudocodex.sty `\RequirePackage{tikz}`; the binding must
load what the package loads), and 3× `#PCDATA isn't allowed in <ltx:listing>` (text
landing directly in the listing instead of a `listingline`). Reconvert the manual
(`grep algpseudocodex ~/data/perfect_kernel/corpus.tsv`, log
`~/data/perfect_kernel_s52/algpseudocodex/algpseudocodex/algpseudocodex.log`) to 0
FIRST, with a guard per construct; a binding that handles less than the raw package
is a regression, not a feature. Then read algpseudocodex.sty (`kpsewhich
algpseudocodex.sty`) once more and close the gaps the review found: `\Require`/`\Ensure` are not hooked for `\algpx@endCodeCommand`
(`spaceRequire` dropped), `\Comment`'s trailing `\ignorespaces` (:911) is not
reproduced, `indLines` is inert. Add a guard per closed gap with a control; keep the
`ltx_border_<color>`/`ltx_thick` class names.

### N3 — a PDF page count for the DVI persona (design + red repro; core/engine edits are the orchestrator's)

notebeamer's `pages=-` loops over `\l__ntbm_include_filepages_int`, which today is
l3's fallback 1 (`latexml_sty/mod.rs`, hook `file/l3backend-dvips.def/after`).
`\pdflastximagepages` is a stub (`latexml_engine/src/pdftex.rs`), and
`latexml_core/src/util/image.rs` parses PDF page boxes but no page count. Deliver the
design (one page-count reader feeding both) and a ≤20-line red repro under
`repros/graphics-tikz/` whose expected output names the real page count of
`example-image-a4.pdf` (or a small local PDF you generate with pdflatex).

### N4 — verify rounds 7 and the 56ac–56ae batches on sweeps 52 → 54

`~/data/perfect_kernel_s52..s54/sweep_verdicts.tsv` exist (s54 finishes tonight). Repeat
round 7's M5 over the oracle-clean set for s52 → s54: newly clean, regressions (first
error side by side), and confirm at 0: biblatex-cheatsheet, kksymbols-doc,
notebeamer-demo, istgame-doc, uspatent (round 7); msc, ribbonproofsmanual (56ac);
modernposter/demo (56ad). Also explain the non-oracle rises in s53: datatool-user 8→14,
glossaries-user 4→5, tcolorbox 10→21 (memory fuse — report only).

### N5 — pgf layers in the SVG driver (binding)

pgf-PeriodicTableManual (non-oracle) carries ~200 errors, 68 of them
`Package pgf Error: Sorry, the requested layer 'pgfPTbacklayer'/'pgfPTpaperlayer' is not
part of the layer list`: `\pgfdeclarelayer`/`\pgfsetlayers`/`{pgfonlayer}` under our SVG
driver (`latexml_package/src/package/pgfsys_latexml_def.rs`, `pgfcorelayers.code.tex`).
Read pgfcorelayers.code.tex (`\pgfsetlayers` builds `\pgf@layerlist`; `\pgfonlayer`
checks membership at :90-110) and find why the list the manual sets is not seen — a
binding-defined `\pgfsetlayers` shadowing the raw one, a `\pgfsys@…` layer hook missing
(`\pgfsys@beginlayer`?), or the list assignment landing in a group. Deliver a ≤20-line
repro (`\pgfdeclarelayer{bg}\pgfsetlayers{bg,main}` + `\begin{pgfonlayer}{bg}`), the
fix, a guard asserting the layered content renders (0 errors + an `<svg:g>` per layer in
order), and the manual's before/after count.

## Status (Gemini → orchestrator; append-only, newest last; round 8 only)

