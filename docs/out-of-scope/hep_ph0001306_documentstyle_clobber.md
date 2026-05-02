# Out of scope (moved from SYNC_STATUS.md 2026-05-01)

Empirically verified: Perl LaTeXML on TL2025 with --preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings does NOT produce 0 errors on this paper, so it fails the in-scope predicate ("in scope iff Perl produces 0 errors").

Original SYNC_STATUS.md task content preserved below for future reference.

### 1.7 hep-ph0001306 — `\def`s BEFORE `\documentstyle` clobbered by class load

**Root cause isolated 2026-04-30 via 4-test bisection matrix:**

| Setup | Outcome |
|-------|---------|
| `\documentstyle\nl\def\d{\delta}\nl…\$\d\$\$O_{3L}\$` | 0 errors ✓ |
| `\documentclass\nl\def\d{\delta}\nl…\$\d\$\$O_{3L}\$` | 0 errors ✓ |
| `\def\d{\delta}\nl\documentstyle\nl…\$\d\$\$O_{3L}\$` | **2 errors** ✗ |
| `\gdef\d{\delta}\nl\documentstyle\nl…` | 0 errors ✓ (forces global) |
| `\def\d{REDEF}\nl\documentstyle\nl…` (text body) | 0 errors ✓ |

**Conclusion:** When `\def\d{...}` precedes `\documentstyle`, Rust's
class-load path re-installs the kernel `\d{}` macro (a 1-arg accent
form: `\ifmmode\@math@daccent{#1}\else\@text@daccent{#1}\fi`),
clobbering the user's earlier 0-arg redef. Subsequent `$\d$` then
hits the kernel `\d{}` which consumes the closing `$` as `#1`,
corrupting math state and cascading 90 `_` + 36 `^` script errors.

In Perl LaTeXML, the same load order works (paper is in no-problem
canvas), so Perl either preserves prior user `\def`s during kernel
reload or doesn't reload `\d`. Rust's class-load needs to either
(a) skip kernel `\d`/`\b`/etc. if user has redefined, or (b) load
plain_constructs' `\d{}`/`\b{}` once at engine bootstrap and never
re-install. **Code locus:** `tex_job.rs:146` `input_definitions("LaTeX", pool=>pool)`
re-runs the LaTeX pool's LoadDefinitions! block at every `\documentstyle`
invocation, transitively re-running `plain_constructs.rs:86`'s
`DefMacro!("\\d{}", …)` and clobbering the user's earlier `\def\d{...}`.
`\documentclass{article}` does NOT take this code path (probably
preloads via a different chain).

**Attempts (both reverted, both correctly so):**
1. State-flag guard at `tex_job.rs:146` — didn't help (flag never set).
2. Pre-mark `LaTeX.pool_loaded=true` BEFORE input_definitions call —
   2 errors → 1, but **also broke `\compat@loadpackages` (undefined)**
   and **cortex_worker core-dumped** on hep-ph0001306. So the LaTeX
   pool's LoadDefinitions! block IS the first/only load — Rust does
   NOT preload plain_constructs at engine bootstrap. **Corrected
   diagnosis:** the chain is `\documentstyle{article}` →
   `input_definitions("LaTeX",pool)` → first-time install of `\d{}`
   etc. The user's `\def\d{...}` ran BEFORE this — i.e. before `\d`
   was even defined — so the override doesn't exist when the kernel
   later installs `\d{}`. Perl LaTeXML preloads its kernel pool at
   engine bootstrap (before reading the user TeX file), so user
   `\def\d{...}` cleanly overrides existing kernel `\d`.

**Real fix path:** engine bootstrap must load plain_constructs +
latex_constructs + base_constructs at startup time, BEFORE the user
TeX file is read. Then user `\def\d{...}` (anywhere) overrides as
expected. Currently these pools are lazy-loaded via `\documentstyle`'s
`input_definitions` call. This is a substantial architectural change
to engine init; deferred. Tests still 1110/0/0.

**Smoking-gun confirmation (2026-05-01):** `\def\d{MARKER}` before
`\documentstyle [12pt]{article}` followed by `\d end` produces
`test ẹnd` — the kernel 1-arg accent fires on `e` (ẹ = e with dot
below), NOT `test MARKERend`. So the user's redef IS clobbered.

