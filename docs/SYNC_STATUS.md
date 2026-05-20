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

Per-stage first-pass tallies (each row = 10k papers):

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

Combined first-pass: **99,390 / 100,000 OK = 99.39%**. With targeted
per-stage re-runs against the iteratively rebuilt release binary
(+51 recovered): **~99,441 / 100,000 = 99.44%**. Round-26 close.

**Round-27 cluster work plan (opened 2026-05-13)**: the 220-paper
classified-cluster cohort is worked from kernel-and-core quality
outward to individual bindings. Open clusters are described below;
closed clusters are dropped from this doc once their fix lands and
generalizes.

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
- `\thanks OptionalSemiverbatim {}` (2026-05-18 session) — `[opt]`
  is identifier-shape (label tag, often discarded by the constructor
  anyway). Per cluster-A principled approach, switch to
  OptionalSemiverbatim to neutralize `_`/`^`/`~`/`&`/`$`/`#`/`'`
  catcodes in the optional label arg.

**Audit candidates — verified 2026-05-18:**
- `\ref`/`\pageref`/`\eqref` — ✅ `OptionalMatch:* Semiverbatim`
  (latex_constructs.rs:7421; pageref Let-aliased to ref).
- `\label` — ✅ `Semiverbatim` (latex_constructs.rs:7358).
- `\cite[]Semiverbatim` — ✅ key arg Semiverbatim
  (latex_constructs.rs:7816). `\citep`/`\citet`/`\citealp` forward
  via `Semiverbatim` in biblatex_sty.rs.
- `\href HyperVerbatim {}` — ✅ HyperVerbatim neutralizes catcodes
  (hyperref_sty.rs:305 + base_parameter_types.rs:553).
- `\url` — ✅ url_sty.rs reads via begin_semiverbatim internally.
- `\hyperref` — ✅ dispatches to `OptionalSemiverbatim {}` or
  `Semiverbatim×4` (hyperref_sty.rs:386-396).
- `\bibitem` — ✅ delegates to `\lx@bibitem[] Semiverbatim`
  (latex_constructs.rs:7629).
- `\caption`/`\subcaption` `[short]` — content-shape after
  re-evaluation; the optional short caption is real text content
  (allows `$x^2$`), not identifier-shape. NO change.
- `\index` — ✅ `SanitizedVerbatim` (base_parameter_types.rs:526).

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

**Status:** SHARED-FAILURE confirmed (5/6 sampled). **WONT-FIX**
(user directive 2026-05-19): "`\font` on a locked primitive
shouldn't work." Accept the SHARED-FAILURE. ~46 first-error
papers.

**Root cause.** Pre-2000 papers use `{\abstract \ni …}` as a
font-switch group (`\font\abstract=cmr8`), then `}` closes the
group but the abstract environment is still open and in
internal_vertical mode. `\abstract` in our binding is
"locked" — the user's `\font\abstract=cmr8` is correctly a no-op
(Info!("ignore", "\\abstract:locked", ...)). The downstream
cascade Error from `{\abstract …}` opening the env unexpectedly
is left as-is.

**Why not surpass-Perl.** The author's source violates LaTeX
convention by shadowing a class-provided macro (`\abstract` is
reserved by `article.cls`). They should have used `\newfont` (which
does `\@ifundefined` and errors loudly on the collision) or chosen
a non-clashing CS name. Bypassing the lock specifically for `\font`
would accommodate the anti-pattern; we instead match Perl
LaTeXML's defensive behaviour. An earlier attempt to bypass the
lock (commit reverted same session) ran cleanly on the minimal
repro but was retracted to preserve the lock invariant.

### Cluster D — babel "Unknown option" languages on TL2025

**Status:** ✅ Effectively resolved 2026-05-19 (re-verified). The
witness cluster behaviour has been closed by `babel_lang_stubs.rs`
(commits `6249382abb` 2026-05-16 + `8acb8135cf` 2026-05-17), which
landed AFTER the sweeps that produced the ~58-paper count.

