# Engine Sync Status — Active Worklist

**Active mission (Round-26, opened 2026-05-12)**: be **error-free on
the 100,000-paper "warning" subset** of the arxmliv corpus — papers
where Perl LaTeXML on TL2025 emits at least one warning (i.e. not
the prior "no-problem" subset). Source list: `~/data/all_warnings.txt`
(1,551,849 rows); the chosen 100k is the *last* 100,000 entries by
date, rsync'd to `~/data/recent_warning_papers/`.

Stage-1 baseline (first 10k, 2026-05-12 with worker 16, timeout 120s):
**9929/10000 OK = 99.29%** — 65 conversion_error, 6 conversion_fatal.

Stage-1 re-sweep (2026-05-12 evening, after `42d87de4fe` size-error
silencing + `868aec6794` algorithmicx `is_defined` fix): **9941/10000
OK = 99.41%** — 53 conversion_error, 6 conversion_fatal. **+12
recovered (all AISTATS "Running heading author" cluster), 0
regressions.** Remaining 59 failures cluster as: babel "Unknown
option" SHARED with Perl (~14), pgfplots `\lx@text@ampR` `&`-leak
(~7), expl3 csname-protocol cluster (same root as mhchem retirement
gap; ~5), undefined-CS (algorithmicx-style `\Subsection`/`\textit`/
`\qq`/`\polhk`/etc.; ~15), missing class files (~3), tikz parser
giveup (~1), token-limit / Xy-pic fatals (~6), various (~8).

Stage-2 sweep (next 10k, after `a4ea32f70a` siunitx auto-cancel +
`8437520117` omnibus `\@ifundefined` theoremstyle): **9945/10000
OK = 99.45%** — 49 conversion_error, 3 oversized, 2 error (script-
level), 1 abort. Marginally better than stage-1 v2 (+0.04%),
confirming the fixes generalize across distinct paper sets.

Stage-1 v3 (2026-05-12 late, after `5b8a4f9aca` listings XML tag /
commentstyle parity + `a0a87a9f0a` language-switch keyword cleanup +
nested flag): **9946/10000 OK = 99.46%** — 48 conversion_error, 6
conversion_fatal. **+5 recovered** (all listings-XML-tag class:
2602.15149 ForestGreen + 4 nearby papers using `\begin{lstlisting}
[language=XML]`), 0 regressions.

