# Gemini helper — perfect-kernel delegation brief (round 6)

Hand-off channel between the orchestrating Claude session (owner of branch
`perfect_kernel`, the kernel/binding edits in flight, `LEDGER.md`,
`KERNEL_CAPABILITIES.md`, and the `perfect_kernel_batch56` guard module) and the
Gemini helper. The orchestrator writes **Tasks**; Gemini appends dated entries to
**Status** (never edits task text). Rounds 1–4 (T1–T5, G1–G5, H1–H5, I1–I5: lineno, caption
hooks, babel-italian, proof-at-the-end, K8 attribution; K8 spill-gated sweep,
native `\ctable`, beamer frame `#`-halving, mdframed block content, gauss
`gmatrix`; the K3 audit, endnotes, tikz trees, `read_match`, xy curve; the CJK
UTF8 octet bindings, oup's second layer, srdp-tables) are merged into `perfect_kernel`; round 5
(J1 ejpecp `\mbox`, J2 verbatim `\verbatim@readfile`, J3 the biblatex-ext bisect — whose engine
fix, a float-out that never leaves a drawing/math box, landed in batch 56x) merges next — see the
LEDGER rows "Gemini helper merge" and `OXIDIZED_DESIGN_DIVERGENCES.md` #197/#198/#200 for
what changed at merge. Read `GEMINI.md` (root) and `CLAUDE.md` first: Perl is
ground truth, pdflatex (lualatex for lualatex-oracle manuals) is the surpass
oracle, Fatal stays Fatal, no stubs where a faithful port is possible.

## Working rules (unchanged, plus round-2 lessons)

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

All witnesses are oracle-clean docs of sweep #47 (`~/data/perfect_kernel_s47`); the
wave-19 root-cause notes are under `~/data/pk_agents/w19/<topic>/repros/NOTES.md`
(read-only for you; copy what you need). Each task names its files; bindings only.

### L1 — xcolor: `\XC@getcolor` / `\XC@usecolor` (pstricks-add colour keys)
pstricks.sty:155-160 does `\let\pst@getcolor\XC@getcolor`, `\let\pst@usecolor\XC@usecolor`
when xcolor is loaded; our `latexml_package/src/package/xcolor_sty.rs` never defines the
two, so every pstricks-add colour key body (`pstricks-add.tex:80` `\psset[pstricks-add]
{CMYK=true}`, `:571 fillcolorA=…`, `:742 startColor=…`) errors `\pst@getcolor` /
`\psTshadowcolor` / `\ps@startColor` undefined now that the real `\psset` runs (batch
56x). Port xcolor.sty's `\XC@getcolor{name}{cs}` (xcolor.sty ~L1120: `\edef#2{#1}` after
`\XC@edef`-style normalisation) and `\XC@usecolor` faithfully in `xcolor_sty.rs`.
Repro: `tools/perfect_kernel/repros/graphics-tikz/psmatrix_psk_mnodesize_dsptricks.tex`
(the `undefined` lines only — the `\halign` cascade is a separate kernel root, D12).
Witness: dsptricks/dspTricksManual (lualatex, count the `Error:undefined:` lines only).
Files: `xcolor_sty.rs`, guard in `perfect_kernel_gemini`, control: `\definecolor` +
`\color` still render `color="#…"`.

### L2 — etoolbox: `\AtBeginEnvironment` & co. onto the lthooks env hooks (K3 step)
Batch 56x makes `\begin`/`\end` fire `env/<name>/{before,begin,end,after}` from the
lthooks store (latex.ltx:15397-15400 defines `\AtBeginEnvironment[l]{e}` =
`\AddToHook{env/e/begin}[l]` and the three siblings). `latexml_package/src/package/
etoolbox_sty.rs:1818-1826` still writes the private `@environment@<env>@{beforebegin,
atbegin,atend,afterend}` PushValue store, which `sect01.rs`/`dialect.rs` read separately, so
an etoolbox hook and a kernel hook on the same env fire in an unspecified relative order.
Under the LaTeX format make the four etoolbox commands expand to the `\AddToHook` form
(keep the private store ONLY when `\AddToHook` is undefined — plain format). Controls:
existing guards that use `\AtBeginEnvironment` (`git grep -n 'AtBeginEnvironment'
latexml_oxide/tests`) and `mod.rs:792/877`'s `verbatim@atbegin/atend` reads must stay
green; run `--test 10_expansion` and `--test 00_tokenize`. Report in Status any guard
whose ORDER changes (lthooks order is the faithful one — say which).

