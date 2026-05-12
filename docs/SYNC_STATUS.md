# Engine Sync Status — Active Worklist

**Mission (closed 2026-05-12)**: 100k "no-problem" sandbox parity on
the 426,555-paper arxmliv corpus, in-scope iff Perl LaTeXML on TL2025
(`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings`) produced
0 errors. Round-25 stages 1-43 closed at **~99.85% aggregate OK**,
stage 41 = **100.00%**, 30 RUST-REGRESSIONs fixed; ~15 deferred (all
single-paper niche or cascade-amplification). Pre-Round-25 sprint
records live in [`archive/round19_iteration_log.md`].

**Active focus**: retire hand-stub bindings via raw-load. Remaining
blocker is the **mhchem 77-error expl3 csname-protocol gap** — see
"mhchem retirement" below.

`cargo test --tests` = **1185/0/0**.

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
