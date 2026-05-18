# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML
> converts a paper without a downgrade, the Rust translation must
> match by improving the core engine — never by silencing
> diagnostics. Acceptable pre-existing exception:
> `is_typesetting_only_message` entries that match Perl's behavior
> on the SAME paper (e.g. "Running heading author exceeds size
> limitations" per WISDOM #50). Any NEW downgrade requires explicit
> proof Perl emits the same severity, otherwise it's hiding a real
> engine gap. User directive (2026-05-15): "downgrading errors is
> generally cheating at the task and must not be attempted."

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

**Round-27 cluster work plan (opened 2026-05-13, official)**:

The 220-paper classified-cluster cohort below is being worked
from kernel-and-core-quality outward to individual macro
bindings, per user directive. Each cluster gets a root-cause
analysis and a principled fix path. The first
surpass-Perl improvement on the cohort landed in `f54df88c22`
(`\lx@notetext` optional `[id]` → `OptionalSemiverbatim`)
which fixes the `\fntext[footnote_label2]` family.

### Cluster A — Catcode-leak through optional-arg digestion (math-mode-as-symptom)

**Status:** OPEN, in progress 2026-05-13. First fix landed
(`f54df88c22`). ~78 remaining first-error candidates.

**Root cause.** Constructors (and macros) that declare an
optional `[]` slot read with the *default* catcode regime —
`_`, `^`, `~`, `&`, `$`, `#`, `'` all keep their special TeX
catcodes. When a paper writes `_` literally in a slot that's
semantically an identifier (xml:id, label, URL, file path,
keyword), the SUB-catcode token bleeds into the digester via
`Parameter::digest → Tokens::be_digested → stomach::digest`,
runs through `invoke_token` on `T_SUB!`, hits the text-mode
branch of `script_handler`, and errors.

Perl LaTeXML has the same `[]`-default-catcodes behaviour and
fires the same error at the same source line on the same
papers, so this cluster is currently **SHARED-FAILURE**. The
surpass-Perl path is to change those parameter slots to
`OptionalSemiverbatim` (or `Semiverbatim` for the mandatory
`{}` variant) which sets `_`/`^`/`~`/`&`/`$`/`#`/`'` to OTHER
catcode at read time, making the identifier read as plain text.

**Principled approach.** Audit constructors whose optional /
mandatory slots are semantically identifiers (`xml:id`,
`label`, `href`, `key`, `bib-key`, `filename`, `\ref` target).
Change those slots to `OptionalSemiverbatim` /
`Semiverbatim`. Constructors whose slots are semantically
*content* (caption text, note body, figure body) stay as
default-catcoded — those slots SHOULD allow `_`/`^` inside
inline math `$x_1$` correctly.

**Already fixed:**
- `\lx@notetext OptionalSemiverbatim {} [] {}`
  (commit `f54df88c22`) — fixes `\fntext`, `\tnotetext`,
  `\footnotetext`. Witness: 2604.00193.

**Audit candidates (next sprint):**
- `\ref`, `\pageref`, `\eqref` — already partially handled, audit
- `\label` — already `Semiverbatim` (verify)
- `\cite`, `\citep`, `\citet`, `\citealp` — `key` arg
- `\href`, `\url`, `\hyperref` — URL slot
- `\bibitem[opt]{key}` — key arg
- `\caption`/`\subcaption` — `[short]` is identifier-shape
- `\thanks[opt]` — same pattern as `\fntext`
- `\index` — entry key

Each fix gets a witness recovery count noted here.

**Acceptance:** Re-sample the 79 math-mode-first papers after
each binding change; track recovery delta in this section.

### Cluster B — `\@math@daccent` / `\@math@baccent` paper-side `\def\d`

**Status:** SHARED-FAILURE confirmed. CANDIDATE FOR
"surpass-Perl" if a kernel-side fix can detect paper-local
`\def\<one-letter-CS>` before docclass and protect the user's
intent.

**Root cause.** Standard plain-TeX kernel re-defines `\d` /
`\th` / `\b` to text accents on load. Papers that
`\def\d{...}` before `\documentclass` get over-written.
Witnesses: hep-th0005159, hep-th0010165, hep-ph0001306,
cond-mat0102064, cond-mat0103632, hep-th0005268 (plus 14
math-cascade papers).

**Principled approach.** The kernel SHOULDN'T re-define
already-`\def`-ed one-letter CSes. Option (a): in latex.ltx
processing, check `IsDefinable` before `\let`-ing the text
accent. Option (b): record paper-local `\def\d` defs in a
"user-redefined" set and skip the kernel override for those.

**Acceptance:** the witness cluster errors go to 0; Perl
should be informed of the same surpass-opportunity.

### Cluster C — `\begin{abstract}` mode-switch on plain-TeX-style abstract

**Status:** SHARED-FAILURE confirmed (5/6 sampled). CANDIDATE
FOR surpass-Perl. ~46 first-error papers.

**Root cause.** Pre-2000 papers use `{\abstract \ni …}` as a
font-switch group (`\font\abstract=cmr8`), then `}` closes the
group but the abstract environment is still open and in
internal_vertical mode. `\abstract` in our binding is
"locked" — the user's `\font\abstract=cmr8` can't override it.

**Principled approach.** Make our `\font` primitive recognise
"redefining a locked CS to a font" as a USER OVERRIDE
indicator and bypass the lock for that CS. This is a kernel
quality improvement: `\font` is supposed to fully replace the
CS's meaning per TeX semantics.

### Cluster D — babel "Unknown option" languages on TL2025

**Status:** SHARED-FAILURE confirmed. ~58 first-error papers.

**Root cause.** TL2025 babel dropped `italian.ldf`,
`spanish.ldf`, etc. in favour of `locale/<lang>/babel-<lang>.tex`
(ini-file system). Both engines fail `Package babel Error:
Unknown option 'italian'` on `\usepackage[italian]{babel}`.

**Principled approach.** Patch our `babel.sty` binding to
recognise the new ini-file system: if `<lang>.ldf` not found,
look up `locale/<lang2>/babel-<lang>.tex` (where `<lang2>` is
the ISO code from `babel_support_sty::babel_language_to_iso`)
and load it. Surpass-Perl until upstream catches up.

### Cluster E — expl3 csname-protocol cluster (deferred Task #22)

**Status:** OPEN. Same root cause as the mhchem retirement gap.
~13 first-error papers + the 77-error mhchem residual.

**Root cause and approach** already documented in the
"mhchem retirement" section above. No change.

### Cluster F — `\endgroup`-`\figure` RevTeX 3.x short-form

**Status:** CLOSED. Rust SUPERSEDES Perl on 9/10. SHARED on
the 10th. **No action.** ~10 papers.

**Root cause.** RevTeX 3.x's `\figure{N} caption…` short-form
(aps.sty L616-628, non-`floats` mode) has no binding in either
engine. Rust recovers further from the resulting unclosed-mode
error than Perl. Witness counts: cond-mat9607130 (Rust 1,
Perl 7), hep-th9410220 (Rust 93, Perl 102), …

**Why no Rust binding.** Verified 2026-05-13 against
`~/LaTeXML/lib/LaTeXML/Package/revtex*.ltxml` — Perl has zero
`\figure` definitions (only `\printfigures` in revtex4_support).
A Rust-only `DefMacro!(r"\figure {}", "…")` would be a hotfix
diverging from Perl. Per `feedback_perl_parity_bindings.md`
the project rule is "match Perl, do not innovate" — the
earlier "Principled approach" plan (provide a short-form
binding) was retracted. The cluster's already-recorded
"Rust SUPERSEDES" verdict stands.

### Cluster G — long-tail single-witnesses (~274 papers)

**Status (current, post-Round-34):** Cluster G is effectively
**closed**. The 274-paper sample was progressively triaged across
Rounds 26–34 (2026-05-12 → 2026-05-17). Most papers split into
the SHARED-FAILURE log below or were fixed by engine work
documented elsewhere; remaining single-witness regressions roll
up into the broader corpus pass-rate.

**Cross-corpus validation (2026-05-17):** **4736 / 4736** random
arxiv samples across `next_warning_papers`, `warning_papers_3`,
and historical pre-2000 corpora pass with **0 errors** on the
current binary. Effective pass-rate is statistically
indistinguishable from 100%.

**Remaining deferred work** (none block the mission-success
criterion):
* **Task #10**: math-parser xml:id collision cases.
* **Task #22**: mhchem retirement gap (expl3 csname-protocol
  cascade). See "mhchem retirement" section below.
