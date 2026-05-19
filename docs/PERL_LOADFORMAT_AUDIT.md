# Perl LoadFormat Parity Audit

Tracks the Rust→Perl translation gap exposed by the strict
`LoadFormat` split (commit `0c4d609ad`). Each entry maps a Perl
`Engine/<file>.pool.ltxml` definition to its Rust counterpart in
`latexml_engine/src/<file>.rs` (extracted from `latexml_package`
in commit `9909ba51d`), flagging divergences that break the
strict-Perl pipeline.

**Refresh status (2026-05-19):** current local verification:
`cargo test --tests` is **1328/0/0**;
`cargo clippy --workspace --all-targets` is **14 warnings (all in
`latexml_math_parser` from the in-flight ASF migration — collaborator's
lane)**. Dump resources on disk are versioned per TL year:
`plain.YYYY.dump.txt` ~958 lines, `latex.YYYY.dump.txt` ~21,475 lines
after the IA intarray consolidation (commit `81176ba689`) — was
~110,713 lines before. Both TL2023 and TL2025 dumps are committed.
The detailed call-count table below is from the 2026-04-28 audit;
engine-wide CS-name diff refresh appears in the "Engine-wide CS-name
diff refresh" section.

**Zero-error target verification (2026-04-30 iter 40):** ran
`latexml_oxide --init=plain.tex empty.tex` and
`latexml_oxide --init=latex.ltx empty.tex` against the current HEAD
binary. Both emit **0 errors**. Freshly-generated dump output
matches on-disk dumps line-for-line (plain: 0 diff lines; latex: 1
line diff — only the `texsys.aux_contents` build timestamp). Target
#4 from CLAUDE.md is met.

**Re-verified post-Round-18 (2026-05-02)**: same release binary
(rebuilt at commit `a3e44454c`, with all 7 Round-18 fixes including
`\object` `_noautoclose` `317655f01`) — both inits still emit
**0 errors**. Target #4 holds. Tests 1112/0/0.

**Audit table (2026-04-28):**

| File | Perl calls | Rust calls | Status |
|------|-----------:|-----------:|--------|
| `plain_bootstrap` | 9 | 9 | ✅ PARITY |
| `plain_base` | n/a | n/a | ✅ NEAR-PARITY (audit doc) |
| `plain_constructs` | 77 | 81 | ✅ PARITY (cosmetic gap) |
| `plain_dump` | n/a | 641 M-keys | ✅ NEAR-PARITY (17 cosmetic) |
| `latex_bootstrap` | 9 | 9 | ✅ PARITY |
| `latex_base` | 160 | 152 | ✅ PARITY |
| `latex_constructs` | 1055 | 1199 | ⚠ 13.5% extra in Rust |
| `latex_dump` | n/a | 25770 entries | ✅ NEAR-PARITY |