**Verification 2026-05-19.** Minimal-repro
`\usepackage[<lang>]{babel}` for italian/spanish/portuges/brazil/
czech/polish/romanian/slovene/turkish/vietnamese/icelandic/arabic/
dutch/farsi/hindi/latin/croatian + bulgarian/catalan/danish/
estonian/finnish/galician/greek/hebrew/hungarian/magyar/norsk/
nynorsk/russian/serbian/slovak/swedish/ukrainian/welsh/irish/
afrikaans/esperanto/interlingua/serbianc/slovenian/swissgerman/
friulan/basque/welshb/bahasa — **all 0 errors**. The TL2025
ini-file fallback (`locale/<lang2>/babel-<lang>.tex`) loads cleanly
once `italian.ldf` etc. resolve via our `<lang>.ldf`→stub binding,
which is the on-disk fallback our `find_file` routes to.

Single SHARED-FAILURE outlier: `azerbaijani` errors `Package
azerbaijani Error: No font containing the schwa has been
detected. Please, load a Cyrillic encoding (T2A, T2B, T2C, X2)`.
That is a real package-side requirement, not a babel-options gap.

**Principled approach (HISTORIC).** Patch our `babel.sty` binding
to recognise the new ini-file system: if `<lang>.ldf` not found,
look up `locale/<lang2>/babel-<lang>.tex` (where `<lang2>` is the
ISO code from `babel_support_sty::babel_language_to_iso`) and load
it. Already implemented via the stubs above + `find_file`'s notex
fallback to `locale/<iso>/babel-<lang>.tex` — surpass-Perl is in
effect.

### Cluster E — expl3 csname-protocol cluster (deferred Task #22)

**Status:** OPEN. Same root cause as the mhchem retirement gap.
~13 first-error papers + the 77-error mhchem residual.

**Root cause and approach** already documented in the
"mhchem retirement" section above. No change.

### Cluster F — `\endgroup`-`\figure` RevTeX 3.x short-form

**Status:** CLOSED. Rust SUPERSEDES Perl on 9/10. SHARED on the
10th. Verified 2026-05-13 against Perl revtex*.ltxml — no
`\figure` short-form binding in either engine; Rust just recovers
further from the unclosed-mode error.

### Cluster G — long-tail single-witnesses (~274 papers)

**Status:** effectively CLOSED post-Round-34 (2026-05-17). Most
papers split into the SHARED-FAILURE log; remaining single-witness
regressions roll up into the corpus pass-rate. Cross-corpus check:
4736/4736 random arxiv samples pass with 0 errors.

**Remaining deferred work** (none block the mission):
* **Task #10**: math-parser xml:id collision cases.
* **Task #22**: mhchem retirement gap. See "mhchem retirement"
  below.
* `neurips_2024.sty` mode-switch cluster (~4 papers).

---

## mhchem retirement (Round-26 candidate)

`latexml_contrib/src/mhchem_sty.rs` intercepts TL `mhchem.sty`
(~110 lines as of 2026-05-19). The raw chain is `chemgreek` →
`xparse` → expl3 (group machinery, `\__file_tmp:w`, l3regex,
l3tl-analysis). Driver: arXiv:1806.06448.

**Minimal repro (2026-05-19)**: set `LATEXML_MHCHEM_NOLTXML=1` to
bypass the stub (env-var probe in `mhchem_sty.rs`). With:
```
\documentclass{article}
\usepackage[version=3]{mhchem}
\begin{document}
\ce{H}
\end{document}
```
HYBRID + release binary yields **77 errors** (matches the
SYNC_STATUS-recorded baseline exactly). Just `\usepackage{mhchem}`
without any `\ce{...}` invocation produces **0 errors** — the
77-error cascade is triggered specifically by the first `\ce{...}`
invocation, which forces the (lazy) chemgreek load chain to
execute inside the `\ce{...}` argument-handling code path.

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

**First diagnostic anomaly (2026-05-19)**: the cascade begins with
`Warn:expected:<number> Missing number, treated as zero while
processing "\int_value:w", next token is Some(";")` at line 6 col 1
(the `\ce{H}` line). That is, `\int_value:w` (PA→`\number`) is
called and sees `;` directly with no leading digit — so the
expected preceding digit-producing expansion produced **no digits**.
Once the `;` is consumed mid-`\int_value:w` read, every following
expl3 token (`\__int_eval_end:`, `\fi:`, `\else:`, `\s__tl`,
`\tex_skip:D`, etc.) shifts left by one slot, surfacing in
`\csname...\endcsname` reads where it shouldn't — the 77-error
cascade is purely downstream from this single mis-evaluation.