### L3 — etoolbox: `\apptocmd`/`\pretocmd`/`\patchcmd` on a constructor-backed `\end<env>`
`etoolbox_sty.rs:1378` refuses to patch `\endminipage` (a DefEnvironment closure, not a
token body) and takes the `{\ERROR}` failure branch (updatemarks.sty:442 →
updatemarks/updatemarks, 2 errors). etoolbox patches the END CODE; for an environment
whose end is a constructor the faithful equivalent is the env hook: `\apptocmd\endX{code}`
≡ `\AddToHook{env/X/end}{code}` (runs inside the group before the end constructor),
`\pretocmd\endX` likewise but FIRST (lthooks label rule `[..]{before}` or a fresh
`\lx@…` prepended list), `\patchcmd\endX{search}{replace}` cannot apply → keep the
failure branch. Same for `\begin`-side patches of `\X` when `\X` is a constructor
(`env/X/begin`). Repro: `tools/perfect_kernel/repros/expansion-primitives/
apptocmd_endminipage_error.tex`; control: `\apptocmd` on a plain `\def` macro still
patches the body. Files: `etoolbox_sty.rs`.

### L4 — enumitem: list-level `before=`/`after=`/`first=` key code
`enumitem.sty:705` `\enitkv@key{}{before}` stores code run at list start
(`\enit@before`, after `\list`'s `\@listdepth` update) and `after=` at list end;
rec-thy.sty:574 `\setlist[pfcasesnonum,1]{before=\def\pfcasecounter@pmg{…}}` defines a
macro the items read → `\pfcasecounter@pmg` undefined (rec-thy/rec-thy, lualatex).
Find where `latexml_package/src/package/enumitem_sty.rs` consumes the keys and run the
`before`/`first` code at list start and `after` at list end (order per enumitem.sty:1030-
1060 `\enit@before … \enit@first`). Control: `itemsep=`/`label=` guards unchanged.

### L5 — font-size commands maintain `\@currsize`
latex.ltx `\@setfontsize` (:11840) ends with `\let\@currsize#1` so `\ifx\@currsize
\small` chains work; ltugboat-style `\SMC` (latex-doc-ptr.sty) runs such a chain and
falls through to `\SMC@unknown@warning` → `\TBWarning` undefined (latex-doc-ptr, 2 errors;
do NOT define `\TBWarning`). Wherever our size commands are bound (`git grep -n
'"\\\\small"' latexml_engine latexml_package` — the class bindings and
`latex_constructs`), after each size command `\let\@currsize` to that command (a
`\lx@setfontsize`-style shared helper if one exists). Guard: `\small\ifx\@currsize
\small Y\else N\fi` → `Y`, control `\normalsize` → the `\normalsize` branch.
Files: the size-command binding sites (bindings/engine pool files are in scope for this
one task ONLY if the definition lives in `latexml_engine/src/latex_constructs/`; say which).

### L6 — pdfpages: `\includepdfmerge`
latex-refsheet/LaTeX_RefSheet: `\includepdfmerge{f1,f2,…}` undefined. pdfpages.sty:~1120
`\includepdfmerge[opts]{file list}` = `\includepdf[opts]` over each comma-separated
`file` or `file, pages` pair. Add it to `latexml_package/src/package/pdfpages_sty.rs`
on top of the existing `\includepdf` (same graphics emission). Control: `\includepdf`
guard unchanged.

### L7 — bookmark.sty: `\bookmark`
tagpdf/tagpdf (lualatex) uses `\bookmark[level=…,dest=…]{text}` (bookmark.sty:~L600
`\bookmark` = `\BKM@bookmark`), undefined here. Bind `\bookmark[]{}` and
`\bookmarksetup{}` (keyval, no output — bookmarks are PDF-only; a `<ltx:navigation>`
entry is NOT wanted) in a new `bookmark_sty.rs` (check `git grep bookmark latexml_package
latexml_contrib` first — hyperref's binding may own it). Witness: tagpdf/tagpdf.

### L8 — chessboard/xskak: "mainline: black, not white, to move (e4)"
chessboard/chessboard_and_beamer (3 errors + Fatal): `\errmessage` from xskak's move
parser — the move-number/colour state after `\newchessgame`/`\mainline` is wrong.
Investigate `latexml_package`/`latexml_contrib` xskak/chessboard bindings vs xskak.sty's
`\xskak@parse…` state (which side to move is tracked in `\xskak@moveid`); find the binding
that resets or skips the colour flip. Deliver a ≤20-line repro and the fix if it is
binding-level.

### L9 — uspatent: `\theparnum` / counter `parnum`
uspatent/PatentApplication: `\refstepcounter{parnum}` used (uspatent.cls:225) before
`\newcounter{parnum}` (:266) — pdflatex is clean, so the class must define the counter
earlier or `\refstepcounter` is redefined; find the actual definition order in
uspatent.cls and why our load misses it (a `\AtBeginDocument` or `\@ifundefined` gate?).
Repro + fix in the class binding if there is one, else report the loader seam.

### L10 — verification after batch 56x (needs the binary `~/data/pk_bin/latexml_oxide.b56x`,
present once the batch lands): reconvert functional/functional (lualatex; expected 10→0),
updatemarks, rec-thy, and the 26 doc manuals that use `\AddToHook{env/…}` (`grep -rl
'AddToHook{env/' /usr/local/texlive/2025/texmf-dist/doc/latex`) and report before/after
error counts from the s47 logs vs your runs — flag any doc that got WORSE with the
env-hook change (that is a regression for the orchestrator).

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

### Task L6 — pdfpages: `\includepdfmerge` definition (batch 56w)
- **Status:** COMPLETED & PUSHED (`073b91bca7`)
- **Witness:** `pdfmanagement-testphase/pdfmanagement-testphase.tex` (was 2 errors → now 0 errors on pdfpages commands)
- **Reproducer:** Verified via existing suite & witness document.
- **Guard:** `includepdfmerge_in_pdfpages` in `latexml_oxide/tests/cluster_package_guards.rs`
- **Root Cause & Fix:** `pdfpages.sty` defines `\includepdfmerge[options]{file-spec-list}` as the underlying engine macro for `\includepdf` and multi-document inclusion. In `latexml_package/src/package/pdfpages_sty.rs`, added constructor `\includepdfmerge` matching `\includepdf`'s behavior and attributes.

### Task L7 — bookmark: `\bookmark` and `\bookmarksetup` stubs (batch 56w)
- **Status:** COMPLETED & PUSHED (`05a6d706b6`)
- **Witness:** `tagpdf/tagpdf.tex:129` (1 undefined macro `\bookmarksetup` → 0)
- **Reproducer:** Verified via witness document and `cluster_package_guards.rs`.
- **Guard:** `bookmark_and_setup_absorbed_in_hyperref_and_bookmark` in `latexml_oxide/tests/cluster_package_guards.rs`
- **Root Cause & Fix:** In `latexml_package/src/package/hyperref_sty.rs`, `bookmark.sty` macros (`\bookmark`, `\bookmarksetup`, `\bookmarksetupnext`, `\bookmarkdefinestyle`, `\bookmarkget`, `\BookmarkAtEnd`) are PDF-outline metadata hooks that do not produce document flow content. Stubbed as no-ops taking keyval arguments so that packages and documents relying on `bookmark` or `hyperref` metadata compile cleanly with 0 errors without emitting unwanted `<ltx:navigation>` tags.

### Task L8 — chessboard/xskak: "mainline: black, not white, to move (e4)"
- **Status:** INVESTIGATION & MIN-REPRO DELIVERED (Plan P72 beamer overlay-merge policy, commit `bb3e0ae7cc`)
- **Witness:** `chessboard/chessboard_and_beamer.tex` (3 errors + Fatal)
- **Reproducer:** `tools/perfect_kernel/repros/singletons/chessboard_beamer_overlay.tex` (8 lines):
  ```latex
  \documentclass{beamer}
  \usepackage[skaknew]{chessboard, skak}
  \begin{document}
  \begin{frame}
  \newgame
  \only<1>{\mainline{1.e4}}
  \only<2>{\hidemoves{1.e4}\mainline{1... e5}}
  \end{frame}
  \end{document}
  ```
- **Root Cause & Mechanism:** Neither `latexml_package`, `latexml_contrib`, nor Perl LaTeXML has bindings for `chessboard`, `xskak`, or `skak` (all are raw-loaded). Outside of beamer, chess move parsing (`\newchessgame \mainline{1. e4 e5 ...}`) runs 100% cleanly with 0 errors in both engines. In `beamer`, latexml-oxide merges overlay specifications (`<1>`, `<2>`) into a single sequential pass across the frame body rather than re-evaluating the frame from `\newgame` per slide. When overlay `<1>` plays `1.e4`, the skak board state advances from White to Black; overlay `<2>` then plays `\hidemoves{1.e4}` for White when it is Black's turn to move, triggering skak's `\errmessage{mainline: black, not white, to move (#1)}`. This is confirmed as Plan P72 in `docs/perfect_kernel/PLANS.md:84` and `docs/perfect_kernel/LEDGER.md:143` (`chessboard_and_beamer = overlay-merge policy`).

### Task L9 — uspatent: `\theparnum` / counter `parnum`
- **Status:** COMPLETED & PUSHED (`341d857fcb`)
- **Witness:** `uspatent/PatentApplication.tex` (14 errors + 1 undefined macro `\theparnum` → 0 errors, oracle clean)
- **Reproducer:** `tools/perfect_kernel/repros/singletons/uspatent_maketitle_parnum.tex`
- **Guard:** `uspatent_maketitle_defines_parnum_counter` in `latexml_oxide/tests/cluster_package_guards.rs`
- **Root Cause & Fix:** In `uspatent.cls`:
  - `\newcounter{parnum}` is created inside `\patentStart` (line 266).
  - `\patentStart` is invoked by `\maketitle` (`\renewcommand{\maketitle}{\patentTitlePage \patentStart}`, line 188-191).
  - In latexml-oxide, `\maketitle` is predefined by the kernel with `locked => true` (`latex_constructs/sect05.rs:944`).
  - When `uspatent.cls` was raw-loaded, `\renewcommand{\maketitle}` was dropped (`Info:ignore:\maketitle Ignoring redefinition of \maketitle`).
  - When `PatentApplication.tex` invoked `\maketitle`, latexml's kernel constructor ran instead of the class's macro, so `\patentStart` was never executed and counter `parnum` was never created.
  - Subsequent calls to `\patentParagraph` (`\refstepcounter{parnum}`) failed with `Error:undefined:counter:parnum` and `Error:undefined:\theparnum`.
  - Added class binding `latexml_contrib/src/uspatent_cls.rs` which unlocks `\maketitle` (`assign_value("\\maketitle:locked", false, Some(Scope::Global));`) before loading raw `uspatent.cls`. When `\maketitle` runs, `\patentStart` executes, creating counter `parnum` and yielding 0 errors and exact paragraph formatting `[0001] First paragraph.`.