* `neurips_2024.sty` mode-switch cluster (~4 papers).

**SHARED-FAILURE clusters confirmed** (Perl and Rust both fail
identically; no engine action required):
* `\@math@daccent`/`\@math@baccent` paper-side `\def\d` (~14
  papers).
* `\begin{abstract}` mode-switch on plain-TeX-style abstracts
  (~46 papers).
* babel "Unknown option" PackageError on TL2025.
* apacite/`\citep`/`\citet`/`\citealp` chain (not in TL).

Round-26 mission summary (compact): the 100,000-paper "warning"
subset converted at **99.39–99.44%** end-to-end OK; residuals
(~0.56%) overwhelmingly SHARED-FAILURE. Per-round iteration
logs from this period are archived at
[`archive/round19_iteration_log.md`] and were pruned from this
doc on 2026-05-18 (kept the corpus state, dropped the play-by-play).

`cargo test --tests` was **1190/0/0** at Round-26 close
(commit visibility); current local verification is in
[`docs/SYNC_STATUS.md`](.) header.

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
| `\halign` body `\rowEnd` | row separator at digest time | ✓ (2026-05-15) |
| `\csname` consumption | Knuth: error; we: soft-substitute | divergence (`f8e20b648e`) |

The body-side implicit-`\cr` gap was closed 2026-05-15 by fixing
`is_implicit_cr` (`latexml_engine/src/tex_tables.rs`) to do meaning-
equality against `lookup_meaning(\cr)` / `lookup_meaning(\crcr)`,
mirroring `gullet::is_column_end`'s body-side approach. The original
preamble-side fix in `6a7d8fee7d` only matched `Stored::Token(\cr)`
shape, but `\let \rowEnd \cr` against the LaTeXML Constructor `\cr`
produces `Stored::Constructor` — so the preamble parser was missing
implicit-CR for the common case, eating the entire halign body as
template and silently producing no tabular. Regression test:
`tests/trip/halign_body_implicit_cr.tex` with content-shape
assertion (not just code == 0; the bug had code == 0).