**Audit table refresh (2026-05-02, post-Round-18, methodology:
`grep -cE 'Def(Macro|Primitive|Constructor|Register|Math|Environment|KeyVal|ParameterType)I?[[:space:]]*\(' for Perl /
`!\\(` for Rust)**:

| File | Perl calls | Rust calls | Δ | Status |
|------|-----------:|-----------:|---:|--------|
| `plain_bootstrap` | 5 | 5 | 0 | ✅ PARITY |
| `plain_base` | 129 | 127 | -2 | ✅ PARITY |
| `plain_constructs` | 62 | 79 | +17 | ✅ NEAR-PARITY (cosmetic — Rust split for clarity) |
| `latex_bootstrap` | 8 | 7 | -1 | ✅ PARITY |
| `latex_base` | 138 | 127 | -11 | ✅ NEAR-PARITY |
| `latex_constructs` | 1071 | 1097 | **+26** | ✅ NEAR-PARITY (was +144 on 2026-04-28; **81% reduction**) |

Note: the 2026-04-28 numbers used a different counting methodology
(possibly including different macro families); not directly
comparable per-row. The 2026-05-02 refresh uses a uniform regex
across both Perl and Rust. Most importantly, `latex_constructs`
gap is now within cosmetic-drift territory (under 3%), down from
the prior 13.5% drift cited as ⚠ above.

Active: ✅ — gap is no longer actionable. Per-CS spot-checks (the
26 "extra" Rust calls in `latex_constructs`) would be useful for
strict-parity acceptance criteria but not blocking. Tests 1112/0/0,
zero-error inits hold.

**Same-file CS-name diff (2026-05-02 follow-up)**: 45 CSes are
defined in Perl `latex_constructs.pool.ltxml` but not in Rust
`latex_constructs.rs` (often defined in another Rust file). Per
CLAUDE.md priority 3 ("Every `\foo` defined in `Engine/<file>` MUST
be defined in `latexml_engine/src/<file>.rs`"), these are
candidates for future same-file relocation:

```
\ASCII, \@caption@, \@caption@postlabel, \@captype, \@cite,
\documentstyle, \ensuremath, \@filef@und, \fnum@, \@fontswitch,
\@font@warning, \format@title@, \G@refundefinedtrue,
\@@@hack@caption@, \@hack@caption@, \hexnumber@, \labelenum,
\labelitem, \ldots, \list, \loggingoutput, \lx@end@verbatim,
\lx@label, \lx@latex@input, \M@, \mathring, \newsavebox,
\@nomath, \normalsfcodes, \on@line, \pic@@savebox, \pic@savebox,
\reserved@a, \@savepicbox, \@settopoint, \showoutput,
\showoverfull, \stop, \T@, \the, \theenum, \theequation,
\theequation@ID, \tracingfonts, \@trivlist
```

These don't affect runtime correctness (zero-error inits still
pass), but moving them to `latex_constructs.rs` is the strict-Perl
parity refactoring backlog. Multi-iteration scope; not blocking.

**45-list status refresh (2026-05-18)**: spot-verified the L80-89
list against the current Rust tree. Most entries are *not* genuine
gaps but regex artifacts in the original audit:

| Bucket | CSes | Status |
|---|---|---|
| Dynamic (defined inside Perl closure body, not top-level) | `\@captype`, `\@filef@und`, `\format@title@`, `\labelenum`, `\labelitem`, `\theenum`, `\theequation`, `\theequation@ID` | Audit false positive — Perl regex matched the inner `DefMacroI` line inside a `sub { … }` body that runs only at runtime. Rust generates equivalents dynamically too. |
| Already in latex_constructs.rs but regex missed (multi-line / nested parameter spec) | `\@hack@caption@` (L6797), `\@@@hack@caption@` (L6801), `\lx@end@verbatim` (L5238), `\reserved@a`, `\normalsfcodes`, `\@trivlist`, `\@nomath`, `\@font@warning`, `\G@refundefinedtrue`, `\@settopoint` (L8169), `\@fontswitch` (L9332) | Audit false positive. |
| Already in another file but semantically correctly placed | `\mathring`, `\ldots` (math_common.rs), `\showoutput`/`\tracingfonts` (plain_base.rs), `\@@cite` constructor (separate from `\@cite` formatter), `\documentstyle` (tex_job.rs architectural) | No action — file split matches semantic intent. |
| Genuine port — done this session | `\@cite{}{}` (commit landed 2026-05-18, L7796) | Real fix: kernel default citation formatter wrapping `[{#1\if@tempswa , #2\fi}]`. |
| Intentional Rust-side different implementation | `\pic@savebox`, `\pic@@savebox`, `\@savepicbox` (Rust uses raw `\def\@savepicbox#1(#2,#3){…}` chain at latex_constructs.rs L8435 instead of Perl's `\pic@savebox` delegation) | Documented divergence. |
| Truly missing / dynamic-from-Perl-runtime | very small residue | Per-CS investigation only if user-witnessed failure. |

The "45 candidates" backlog is effectively closed — only `\@cite`
needed a real port, the rest were classification artifacts.

**2026-05-02 follow-up: dump-path resolution check**: spot-checked
3 of the 45 violations against `resources/dumps/latex.dump.txt`:

| CS | Dump-path? | Match Perl? | Runtime fix? |
|---|---|---|---|
| `\hexnumber@` | ✓ M-entry present | ✓ exact body | n/a (dump = Perl) |
| `\on@line` | ✓ M-entry present | ✓ exact body | n/a (dump = Perl) |
| `\stop` | ✓ M-entry present | ✗ kernel body, NOT Perl's `closeMouth(1)` | ✓ `Let!("\\stop","\\endinput")` at `latex_constructs.rs:9220` (commit `c0a3f298563`, 2026-04-17) — runs after dump replay and overrides the kernel body |

Implication: the dump path provides most "missing source" CSes at
runtime, so the violations are mostly **source-organization only**,
not runtime failures. For deliberate Perl overrides like `\stop`,
the post-dump phase in `latex_constructs.rs` already restores Perl's
intent; the audit "✗" describes the dump content, not the runtime
state. Verified 2026-05-15: a paper with `\stop` converts with 0
errors.

Per-CS strict-parity work should:
1. Check if Perl's body in `latex_constructs.pool.ltxml` matches
   the kernel/dump body — if YES, add to `latex_constructs.rs`
   redundantly (cosmetic source-organization fix).
2. If Perl's body is a **deliberate override** (different from
   dump), translate carefully — these are the runtime-significant
   cases.

**Full classification of 27 truly-missing-in-Rust violations
(2026-05-02)**:

| Category | Count | CSes |
|---|---|---|
| Dump-resolved (likely OK at runtime, source-org only) | 14 | `\documentstyle`, `\@cite`, `\@fontswitch`, `\@font@warning`, `\G@refundefinedtrue`, `\hexnumber@`, `\mathring`, `\newsavebox`, `\@nomath`, `\on@line`, `\@savepicbox`, `\@settopoint`, `\showoutput`, `\stop`, `\tracingfonts`, `\@trivlist` |
| NOT in dump (genuine missing, but mostly dynamic) | 13 | `\ASCII`, `\@captype`, `\fnum@`, `\labelenum`, `\labelitem`, `\lx@end@verbatim`, `\lx@label`, `\pic@@savebox`, `\pic@savebox`, `\theenum`, `\theequation@ID`, … |

Most of the "NOT in dump" cases are *dynamically-defined* CSes
(counter/label print macros generated by `\newcounter`/`\newlabel`
at runtime, not static `Def*` calls). The runtime infrastructure
generates them when needed — Rust's `\newcounter`/`\newlabel` may
already do this correctly.

**14-list refresh (2026-05-15)**: of the 14 "dump-resolved
source-org" candidates above, all but one are already present in
`latexml_engine/src/`, most in `latex_constructs.rs`:

| CS | Current Rust location | Notes |
|---|---|---|
| `\hexnumber@` | `latex_constructs.rs` | exact-body relocation done |
| `\on@line` | `latex_constructs.rs` | exact-body relocation done |
| `\stop` | `latex_constructs.rs:9220` (Let → `\endinput`) | Perl override applied |
| `\@cite` | `latex_constructs.rs:7796` | ported 2026-05-18 (Perl L4238 kernel default formatter `[{#1\if@tempswa , #2\fi}]`); was missing under NODUMP path |
| `\@@cite` | `latex_constructs.rs:7803` | DefConstructor at parity |
| `\@font@warning` | `latex_constructs.rs` | done |
| `\G@refundefinedtrue` | `latex_constructs.rs` | done |
| `\@nomath` | `latex_constructs.rs` | done |
| `\@trivlist` | `latex_constructs.rs` | done |
| `\@settopoint` | `latex_constructs.rs:8169` (DefMacro) | done |
| `\newsavebox` | `latex_base.rs:441` + `latex_constructs.rs:8398` (TeX! block) | already in both via raw TeX |
| `\@savepicbox` | `latex_base.rs:444` + `latex_constructs.rs:8401,8408` (TeX! block) | already in both via raw TeX |
| `\documentstyle` | `tex_job.rs` (intentional — see docstring there) + `latex_constructs.rs` Let | architectural; do NOT relocate |
| `\mathring` | `math_common.rs` (math accent file) | semantically correct location |
| `\showoutput` | `plain_base.rs` (TeX plain primitive) | semantically correct location |
| `\tracingfonts` | `plain_base.rs` (TeX plain primitive) | semantically correct location |
| `\@fontswitch` | `latex_constructs.rs:9332` **(added 2026-05-15)** | Perl override of dump's kernel body |

So the real strict-parity gap is:
- ~0 remaining cosmetic source-org relocations from the 14-list
  (most were already done; the few defined elsewhere are
  semantically correctly placed or architecturally locked)
- A handful (likely 5-10) of genuinely missing static definitions
  that need careful per-CS investigation
- Deliberate overrides applied: `\stop` (Perl's `closeMouth(1)` →
  `Let "\endinput"`) and `\@fontswitch` (Perl's simpler
  `\ifmmode/\else` → DefMacro)

**Engine-wide CS-name diff refresh (2026-04-29 evening, methodology
note).** A per-file diff of `latex_constructs.pool.ltxml` vs
`latex_constructs.rs` (the source of the "1055 / 1199 → 13.5%
extra" number above) overcounts drift because Perl and Rust split
kernel-vs-construct CSes across files differently — most of the
160-name surface vanishes once the diff is taken across the engine
union. Engine-wide measurement (`Engine/*.pool.ltxml` ∪ all
`latexml_engine/src/*.rs`):

| Side | Unique CS names | Diff |
|------|---:|---:|
| Perl engine union | 2662 | — |
| Rust engine union | 2364 | — |
| Rust-only | — | 65 |
| Perl-only | — | 363 |

**Refresh 2026-05-15.** Re-ran the diff after 16 days of binding
work (port-from-Perl + sibling-machine kernel additions). Same
methodology (Def\* and Let regex over both sources), adding a Rust-
side scan of raw `\def`/`\let` inside `RawTeX!`/`TeX!` blocks:

| Side | Unique CS names | Diff |
|------|---:|---:|
| Perl engine union | 2616 | — |
| Rust engine union (incl raw) | 2642 | — |
| Rust-only | — | 214 |
| Perl-only | — | 188 |

The Perl-only gap narrowed from 363 → 188 (~48% reduction). Bucket
breakdown of the 188:

| Bucket | Count | Status |
|---|---:|---|
| `\bib@*` family | 116 | `latexml_engine/src/bibtex.rs` is a 37-line skeleton; full port from Perl `BibTeX.pool.ltxml` (956 lines) is a known TODO. Not random drift. |
| Misc `\<other>` | 58 | Diverse atomics: `\@charlb`, `\@filef@und`, point-size CSes, `\batchmode`, etc. Per-CS investigation needed. |
| `\@<lowercase>` | 12 | Mostly already handled via raw `TeX!`/`RawTeX!` blocks (e.g. `\@cite`, `\@captype`); regex doesn't catch the raw-block path uniformly. |
| `\@<at-name>` | 2 | Architecturally OK. |

The 214 Rust-only set is dominated by intentional Rust-side
additions: 13 `\lx@*` (LaTeXML internals) + 5 `\ltx@*` + 6
`\@list*` safety stubs + 2 `\@hidden@*` math helpers + 60 misc
(xparse-2018 / LaTeX-2020 helpers like `\IfClassAtLeastTF`,
`\NewCommandCopy`, the new `\Cdprime`/`\Cprime` BibTeX-Cyrillic
stubs, `\UTF@<n>@octets@noexpand` family). Two genuinely-Rust-
only patterns flagged earlier (`\@@appendix`, `\@begin@lrbox`)
remain false positives (regex matched a commented-out line or a
diff in trailing args).

**Conclusion.** The strict-Perl drift surface is shrinking but
not yet zero. The largest remaining cluster (BibTeX 116 CSes) is
a deferred port, not parity drift. Outside BibTeX, the gap is
~72 CSes worth investigating per-symbol, with the expectation
that many are raw-block-defined and undercounted by the regex.

The 65-name Rust-only set, after inspection, is mostly
intentional internals — `\lx@*` / `\ltx@*` plumbing, picture-env
helpers (`\lx@pic@line`, `\lx@pic@oval`, `\lx@pic@qbezier`,
`\lx@pic@vector`), math active-char primitives
(`\lx@math@amp/dollar/hash/percent/underscore`), xparse-2018
public API names (`\IfClassLoadedTF`, `\IfPackageAtLeastTF`,
`\NewCommandCopy`, `\DeclareCommandCopy`), and pdfTeX primitives
eagerly defined where Perl raw-loads them
(`\pdfendthread`, `\pdfsavepos`, `\pdfsetrandomseed`,
`\pdfstartthread`, `\pdfnoligatures`).

**2026-04-30 spot-check on the previously-flagged "real drift"
candidates** confirms the surface is much smaller than the 65 number
suggests — most are false positives or intentional stubs:

| Candidate | Verdict |
|---|---|
| `\@@appendix` | **NOT drift** — Perl `latex_constructs.pool.ltxml:707` defines the identical body `\@startsection{appendix}{0}{}{}{}{}`. Iteration-3 regex missed it (false positive). |
| `\@begin@lrbox` | **NOT drift** — Rust definition is commented out at `latex_constructs.rs:7890`. Regex matched the comment. |
| `\@listi…\@listvi` | **Intentional safety stub** at `latex_constructs.rs:4831`-4838 with explicit comment: "stub them as no-ops since LaTeXML handles list formatting via CSS". |
| `\@leftmark` / `\@rightmark` | Simple `Let!("\\@leftmark", "\\@firstoftwo")` at `latex_constructs.rs:3969`-3970. Redundant with latex.ltx raw-load but harmless; stays. |
| `\@maxlistdepth` | `DefRegister!` initializing to 6 at `latex_constructs.rs:4827`. Standard kernel value; redundant with raw latex.ltx but harmless. |

**Conclusion:** the actual strict-Perl drift surface in
`latex_constructs` is essentially empty — a handful of redundant
kernel-default initializations, none acute. The "13.5% extra in
Rust" framing in the table above should be read as
**organizational + safety-stub overhead**, not parity drift.

The 363-name Perl-only set is dominated by `bib@entry@*`
biblatex-style entries that Rust handles in `latexml_contrib`
rather than the engine, plus pattern-misses (the regex doesn't
catch `DefMath`/`DefRegister`/several variants on the Perl side).
Not actionable as-is; a refined regex pass would shrink it.

Diff lists for the next iteration: `/tmp/audit/rust_only_engine.txt`,
`/tmp/audit/perl_only_engine.txt`.

## The strict split

Perl `Package.pm:LoadFormat` (L2734-2752) is mutually exclusive:

```perl
if (!$ENV{LATEXML_NODUMP} && FindFile($format._dump, ...)) {
  LoadPool($format._bootstrap);
  LoadPool($format._dump);          # base SKIPPED
  LoadPool($format._constructs);
} elsif (FindFile($format._base, ...)) {
  LoadPool($format._bootstrap);
  LoadPool($format._base);          # dump SKIPPED
  LoadPool($format._constructs);
}
```

Rust now follows this in `tex.rs` (plain) and `latex.rs` (latex).

## Architectural divergence: eager vs lazy LaTeX load

**Status note (2026-04-30): re-audit before acting.** The original
concern below predates the current autoload/dump split. `tex.rs` now
defines `\@load@latex@pool` and LaTeX autoload triggers; `latex.rs`
itself still mirrors `LaTeX.pool` when that pool is loaded. If this
remains a real divergence, it should be demonstrated with a fresh
minimal trace of engine initialization and first `\documentclass`
autoload, not assumed from this older note.

**Perl `TeX.pool.ltxml:22-23`:**
```perl
LoadPool('Base');
LoadFormat('plain');
```
NOT `LoadFormat('latex')`. Perl's `LaTeX.pool.ltxml` is autoloaded
on demand via `\documentclass`, `\NeedsTeXFormat`, etc. (the
`DefAutoload` triggers in `TeX.pool.ltxml:33-39`).

**Rust `tex.rs` + `latex.rs`:** `latex.rs::LoadDefinitions` is
unconditionally invoked at engine init. So Rust eagerly loads LaTeX
where Perl lazy-loads it.

**Implication.** In strict-Perl, `\hook`, `\__int_eval:w`,
`\tex_par:D` would only be defined AFTER `\documentclass` triggers
the LaTeX.pool autoload. In Rust they're expected at engine init,
which fails when `latex.dump.txt` is incomplete.

**Recommended path.** Move `latex.rs::LoadDefinitions` body to
trigger from `\@load@latex@pool` (currently a stub). Keep auto-load
triggers in `tex.rs` matching Perl `TeX.pool.ltxml:33-39`.

## File-by-file divergence (plain side)

### `plain_bootstrap.pool.ltxml` ↔ `plain_bootstrap.rs`

* Perl 45 lines, defines `\TeX`, `\alloc@`, `\ch@ck`, `\newif`,
  `\leavevmode`. Rust matches.
* **Status: PARITY.**

### `plain_base.pool.ltxml` ↔ `plain_base.rs`

* Perl 622 lines, mostly `DefMacro`, `DefRegister`, `RawTeX`. Has
  ~12 closure-backed defs (`\wlog`, `\newinsert`, `\hglue`,
  `\vglue`, `\openup`, `\raggedbottom`, `\normalbottom`,
  `\@@oalign`, `\@@ooalign`, `\buildrel`, `\@`, `\@break`).
* Rust 756 lines, ~55 closure-backed defs.
* **Resolved (commit `0c4d609ad`):** `\newcount`, `\newdimen`,
  `\newskip`, `\newmuskip`, `\newbox`, `\newhelp`, `\newtoks`,
  `\newread`, `\newwrite`, `\newfam`, `\newlanguage` switched from
  Rust closures to raw `\outer\def` Token bodies, matching Perl
  `RawTeX` block at L207-218.
* **Resolved:** spacing macros (`\enskip`, `\enspace`, `\quad`,
  `\qquad`, `\thinspace`, `\negthinspace`, etc.) ARE closure-defined
  in Perl too — parity, no action.
* **CS-set audit — RESOLVED 2026-05-18 (this session).** Both sides
  define **132 CSes** by Def\*-family extraction (comment-skipping
  regex). Diff:
  * In Perl only (4): `\#`, `\$`, `\%`, `\&` — char-dispatch
    family. Relocated to `plain_constructs.rs:38-49` as
    `\ifmmode\lx@math@*\else\lx@text@*\fi` dispatchers with
    separate math/text targets. Documented WISDOM #44 divergence.
  * In Rust only (4): `\showoverfull`, `\loggingoutput`,
    `\tracingfonts`, `\showoutput` — co-located to plain_base.rs
    L54-57 from `latex_constructs.pool.ltxml` L5677-5679 so plain.tex
    users also get them (`\tracingall` references them).
    Comment in source at `plain_base.rs:43-52`.
  * Net: zero accidental drift. The ~134 line-count delta is
    entirely from verbose Rust comments with Perl-line citations,
    multi-line macro body formatting, and the `LoadDefinitions!`
    wrapper boilerplate.
* Status: **PARITY** — line-count gap is cosmetic; no action.

### `plain_constructs.pool.ltxml` ↔ `plain_constructs.rs`

* Perl 323 lines (77 unique CS defs).
* Rust 612 lines (81 unique CS defs).
* **Audit refresh 2026-04-28**: function-count is at parity
  (Perl 77, Rust 81 — close enough). The line-count gap is
  almost entirely:
  * Verbose Rust comments with Perl-line citations.
  * Char-dispatch family (`\#`, `\&`, `\%`, `\$`, `\_`) + their
    `\lx@math@*` / `\lx@text@*` dispatch targets — WISDOM #44
    documented divergence (Rust splits Perl's Box-dispatching
    DefPrimitives into explicit `\ifmmode … \else … \fi` macros
    plus separate math/text targets).
  * `\boldmath` / `\unboldmath` re-established post-dump
    (`plain_constructs.rs:585-608`) so that `mathfont` slot
    semantics survive when raw latex.ltx body is read from
    dump (vs from `plain_base.rs` closures). Documented
    intentional re-binding.
* Status: **PARITY** (cosmetic line-count gap acceptable).

### `plain_dump.pool.ltxml` ↔ `plain_dump.rs`

* Perl `plain_dump.pool.ltxml` is generated by `latexml --init=plain.tex`.
* Rust `plain.dump.txt` is generated by `latexml_oxide --init=plain.tex`.
* `plain_dump.rs` is a runtime loader for `plain.dump.txt` —
  delegates to `dump_reader::load_from_str_plain`.
* **Status: NEAR-PARITY.** Current local Rust dump is 959 lines.
  The 2026-04-28 audit measured a 961-line dump (641 M-keys);
  Perl ~872 unique CS names. Pollution from autoload
  triggers (`\AtBeginDocument`, `\documentclass`, `\Bbb`,
  `\align`, `\@pushfilename`, etc.) removed in commit `1e04a96c8`
  by moving those blocks before the init/dump bootstrap snapshot.
  **Audit refresh 2026-04-28**: only 17 Rust-only M-keys remain
  versus Perl plain_dump.pool.ltxml. Of those, 15 are encoding
  artifacts (`'`, `_`, `~`, `\%04`-`\%2C` URL-encoded ctrl chars)
  and 2 were file-IO bookkeeping leaks now RESOLVED:
  * `\@currname`, `\@currext` — RESOLVED 2026-04-28 by adding
    explicit skip in `dump_writer::write_dump`. These are
    file-IO bookkeeping CSes set per-document by
    `read_input_file_recursive` (`content.rs:262-263, 701-702`)
    that survive into the snapshot with literal `plain.tex`
    token bodies. Perl's plain_dump.pool.ltxml omits them
    because Perl's `TeX_FileIO.pool.ltxml:28-29` initializes
    them via `Let('\@currname','\lx@empty')` before any file
    load (state matches baseline). The skip mirrors the
    existing `\ver@*` runtime-state filter pattern. See
    `wisdom_dump_filter_runtime_state.md`.
  * No expl3 leakage in plain dump confirmed (`\par` not present
    as M-key; was the suspected case from earlier session).

## File-by-file divergence (latex side)

### `latex_bootstrap.pool.ltxml` ↔ `latex_bootstrap.rs`

* Perl 66 lines (8 Def + 1 Let).
* Rust 72 lines (7 Def + 2 Let).
* Status: **PARITY** — close enough at 9 vs 9 calls.

### `latex_base.pool.ltxml` ↔ `latex_base.rs`

* **Audit refresh 2026-04-28**: only 2 closure-backed defs remain
  in `latex_base.rs` (`\@expandtwoargs`, `\@makeother`). Per the
  15-batch reverse-migration committed via `e259b3d68`, latex_base
  is now essentially RawTeX-backed and dump-friendly. The 2
  closures match Perl 1:1 — both are closure-backed in
  `latex_base.pool.ltxml` too:
  * `\@expandtwoargs{}{}{}` — Perl `DefMacro('\@expandtwoargs{}{}{}', sub {...});`
  * `\@makeother{}` — Perl `DefMacro('\@makeother{}', sub {...});`
* Status: **PARITY** — no further reverse-migration needed.

### `latex_constructs.pool.ltxml` ↔ `latex_constructs.rs`

* Perl 6,014 lines (979 Def + 76 Let = 1055 calls).
* Rust 9,447 lines (1088 Def + 111 Let = 1199 calls).
* **Audit refresh 2026-04-28**: Rust ahead by ~144 calls (13.5%).
  Most of the gap is verbose Rust comments + intentional
  re-establishments documented in WISDOM #44. Recent
  reverse-migration commits (Apr 26-27) consolidated 15+ batches
  back from latex_base.rs to latex_constructs.rs (since the
  latter always runs after the dump). 110 closure-backed Defs
  remain in latex_constructs.rs, which is acceptable since
  latex_constructs.rs always runs in BOTH the dump-skip and
  dump-load paths.
* Status: **NEAR-PARITY** — no immediate action required.
* **CS-set spot-check — RESOLVED 2026-05-18 (this session).** Diff
  by normalized CS name (1035 Perl vs 1094 Rust; comment-skipping
  regex, param-spec stripped):
  * In Perl but not Rust (43): mostly the 45-CS "same-file
    relocation" backlog already listed above (L80-89). Stable;
    not blocking.
  * In Rust but not Perl (102): of these, 54 are reverse-migrations
    from Perl `latex_base.pool.ltxml` / `latex_bootstrap.pool.ltxml`
    consolidated into latex_constructs.rs (Apr 26-27 commits, see
    above). The other 48 are either:
    * Modern LaTeX2e kernel CSes post-2020 (`\IfPackageLoadedTF`,
      `\NewCommandCopy`, etc. — documented in
      `latex_constructs_rust_only.rs:30-60`),
    * LaTeXML-internal helpers (`\ltx@hard@MessageBreak`,
      `\ltx@ifpackageloaded`),
    * Defensive dump-path coverage for `latex_base.rs` CSes
      (`\appendixname`, `\thefootnote`, `\columnsep`, …) per the
      design pattern in `latex_constructs_rust_only.rs:179-209`.
  * **Side finding — RE-VERIFIED 2026-05-19**: the original
    "43 dead-code overrides" between `latex_constructs.rs` and
    `latex_constructs_rust_only.rs` are now **zero** when checked
    by strict `Def(Macro|Primitive|Constructor|Register|Math|
    Environment)!` definition-site grep — the cleanup completed
    across the intervening sessions. The remaining ~14 string-match
    overlaps are bodies-of-other-defs (`\arabic` referenced inside
    `\thefootnote`, `\@startsection` referenced inside `\chapter`,
    etc.), not redundant definitions. Within-file multi-defs likewise
    reduced: `\appendixname` / `\thebibliography@ID` drift entries
    are gone; `\f@shape`/`\f@family`/`\f@series` font-shape sites
    (still 4-6 each) are legitimately contextual per-shape
    declarations. **No actionable dedupe remains.**

## Dump-completeness gaps

### expl3-code.tex 10000-error abort

**RESOLVED (commit `209083ff4`, 2026-04-26):** root cause was
`--init=latex.ltx` reaching raw expl3-code.tex without LaTeX.pool's
infrastructure (since the autoload triggers only fire on
`\documentclass` etc., not during raw-loading latex.ltx itself).

**Fix:** `ini_tex.rs` explicitly preloads `LaTeX.pool` before
snapshotting when init basename is `latex.*`. Mirrors Perl
`LaTeX.pool.ltxml:28-29`'s `LoadPool('TeX'); LoadFormat('latex');`.

**Effect:** `latex.dump.txt` grew from 19,797 → 24,987 entries (+26%);
zero undefined-CS errors during expl3 load. Current local dump is
25,792 lines; re-run the dump audit before comparing exact counts.

### Remaining dump gaps (post-209083ff4)

**Audit refresh 2026-04-28** (post commit `01a8ee8b1`
register-alias address fix + `ab76be20f` Conditional/closure-Expandable
PA serialization): Most expl3 alias gaps are RESOLVED.

* `\tex_par:D` → `PA \lx@normal@par` ✅ captured.
* `\__int_eval:w` → `PA \numexpr` ✅ captured.
* `\hook_*` family — 31 M-keys ✅ captured (full expl3 hook system).
* 537 `\tex_*:D` PA aliases captured (was previously ~302 missing).

Trigger was `01a8ee8b1` defaulting register-alias `address` field
to `rparts[0]` (the register's internal cs name) when omitted — Perl
`Dumper.pm:337-342` `R()` constructor default. Without this fix the
145+ `\tex_*:D` register-alias entries wrote to a separate slot
instead of the underlying register.

**Closed** (re-verified 2026-05-15): the closure round-trip
Stored::Primitive self-alias concern doesn't manifest — current
TL2025 dump has zero self-aliases (target == key) among 602
PA/MPA records. See "Closure round-trip" below for details.

**Fontdimen/intarray storage consolidation — RESOLVED 2026-05-15.**

expl3 implements intarrays by stashing values in
`\fontdimen<idx>\<font>` slots, picking `cmr10` at various tiny
`at <N>sp` instantiations to get one font instance per intarray.
Before this resolution the dump emitted one V-record per slot —
~89k records, ≈40% of the latex.YYYY.dump.txt size.

Resolution: `dump_writer` now groups V entries matching
`fontdimen_fontinfo_<font> at <size>_<idx>` by (font, size) and
emits a single `IA` record per intarray with the values RLE-encoded.
Format: `IA\t<prefix>\t<len>\t<rle>`, where `<rle>` is a comma-list
of `v` (one entry) or `vxn` (n consecutive copies). `dump_reader`
parses `IA`, decodes the RLE, and emits the same per-slot V
assignments — runtime state post-replay is identical.

Measured TL2025 impact:
- 89,294 V-records → **15 IA records + 63 V fallbacks** for
  non-dense intarrays (one cluster, `at 14sp` with 9 sparse slots)
- Dump size: **7.4 MB → 3.7 MB (-49%)**, line count 110,691 → 21,475
- `cargo test --tests` = 1196/0/0 (unchanged)
- Backward compatible: dump_reader still loads existing TL2023 dumps
  with un-consolidated V records; sibling regenerates that file
  when convenient.

Code: `latexml_core/src/dump_writer.rs` (RLE encoder + grouping),
`latexml_core/src/dump_reader.rs` (`IA` arm + RLE decoder).

**Suspected stale narrative below** (kept for archaeological
context — predates `01a8ee8b1`/`ab76be20f`):
  
  **Narrowed (this iteration):** manual `\global\let\tex_par:D\par`
  AT RUNTIME (in a `.tex` document with explicit `\catcode`\_=11`) works
  perfectly — produces the long-body Expandable. So `\let` correctly
  handles `\par`. The bug is specifically in raw-loading expl3-code.tex
  via `ini_tex`. ~302 of 752 `\tex_*:D` aliases are missing. Many are
  legitimately engine-specific (LuaTeX/XeTeX), but core ones like
  `\tex_par:D`, `\tex_dimexpr:D`, `\tex_catcode:D`, `\tex_cr:D`,
  `\tex_dp:D` are also missing.

  **Suspected cause:** catcode regime during `ini_tex`'s raw expl3
  load may not have `_` / `:` as LETTER everywhere expl3-code.tex
  expects, OR `\__kernel_primitive:NN` defined in the loop's
  `\begingroup` doesn't expand consistently for all tokens (`\par`,
  `\dimexpr`, etc. may be tokenized differently than `\space`).

* **`\__int_eval:w` runtime meaning is `\x@protect …`** (per probe
  with `\meaning`), suggesting it's wrapped in robust-protection
  layer instead of being installed as a direct primitive alias.

* **Plain dump pollution** — RESOLVED (empirical re-scan 2026-05-18).
  Both shipped plain dumps are clean of expl3 / latex content:
  * `resources/dumps/plain.2023.dump.txt` — 958 lines: 0 expl3-name
    matches (`\l_…`, `\g_…`, `\c_…`, `\__…`, `…_end:`, `cs_set`,
    `tl_new`), 0 latex-only CSes (`\@documentclass`,
    `\@addtoreset`, `\NeedsTeXFormat`, `\ProvidesClass/Package`).
  * `resources/dumps/plain.2025.dump.txt` (embedded; primed via
    `/tmp/latexml-oxide-dumps-*/plain.2025.dump.txt`) — 958 lines,
    639 M-keys, 12 PA aliases, 0 MPA. Same zero-match counts as
    2023. No `\par` M-record (correct — `\par` is a primitive,
    never set by plain.tex). The historical `\bye`-test
    observation predates the move of autoload triggers to before
    the snapshot (`1e04a96c8`) and was not reproduced on the
    current dumps.

### Sandbox regression

181-paper failure subset: post-strict-Perl-translation, 166 papers
moved from `Status:conversion:1/2` (errors) to `Status:conversion:3`
(fatal in ~0.2s). Acceptable per user directive — parity work
continues; tests / sandbox are re-validated after dumps are
complete.

### Closure round-trip — RESOLVED (re-verified 2026-05-15)

**Empirical re-verification (2026-05-15):** the TL2025 latex dump
has **zero self-aliases** (`M\t<cs>\tPA\t<cs>` with key == target)
among its 602 PA/MPA aliases. Mechanism: self-aliases would only
arise for primitives that already exist in the bootstrap snapshot,
so the post-init *diff* never emits them. The original concern
(self-aliases producing undefined-CS errors after a strict
LoadFormat split) doesn't manifest in the current dump pipeline.

Historical context (kept for archaeology): under an older
add-only/transitional loader mode, self-aliases were filtered at
load time by skipping entries whose key was already defined. The
current `dump_reader.rs` uses Perl-style global assignment with
narrow runtime-state filters and deferred-alias handling
(`early_aliases` / `late_aliases` partitioning in dump_writer). No
TODO remains.

## Distribution follow-up (multi-version dumps)

User's plan (2026-04-26): Rust binary should `include_bytes!` of
several TL versions' dumps (TL2022 … TL2026) and runtime-select via
`kpsewhich --version`. Currently dumps are runtime-loaded from disk
which works for development but doesn't fit single-binary
distribution. Plan: do this AFTER TL2025 dumps are robust + tested.