Isolated `\int_value:w \int_eval:n {2+3}` outside mhchem works
fine (`= 5`, 0 errors), so the basic PA-aliased numexpr chain is
correct (`\__int_eval_end:` PA→`\relax` is correctly recognized
by `latexml_engine/src/etex.rs::is_relax_meaning` via
`Stored::Primitive(p) if *p.get_cs() == *TOKEN_RELAX`). The mhchem
mis-expansion is a more elaborate pattern — likely a deeper
`\__mhchem_*` or chemgreek `\use:c { ... }` chain where one of
the intermediate macros isn't expanding. **Next debugging step**:
instrument `read_x_token` to log token + meaning class around
line 6 col 1 in the minimal repro, narrow to the first non-empty
return that doesn't match the expected expansion.

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

- **xcolor double-load Option clash** — paper-local `.cls` runs
  `\usepackage{xcolor}` (no options), then user preamble runs
  `\usepackage[svgnames,x11names]{xcolor}` — the second load's
  Option clash silently drops the svgnames/x11names InputDefinitions,
  so `\color{Gainsboro}`/`\color{Green4}` etc. error out. Both Perl
  and Rust `xcolor.sty.ltxml` are purely option-driven, so both engines
  fail identically here. Witnesses: 2204.01429 (24 errors), 2204.01753
  (1 error). An earlier Rust-only unconditional pre-load
  (commit `c5c16953e5`, reverted 2026-05-20 in this branch) traded
  these papers for a `xcolors_test` regression — x11nam's DarkOrchid
  / LimeGreen overrode the test's `[dvipsnames]`-only expectations.
  **Proper surpass-Perl path** (not yet designed): when xcolor is
  re-loaded with options that weren't on the first load, process the
  new options instead of suppressing the second `\usepackage`.

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