---

## Engine file open gaps (MINOR)

- ~~`base_parameter_types.rs` — `CommaList:Type` parameterised
  form unported.~~ **CLOSED 2026-05-15** (commit `bb17c1adb0`).
  Reads each item through the inner-type Parameter via
  `Parameters::reparse_argument`, mirroring Perl
  `$typedef->reparseArgument`. Tests 1220/0/0 (no Perl users
  in current corpora; pure parity infrastructure).
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

| Gate | Current (2026-05-15) | Target |
|---|---|---|
| `cargo test --tests` | **1220/0/0** | unchanged |
| `cargo clippy --workspace --all-targets` | **0 warnings** | unchanged |
| `latexml_oxide --init=plain.tex` | 0 errors (dump + `LATEXML_NODUMP=1` paths) | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors (dump + `LATEXML_NODUMP=1` paths) | 0 errors |
| Round-25 cumulative regressions | 31 fixed, ~14 deferred | drive deferred to zero |
| 1910.01256 mini-benchmark vs pdflatex×2 | **1.18s** vs **1.11s** idle (tied within noise) | beat 2× pdflatex (currently met at 0.4× the stretch goal) |

Distribution follow-up — **LANDED 2026-05-15** (branch
`distribution-include-bytes-bundling`, merged into the testing
branch). Versioned dump filenames + compile-time embedded fallback
via `include_bytes!` ship multiple TL years (TL2023 + TL2025 currently
committed). Runtime year detection uses
`kpsewhich -var-value=SELFAUTOPARENT` with `pdflatex --version`
fallback (note: `kpsewhich --version` returns the same kpathsea
string across TL releases, so it's NOT a reliable discriminator —
the as-built doc was misleading). Resolution chain:
`$LATEXML_NODUMP` → `$LATEXML_DUMP_PATH` → `$LATEXML_DUMP_DIR/<kind>.YYYY.dump.txt`
→ exe-relative → dev-tree → embedded fallback.

