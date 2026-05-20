# Perl LoadFormat Parity Audit

Tracks the Rust→Perl translation gap exposed by the strict
`LoadFormat` split (commit `0c4d609ad`). Each entry maps a Perl
`Engine/<file>.pool.ltxml` definition to its Rust counterpart in
`latexml_engine/src/<file>.rs` (extracted from `latexml_package`
in commit `9909ba51d`), flagging divergences that break the
strict-Perl pipeline.

**Refresh status (2026-05-20):** current local verification:
`cargo test --tests` is **1328/0/0**;
`cargo clippy --workspace --all-targets` is **14 warnings (all in
`latexml_math_parser` from the in-flight ASF migration — collaborator's
lane)**. Dump resources on disk are versioned per TL year:
`plain.YYYY.dump.txt` ~958 lines, `latex.YYYY.dump.txt` ~21,475 lines
after the IA intarray consolidation (commit `81176ba689`) — was
~110,713 lines before. Both TL2023 and TL2025 dumps are committed
and `include_str!`-embedded into the binary.

**Zero-error target verification (2026-04-30 iter 40):** ran
`latexml_oxide --init=plain.tex empty.tex` and
`latexml_oxide --init=latex.ltx empty.tex` against the current HEAD
binary. Both emit **0 errors**. Freshly-generated dump output
matches on-disk dumps line-for-line (plain: 0 diff lines; latex: 1
line diff — only the `texsys.aux_contents` build timestamp). Target
#4 from CLAUDE.md is met.

**Audit table (2026-05-02 methodology — uniform `Def*`+`Let` regex
across both sides):**

| File | Perl | Rust | Δ | Status |
|------|-----:|-----:|---:|--------|
| `plain_bootstrap` | 5 | 5 | 0 | ✅ PARITY |
| `plain_base` | 129 | 127 | -2 | ✅ PARITY |
| `plain_constructs` | 62 | 79 | +17 | ✅ NEAR-PARITY (cosmetic — Rust split for clarity) |
| `latex_bootstrap` | 8 | 7 | -1 | ✅ PARITY |
| `latex_base` | 138 | 127 | -11 | ✅ NEAR-PARITY |
| `latex_constructs` | 1071 | 1097 | +26 | ✅ NEAR-PARITY (<3% drift) |

Per-file gap is **closed for action** — the residual differences
are cosmetic source organization (latex_constructs Rust-only set
is mostly intentional internals: `\lx@*` / `\ltx@*` plumbing,
picture-env helpers, math active-char primitives, xparse-2018
public APIs, pdfTeX primitives). Zero-error inits hold for both
`--init=plain.tex` and `--init=latex.ltx`.

**Engine-wide CS-name diff (most recent refresh 2026-05-15)**:

| Side | Unique CS names | Diff |
|------|---:|---:|
| Perl engine union | 2616 | — |
| Rust engine union (incl raw blocks) | 2642 | — |
| Rust-only | — | 214 (mostly intentional internals) |
| Perl-only | — | 188 |

Bucket breakdown of the 188 Perl-only:

| Bucket | Count | Status |
|---|---:|---|
| `\bib@*` family | 116 | `latexml_engine/src/bibtex.rs` is a 37-line skeleton; full port from `BibTeX.pool.ltxml` (956 lines) is a known TODO. |
| Misc atomics | 58 | `\@charlb`, point-size CSes, `\batchmode`, etc. Per-CS investigation if user-witnessed. |
| `\@<lowercase>` | 12 | Mostly handled via raw `TeX!`/`RawTeX!` blocks; regex undercounts. |
| `\@<at-name>` | 2 | Architecturally OK. |

Outside BibTeX (deferred port), the gap is ~72 CSes worth per-CS
investigation. Diff lists for next iteration in `/tmp/audit/`.

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

**Plain dump pollution** — RESOLVED (empirical re-scan 2026-05-18).
Both shipped plain dumps are clean of expl3 / latex content
(0 `\l_…`/`\g_…`/`\c_…`/`\__…`/`…_end:`/`cs_set`/`tl_new`/
`\@documentclass`/`\@addtoreset`/`\NeedsTeXFormat`/`\ProvidesClass`).
The historical `\bye`-test observation predates the move of
autoload triggers to before the snapshot (`1e04a96c8`).

### Closure round-trip — RESOLVED (re-verified 2026-05-15)

TL2025 latex dump has **zero self-aliases** (`M\t<cs>\tPA\t<cs>`
with key == target) among its 602 PA/MPA aliases. Mechanism:
self-aliases would only arise for primitives that already exist
in the bootstrap snapshot, so the post-init *diff* never emits
them. No TODO remains.

## Distribution follow-up (multi-version dumps)

**LANDED 2026-05-15.** Per-TL-year dumps
(`resources/dumps/{plain,latex}.YYYY.dump.txt` +
`texlive.YYYY.version`) committed and embedded into the binary at
build time via `include_str!`. Runtime resolves the ambient year
via `kpsewhich -var-value=SELFAUTOPARENT` with `pdflatex --version`
fallback (`kpsewhich --version` returns the same kpathsea library
string across TL releases — NOT a reliable discriminator). TL2023
+ TL2025 bundled currently; add years via
`tools/make_formats.sh`. Follow-up IA consolidation
(`81176ba689`, 2026-05-15) halved `latex.YYYY.dump.txt` by
collapsing per-slot fontdimen V-records into per-(font,size) `IA`
records with RLE-encoded data. See `CLAUDE.md` and
`docs/SYNC_STATUS.md` for the canonical record.