| Gate | Current (2026-05-20) | Target |
|---|---|---|
| `cargo test --tests` | **1328/0/0** | unchanged |
| `cargo clippy --workspace --all-targets` | 14 warnings (all in `latexml_math_parser`, residual clippy cleanup of post-ASF-migration code — collaborator's lane) | 0 warnings (clippy cleanup landed) |
| `latexml_oxide --init=plain.tex` | 0 errors (dump + `LATEXML_NODUMP=1` paths) | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors (dump + `LATEXML_NODUMP=1` paths) | 0 errors |
| 1910.01256 mini-benchmark vs pdflatex×2 | release (`--dest=.html`, full post-processing): **0.71s** post-DEP-19 (was 0.73s); pdflatex idle ~1.11s. `.xml`-only is **0.60s** but not a fair comparison since pdflatex always runs graphics + bibliography. | beat 2× pdflatex (met: 0.71s ≪ 2.22s) |
| Distribution build | Release profile (post-DEP-22, 2026-05-19): **44.38 MB**. `--no-default-features --profile maxperf` previously measured **44.98 MB** (pre-DEP-18h + pre-DEP-22, 2026-05-18); current maxperf is expected slightly lower after helper consolidation. | maxperf ~55 MB (overshot — gate met) |

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

- **P1 graphics** ✅: primary rasterizer optimization done 2026-05-12
  (`5244a5a4e2` → `feaf8bcd16`); graphics phase 1031 ms → ~480 ms
  on 1910.01256. Follow-ups ALSO done: content-identity conversion
  cache (`latexml_post/src/graphics_cache.rs`, `bba00c0c83` 2026-05-16);
  cross-document duplicate coalescing
  (`graphics.rs::process_coalesces_only_matching_conversion_options`
  test verifies it).
- **P1 digest+build** ✅ CLOSED 2026-05-19: profile-driven sweep on
  `2305.06773` confirmed the residual cost is structural to the TeX
  read-then-invoke pattern (the same meaning is probed in
  `read_x_token` to decide expansion, then again in `invoke_token` to
  decide invocation). Combining the two probes would require an API
  change on the gullet — explicitly out of scope, the gullet API
  mirrors TeX by design (user directive 2026-05-19). Internal wins
  landed: `Catcode::name_sym` in `lookup_digestable_definition`
  (`f2e23d9570`), `has_meaning` migration for 8 sites doing
  `lookup_meaning(t).is_some()/.is_none()` (`3f06ecebd6`),
  `Token::pin_cs_name` in `lookup_conditional` (`2b63a1a0a1`), plus
  6 companion clippy-driven function-body sweeps (redundant_clone /
  or_fun_call / needless_collect / stable_sort_primitive /
  implicit_clone / manual_string_new). Full close-out in
  `docs/PERFORMANCE.md` under "P1 digest + build … CLOSED 2026-05-19".
  Do not reopen without new digest-bound witnesses that diverge from
  the recorded SwissTable-probe-floor pattern.
- **P1 math/large-doc**: `LATEXML_PARSE_AUDIT=1` on astro-ph0204009,
  0911.0884, astro-ph0401354, 0809.5174, astro-ph0507615.
- **P2 allocation/startup**: partial landings 2026-05-12 (arena
  pre-alloc, `State::meaning` pre-alloc, dump_reader Vec elimination)
  + 2026-05-19 (`*_sym` accessors converted at the two hot sites
  identified by perf — `lookup_digestable_definition` /
  `lookup_conditional`). Remaining open: `Tokens` conversions,
  `Stored` deep copies, package lookup caching — land only when a
  fresh profile shows them above the SwissTable-probe floor.

---

## Distribution-readiness dependency cleanup — closed audit

Closed 2026-05-19. Release binary **44.60 MiB stripped** (down
from 57.12 MiB pre-audit); .text ≈ 34.3 MiB, .rodata = 2.2 MiB
(TL2023+TL2025 dumps gzipped). The remaining .text is OUR
macro-arm bindings (latexml_package 41%, engine 16%, contrib 13%,
core 10%) — i.e. payload, not dependencies.

**Settled lessons (do not retry):**
* Generic `T: Into<X>` helpers GROW the binary via
  per-call-site monomorphization
  ([[wisdom_helper_monomorphization_trap]]). Only concrete-value
  helpers shrink.
* Data-drive helpers need ≥5 dominant call-sites per file to
  net-shrink ([[wisdom_data_drive_min_call_sites]]).
* Helpers needing complex option structures (e.g. textcomp's
  `bounded => true, font => { encoding => "TS1" }`) cross the
  ergonomics-vs-savings line.

**Remaining unconsolidated text-section consumers** (per fresh
`cargo bloat`, future re-audit input):

| Candidate | .text | Notes |
|---|---:|---|
| `latex_constructs::load_definitions` | ~1.0 MiB | varied; sub-module split would be next lever |
| `STDMETRICS::{closure#0}` | 810 KiB | font-metric data tables, not a macro-arm pattern |
| `_ModelLoader::build_model` | 602 KiB | RelaxNG schema (DEP-16 already collapsed 2 sites to 1) |
| `proofwiki_sty` | 201 KiB | 254 distinct-body DefMacros |
| `textcomp_sty` | 137 KiB | 89 DefPrimitive with font directive |

`panic = "abort"` is `maxperf`-only (NOT release —
`cortex_worker` per-paper isolation needs unwinding). Distribution
build recipe is in `CLAUDE.md`
(`--no-default-features --profile maxperf`).

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
4. ✅ **CLOSED 2026-05-19**: the 9-test list referenced below
   was already obsolete (down to 1 — `physics_test`); the residual
   `physics_test` failure under `LATEXML_MARPA_ASF_ONLY=1` is now
   resolved. Both `cargo test --tests` (HYBRID, default) and
   `LATEXML_MARPA_ASF_ONLY=1 cargo test --tests` report
   **1328/0/0** on this branch.
   Root cause: the grammar had two rules matching `\sin[arg]` in
   `applied_func` — `opfunction tight_term => prefix_apply` AND
   `opfunction lbracket formula rbracket => apply_delimited`
   (`[arg]` is also a `fenced_factor` → `tight_term` via
   `lbracket formula rbracket => fenced`). HYBRID's Tree-iter
   landed on `prefix_apply` and capped via `max_unique`; ASF's
   Cartesian-product enumeration ran BOTH rules. `apply_delimited`
   eagerly XMRefs its `func` operand through `create_xmrefs` →
   `Document::generate_id`, bumping `_ID_counter_` on the math
   ancestor for a tree that's then pruned in favor of
   `prefix_apply`'s output. The wasted xml:id slot shifted
   surviving lexemes' IDs by +1 (`S1.Ex14.m1.15` vs expected
   `S1.Ex14.m1.14`).
   Fix: removed the redundant `opfunction lbracket formula
   rbracket => apply_delimited` rule in
   `latexml_math_parser/src/grammar/builder.rs`. Both modes now
   converge on `prefix_apply` for `OPFUNCTION+[…]`, eliminating
   the spurious action call. The paren variant
   (`opfunction lparen formula rparen => apply_delimited`)
   remains — `\sin(x)` is the canonical function-call notation
   that warrants the XMDual structure. `function lbracket`
   and `trigfunction lbracket` rules left intact for now (their
   rule-id signatures didn't fire on the failing case; revisit
   if a future witness emerges). Test fixture
   `tests/complex/physics.xml` re-blessed (23 xml:id
   renumberings; tighter contiguous numbering — closer to
   Perl's `t/complex/physics.xml` ID pattern, no structural
   changes).
   Historical context: the old 9-test list was
   `ambiguous_relations, count_parses, mathtools,
   metarelation_elision, physics, plainfonts, qm,
   standalone_modifiers, vertbars` — those were the ASF failures
   as of 2026-05-17 / 2026-05-18; subsequent landings (pragma
   refinements documented in `MATH_PARSER_ASF_TIEBREAKING.md`)
   closed all but `physics`, which this fix addresses.