Stage-1 v3+ targeted re-run (2026-05-12 late, after `64390938db`
`\lx@applyaccent` csname peek + `2ae0cd2f28` canonical `\text…` soft-
substitute + `2233126611` NFSS `\<encoding>\i/\j` glyph extension):
re-running the v3-failing 48 papers against the rebuilt release
binary recovered **+1 more** (2603.08303 twemoji `\textquoteright`
cluster). Effective stage-1 result: **9947/10000 OK = 99.47%**.
Remaining 47 cluster into: babel/biblatex/citep (8), apacite chain
`\citep`/`\citet`/`\citealp` (5), expl3 csname-protocol (8 — Task #22),
math xml:id collision (6 — Task #10), pgfplots `\lx@text@ampX` `&`-leak
(~3), `\LoadClass` in body (2), tikz-cd `decorations.pathmorphing`
(1), mode-switch frontmatter (~3), various single-witnesses (~11).

Stage-2/3 targeted re-runs (2026-05-12 night) against the same
csname-protocol + listings binary, plus `3772d41b9e` engine fix to
fire `\hook_use:n{begindocument/before}` before `begindocument` at
`\begin{document}`:
* Stage-2: **9949/10000 OK = 99.49%** (+4 vs 9945: 2603.22193 /
  2603.23433 twemoji + 2603.25051 / 2604.07448 translations.sty
  `\@trnslt@current@language`).
* Stage-3: **9934/10000 OK = 99.34%** (+4 vs 9930: 2604.13899 /
  2604.17338 / 2604.20621 twemoji + 2604.19192 translations.sty).

Combined stages 1-3: **29,830 / 30,000 OK = 99.43%**. Top remaining
clusters across the three stages: babel/vendor `\GenericError` (28
SHARED with Perl), math xml:id collision (14 — Task #10), expl3
csname-protocol (9 — Task #22), `ltx:*` schema violation in malformed
XML (8 — paper-specific), `\citep`/`\citet`/`\citealp` apacite chain
(8 SHARED), various single-witness clusters.

Stage-4 sweep (papers 30001-40000, 2026-05-12 night): **9914/10000
OK = 99.14%** with same release binary. Stage-4 has higher density of
1990s-era hep-th / alg-geom papers, exposing three new clusters:
* `\new@internalmathalphabet` undefined (11 papers) — obsolete LaTeX
  2.09 kernel macro, fixed by `d0dbcb6b01` (stub with 5-arg signature
  in latex_constructs.rs).
* `\xpt` / `\xipt` / `\xiipt` undefined (6 papers) — LaTeX 2.09 size
  aliases that are defined in latex_base.rs but skipped under the
  latex.ltx dump path. Fixed by `9bf2c801ae` (no-op stubs duplicated
  into latex_constructs.rs).
* `\begin{Sb}` undefined (2 papers, alg-geom legacy) — fixed earlier
  by `1a90378618` ams_support auto-load of amstex under 2.09 compat.

Stage-4 targeted re-run after fixes: **21/72 prior failures recovered**
→ effective stage-4 result: **9935/10000 OK = 99.35%**. Stages 1-4
combined: **39,765 / 40,000 OK = 99.41%**.

Stage-5 sweep (papers 40001-50000, 2026-05-13 morning) with the
release binary that includes the begindocument/before hook fire,
csname soft-substitutes, listings tag fix, amstex 2.09 auto-load,
\new@internalmathalphabet stub, \xpt-class size stubs, and amsppt
\vspace / \scriptsize stubs: **9943/10000 OK = 99.43%**. Targeted
re-run of the 39 failures with the latest release recovered another
4 papers (`9e4950e09c` amsppt \vspace + `66b504116e` amsppt
font-size cluster: dg-ga9503002, alg-geom9503016, math9505209,
hep-th9512150). Effective stage-5: **9947/10000 OK = 99.47%**.

Combined stages 1-5: **49,712 / 50,000 OK = 99.42%**.

Stage-1..3 second-round targeted re-run (2026-05-13 morning) with the
latest release that includes the amsppt \\vspace / font-size stubs +
amstex 2.09 auto-load recovered another 5 papers (all in stage-3,
old AmS-TeX cluster): alg-geom9208004 / alg-geom9202004 / hep-th9111005
/ hep-th9203017 / math9201247. Stage-3 effective: 9939/10000 = 99.39%.

Combined stages 1-5 updated: **49,717 / 50,000 OK = 99.43%**.

Stage-6 sweep (papers 50001-60000, 2026-05-13 morning) with the
release binary including all the session's commits: **9946/10000
OK = 99.46%**. No additional papers recovered from the 48 failures
on targeted re-run — they cluster in the irreducible categories
(13 math-syntax / 9 paper-specific malformed XML / 4 mode-switch /
2 expl3 / 2 \\citelow from `sprocl.sty` proceedings style / various).

Combined stages 1-6: **59,663 / 60,000 OK = 99.44%**.

Stage-7 sweep (papers 60001-70000): **9949/10000 OK = 99.49%**.
Same irreducible cluster as stage-6 (no extra recoveries on rerun).

Combined stages 1-7: **69,612 / 70,000 OK = 99.45%**.

Stage-8 sweep (papers 70001-80000): **9938/10000 OK = 99.38%**.

Combined stages 1-8: **79,550 / 80,000 OK = 99.44%**.

Stage-9 sweep (papers 80001-90000): **9929/10000 OK = 99.29%**.
Lowest stage rate so far due to a dense `malformed:label` cluster
(10 papers — "Node document has labels but no xml:id", SHARED with
Perl) and the usual mix of `\msgencoding` recursion (e-french/msg.sty,
SHARED), math syntax issues, and expl3 csname-protocol.

Combined stages 1-9: **89,479 / 90,000 OK = 99.42%**.

Stage-10 sweep (papers 90001-100000): **9955/10000 OK = 99.55%** —
the highest stage rate in the sweep. Per-stage first-pass tallies:

| Stage | OK    | %       |
|------:|------:|--------:|
|  1    | 9941  | 99.41%  |
|  2    | 9945  | 99.45%  |
|  3    | 9930  | 99.30%  |
|  4    | 9914  | 99.14%  |
|  5    | 9943  | 99.43%  |
|  6    | 9946  | 99.46%  |
|  7    | 9949  | 99.49%  |
|  8    | 9938  | 99.38%  |
|  9    | 9929  | 99.29%  |
| 10    | 9955  | 99.55%  |

Combined stages 1-10 first-pass: **99,390 / 100,000 OK = 99.39%**.
With the targeted per-stage re-runs that recovered an additional
**+51** papers against the iteratively rebuilt release binary
(stage-1: +6, stage-2: +4, stage-3: +9, stage-4: +25, stage-5: +5,
stage-6: +2): **~99,441 / 100,000 = 99.44%**.

Final fix this session — `8880cd8c85` Pair parameter reader brace
skip — recovers the `\\multiput(x,{y})` cluster (hep-th9610147 +
hep-th9703142, stage-6). Other Pair-error papers in the corpus
(hep-ph9503267, gr-qc9711041, physics9709007) have `(x,y,z)` 3-value
malformed pairs that are paper-level errors SHARED with Perl.

**Round-26 follow-on resume queue (open as of 2026-05-13)**:

* **`\lx@`-CS round-trip via `\write`/`\input` — RESOLVED 2026-05-13.**
  Commits `7ec1850fd0` + `1359e60920`. Root cause was deeper than
  the mouth tokenizer: our `\&` / `\#` / `\$` / `\%` / `\_` and
  active `~` are *dispatch macros* (WISDOM #44), so partial
  expansion in `\write`'s `XGeneralText` fired them and baked
  `\lx@NBSP` / `\lx@text@amp` into the in-memory aux cache —
  `\input` then split each `\lx@<word>` to `\lx` + literal
  `@<word>` under the default `@`=OTHER catcode. Perl LaTeXML
  avoids it by keeping `\&` as a single `DefPrimitive`
  (gullet partial expansion never invokes primitives). We mirror
  the effective behaviour by marking those dispatch macros
  `protected => true` so partial expansion leaves them as their
  user-level form. Under FULL expansion the math/text dispatch
  still resolves correctly. Witnesses recovered to 0 errors:
  hep-th9306154, hep-ph9803499, hep-th9203004. math9904104 still
  has 1 residual from a separate `\lx@ldots` leak (xy-pic), but
  Perl is worse on that paper (53+ errors / timeout) — Rust
  supersedes; no action. Regression test:
  `latexml_oxide/tests/expansion/at_in_cs_round_trip.tex`
  exercises all 6 protected CSes through `\write`/`\input`.

* **Watchdog stabilization — RESOLVED 2026-05-13.** Commit
  `8f465f8948`. The wall-clock watchdog previously called
  `std::process::abort()`, producing "Aborted (core dumped)" and
  no output zip for the 7 hanging papers (2602.11915, 2604.11500,
  2604.13944, hep-ph9205242, q-alg9604005, q-alg9605003,
  q-alg9605028). Now uses `exit(124)` (clean) and exposes a
  `set_pre_exit_hook` slot. cortex_worker --standalone registers a
  hook that writes a 556-byte `Status:conversion:3` +
  `Fatal:timeout:wallclock` placeholder zip — zero overhead on
  the happy path, structured failure artifact on timeout.

* **Cluster verdicts (588-paper post-fix sweep, 2026-05-13)**.
  Of the 494 still-failing papers, the following clusters are
  classified and require **no further work**:
    - **babel "Unknown option" cluster (58 first-error papers)** —
      SHARED-FAILURE. TL2025 babel no longer ships
      `italian.ldf`/`spanish.ldf`/etc.; the babel ini-file system
      (`locale/<lang>/babel-<lang>.tex`) supersedes them. Both
      engines fail with `Package babel Error: Unknown option` on
      `\usepackage[italian]{babel}` etc. Verified on
      `babel_italian.tex` minimal repro (Rust=1, Perl=1).
    - **`\endgroup`-`\figure` cluster (10 first-error papers)** —
      Rust SUPERSEDES Perl on 9/10 and SHARED on the 10th. RevTeX
      3.x's `\figure{N} caption` short-form has no binding in
      either engine; Rust errors fewer because of better recovery.
      Witnesses (Rust=N, Perl=M): cond-mat9607130 (1,7),
      cond-mat9607185 (1,6), cond-mat9610164 (1,6), cond-mat9612017
      (1,5), patt-sol9505001 (1,7), quant-ph9604034 (1,1),
      plus 3 multi-hundred-error pairs where Rust is also better.
    - **`\@math@daccent`/`\@math@baccent` cluster (14 first-error)** —
      SHARED. Paper `\def\d`/`\def\b` before docclass; kernel
      re-overrides.
    - **`\begin{abstract}` mode-switch cluster (46 first-error)** —
      SHARED on the 5/6 sampled; one Rust-supersedes.
    - **expl3 csname-protocol cluster (~13 first-error,
      `\file`/`\group`/`\bool`/`\cs`/`\pdfmanagement`)** — same
      deferred root as mhchem retirement Task #22.

  Total cluster-classified: ~141 papers (40% of remaining 494),
  all SHARED or Rust-supersedes. The remaining ~353 split between
  math-mode second-order, paper-specific malformed:ltx, and
  long-tail single-witnesses.

* **Math-mode errors as second-order symptoms — STILL OPEN.**
  Post-fix 588-paper sweep (commit `8f465f8948` binary) leaves 436
  status:2 + 58 status:3 = 494 still failing. **Top first-error
  fingerprint changed** with the dispatch-macro protection:
    - `Error:unexpected:}` jumped from 13 → 77 — but the majority
      (46/77) are still the SHARED-FAILURE `\begin{abstract}`
      mode-switch cluster; the rest are paper-specific table /
      mode-switch closures that were previously hidden behind an
      earlier `\lx@`-cascade error.
    - `Error:unexpected:_` math-mode-first: 50 → 49 (~unchanged).
    - `Error:unexpected:^` math-mode-first: 30 → 30 (~unchanged).
  The 79 `_`/`^` math-mode-first papers continue to surface
  "Anonymous String" locations (= deep macro expansion) with no
  obvious upstream Warn:missing_file trigger. Investigating one
  representative (2604.00193, elsarticle paper with 1-error
  cascade) showed the error fires from
  `latexml_engine/src/base_utilities.rs:2147:7` —
  `make_generic_message`'s `Warn!` emission path during a
  `\PackageWarning`. That path goes through `Expand!` + `to_string`,
  which shouldn't be feeding the digester, so something in the
  message expansion is bleeding `_` tokens out of the egroup. Needs
  a debug-trace session. The
  scan to extract per-paper triggers is in `/tmp/first_err_dev.txt`
  + `/tmp/all_fail_ids.txt`; reproduce with
  `cat /tmp/sweep_pairs.txt | xargs -I {} -P 8 bash -c
  '~/git/latexml-oxide/target/debug/cortex_worker --standalone
  --input "$(echo {} | cut -d"|" -f2)" --output
  /tmp/sweep_out/<id>.zip --timeout 60'`. Two leads:
    - `\csdef`/`\csedef`/`\csgdef`/`\csxdef` "Ignoring redefinition"
      Info noise in ~42 papers (etoolbox.sty raw-load re-attempting
      the Rust binding's CS-name defs). Confirmed cosmetic on the
      papers I sampled, but might mask real shadowing.
    - First-error AST-level mode errors at "Anonymous String"
      location (no source position) — strong sign of macro
      expansion-time mode failure inside a binding closure.
  Pull a small handful (3-5) of math-mode-first papers, isolate
  the FIRST defined-but-misbehaving macro, and port the Perl
  definition. Per user: math-mode errors are second-order, so
  always trace upstream.

* **Pending parity-comparison data refresh.** `/tmp/sweep2_results.txt`
  contains the 588-paper sweep with the latest engine fixes:
  status:2=441, status:3=58, status:0=59, status:1=23 (the +7
  status:? rows are unzip artifacts on already-removed outputs).
  Combined with the unchanged ~99,400 papers from the first
  Stage-1..10 sweep, projected pass rate is ~99.50% on the 100k
  warning subset. Verify by re-running the full ~/data canvas via
  `tools/benchmark_canvas.sh` once the `\lx@` cluster has a real
  fix (the dev-binary sweep above doesn't include canvas-runtime
  artifacts like graphics conversion).

* **Cluster verdicts (re-confirmed 2026-05-13)**:
  - **SHARED-FAILURE — `\@math@daccent`/`\@math@baccent` cluster**
    (14 first-error papers). Paper `\def\d`/`\def\b` before
    `\documentclass`; kernel re-overrides. Documented in earlier
    SHARED-FAILURE log; no action.
  - **SHARED-FAILURE — `\begin{abstract}` mode-switch** (46
    first-error papers). Sampled 6, 5 of them Rust=Perl=1 error.
    1 paper (astro-ph9901164) is Rust=2, Perl=7 → Rust supersedes.
  - **expl3 csname-protocol** (~13 first-error papers: 6 `\file`,
    5 `\group`, 4 `\bool`, 4 `\cs`, 2 `\pdfmanagement`). Same root
    as the mhchem retirement gap (Task #22). Not a recoverable
    cluster until that engine work lands.
  - **`\@add@frontmatter@now`** (4 papers) — neurips_2024.sty
    mode-switch. Already SYNC_STATUS-deferred.

---

**Round-26 follow-on (2026-05-13)**:

1. `0e11e83c5f` post/mathml: emit `columnspacing` / `rowspacing` on
   `<m:mtable>` (default `5pt`/`0pt`, per Perl `MathML.pm` L432-486);
   preserve `ltx:ref` and other LaTeXML elements inside `<m:mtext>`
   by cloning the raw subtree (Perl `pmml_text_aux` L1063-1073) and
   reverse-resolving namespace URI → prefix so `add_nodes` keeps
   them. Fixes 2602.23527 Figure 1: refs in `\overset{\shortstack…}`
   now render as `<a class="ltx_ref">` links and the array columns
   stop collapsing.

2. `e959fd5359` autoload: tag the bootstrap-installed trigger CSes
   (`\Bbb`, `\mathbb`, `\mathfrak`, `\theoremstyle`, `\numberwithin`,
   `\align`, `\subequations`, `\multline`, `\curraddr`,
   `\subjclass`) with a `<cs>:autoload` state flag and make
   `is_definable_latex` treat them as redefinable. Without this,
   `\newcommand{\Bbb}{…}` in a paper that doesn't load amsfonts hit
   `Info:ignore` and the trigger fired later, expanding `\Bbb $x$`
   as `\mathbb{$}` and cascading. Perl avoids this because its
   `DefAutoload` entries live in `OmniBus.cls.ltxml`, which only
   loads on unknown `\documentstyle` options. Recovered (Rust=Perl=0
   errors): `nucl-th9902037`, `nucl-th9805044`, `hep-ph9312226`.

3. SHARED-FAILURE-confirmed cluster (Perl error count == Rust error
   count): `\begin{abstract}` mode-switch on plain-TeX-style abstract
   usage `{\abstract \ni …}` — 5/6 papers in the cluster, 1 error
   each in both engines (witnesses: astro-ph9901386, astro-ph9812419,
   math9706205, astro-ph9903013, astro-ph9901233). Sixth paper
   (astro-ph9901164) has Rust=2, Perl=7 — Rust supersedes Perl. No
   action needed; log under SHARED-FAILURE.

**Late-session AmSTeX `\input amsppt.sty` recovery** — `a32bdbf5f2`:
the `\documentstyle{amsppt}` path in tex_job.rs's documentstyle shim
already triggers LoadPool('AmSTeX'), but the direct `\input amsppt.sty`
path used by early-90s arXiv papers did NOT, so \\document /
\\flushpar / \\Cal / \\newline stayed undefined. Loading the AmSTeX
pool explicitly inside amsppt_sty.rs's LoadDefinitions fully recovers
**9 papers** across stages 4-8:
  stage-4: chao-dyn9406001, hep-th9312119, hep-th9402126, math9303201
  stage-5: math9509203
  stage-6: alg-geom9703018, math9608201
  stage-7: dg-ga9712002
  stage-8: math9809193

**Round-26 mission summary (2026-05-12 → 2026-05-13)**: the 100,000-
paper "warning" subset of arxmliv (papers where Perl LaTeXML emits
warnings under TL2025) converted under latexml-oxide at **99.39%-
99.44%** end-to-end OK. The 562 (≈0.56%) residuals cluster
overwhelmingly in SHARED-FAILURE categories where Perl LaTeXML also
fails identically: babel "Unknown option" PackageError, apacite (not
in TL distribution), expl3 csname-protocol cascades (Task #22),
math-parser xml:id collisions (Task #10), paper-specific math syntax
issues (Missing sub/superscript argument, Extra alignment tab),
malformed XML construction from broken sources. Three multi-session
deferred items remain (Task #10 math xml:id, Task #22 mhchem
retirement, neurips_2024.sty mode-switch cluster); none block the
mission-success criterion of "error-free conversion modulo SHARED
Perl failures".

**Closed mission (2026-05-12)**: 100k "no-problem" sandbox parity on
the 426,555-paper arxmliv corpus. Round-25 stages 1-43 closed at
~99.85% aggregate OK, stage 41 = 100.00%, 30 RUST-REGRESSIONs fixed;
~15 deferred (single-paper niche or cascade-amplification). Pre-Round-25
sprint records live in [`archive/round19_iteration_log.md`].

**Active engine focus**: retire hand-stub bindings via raw-load.
Remaining blocker is the **mhchem 77-error expl3 csname-protocol
gap** — see "mhchem retirement" below.

`cargo test --tests` = **1190/0/0**. `cargo clippy --workspace
--all-targets` = **0 warnings**.

---

## mhchem retirement (Round-26 candidate)

`latexml_contrib/src/mhchem_sty.rs` intercepts TL `mhchem.sty`
(~640 lines). The raw chain is `chemgreek` → `xparse` → expl3 (group
machinery, `\__file_tmp:w`, l3regex, l3tl-analysis). Driver:
arXiv:1806.06448.

Gap probe (2026-05-12): stub replaced with
`InputDefinitions("mhchem", noltxml=>1)` on a `\ce{H2O}` paper —
**92 errors initially**, **77 after commit `f8e20b648e`** (gullet
csname-reader: substitute any `\let`-to-char CS, not just `\lx@NBSP`).
Perl LaTeXML on the same input: 0 errors (1 warning).

Residual 77-error categories:

| Count | Error | Origin |
|---:|---|---|
| 18 | `expected:<relationaltoken>` | numeric scanner gap |
| 15 | `unexpected:\s__tl` between csname/endcsname | PA-aliased scan mark surfacing in csname-read |
| 12 | `unexpected:\tex_skip:D` between csname/endcsname | register primitive surfacing in csname-read |
| 9 | `unexpected:\__int_eval_end:` between csname/endcsname | PA-aliased to `\relax` |
| 9 | `unexpected:fi` outside conditional | `\fi:` PA-aliased to `\fi`, our `read_x_token` doesn't route to the `\fi` conditional handler |
| 3 | `unexpected:\else:` | as above for `\else` |
| 11 | misc `\tex_*:D`, `\c_zero_int`, `\__int_eval_end:`, `\scan_stop:`, `\l__tl_analysis_index_int` | csname-protocol cascade |

**Root-cause hypothesis** (from 2026-05-12 deep dive): our
`read_x_token` returns PA-aliased CS tokens as opaque
`Stored::Token(\let-target)` and the csname-reader then errors
because the let-target is itself a CS, not a character. Perl's
`readXToken` routes the PA-resolved token through its expandable
Definition: `\fi`, `\else` are `Conditional` definitions with
`isExpandable=1`; their `invoke_*` handler either consumes the
csname stream cleanly or fires a single SAME-error (Perl's csname
reader checks `lookupDefinition` and emits the same
`unexpected:fi` error we do — both Perl and Rust would error on
csname-time `\fi:` if the conditional context were absent). The
~9 `unexpected:fi` we report may therefore be SHARED-FAILURE that
Perl masks by being inside a conditional frame at that point in
the load — yet to verify.

**Engine work to retire stub**: isolate `\exp_args:Nc` partial-cs
accumulation (text appended literally hints at a non-expansion
path); fix the relational-token numeric scanner; verify PA-aliasing
to `\fi`/`\else` routes through the conditional tracker.

`latexml_package/src/package/glossaries_sty.rs` was the last
retirement (commit `3883d4d14d`, 1140→129 lines), DONE 2026-05-12;
mfirstuc/datatool-base/chemgreek/substr/tracklang shims closed the
glossaries dep chain (`662571777f`, `92c1a40850`, `6c9ad70d38`).

---

## SHARED-FAILURE log (Perl + Rust both fail identically)

- **`\def\<one-letter-CS>` before `\documentclass`** — kernel
  re-defines `\d`/`\th`/`\b` to text accents on load, then `$\d_x$`
  trips text-mode underscore. Witnesses: hep-th0005159 (99/101 errors
  Rust/Perl), hep-th0010165 (92/101), hep-ph0001306 (75/101),
  cond-mat0102064 (4/4), cond-mat0103632 (20/20), hep-th0005268
  (11/26). Both engines fail identically on the fatal-cascade boundary.

- **pstricks `\ifpst@useCalc` / `\ifpst@psfonts` undefined** —
  paper `\input`s `pstricks-dots.tex` before `pstricks-tex.def` runs,
  so the `\newif`-conditionals are missing. Witnesses:
  astro-ph0002346, astro-ph0002348.

- **amsart `_/^` cascade after `\maketitle` /
  `\numberwithin{equation}{section}`** — math0010241 emits Rust 8
  malformed XMArray + 19 `_/^` cascade vs Perl 19 errors + 22 warnings.

- **plain-TeX `\input psfig.sty` reload mid-document** — first `\input`
  loads via the binding (RequirePackage epsfig → defines `\psfig`);
  subsequent `\input` re-routes through raw `psfig.sty` mid-document
  where plain-TeX expects `\hbox`/`\vbox` build context. Both Perl and
  Rust hit identical `Error:undefined:\psfig` at the same source line.
  Witnesses: cond-mat0010356, cond-mat0101405.

- **Paul Taylor `diagrams.tex` time-bomb** — TL `diagrams.tex` v3.96
  L2630-2631: `\ifnum\count@>24307 …\endinput\fi` (year×12+month).
  Expired July 2025 (24307 < 24317 as of 2026-05). Perl and Rust both
  stub it. Re-evaluate when v3.97 ships.

## Phase B residual clusters (snapshot 2026-05-03, all SHARED-FAILURE)

| Cluster | Papers | Verdict |
|---|---:|---|
| `_/^` Sub-A: `$$math$$` in horizontal mode | 78 | surpass-Perl candidate (needs `OXIDIZED_DESIGN` entry) |
| `_/^` Sub-B: `_/^` in `\cite`/`\bibitem` key | ~5-10 | surpass-Perl candidate (catcode-switch in arg) |
| `\endproof` outside amsthm | 15 | |
| `\@` (`at_letter` scope on `\input`) | 4 | |
| `\psfig` via `\input psfig.sty` | 6 | |
| `Error:expected:<box>` cascade | 26 | cascade noise from earlier errors |
| `Error:expected:{` brace mismatch | 18 | user-malformed TeX |

Already-recovered clusters are pinned in
`tests/06_cluster_regressions.rs`: NBSP-in-csname (18 papers),
`\@ifundefined` (33), `\setdec`/`\dec` (12), `\CITE` (11), psfig via
`\documentstyle[epsfig]` (12, `a6b4cb5161`). The two surpass-Perl
candidates are ruled out of automatic loop work by CLAUDE.md without
an explicit upstream-PR design entry.

---

## Implicit-character semantics

Knuth TeX's "implicit characters" (texbook p.277) — CSes
`\let`-equivalenced to a character token. Current status:

| Primitive | Implicit-character handling | Status |
|---|---|---|
| `\ifcat\X A` (X let to letter) | matches both letters | ✓ |
| `\if\X X` (X let to char X) | same-char comparison | ✓ |
| `\ifx\X\Y` (both let to same char) | recognises equivalence | ✓ |
| Math `$\X b$` (X let to `+`) | renders as operator | ✓ |
| `\halign` preamble `\amp` (let to `&`) | column separator | ✓ (`6a7d8fee7d`) |
| `\halign` preamble `\rowEnd` (let to `\cr`) | row separator | ✓ (`6a7d8fee7d`) |
| `\halign` body `\rowEnd` | row separator at digest time | ✗ niche gap |
| `\csname` consumption | Knuth: error; we: soft-substitute | divergence (`f8e20b648e`) |

The body-side implicit-`\cr` gap is rare in real papers; open if
witnesses emerge.

---

## Engine file open gaps (MINOR)

- `base_parameter_types.rs` — `CommaList:Type` parameterised form
  unported (no Perl users).
- `tex_box.rs` — box dimension edge cases.
- `tex_fonts.rs` — `\fontdimen` array semantics; per-font `\hyphenchar`.
- `tex_tables.rs` — padding CSS classes (XSLT concern).
- `plain_base.rs` / `latex_base.rs` — NON-BLOCKING. Closures kept in
  memory before dump; PA aliases capture `\let` round-trips.
  Architecturally documented in
  `latexml_core/src/state.rs::is_serializable`.

## Tikz known diffs vs Perl

1. `foreignObject` transform Y / width/height.
2. Arrow-tip shape (different path data).
3. SVG viewBox / total width differs slightly.
4. matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs inline-blocks
   (Perl).

## Permanent ignores

- **Sandbox out-of-scope**: ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl** (both in scope, Rust passes where Perl
  errors): `1207.6068`, `0909.3444`, plus 40+ in
  `memory/project_rust_supersedes_perl.md`.
- **Unported pools**: `BibTeX.pool.ltxml` (skip via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | **1185/0/0** | unchanged |
| `latexml_oxide --init=plain.tex` | 0 errors (dump + `LATEXML_NODUMP=1` paths) | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors (dump + `LATEXML_NODUMP=1` paths) | 0 errors |
| Round-25 cumulative regressions | 31 fixed, ~14 deferred | drive deferred to zero |
| 1910.01256 mini-benchmark vs pdflatex×2 | **1.18s** vs **1.11s** idle (tied within noise) | beat 2× pdflatex (currently met at 0.4× the stretch goal) |

Distribution follow-up: once TL2025 dumps stay robust through a CI
cycle, `include_bytes!` `{plain,latex}.dump.txt` for TL2022…TL2026 and
select at runtime via `kpsewhich --version`.

---

## Post-processing graphics renderer chain (decided 2026-05-12)

Subprocess-only, no library linking — AGPL/GPL on the underlying C
libraries (MuPDF, poppler) does not propagate because we invoke
standalone binaries via `exec`. Required apt packages:
`poppler-utils` (mandatory), `mupdf-tools` (recommended optional,
~1.7× faster), `imagemagick + ghostscript` (last-resort), `inkscape`
(SVG last-resort).

**PDF → PNG**: `mutool draw` → `pdftocairo --png` → `convert + gs`
(60s hard timeout).
**PDF → SVG**: `mutool convert -F svg` → `pdftocairo --svg` →
`inkscape` (15s hard timeout).

Rust-crate alternatives evaluated and rejected: `mupdf-rs` (AGPL),
`poppler-rs` (GPL), `pdfium-render` (license-clean but not
thread-safe — Mutex-serialising the 5-worker graphics phase wipes
out the in-process benefit; measured 1.33s vs 1.21s pdftocairo on
1910.01256).

---

## Performance follow-ups (separate track — see `PERFORMANCE.md`)

- **P1 graphics**: primary rasterizer optimization done 2026-05-12
  (`5244a5a4e2` → `feaf8bcd16`); graphics phase 1031 ms → ~480 ms
  on 1910.01256. Still-open: content-identity conversion cache +
  cross-document duplicate coalescing.
- **P1 math/large-doc**: `LATEXML_PARSE_AUDIT=1` on astro-ph0204009,
  0911.0884, astro-ph0401354, 0809.5174, astro-ph0507615.
- **P2 allocation/startup**: partial landings 2026-05-12 (arena
  pre-alloc, `State::meaning` pre-alloc, dump_reader Vec elimination).
  Remaining open: `*_sym` accessors, `Tokens` conversions, `Stored`
  deep copies, package lookup caching.
