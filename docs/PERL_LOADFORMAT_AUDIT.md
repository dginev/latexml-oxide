# Perl LoadFormat Parity Audit

Tracks the Rust→Perl translation gap exposed by the strict
`LoadFormat` split (commit `0c4d609ad`). Each entry maps a Perl
`Engine/<file>.pool.ltxml` definition to its Rust counterpart in
`latexml_engine/src/<file>.rs` (extracted from `latexml_package`
in commit `9909ba51d`), flagging divergences that break the
strict-Perl pipeline.

**Refresh status (2026-04-30):** current local verification:
`cargo test --tests` is **1109/0/0**; dump resources present locally
are `plain.dump.txt` at 959 lines and `latex.dump.txt` at 25,792
lines. The detailed call-count table below is from the 2026-04-28
audit and should be re-run before using exact counts as acceptance
criteria.

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

Active: spot-check `latex_constructs` extras for accidental drift vs
intentional WISDOM #44 divergences, and re-run the count audit after
the Apr 29-30 package/class fixes.

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

The 65-name Rust-only set, after inspection, is mostly
intentional internals — `\lx@*` / `\ltx@*` plumbing, picture-env
helpers (`\lx@pic@line`, `\lx@pic@oval`, `\lx@pic@qbezier`,
`\lx@pic@vector`), math active-char primitives
(`\lx@math@amp/dollar/hash/percent/underscore`), xparse-2018
public API names (`\IfClassLoadedTF`, `\IfPackageAtLeastTF`,
`\NewCommandCopy`, `\DeclareCommandCopy`), and pdfTeX primitives
eagerly defined where Perl raw-loads them
(`\pdfendthread`, `\pdfsavepos`, `\pdfsetrandomseed`,
`\pdfstartthread`, `\pdfnoligatures`). Real drift candidates are
small and mostly LaTeX kernel internals
(`\@@appendix`, `\@begin@lrbox`, `\@listi…\@listvi`,
`\@maxlistdepth`, `\@leftmark`, `\@rightmark`); these need
case-by-case audit against `latex.ltx` raw load to confirm
whether they should be deleted or renamed.

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
* Rust 786 lines, ~55 closure-backed defs.
* **Resolved (commit `0c4d609ad`):** `\newcount`, `\newdimen`,
  `\newskip`, `\newmuskip`, `\newbox`, `\newhelp`, `\newtoks`,
  `\newread`, `\newwrite`, `\newfam`, `\newlanguage` switched from
  Rust closures to raw `\outer\def` Token bodies, matching Perl
  `RawTeX` block at L207-218.
* **Open:** spacing macros (`\enskip`, `\enspace`, `\quad`,
  `\qquad`, `\thinspace`, `\negthinspace`, etc.) ARE closure-defined
  in Perl too — parity, no action.
* **Open:** Rust has additional CSes not in Perl plain_base
  (~150 lines extra). Need line-by-line audit to determine if
  they belong here or in plain_constructs.

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
* TODO: spot-check whether any of the 144 extras are
  documented intentional divergences vs accidental drift.

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

**Open**: see "Closure round-trip" (line 197+) — Stored::Primitive
self-aliases (where the dump entry's CS equals the primitive's
.cs) are no-op at load time and need a `dump_writer` marker that
`dump_reader` recognizes as "engine has primary; nothing to install".

**New (audit 2026-04-28): fontdimen/intarray storage divergence.**
Rust dumps 3094 V-records of form `V \fontdimen<N>\<font-cs>`
(one per slot per font), where Perl dumps 0 such records and
encodes the same data inside `fontinfo_<name>_at_<size>pt`
V-records (one record per font with embedded `data` array).
Affected fonts: `\c__fp_exp_intarray` (216 slots),
`\c__fp_trig_intarray` (1264), `\c_initex_cctab`/`_other`/`_str`/
`_document` (257 each), `\g_tmpa_cctab`/`_tmpb` (257 each),
`\g__regex_*_intarray` (9 each). Total 3094 records bloating the
dump (~80KB). Same runtime semantics but different storage layout.
Project doc:
[`memory/project_fontdimen_intarray_storage.md`]
— deferred until tests fail or dump becomes a bottleneck.

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

* **Plain dump pollution**: running `\bye` plain-TeX test shows
  `\par` defined as `\para_end:`-style expl3 chain, which means
  plain.dump.txt OR plain_dump.rs is loading latex content. Need
  to verify plain.dump.txt is clean (just plain.tex bindings).

### Sandbox regression

181-paper failure subset: post-strict-Perl-translation, 166 papers
moved from `Status:conversion:1/2` (errors) to `Status:conversion:3`
(fatal in ~0.2s). Acceptable per user directive — parity work
continues; tests / sandbox are re-validated after dumps are
complete.

### Closure round-trip

**Historical note pending re-audit.** The paragraph below was written
for an older add-only/transitional loader mode. `dump_reader.rs` now
routes meaning/value entries through Perl-style global assignment
semantics, with narrow runtime-state filters and deferred alias
handling. Re-test any self-alias failure before reviving this TODO.

Rust closure bodies (`Stored::Primitive`, `Stored::Conditional`)
serialize as `PA\t<target_cs>`. If the entry's CS equals the
primitive's `.cs`, it's a self-alias and effectively a no-op at
load time. Workaround for now is the `add-only` policy at engine
init: `_base.rs` runs first, then dump entries that would self-alias
get rejected. With strict split, `_base` doesn't run, so these
self-aliases produce undefined-CS errors at runtime.

**TODO:** dump_writer should emit self-aliases as a marker that
dump_reader recognizes as "engine has primary; nothing to install".

## Distribution follow-up (multi-version dumps)

User's plan (2026-04-26): Rust binary should `include_bytes!` of
several TL versions' dumps (TL2022 … TL2026) and runtime-select via
`kpsewhich --version`. Currently dumps are runtime-loaded from disk
which works for development but doesn't fit single-binary
distribution. Plan: do this AFTER TL2025 dumps are robust + tested.