5. ✅ **LANDED 2026-05-19**: `modified_term` grammar category
   (Phase 1 + Phase 2). Concrete witness `P(x = 0, y < 0)` —
   previously `ltx_math_unparsed`, now parses cleanly as
   `P @ vector(x = 0, y < 0)`.
   * **Phase 1 (a16cce3ddc):** narrow grammar additions —
     `modified_term = tight_term relop expression =>
     infix_relation` (single-relop only; multi-relop chains keep
     the existing multirelation path) plus
     `formula_list += modified_term punct modified_term |
     formula_list punct modified_term => modified_list_apply`.
     Early-action prune in `infix_relation` rejects `Apply(relop,
     lhs, list@(…))` when the list contains a relational item,
     forcing Marpa to commit to the modified_term + fenced path.
     `cargo test --tests` and `LATEXML_MARPA_ASF_ONLY=1 cargo
     test --tests` both **1328/0/0**.
   * **Phase 2 (994cbcfa1a):** retired the now-redundant
     `prefer_zero_absent_when_available` pragma (no dedicated
     test witness; conceptual target already covered by qm
     pragmas + angle-bracket grammar). Function body removed
     from `semantics/tree.rs`; placeholder comment in
     `parser.rs::parse_marpa` references the commit.
   * **Discipline notes:** the earlier (deferred) additive
     prototype broke 8 tests because it added a wider
     `modified_term` form at the `statement` level alongside the
     `formula relop expression` chain — additive co-existence
     multiplied ambiguity. Phase 1 stays narrow (all-modified-
     terms list variants only); mixed-content variants
     (`modified_term punct expression`, etc.) deferred until a
     witness justifies them. `parse_tree_count_limits` regression
     test is the canary.
6. ⏳ Delete 5 of the 6 convergence caps in `parser.rs` (only
   `max_time` stays). Delete online `parses.contains(&tree)` dedup.
   **Note (refreshed 2026-05-19):** the code comment at
   `parser.rs::parse_marpa` line ~1576-1589 explicitly keeps the
   caps as the LEGACY-path debug-escape-hatch protection — without
   them the legacy escape would hang on real ambiguous inputs.
   The intent of this item was the ASF/HYBRID hot path, where
   the caps don't fire anyway. Treat as a documentation cleanup
   rather than a code change.
7. ✅ **CLOSED**: marpa dep is on `dginev/marpa` master
   (`Cargo.toml` shows `git = "https://github.com/dginev/marpa"`
   with no branch; commit `0bf241116fcef…` in `Cargo.lock`).
   The asf-step3-generic-traverser branch was merged via marpa
   PRs #3 + #4 (`cdb5fa5f99` "marpa back to master (PR #4 merged,
   large-bocage fallback landed)").

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