Follow-up IA consolidation (`81176ba689`): the latex dump shrank from
~7.4 MB → ~3.7 MB by collapsing per-slot fontdimen V-records into
per-(font, size) `IA` records with RLE-encoded data. 25 new unit
tests pin the round-trip + RLE edge cases + V-record backward compat.

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

---

## Distribution-readiness dependency cleanup (audit 2026-05-17)

Snapshot of release binary: **57 MiB stripped / 72.5 MiB before
strip**; .text = 37.1 MiB, .rodata = ~13 MiB (embedded TL2023+TL2025
dumps via `include_str!`), `.eh_frame + .gcc_except_table` = ~3.5
MiB. **Bulk of .text is OUR code** (latexml_package 41%, engine 16%,
contrib 13%, core 10%); third-party deps combined ≈ 8%. So dep
cleanup is **compile-time hygiene** more than binary-size, but
duplicate-version pairs are still painful for cache / build time.

Tasks below ordered by ratio of payoff to risk.

### Tier 1 — Cargo.toml hygiene (no runtime change, no risk)

- **DEP-01 — Remove unused direct deps from Cargo.toml** ✅
  Re-audit on 2026-05-18 found the original three items already
  resolved before the audit was filed: `base64` already lived in
  `latexml_package` (where it's actually used), `chrono` already
  in `latexml_engine`, `string-interner` no longer present in
  engine/package/math_parser. **Plus newly-found unused dep
  removed** in commit `c57bcf8760`: `unicode-normalization` was
  in `latexml_package/Cargo.toml` with zero use sites.
- **DEP-02 — Move test-only deps out of the runtime tree** ✅
  Landed 2026-05-18 (commit `c57bcf8760`): split `util/test.rs`
  → `util/preset.rs` + feature-gated `util::test` behind
  `test-utils` (default on). `latexmlmath_oxide` now imports
  from `util::preset` so the production binary builds cleanly
  with `--no-default-features`. Drops 5 transitive crates
  (`phf` + `phf_generator` + `phf_macros` + `phf_shared` +
  `siphasher`) from the runtime dep graph.

### Tier 2 — Eliminate duplicate-version pairs

Re-audit 2026-05-18 (`cargo tree --duplicates`):

| Crate | Status as of 2026-05-18 |
|---|---|
| `syn` 1.0 vs 2.0 | ✅ **DEP-03 resolved**: no longer duped; only `syn 2.x` in the workspace. |
| `regex-syntax` 0.6 vs 0.8 | ✅ **DEP-04 resolved**: marpa fork bumped to 0.8. |
| `rustix` 0.38 vs 1.1 | ✅ **DEP-05 resolved upstream**: `kpathsea v0.2.5` now pulls `which v8` → libc only. The 0.38 path is gone. |
| `hashbrown` 0.16 vs 0.17 | ⏳ **DEP-06 still open**: `string-interner 0.20` pins 0.16, `indexmap 2.14` (via `zip 8`) pulls 0.17. Upstream-blocked. |
| `tar` v0.4.45 (×2) | ℹ️ Same version, different features: runtime build (no `xattr` after `3e7c039eb1`) vs `libmarpa-sys` build-dep (default). Build-dep doesn't link into runtime binary — benign. |

### Tier 3 — Slim feature sets / drop unmaintained crates

- **DEP-07 — Replace `ansi_term v0.12` with `anstyle`** ✅ Done:
  `latexml_core/src/util/logger.rs` now uses raw ANSI SGR escape
  sequences (no external crate). `ansi_term` no longer in tree.
- **DEP-08 — Drop `dirs v6.0` for `std::env::var_os("HOME")`** ✅
  Done: `latexml_core/src/util/pathname.rs` uses `var_os` (with
  inline comment documenting the replacement).
- **DEP-09 — Slim `chrono`** ✅ Done: `latexml_engine/Cargo.toml`
  has `chrono = { version = "0.4", default-features = false,
  features = ["clock", "std"] }`.
- **DEP-10 — Audit `regex` feature flags** ⏳ Partially blocked:
  audit found we use `\p{Latin}`, `\p{Greek}`, `\p{Lu}`, `\p{N}`,
  and `\w` patterns. Fully disabling `unicode` would break those.
  Selective disable of `unicode-age`, `unicode-bool`, `unicode-case`,
  `unicode-segment` may be safe — not yet attempted.
- **DEP-NEW: slim `sha2`** ✅ Done (commit `c57bcf8760`):
  `default-features = false` drops the `oid` feature (DER object-id
  tables) which we never touch.
- **DEP-NEW: slim `tar`** ✅ Done (commit `3e7c039eb1`):
  `default-features = false` drops the `xattr` crate; we only need
  basic `tar::Archive::new(...).unpack(dest)` for arxiv zips.

### Tier 4 — Profile / packaging for distribution

- **DEP-11 — `panic = "abort"`** ✅ **Refined** and landed on
  `maxperf` only (commit `c57bcf8760`), **NOT on `release`**. The
  user's canvas sweeps via `cortex_worker` use `release` and rely
  on `thread::spawn().join()` for per-paper panic isolation — that
  pattern silently breaks under `panic = "abort"` (the whole worker
  process aborts instead of recording the failure). `maxperf` is
  the public-distribution profile (no debugging requirement); it
  gets `panic = "abort"` for the 1.9 MiB size saving + slightly
  better optimization. Comments in `Cargo.toml` document the
  distinction explicitly.
- **DEP-12 — TL-dump distribution model**. Unchanged — design
  call, owner input required.
- **DEP-13 — Document ship-build recipe**. ⏳ Partially: `Cargo.toml`
  comments now explain the release-vs-maxperf distinction. `CLAUDE.md`
  already documents `cargo build --profile maxperf --bin
  latexml_oxide` for distribution. Open: add `--no-default-features`
  to that line so phf/glob are also dropped.

### Tier 5 — Code-architecture wins worth flagging

- **DEP-14 — Feature-gate `proc-macro2` + `quote` in
  `latexml_core`** ✅ Landed 2026-05-18 (commit `1365989630`):
  added `codegen` feature, made `proc-macro2` + `quote` optional,
  wrapped the 5 `impl ToTokens for X` blocks (in `tokens.rs` and
  `parameter.rs`) with `#[cfg(feature = "codegen")]`.
  `latexml_codegen` activates the feature on its dep edge;
  resolver v2 keeps proc-macro feature unification isolated so
  the runtime `latexml_core` doesn't compile those impls.
  **Reality check**: binary size delta was essentially zero
  (+448 bytes on release) — LTO had already been dead-stripping
  those symbols. The audit's "~93 KiB" claim was overstated. The
  win is architectural (compile-time clarity, smaller per-build
  `latexml_core` graph), not binary size.
- **DEP-15 — Investigate the per-`load_definitions` size
  bloat**. ⏳ Open — see post-audit notes below for the concrete
  data and a proposed approach.

### DEP-15 follow-up — cargo-bloat data + approach (2026-05-18)

Top `.text` consumers on `target/release/latexml_oxide`:

| Function | Size | % of `.text` |
|---|---:|---:|
| `latexml_contrib::fontawesome5_sty::load_definitions` | 2.0 MiB | 5.3% |
| `latexml_engine::latex_constructs::load_definitions`  | 1.1 MiB | 3.0% |
| `latexml_core::common::font::standard_metrics::STDMETRICS::{closure#0}` | 811 KiB | 2.1% |
| `latexml::dump_compiled_latexml_model::_ModelLoader::build_model` × 2 | 1.2 MiB | 3.2% |
| `latexml_package::package::jhep_cls::load_definitions` | 511 KiB | 1.3% |
| `latexml_contrib::fontawesome_sty::load_definitions` | 480 KiB | 1.3% |
| `latexml_package::package::mathabx_sty::load_definitions` | 438 KiB | 1.1% |
| (10 more in the 200–340 KiB range) | ~2.3 MiB | ~6% |

Top 16 functions account for ~22% of `.text` (~9 MiB on a
40-MiB code section). Universally they're `LoadDefinitions!`
bodies with hundreds of repeated `DefMacro!` / `DefConstructor!`
invocations.

Concrete approach for `fontawesome5_sty` (1373 trivial calls,
all pattern `DefMacro!("\\faXxx[]", "\\faIcon[#1]{kebab-name}")`):

1. Add a runtime helper `def_fa5_icon(suffix: &'static str,
   kebab: &'static str)` in `fontawesome5_sty.rs`. Internally
   constructs the CS via `T_CS!`, parses the parameter spec
   `[]` once, and constructs the expansion `Tokens` at runtime
   instead of compile-time inlining.
2. Replace the 1373 trivial `DefMacro!` lines with single-line
   `def_fa5_icon(b"AccessibleIcon", b"accessible-icon");` calls.
3. Keep the ~20 non-trivial `Match:N` / `OptionalMatch:*` /
   `Number[]` variants as full `DefMacro!` calls (low count, no
   benefit to data-driving).

Expected impact: 2.0 MiB → ~50–100 KiB on `fontawesome5_sty`
alone. Similar mechanical refactor on `fontawesome_sty` (~480
KiB → ~30 KiB).

Risk: each `DefMacro!` arm runs through `parse_prototype!` +
`compile_expansion!` at compile time; building the same `Tokens`
structure at runtime requires the helper to mirror the exact
shape. Validation: byte-for-byte XML output equality on the
`fontawesome*` test fixtures.

Profile-guided, not a quick win — but the upper bound (2.5
MiB combined fontawesome saving) is the biggest single binary-
size lever remaining after DEP-1 through DEP-14.

---

## Math parser ↔ Marpa ASF migration (planned 2026-05-17)

A multi-session effort to swap the math parser's Tree-iteration
+ per-tree-pruning loop for ASF-driven traversal.

**Working docs**:
* [`docs/MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md) — full
  rationalization: where the existing three stages (grammar
  categories, early semantic pruning in actions, late semantic
  pruning in pragmas) map onto ASF, a worked example, pseudocode
  for the new driver, and a four-gate test plan. **Read first.**
* [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md)
  on the `asf-completion` branch of dginev/marpa — what's
  scaffolding vs functional on the marpa side, with a 7-step
  completion plan and the target Rust API sketch.

**Status snapshot 2026-05-17 (end of session)**:
* Marpa fork `asf-step3-generic-traverser` branch — **Steps 2-6
  LANDED**:
  * `compute_symches` ported (Perl `ASF.pm`-faithful: contiguous
    same-predecessor and-nodes unify into multi-source glades).
  * `Glade` query API: `rule_id`, `symch_count`, `factor_count`,
    `is_factored`, `rh_length`, `rh_glade_id`, `next`, `rewind`,
    `is_token`, `cursor`, `symches()`. (`literal()` deferred —
    needs SLR; math parser is a token-stream consumer, doesn't
    need text spans.)
  * `ASF::traverse` is now a post-order recursive driver with
    per-glade `HashMap<usize, PT>` memoization. Cycle-safe via
    `visited` flag.
  * `Traverser` trait: generic + `&mut TR` (no `Box<dyn>`). Allows
    borrowing traversers like `MathTraverser<'a>` that hold
    `&'a mut Document` + `&'a Actions`. Single-threaded by design.
  * `asf_three_parses_via_exhaustive_traverser` substantive test:
    panda grammar produces exactly 3 distinct Penn-tagged strings
    via post-order memoized traversal — the substantive end-to-end
    validation.
  * 17 marpa tests pass (was 13 before this session).
* latexml-oxide:
  * Cargo.toml marpa dep switched to
    `branch = "asf-step3-generic-traverser"`.
  * Full test suite (1301/0/0) passes against the new marpa branch.
  * `latexml_math_parser/src/asf_traverser.rs` — **scaffolding
    landed**: `MathTraverser` struct implementing
    `marpa::asf::Traverser`. Handles byte glades, lexeme-rule glades
    (matches `TreeBuilder::rollup_token` semantics), standard rule
    glades (Cartesian product + `Actions::action_on`).
    **Not yet wired into `parse_marpa`** — that's the next-session
    task.

**Remaining sequence**:
1. ✅ **LANDED**: `MathTraverser` wired behind `LATEXML_MARPA_ASF=1`.
   Side-by-side runs validated.
2. ✅ **MOSTLY LANDED**: pragma/action prunes for ambiguity classes
   (1272 → 1292 ASF; LEGACY 1301/0 preserved).
3. ⏳ Validate on the 10k canvas stage. Expect 0 test regressions,
   measurable perf gain on ambiguous formulas.
4. ⏳ **Open**: 9 remaining ASF failures — ambiguous_relations,
   count_parses, mathtools, metarelation_elision, physics,
   plainfonts, qm, standalone_modifiers, vertbars. See research
   notes in `docs/MATH_PARSER_ASF_TIEBREAKING.md`.
5. ⏳ **Open principled refinement**: `modified_term` grammar
   category (proposed 2026-05-17; user-articulated). Expected to
   subsume 5-6 of the remaining 9 by structural change at the
   grammar level. Deferred to its own session.
6. ⏳ Delete 5 of the 6 convergence caps in `parser.rs` (only
   `max_time` stays). Delete online `parses.contains(&tree)` dedup.
7. ⏳ Once stable, ask user to merge the marpa PR into dginev/marpa
   master, then switch latexml-oxide's marpa dep back to master.

**Session progress (2026-05-17, second push)**: ASF parity
**1272/29 → 1292/9** (20 tests fixed) via:
* `FencedLettersAreFunctionArguments` Dual-aware + tier move (12)
* `prefer_named_interval_at_root` for `(a,b)`, `[a,b]` (2)
* `prefer_non_self_wrapping_root` for `set@(set@(...))` (2)
* `prefer_combined_relop_over_multirelation_with_absent` (subcase fix)
* Early-action prune for `Apply(OPERATOR, [single]) * simple_RHS` (1)
* Compose left-associativity in `infix_apply` (1)
* `bare_conditional` reject in `list_apply` (1)
* `prefer_zero_absent_when_available` + ncases.xml bless (1)

**The win**: eliminates the 5000-tree cap. Per-formula action cost
drops from O(trees × occurrences) to O(glades). Removes the five
convergence bandages (`max_trees`, `max_consecutive_dupes`,
`pruned_only_time_budget`, `converge_budget`, `max_unique`) that
exist purely to dodge the wrong-paradigm cost. `max_time` is the
only cap that needs to stay.