`InnerPool!(plain_constructs)` IS idempotent (`setup_binding_language.rs:60`,
checks `plain_constructs.pool_loaded` flag, skips reload). So
plain_constructs's `DefMacro!("\\d{}", ...)` does NOT re-run during
`\documentstyle`. Yet the kernel `\d` is restored.

**ROOT CAUSE LOCATED (2026-05-01):** `latex_constructs.rs:2379-2401`
intentionally CLEARS `plain_constructs.pool_loaded` and re-runs
`InnerPool!(plain_constructs)` (and `math_common`) with
`state_unlocked(true)`. Per the in-code comment, this is needed for
locked math CSes (`\prime`, `\active@math@prime`) to be properly
re-installed during the dump-load path — without it, the
`\mathchardef\prime="0230` from the dump beats the engine's
`\prime` redef and renders as digit `0`.

Stack trace (instrumented eprintln):
```
plain_constructs::load_definitions (install #2)
  ← latex_constructs::load_definitions (setup_binding_language.rs:69 = InnerPool!)
  ← latex::load_definitions (setup_binding_language.rs:69 = InnerPool!)
  ← input_definitions("LaTeX")
  ← tex_job.rs:146 (\documentstyle handler)
```

**Fix paths:**
* (a) Save+restore user-redefined CSes around the locked-CS reload.
  Snapshot `\d`/`\b` etc. before clearing pool_loaded; if user has
  overridden, restore after.
* (b) Refactor the locked-CS reload to be SURGICAL — only re-install
  the specific math CSes that need re-locking (`\prime`,
  `\active@math@prime`, etc.), not the whole `plain_constructs` pool.
* (c) Move the locked-CS reload to engine bootstrap so it runs ONCE
  and is finished before user TeX is read.

**Attempted fix #1 (2026-05-01, reverted):** Added `if !IsDefined!`
guard around `\d{}` and `\b{}` install in `plain_constructs.rs:86-93`.
Result on min-repro: 2 errors → 0 errors ✓. Result on
hep-ph0001306 full paper: **150 errors → 0 errors** (just 100
warnings) ✓✓. **BUT** broke `accents_test` (`\r` accent now
produces extra `c̊c̊c̊c̊` content). Reverted.

The accents_test regression suggests the second-pass install is
load-bearing for downstream state — skipping it leaves the engine in
a different state than expected. Specifically the test exercises `\r`
accent which doesn't have IsDefined guard, so it gets re-installed —
but possibly some interaction between `\d{}`/`\b{}` skip and other
plain_constructs CSes (like `\@math@daccent` constructor) breaks.

**Next attempt:** approach (a) — snapshot `\d`/`\b` before
`latex_constructs.rs:2379` clears the pool flag, restore AFTER the
reload IF user has redefined. This way the reload still runs (so
downstream state stays consistent) but the user's override is
preserved. More surgical than the IsDefined! guard. Tests still
1110/0/0; deferred.

**Attempted fix #2 (2026-05-01, reverted):** Snapshot+restore using
`state::lookup_meaning` / `state::assign_meaning` around the reload at
`latex_constructs.rs:2398-2401`. Build clean, B test 0 errors,
hep-ph0001306 0 errors, **same accents_test regression** (`\r` ring
accent now produces extra `c̊c̊c̊c̊` content). Reverted.

The persistent accents_test regression — across BOTH the IsDefined!
guard AND the snapshot+restore — proves that `\d{}`/`\b{}`'s second-
pass install carries side effects beyond just the meaning slot. The
two installs are NOT idempotent at some downstream layer. Possibly:
(i) the meaning's identity (Rc pointer) matters for `\@math@daccent`
construction; (ii) there's a property table tied to install order;
(iii) accents_test exercises caching or memoization that's
install-instance-specific.

**Path forward:** option (b) — surgical refactor. Don't blanket-reload
plain_constructs; instead, identify the specific locked CSes that
need re-locking (`\prime`, `\active@math@prime`, etc. — comment lists
two) and re-install just those with `state_unlocked(true)`. This
avoids touching `\d`/`\b` entirely. Requires identifying the full
locked-CS list and extracting their definitions out of plain_constructs
into a separately callable function. Substantial refactor; deferred.

**Attempted fix #3 (2026-05-01, reverted):** Skip `InnerPool!(plain_constructs)`
entirely; keep only `InnerPool!(math_common)` reload (which has
`\prime`). B test 0 errors ✓, hep-ph0001306 0 errors ✓ (same as
attempts #1 and #2), but **`mathtokens_test` regression** (different
from accents_test). So plain_constructs reload IS needed for some
math state — can't just skip it. Reverted.

Confirmed: Perl `latex_constructs.pool.ltxml:19-21` has the comment
`"Won't reload ?????"` next to `LoadPool('plain_constructs')`,
suggesting Perl's author was uncertain whether reload happens. So
Perl LaTeXML may be running an idempotent no-op here, NOT reloading.
This explains why Perl handles hep-ph0001306 cleanly — Perl never
clobbers the user `\def\d`. The difference is Rust's `InnerPool!`
+ explicit pool_loaded clear DOES reload, while Perl's `LoadPool`
may detect already-loaded and skip.

**Final fix path (engine-level):** make Rust's pool reload truly
idempotent at the symbol-table level (skip CSes that have user
definitions on top of the kernel default), OR identify the precise
subset of plain_constructs CSes that need re-locking and refactor
them into a re-runnable subset. Both are substantial engine changes.

**Attempted fix #4 (2026-05-01, reverted):** Snapshot+restore extended
to include `\@math@daccent`/`\@math@baccent` constructors in addition
to `\d`/`\b` macros (theory: Rc identity matters since `\d` body
references the constructor). Build clean, B test 0 errors,
**accents_test still fails identically**. So Rc-identity-of-constructor
isn't the issue either. The downstream state coupled to the second
plain_constructs install is deeper than the four CSes I tried to
preserve. Reverted.

**Summary of 4 attempted fixes for Task 1.7:**
| # | Strategy | hep-ph0001306 | Regression |
|---|----------|----------------|------------|
| 1 | `if !IsDefined!` guard on `\d`/`\b` install | 150→0 ✓ | accents_test |
| 2 | snapshot+restore `\d`/`\b` meaning | 150→0 ✓ | accents_test |
| 3 | skip `InnerPool!(plain_constructs)` reload | 150→0 ✓ | mathtokens_test |
| 4 | snapshot+restore `\d`/`\b`/`\@math@daccent`/`\@math@baccent` | 150→0 ✓ | accents_test |

**Common pattern:** every approach that preserves user `\def\d` value
across the reload also breaks at least one regression test. The
plain_constructs second-pass install is genuinely load-bearing for
state beyond the four accent CSes we've identified. **Conclusion:**
fix requires either (1) a comprehensive snapshot of all CSes touched
by plain_constructs (~hundreds of CSes), or (2) refactoring
plain_constructs to split the locked-CS portion (math symbols) from
the user-overrideable portion (accent macros). Neither fits in a
short iteration. Deferred for a focused debugging session.

**Path forward synthesis (2026-05-01) — clobber path verified:**
Bootstrap order from `tex.rs:233-251` and `latex.rs:65-91` is
`bootstrap → dump → constructs`. So:
1. `plain_bootstrap` → `plain_dump` (installs `\prime` mathchar 560)
   → `plain_constructs` (locks `\prime` = U+2032 ′, beating dump)
2. `latex_bootstrap` → `latex_dump` (re-installs `\prime` mathchar 560
   via inherited plain_dump records, **clobbering the lock**) →
   `latex_constructs` (sees `\prime` clobbered, force-reloads
   plain_constructs to restore lock — this is the existing
   `latex_constructs.rs:2379-2401` workaround which also clobbers
   user `\def\d` etc.)

Per `dump_reader.rs:571-576`, dump_reader intentionally uses direct
table mutation bypassing the `:locked` gate (Perl-faithful — Perl's
`Core/Dumper.pm:assign_internal('global')` does the same). Per
`resources/dumps/plain.dump.txt:552`, `\prime` record reads
`M  \prime  R  \prime  CD  560  8242  Register` — both mathchar 560
AND U+2032 (8242) are stored.

**Surgical-fix candidates (prioritized):**
* (i) Make `latex_dump` SKIP records for CSes already present in
  the running symbol table — i.e. don't have latex_dump replay
  inheritance from plain_dump that plain_constructs already
  superseded. Concretely: when latex_dump_reader sees `\prime`
  already defined (from plain_constructs), skip the record.
  Eliminates the clobber → eliminates the workaround. Risk: the
  add-only behaviour changes semantics for legitimate latex-
  specific overrides.
* (ii) Identify the exact set of CSes that `plain_constructs` locks
  and that `plain_dump` records as mathchar — the union is small
  (probably <50 CSes including `\prime`, `\active@math@prime`,
  the math symbols). Filter latex_dump to skip those records.
  More targeted than (i).
* (iii) Tag locked-CS records in the dump with a `locked-source`
  flag so dump_reader can prefer the existing locked def rather
  than overwriting. Requires dump format extension.

(i) is the smallest change and most Perl-faithful (Perl's
`AssignValue('plain_constructs.pool.ltxml_loaded' => undef)` only
clears one of four flags, so `LoadPool('plain_constructs')` is a
no-op — meaning Perl's `\prime` after latex bootstrap is whatever
plain_constructs left it, NOT the dump's mathchar. The reason is
Perl's flag mechanism, but the OBSERVABLE result is the same:
latex_dump shouldn't clobber). Tagged for next focused session.

**Concrete dump-content bug (2026-05-01):** `latex.dump.txt:21951`
contains `M\t\prime\tR\t\prime\tCD\t560\t48\tRegister` — mathglyph
is **48** (digit '0'), NOT **8242** (U+2032 ′) as in
`plain.dump.txt:552`. So latex.dump.txt itself is corrupt for
`\prime`: it serializes a Register with the wrong mathglyph.

**Bug is widespread (2026-05-01):** `diff plain.dump.txt latex.dump.txt`
on CharDef Register entries shows ~85 corrupted symbols — `\alpha`
(945→11), `\aleph` (8501→64), `\amalg` (8720→113), `\approx`
(8776→25), `\ast` (8727→3), `\asymp` (8781→16), `\beta` (946→12),
… The pattern is `latex_mathglyph = value & 0xFF` (the low byte of
the mathchar). E.g. `\amalg` value=8817, low-byte=113 ✓; `\aleph`
value=576, low-byte=64 ✓. So the corruption is the cmsy-family
font-encoding map returning the slot byte as the literal codepoint,
instead of mapping cmsy slot N → the actual Unicode glyph (U+2032
for slot 0x30, etc.).

**Multi-layer bug chain:**
1. Engine font-encoding map for cmsy (math family 2) lacks
   slot→Unicode entries (cmsy slot 0x30 should map to U+2032 ′,
   but returns U+0030 '0').
2. `\mathchardef\prime="0230` calls `decode_math_char(560)` →
   props.glyph = Some('0') instead of Some('′').
3. `install_definition` writes `\prime` Register with mathglyph=48.
   `:locked` flag would normally refuse, but during latex.fmt dump
   generation, `state_unlocked(true)` is active, so the write
   succeeds.
4. `dump_writer` captures corrupt state → `latex.dump.txt:21951`.
5. On every subsequent run, `latex_dump_reader` reapplies
   mathglyph=48, clobbering plain_constructs's locked U+2032.
6. `latex_constructs.rs:2379-2401` force-reloads plain_constructs
   to restore `\prime`, but this also clobbers user `\def\d` etc.
   → Task 1.7 root cause.

**Real fix (most upstream — most Perl-faithful):** add cmsy
font-encoding entries so `decode_math_char(0x230) = Some('′')`.
That eliminates layers 2-6 in one shot. Investigation locus:
LaTeXML's font encoding `.cmap` / `.enc` files in `resources/` and
the engine's font-decode glue.
Fix paths:
  * (α) Investigate latex_dump generation: when building latex.fmt,
    after plain_constructs locks `\prime` with mathglyph=8242, what
    causes `\prime`'s Register to be stored with mathglyph=48 by
    the time dump_writer runs? Likely `\mathchardef\prime="0230`
    appearing somewhere in latex.ltx's expansion, with the engine
    direct-mutating the mathglyph past the locked guard.
  * (β) As a quick, generation-pipeline-bypass workaround: post-
    process latex.dump.txt to overwrite the bad `\prime` row with
    the plain.dump.txt one (or strip from latex.dump.txt entirely
    so plain_constructs's lock survives). Cheap to test.
  * (γ) Make `dump_reader.apply_record` skip Register records when
    `lookup_meaning(cs)` already exists AND is locked — rejects
    only the genuinely problematic clobbers.

(γ) is most surgical and least invasive. Implementation cost: ~10
lines in dump_reader.rs:734-806's Register arm. Risk: legitimate
latex-only Register overrides (none known but possible).

