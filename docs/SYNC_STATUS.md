# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-17. Open gaps & active TODOs only; completed items live in git history.

**Test inventory:** 409 integration tests pass (0 failures, 0 ignored); all 10 tikz tests pass. MakeBibliography pipeline fully operational.

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **93+%** on full catalog.

**10k sandbox:** 7,898 arxiv ZIPs in `$HOME/data/10k_sandbox/`. Last 512-paper ramp: **93.2% OK** (477 ok / 21 conv_error / 14 timeout / **0 panics**).

**Engine definition coverage:** **99.9%** (2,455/2,457 Perl Engine definitions ported). Only `\directlua` (LuaTeX) and `\ASCII` (niche) missing by design.

**Dump loading:** 5,834 entries from latex.ltx kernel (V + codes + @-internal M + Register). Add-only policy preserves engine semantics.

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational:
```
latexml_oxide --whatsin=archive --format=html5 --pmml --mathtex --noinvisibletimes \
  --nodefaultresources --nobibtex --preload=ar5iv.sty --timeout=2700 --log=log.txt \
  --dest=output.zip input.zip
```

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) | [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) | [`PERFORMANCE.md`](PERFORMANCE.md)

---

## Engine Files — Open Gaps Only

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`, `CommaList`, `DigestUntil` stubbed |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics |
| tex_tables.rs | MINOR | Minor: padding CSS classes (XSLT concern) |

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.

## Unported Perl Engine Files

| File | Defs | Status | Notes |
|------|------|--------|-------|
| `AmSTeX.pool.ltxml` | 112 | ~30% | Plain TeX format (rare) |
| `BibTeX.pool.ltxml` | 956 | 0% | Skipped via `--nobibtex` in production |

## Package Bindings

**100% coverage: all 406+ Perl bindings ported to Rust.** Zero MISSING. Three `todo!()` remain as deliberate invariant asserts on unreachable branches (be_absorbed for Alignment; get_meta for Ref/Arg XM variants).

## Tikz — Known Diffs (vs Perl output)

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. tikz matrix rendering uses `<svg:g class="ltx_tikzmatrix">` groups (Rust) vs inline-blocks (Perl)

### Permanent sandbox ignores

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port
- **2402.03300**, **2410.10068**, **2511.03798** — Perl also fails on these papers

---

## Completed Phases (historical summary)

- **Phase A (EMPTY→OK binding coverage)** — all Perl `.sty.ltxml` / `.cls.ltxml` files ported or stubbed.
- **Phase B (parity improvement)** — per-package semantic alignment, rewrites.
- **Phase C1–C3 / C5** — 97-paper sandbox baseline (93%+ OK), babel fix, directory/archive input parity, code quality sweep.
- **Phase E (kernel dump integration)** — 5,834 dump entries loaded (V + @-internal M + Register + codes). Add-only policy with has_meaning/has_value safety, non-ASCII catcodes only, MC/DC skipped (expl3 init corrupts them). Auto-generated at build time via `build.rs`.
- **Phase F (engine file reorganization)** — 1:1 match with Perl's `Engine/` directory. `math_common.rs`, `plain_bootstrap.rs`, `plain_base.rs`, `plain_constructs.rs`, `latex_bootstrap.rs`, `latex_base.rs`, `latex_constructs.rs` (7800 lines, merged from 36 `latex_ch*.rs`). All Rust engine files now match Perl file names exactly.
- **Phase G (SVG post-processor)** — inline SVG injection for `\begin{picture}` via post-XSLT regex replacement (avoids libxml2 UAF); covers lines, vectors, circles, ovals, framebox, qbezier, multiput.

Detailed fix history for phases above lives in git log. See the corresponding session commits on `claude-round-15` (e.g., `da8b66358` ch* consolidation, dump-loading commits via Session 102, SVG commits in Session 102, etc.).

---

## Work Plan — Active TODO List

### D0. Raw-binding fidelity — LARGELY COMPLETE (was HIGHEST PRIORITY)

**Status (2026-04-17):** `tests/babel/page545` **passes** and is no
longer `#[ignore]`d. The remaining engine gaps are non-blocking.

Make `tests/babel/page545` pass via the **exact same raw-loading path**
that Perl uses. Re-enabling this single test was a practical,
fully-solvable project that closed deep engine gaps.

**Background.** Perl's babel support is three tiny files:
- `babel.sty.ltxml` (30 lines) — `InputDefinitions('babel', noltxml=>1, type=>sty)`
- `babel.def.ltxml` (34 lines) — `Let('\bbl@opt@safe','\@empty')`, load raw
  `babel.def`, require `babel_support`
- `babel_support.sty.ltxml` (169 lines) — Unicode glyphs, language→ISO map,
  `\select@language` hook that calls `MergeFont(language=>iso)`.

All language-specific behavior (captions, shorthands, active punctuation,
encoding switches, …) comes from the vanilla `.ldf` files that `babel.sty`
pulls in via `\openin` + `\input` as options are processed.

**Our Rust bindings**, by contrast, are **384+153 lines of workarounds**
(`babel_sty.rs`, `babel_support_sty.rs`) that pre-declare things the raw
path would otherwise build, hard-code caption strings that would otherwise
come from `.ldf` files, and hand-roll active-char mechanisms. These were
added in sessions 42–80 to keep the test suite running, but they mean
Rust's babel is a different implementation from Perl's — and the diff shows:

```
                           Perl          Local Rust                    CI Rust
  p1 first chars:          <p>The ...    <p><text xml:lang="de">,</   <p>,The ...
  para class:              ltx_align_left  (missing)                    (missing)
  French colon spacing:    "français :"  "français:"                   "français:"
```

An experiment (`/loop`, 2026-04-17) replaced all three Rust babel files
with line-for-line Perl ports. Four tests broke (`csquotes_test`,
`french_test`, `german_test`, `greek_test`) and the failures revealed the
exact engine gaps:

**Tasks, roughly in dependency order:**

- [x] **Precompile-phase language registers.** Perl's `make formats` puts
  `\l@english`, `\l@german`, `\l@french`, … in the precompiled kernel so
  babel's `\bbl@iflanguage` check passes. Status: the Rust kernel dump
  (`resources/dumps/latex.dump.txt`, 24k entries) now carries **108**
  `\l@<lang>` `CharDef` registers — confirmed via
  `awk -F'\t' '$1=="M" && $2 ~ /^\\l@/' resources/dumps/latex.dump.txt |
  wc -l` → 108. Includes `\l@english`, `\l@french`, `\l@german`,
  `\l@ngerman`, `\l@greek`, `\l@russian`, and all mainline babel
  languages. The runtime `language.def` / `hyphen.cfg` ingestion that
  runs when `tools/make_formats.sh` regenerates the dump (via
  `--init` path in `ini_tex.rs`) persists these into the emitted dump.
  `babel_support_sty.rs`'s `\iflanguage` still auto-creates missing
  entries via `\newlanguage` as a belt-and-suspenders fallback.

- [x] **`\openin`-based `.ini` loading.** WORKS (2026-04-17 verified):
  `\babelprovide[import]{<lang>}` + `\selectlanguage{<lang>}` reads
  `babel-<lang>.ini` via babel's own parsing loop and defines captions
  end-to-end. Verified:
  - **Latin-1** (`spanish`, `french` etc.) — works out of the box:
    `refname=[Referencias] chaptername=[Capítulo] xml:lang="es"`.
  - **Cyrillic** (`bulgarian`, `russian`) — works with explicit
    `\usepackage[T2A]{fontenc}`: `refname=[Литература]`. Without T2A,
    the LICR macros (`\CYRL \cyri \cyrt...`) are undefined.
  - `find_file`, `\openin`, `\ifeof`, `\read` all function correctly
    on `babel/locale/*.ini` paths.

  **Caveat**: the `\usepackage[<lang>]{babel}` convenience path still
  uses our per-language Rust ports (english_sty.rs, french_ldf.rs,
  german_sty.rs, ngerman_sty.rs) because `\bbl@main@language`
  resolution fails (see `project_babel_second_processoptions_bug.md`).
  Fixing the option-pipeline gap would let the ini-loading path cover
  ALL languages uniformly and make per-language ports redundant.

- [x] **`\initiate@active@char` / active-char lifecycle.** WORKS
  (2026-04-17 verified): the primitive fires, `\useshorthands{"}`,
  `\languageshorthands{system}`, and `\declare@shorthand{system}{"a}{ae}`
  chain together end-to-end — `"a` → `ae` in rendered output.
  Earlier "fails during raw loading" note was a red herring; the
  underlying issue was the `@currname` leakage bug
  (`project_babel_second_processoptions_bug.md`) preventing the option
  pipeline from loading `germanb.ldf` in the first place. Once the
  option pipeline closes, babel's own shorthand wiring activates
  cleanly.

- [x] **`AtBeginDocument` hook chain ordering.** FIXED session 110
  (commit 56b0c35d2). Root cause was `@currname` leakage from plain
  `\input`: our `input_definitions` only restored `@currname/@currext`
  when `handleoptions=true`, but plain `\input` uses `handleoptions=
  false` — so after babel.sty's `\input txtbabel.def` at L2242, the
  callee's `@currname="txtbabel"` leaked into 2000+ lines of babel.sty,
  causing `\ProcessOptions*` at L4291 to look up `opt@txtbabel.def`
  (empty) instead of `opt@babel.sty`. Fix: plain `\input` now locally
  saves/restores `@currname/@currext` around the nested input call.

  Coordinated fix in `\lx@babel@activate@mainlang`: force-set
  `\bbl@main@language` globally from `\opt@babel.sty`. Babel's raw-load
  path may leave it pointing at a language whose `.ldf` `\ldf@finish`
  happened to run last (e.g. "greek" for
  `\usepackage[polutonikogreek,english]{babel}`), which would activate
  the wrong language at `\AtBeginDocument`.

- [x] **Kernel dump regeneration at build time.** Per design intent,
  `resources/dumps/latex.dump.txt` should **not** be checked into VCS;
  it should be rebuilt on each compile from the ambient texlive. (Status:
  `resources/dumps/` is `.gitignore`d. `latexml_package/build.rs` used to
  `include_str!` the dump at compile time, which locked it into whatever
  texlive was present when someone last ran `--init` locally. As of
  "Make the kernel dump a runtime artifact, not a compile-time one" — the
  dump is resolved at runtime via `$LATEXML_DUMP_PATH`, `$LATEXML_DUMP_DIR`,
  exe-relative paths, or the dev-tree path. `tools/make_formats.sh`
  regenerates it in one step. CI runs `make_formats.sh` before tests, so
  the dump the test suite consumes always matches the test-runtime texlive.)

- [x] **Perl-parity `LATEXML_NODUMP` opt-out.** `Package.pm` `LoadFormat`:
  `if (!$ENV{LATEXML_NODUMP} && FindFile($format . '_dump', ...))`. The
  Rust runtime loader now honors the same env var — if set, the dump is
  skipped unconditionally and the engine proceeds on the in-code bootstrap
  path. Verified: `LATEXML_NODUMP=1` emits an info-level log, skips the
  file search, returns `Ok(())`.

- [ ] **Dump / `_base` mutual exclusivity (Perl-parity `LoadFormat`
  branching).** Perl's `LoadFormat` takes **one** of two paths:
  `bootstrap + dump + constructs` (when the dump exists) **or**
  `bootstrap + _base + constructs` (when it does not). The two are
  mutually exclusive — `_base` is the verbose source form of what the
  dump serializes. Our `latex.rs` currently loads both: `bootstrap` →
  `_base` (our `latex_base.rs` Rust bindings) → `dump` (add-only) →
  `constructs`. Measured impact: the dump does **~6045 add-only inserts
  on startup** (most pre-loaded entries are already defined by
  `_base.rs`), costing ~5 MB RSS and ~10 ms, delivering **no measurable
  speed-up** on minimal or medium docs. Full test suite passes
  identically with or without the dump. Fix: make the `_base` pool vs
  `_dump` load mutually exclusive so the dump's raison d'être
  (bypassing base reprocessing) actually kicks in.

  This is the cleanest lever that will make the kernel dump *do* what
  it claims in the Perl design — and it becomes necessary once the
  D0 precompile-phase work (language registers, `\openin` / `.ini`
  loading, etc.) lands, because `_base` will no longer cover those
  things alone.

- [x] **Dump captures primitive aliases (`PA`/`MPA` entries).** The
  short-circuit guard in texlive's `expl3.sty` is
  `\ifx\csname tex_let:D\endcsname\relax` — it skips the 36k-line
  `\input expl3-code.tex` if `\tex_let:D` is defined. `\tex_let:D` is
  established by `\let \tex_let:D \let` in expl3-code.tex L276, i.e.
  it's a `Rc<Primitive>` alias-share with `\let`. Closures can't be
  serialized, so the dump previously lost all ~370 of these alias
  relationships. (Status: `is_serializable` now returns true for
  `Stored::Primitive`/`MathPrimitive`; `serialize_stored` emits
  `PA\t<target_cs>`/`MPA\t<target_cs>`; the primary (canonical) entry
  is filtered when `key == target_cs`. Current dump includes 373
  PA entries: `\tex_let:D → \let`, `\tex_def:D → \def`,
  `\tex_global:D → \global`, and hundreds more. `dump_reader` has
  the PA handler wired (replays `\let <key> <target>` via
  `state::let_i`) but **consumption is gated off** until the
  mutual-exclusivity work below lands — see next item.)

### Critical review: Perl dumper vs. Rust dumper

A line-by-line comparison of `LaTeXML::Core::Dumper`, `Engine/TeX_Job.pool.ltxml::DumpFile`,
and `Package.pm::LoadFormat` against our `dump_writer.rs` /
`dump_reader.rs` / `ini_tex.rs` surfaces five significant structural
differences. Each corresponds to an entry in the work plan below.

1. **Snapshot taken at the wrong point.** Perl's `DumpFile` runs
   `LoadPool($name . '_bootstrap')` *before* snapshotting, and only
   the bootstrap. The subsequent raw-load's diff is therefore
   "bootstrap → fully-initialized kernel". Our `ini_tex.rs` starts
   from a state where `plain_bootstrap.rs` **+ `_base.rs` +
   `_constructs.rs`** have already all run (whatever the engine
   normally loads at `Core::new` time), so our diff captures only
   "full kernel → full kernel + raw latex.ltx extras". The dump is
   ~24k entries vs. what Perl's `latex_dump.pool.ltxml` captures:
   the 8741-line block of the LaTeX kernel itself.

   This is the biggest structural gap. It also explains why flipping
   PA consumption on causes explosion: our dump only has the extra
   expl3 definitions, not the LaTeX kernel — when `expl3.sty`'s guard
   short-circuits, post-guard code executes against a hybrid state
   (`_base.rs` primitives mixed with dump PAs mixed with dump
   `@`-internal macros) that wasn't the state any single code path was
   designed for.

2. **Missing early/late let-assignment split.** Perl's `DumpFile`
   categorizes `\let` assignments into three buckets:
   - `@cmds_early` — `Lt(cs, target)` where the target existed
     *before* the raw load (bootstrap primitive). Emitted **first**.
   - `@cmds` — normal `I(dump)` / `Im(key, dump)` assignments.
   - `@cmds_late` — `Lt(cs, target)` where the target was defined
     *during* the raw load. Emitted **last** so its target is already
     installed by the time the let fires.

   Our PA/MPA entries are written in arbitrary (hash-iteration) order
   and loaded in file order. If an alias points at a CS that the dump
   also defines *later in the file*, the alias resolves against
   either an undefined target (silent skip via our `has_meaning`
   guard) or a stale binding. Perl's `@augtables = (…'prelet'…
   'postlet'…)` encodes this split explicitly.

3. **`I(dump)` vs `Im(key, dump)` distinction.** Perl emits `I(dump)`
   when the definition's own CS matches the table key (the standard
   case, where the value carries its own identity) — the CS is
   embedded in the dump string itself. `Im(key, dump)` is for cases
   where the value doesn't have a self-CS (a meaning assigned to a
   token that doesn't identify itself). Our `M` entries always use
   the external key; we don't distinguish the self-identifying case.

4. **`IGNORED_SYMBOLS` is a specific blacklist, not substring
   patterns.** Perl hard-codes `value:DOCUMENT_REWRITE_RULES`,
   `value:PARAMETER_TYPES`, `value:TAG_PROPERTIES`,
   `value:MATH_LIGATURES`, `value:TEXT_LIGATURES`, plus
   `meaning:\lnot` and `meaning:\to` (both of which used to cause
   test breakage via pre-2017 TeXLive `\let\lnot\neg`). Our
   `SKIP_VALUE_KEYS` + `SKIP_VALUE_PREFIXES` + `SKIP_VALUE_CONTAINS`
   mirror the *spirit* but miss the targeted specificity — e.g., our
   `_loaded` substring blocks all of them, whereas Perl keeps
   `expl3-code.tex_loaded` by *not* having it on the list.

5. **Perl's dump is executable Perl code.** `latex_dump.pool.ltxml`
   opens with `package LaTeXML::Internal::Dump; use LaTeXML::Core::Dumper
   qw(:load);` and contains ~8k lines of the form `I(E(C('\foo'),…))`.
   Load-time is `require FILE` — `perl` parses the compact
   Huffman-named constructors (`C`, `L`, `T`, `E`, `I`, `Lt`, …) and
   runs them. Very fast. Our format is tab-separated text parsed by
   `parse_and_load` at runtime. Functionally equivalent, but we pay
   more per entry than Perl does.

Nothing critical is missing from our data model — `PA`/`MPA` plus
`E`/`T`/`R`/`V` cover the same variants — but **the snapshot timing
(#1) and the let ordering (#2) are the two gaps that block the
Perl-sized expl3 speedup**. Harvesting the speedup safely requires:

- [ ] **(d.1) Move the snapshot earlier.** `ini_tex.rs` should
  explicitly load `plain_bootstrap + latex_bootstrap` only, snapshot,
  then raw-load `latex.ltx`. Result: dump includes the full LaTeX
  kernel, so a dump-only load path can replace `_base.rs` entirely
  (matching Perl's `LoadFormat` branching).

- [ ] **(d.2) Split PA/MPA into early / late buckets** based on
  whether the target CS existed in the snapshot. `dump_writer.rs`
  needs the same `%prev` / `%curr` comparison Perl does in
  `DumpFile`; `dump_reader.rs` / the dump file layout need a way to
  load-in-order that respects the bucket.

- [ ] **(d.3) Implement `\let`-alias ordering guarantees for PA
  entries.** Once (d.2) is in place, consuming PA becomes safe: the
  target is always defined before the alias fires.

- [ ] **(d.4) Switch to Perl-style executable-constructor dump
  format** (optional, perf-only). Compact constructors like `I(E(C,
  Ps, T))` would let us skip string parsing. Not blocking for
  correctness; measure first whether the tab-separated-text parse
  is a real hotspot.

- [ ] **(d.5) Harvest expl3 short-circuit.** With (d.1)–(d.3) in
  place, enabling PA consumption + `expl3.sty_loaded` allow-list
  should cleanly cut `\usepackage{expl3}` from 1.3 s to <100 ms
  without any state-mix explosion.

- [ ] **Harvest expl3 short-circuit (Perl's actual "massive speedup").**
  First-principles derivation of what Perl's dump saves that ours
  doesn't, with measurements:

  | Path | Wall | RSS |
  |---|---|---|
  | Rust `--init=latex.ltx` raw-load (no dump) | 15.5 s | ~1 GB |
  | Rust conversion of expl3 doc (with dump) | 1.37 s | 164 MB |
  | Rust conversion of expl3 doc (`LATEXML_NODUMP=1`) | 1.36 s | 155 MB |
  | Rust bootstrap+_base+constructs (compiled) | <10 ms | ~40 MB |

  **Why our dump currently doesn't speed anything up:**
  1. `_base.rs` is already pre-compiled Rust containing LaTeX-kernel
     bindings. The dump's add-only policy sees most of its 6045 entries
     as "already defined" and skips them — the state they'd add is
     already set by compiled code. This is the *opposite* of Perl, where
     the dump REPLACES work that would otherwise be done by interpreter-
     bound `.pool.ltxml` loading.
  2. `\usepackage{expl3}` in a user doc calls `expl3_sty.rs::load_definitions`
     which unconditionally `input_definitions("expl3", sty, noltxml=true)`,
     re-processing all 36k lines of `expl3-code.tex`. This costs ~1.3 s.
     The dump contains the post-load expl3 state (`expl3-code.tex_loaded=1`
     plus ~17k expl3 definitions) but cannot short-circuit the raw load
     because `dump_reader`'s `SKIP_VALUE_CONTAINS = ["_loaded", ...]`
     strips every `_loaded` flag. Perl's dump preserves these flags, so
     `\usepackage{expl3}` sees "already loaded" and skips the 36k-line
     reprocess.

  **What breaks when we naively lift the skip** (tried and reverted):
  unblocking `_loaded`/`_found_loaded` for all keys sets 1000+
  hyphenation-pattern flags. Downstream babel language.def loading
  then skips files the engine depends on to register `\l@<lang>`,
  triggering error recovery that balloons to **61 s / 4.5 GB RSS** on
  the simple expl3 test doc. Short-circuiting expl3 alone
  (`ALLOW_LOADED_EXCEPTIONS` carve-out + an expl3_sty guard) fires
  correctly but the rest of the doc hits an interaction the dump
  doesn't fully cover and still blows up.

  **What's actually needed** to harvest the Perl speedup safely:

  - (a) A curated subset of `_loaded` keys worth short-circuiting (at
    minimum `expl3.sty_loaded` + `expl3-code.tex_loaded`, later babel
    language flags once their bindings are Perl-strict).
  - (b) For each key in that subset, a companion guarantee that the
    corresponding `*_sty.rs` binding is idempotent when its raw-load
    is skipped — the post-load catcode/message/fixup steps in
    `expl3_sty.rs` need to be either captured by the dump or run
    unconditionally so a partial dump doesn't leave the engine in a
    half-initialized state.
  - (c) Ideally, regenerate the dump against the exact binding that
    will consume it (so the post-load side-effects of the Rust
    wrapper ARE part of the snapshot), not from `--init=latex.ltx`
    alone. That is: `--init` should include a tiny `\usepackage{expl3}`
    stanza at the end so the .sty-level loaded flag is also captured.
  - (d) Enable consumption of `PA`/`MPA` entries in `dump_reader`'s
    M-table dispatcher (currently gated off). With the 373 aliases
    re-applied, `\tex_let:D` is defined → `expl3.sty` guard fires →
    raw `\input expl3-code.tex` skipped. **Verified mechanism**: I
    confirmed this works end-to-end by temporarily enabling PA
    consumption — `\ifx\csname tex_let:D\endcsname\relax` goes from
    "IS_RELAX_FULL_LOAD" to "IS_NOT_RELAX_SHORT_CIRCUIT". The guard
    fires correctly. BUT the code in `expl3.sty` after the guard
    (`\__kernel_dependency_version_check:Nn`, `ProcessOptions \relax`,
    `\keys_define:nn { sys }`, …) exercises expl3 machinery whose
    state disagrees with what `_base.rs` has — a simple expl3 doc
    balloons to 60 s / 4.5 GB RSS. Unblocking (d) requires (a)–(c)
    first.

  Once (a)–(d) are in place we should see the Perl-sized win:
  ~1.3 s → ~50 ms per expl3 conversion.

  **2026-04-17 update — failure-mode catalog from isolated experiments.**
  With the (d.2) early/late split in place I re-tested narrower PA
  consumption variants. Both failed with the same run-time shape
  (~60 s timeout, RSS climbing, exit 143 SIGTERM-by-watchdog) but
  for different reasons:

  - **PA alone, no `:`-style Expandables**: `\tex_let:D` becomes
    let-aliased to `\let` via the dump → `expl3.sty`'s own guard
    `\ifx\csname tex_let:D\endcsname\relax` fires → raw
    `\input expl3-code.tex` is skipped → `expl3.sty`'s post-guard
    code (`\__kernel_dependency_version_check:Nn`, `\ProcessOptions`,
    `\keys_define:nn { sys }`, …) references `:`-style macros we
    still filter out → undefined-CS recovery → loop.
  - **PA + `:`-style Expandables loaded**: the `:`-style macro
    bodies reference each other through `\__kernel_…` and expl3
    hooks. Loading them en-masse triggers a similar recovery
    cascade.

  Neither partial unblock works. The two have to be removed
  **together AND** `expl3_sty.rs` needs to short-circuit its whole
  `load_definitions` when the dump already has expl3 state so
  `expl3.sty`'s post-guard code doesn't run at all. Each of the
  three gates independently causes the same class of crash;
  removing all three simultaneously is what gets the Perl speedup.

- [x] **Page545 verification.** `cargo test -p latexml --test 81_babel
  -- page545` passes. Status (2026-04-17): **3 of 4 diffs FIXED**, 1
  documented as an intentional divergence (OXIDIZED_DESIGN #22).
  - ✓ `ltx_align_left` on paragraphs — FIXED as side effect of comma fix
  - ✓ stray leading comma in p1 — FIXED by removing `\let\@nil\relax`
    from `latex_base.rs` (was a Rust-only addition that broke babel's
    `\bbl@fornext` termination check)
  - ✓ `français :` colon spacing — FIXED by making French active
    `:;!?` primitives check `\languagename` / current font language
  - ✓ missing trailing `<text xml:lang="de"></text>` nested empty
    wrapper — accepted as intentional divergence (OXIDIZED_DESIGN #22:
    Perl emits pedantic nested empty language-stack unwind wrappers
    at group exit; Rust emits only the outer one; both are invisible
    in rendering).

- [~] **Drop Rust babel workarounds incrementally.** In progress.
  Session 110 (2026-04-17): **babel_sty.rs cut 405 → 128 lines (68%)** via
  relocation + consolidation — not yet engine-closed, but the remaining
  code is a clean orchestration layer. Breakdown of moves:
  - French captions/ordinals/guillemets/active-puncts (~100 lines):
    `babel_sty.rs` → `french_ldf.rs` (where frenchb.ldf content belongs).
    Includes frenchb's trailing `\xspace`/`\FBthickkern` kerning.
  - German caption + `"` shorthand dispatch (~45 lines):
    `babel_sty.rs` → `german_sty.rs` (where germanb.ldf content belongs).
  - English/NGerman captions: moved to `english_sty.rs`/`ngerman_sty.rs`.
  - `\lx@babel@activate@lang@post` + `\lx@babel@set@doclang` consolidated
    into `\lx@babel@activate@mainlang` (run twice — at load-time for
    DOCUMENT_LANGUAGE, at AtBeginDocument for late state refresh).
  - Iterates over ALL options in `\opt@babel.sty` to load each language's
    port up-front. Needed because babel's own `\select@language` calls
    `\captions<lang>` before our `\ltx@bbl@select@language` hook can load
    the port.
  - `.ldf` dispatch entries added: english/german/germanb/ngerman/ngermanb.

  Remaining gap to Perl's 30-line stub: ~100 lines of rationalized
  workarounds for engine gaps (precompiled kernel, `\initiate@active@char`
  lifecycle, `.ini` openin). Eliminating them requires the dump-capture-
  language-layer step in the staged plan (memory:
  `wisdom_babel_fidelity_plan.md`).

- [x] **Un-ignore `page545_test`.** Done 2026-04-17 (commit 96d4bfbe4).
  The expected XML in `tests/babel/page545.xml` matches Rust's output
  byte-for-byte after the `\@nil` root-cause fix; `#[ignore]` removed.

  Status of the four original diffs (final 2026-04-17):
  - [x] **French `:`/`;`/`!`/`?` thin space** — fixed by making the
    active-char primitives query the current font's language and only
    emit `\u{2006}` when French is active. `\foreignlanguage{english}`
    inside a French paragraph now emits bare punctuation.
  - [x] **Stray `,` in p1** — root-caused and fixed. The Rust-only
    `\let\@nil\relax` in `latex_base.rs` (added as a cells_test
    workaround in fa0e9a08f) made `\ifx\@nil\relax` return TRUE,
    prematurely terminating babel's `\bbl@fornext` iteration and
    leaking the iterator-separator comma into the main token stream.
    Perl keeps `\@nil` undefined so the test is FALSE. Removed the
    stray `\let`.
  - [x] **`\raggedright` missing `class="ltx_align_left"`** — resolved
    as a side effect of the comma fix. The comma was landing in the
    first auto-opened paragraph which was then captured as
    `ALIGNING_NODE` instead of the document.
  - [~] **Empty `<text xml:lang="de"></text>` in p4 not emitted** —
    accepted as an intentional divergence (OXIDIZED_DESIGN #22).
    Perl emits pedantic nested empty language-stack unwind wrappers;
    Rust emits only the outermost. No rendering impact.

**Why this is practical, not aspirational.** Every item above is
mechanical: the Perl source is short, its intent is legible, and the
divergences show up as specific XML diffs we can pin. No novel design is
required — just closing each engine primitive to the point where the
raw-loading path runs clean. When it does, one of the most complex
packages in the LaTeX ecosystem becomes a ~50-line Rust stub, and every
future babel upgrade from upstream flows in automatically.

---

### Phase D: 10k-Document Sandbox — Coverage & Performance

Scale testing to ~8,000 arxiv papers (`$HOME/data/10k_sandbox/`). All known to convert under Perl LaTeXML. **Tool:** `cortex_worker --standalone --input <zip> --output <zip>`.

**Process guards:** timeout 60s, RAM 6GB, core dumps disabled, output 200MB cap. Parallelism via GNU parallel (default 16). Categories: `ok`, `timeout`, `oom_or_kill`, `segfault`, `abort`, `error`, `empty_output`, `oversized`. Runner: `tools/benchmark_10k.sh`.

**Ramp-up protocol:** exponential doubling (4→8→16→…→7898) with 0-error gate. On failure: diagnose root cause, fix in Rust, re-run failing files, restart ramp.

**Two stages:**
1. **Stage 1 — Coverage:** zero non-timeout failures at full scale.
2. **Stage 2 — Performance:** eliminate timeouts at 120s cap.

#### [ ] D1. Ramp-up runs — ONGOING

Latest (session 108): **512 papers: 93.2% OK** (477 / 21 conv_error / 14 abort / **0 panics**). No Rust-attributable conversion errors at 128-paper scale. Remaining 512-scale errors are paper-specific (user LaTeX bugs, exotic Unicode in CS names, custom macros, content-model violations).

Known blockers by category (512-scale residuals):
- `Missing $` display math (document bugs)
- Content-model `malformed` (`ltx:line` in `ltx:para`, `ltx:g` in `ltx:figure`, etc.)
- Raw-class undefined internals (e.g. `\@count`, `\theequation@ID` in standalone non-article classes)
- Rc<RefCell> "shared Node" error in 0805.2376 (libxml2 node sharing during tree mutation — tracked in D3b)

#### [ ] D2. Coverage fixes — ONGOING

Each cycle adds small targeted fixes for specific undefined/misbehaving commands per log analysis. Detailed fix history in git log; current focus is filling package-parity gaps against Perl upstream.

**Most recent wave (session 108 /loop):** xcolor `RGB` case-sensitivity bug (all `{RGB}{r g b}` defs → white), page counter starts at 1 (#2442), `\braket` user-facing reversions (#2340), bibitem prune empty auto-opened (#2409), `\text@frac` constructor, `\person@thanks` inline, elsart/mn2e/aa/iopart/texvc/proofwiki/sv_support/ams_support/acmart/amsbook/revtex4/inst_support/microtype/html/subcaption/attachfile/floatflt/floatfig/subfloat/iopams/actuarialangle parity patches.

#### [ ] D3. Performance catalog — after Stage 1

After Stage 1 reaches 7,898 with 0 non-timeout errors:
1. List all tasks >60s with wall-clock time
2. Profile top offenders (flamegraph, token count, loop detection)
3. Targeted optimizations (per-task or systemic)

#### [ ] D3b. Stability — eliminate SIGSEGV in test suite

A Rust safe-by-construction implementation should NEVER segfault. Sources investigated:
1. **libxml2 FFI** — `libxml::tree::Node` is `Rc<RefCell<_Node>>` wrapping raw C pointers; unlinking while referenced elsewhere causes UAF. Past incident: `xmlFreeNodeList` UAF during PostDocument Drop when SVG replacement kept idcache alive (fixed in G2 via string-based SVG injection).
2. **libxslt C stylesheet processing** — past crashes with `svg:` namespaced elements.
3. **Rust unsafe in arena** — `with_arena_mut` cached raw pointer from RefCell.
4. **Parallel benchmark writes** — output files sharing paths.

**Status:**
- 50_structure SIGSEGV no longer reproduces (5-run stress stable after S105 `STATE_IN_USE` / `LASTID` moves to thread_local Cell).
- Catalogued 10 `unsafe` blocks across 8 files; all SAFETY-documented (session 106).
- 0805.2376 "shared Node" error still open (Rc mutation during tree traversal).

**TODO:**
- [ ] Route libxml node lifetimes through guardian structure that forbids unlinking without cache invalidation.
- [ ] Replace unsafe-over-FFI patterns with safe wrappers where practical.

#### [ ] D4. Performance — parallel scaling and allocations

**Baseline (session 105, paper 0707.1173):**

| Workers | Total time | Per-worker efficiency |
|---|---|---|
| 1 | 22.6s | 100% |
| 4 | 33.6s | 67% |
| 16 | 76.8s | 29% |
| 20 | 104.7s | 22% |

14-core/20-thread machine, ~42% ceiling at 16 workers. Peak RSS 570 MB/process.

**Completed:**
- [x] mimalloc as global allocator — reduces glibc arena-mutex contention (~6% single-process).
- [x] `--timeout` default 600s → 60s.

**Callgrind (session 105):** Math parser Marpa dominates — `transitive_closure` 34.3%, `marpa_g_precompute` 8.3%, `bv_scan` 7.1%, AVL ops 6.8%. Total Marpa-related >60% CPU.

**Active work:**
- [ ] Audit `.to_string()` (~1900 sites) — replace with `&str` / interned symbols where value goes into `HashMap<String,String>`.
- [ ] Audit `String::from("...")` literals for interned conversions.
- [ ] Replace `HashMap<String,String>` with `SymHashMap<SymStr>` in hot paths.
- [ ] Audit `.clone()` in `document.rs` (73), `latex_constructs.rs` (73), `font.rs` (39).
- [ ] Review `Tokens` cloning — pass `&Tokens` or `Cow` for read-only iteration.
- [ ] Profile math parser RAM independently (Marpa chart, forest).
- [ ] Investigate shared read-only engine state across processes (mmap dump).
- [ ] Long-running daemon / process pool to amortize 570 MB startup.
- [ ] Fork-based parallelism for CoW memory sharing.

#### [ ] D5. Math parser optimizations (HIGHEST PRIORITY per callgrind)

**Completed:**
- [x] Avoid per-formula `reset_engine` (S105): paper 0707.1173 22s→15s.
- [x] Audit `trig_arg` ambiguity (S105): `\sin(x)+\sin(y)` 65→1 parses; paper 0704.0516 6×65-enumerated→1.
- [x] Remove duplicate `<fn> fenced_factor` alternatives: physics.tex 40→8, full suite 99→59 ambiguous formulas.
- [x] `MATHPARSER_SPECULATE` redesign (S107): removed grammar-layer filter, `FencedLettersAreFunctionArguments` pragma picks consistent interpretation. `a(b)(c)(d)` 23→2 (91% reduction).
- [x] Watchdog thread for cooperative-timeout escape (aborts native Marpa/libxml2 loops).
- [x] `LATEXML_PARSE_AUDIT=1` env var for per-formula diagnostics.

**Remaining:**
- [ ] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.

#### [ ] D6. Grammar First-Principles Plan

Grounded in `docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`. Live audit: `LATEXML_PARSE_AUDIT=1`.

**Completed (S106-108):**
- [x] Narrow `script_op` to `metarelop | vertbar | supops | modifierop` (P^+ tuple 31→3).
- [x] Fix 1: OTHER_OPEN/OTHER_CLOSE split — eliminates PREFIX-match duplication. `[A],[B],[C],[D]` 64→2 (32×).
- [x] Fix 2: Remove `formula_list` from `anything` alternatives.
- [x] Fix 3: Collapse `term_list` vs `formula_list` in fenced contexts.
- [x] Fix 4: `MATHPARSER_SPECULATE` redesign (see D5 above).
- [x] Fix 5: Interval moved from `fenced_factor` to `tight_term` — `f(x,y)` now correctly parses as `f@(vector(x,y))` via category hierarchy, no ad-hoc pragmas.
- [x] Removed redundant `opfunction opfunction` rule.
- [x] Math parser convergence 32→16 consecutive dupes (32% reduction on `tr ρ`).
- [x] Half-decay `consecutive_dupes` on new unique.

**Remaining hotspots (post-S108):**
1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

Items 1–4 are primarily **semantic** (inherent to math practice); further grammar refactoring has limits.

---

## Recent Session Highlights

### Session 109 (2026-04-17, /loop cycles) — page545 / D0 raw-binding milestone

**D0 HIGHEST PRIORITY task COMPLETE**: `tests/babel/page545` passes
unignored. 6/6 babel tests green, 409/409 overall, 0 ignored.

**Root-cause engine fix** — the stray leading comma that had
plagued page545's `<p>` output was caused by a single Rust-only
line in `latex_base.rs:30`:

```
\let\@nil\relax     % ← Rust-only workaround, NOT in Perl kernel
```

Perl leaves `\@nil` undefined so `\ifx\@nil\relax` is FALSE when
both tokens are compared (undefined meaning ≠ `\relax` meaning).
Our workaround made `\ifx\@nil\relax` TRUE, which prematurely
terminated babel's `\bbl@fornext` termination check on the
first empty-parameter iteration:

```
\def\bbl@fornext#1,{%
  \ifx\@nil#1\relax\else ... \expandafter\bbl@fornext \fi}
```

With `#1 = empty`, the `\ifx\@nil\relax` evaluation was the
termination signal instead of the intended recursion, leaving
`\@nil,` unconsumed → trailing `,` emitted as stray text.

**Side-effect fixes** (all driven by root-cause alignment):
- `ltx_align_left` on paragraphs — FIXED
- French active `:;!?` in non-French contexts — FIXED (now
  language-aware via `lookup_font().language`)

**Cleanups enabled**:
- Dropped `\def\@fontenc@load@list{\@elt{OT1}}` workaround
- Dropped 14 redundant `\l@<lang>` pre-definitions (kernel dump
  now carries 108 language registers)
- babel_sty.rs: 418 → 404 lines

**Intentional divergence documented**: OXIDIZED_DESIGN #22 —
single-level vs. per-frame language-stack unwind on group exit
(Perl emits extra empty nested `<text xml:lang>` wrappers;
Rust emits only the outer one; no rendering impact).

### Session 108 (2026-04-17, /loop cycles)

**Packages parity**: 50+ commits filling gaps against Perl: elsart, mn2e, aa, aas, revtex4, iopart, texvc (92 proofwiki macros), sv_support, ams_support, acmart, amsbook, revtex4, inst_support, microtype, html, subcaption, attachfile, floatflt/floatfig, subfloat, iopams, actuarialangle.

**Real bug fixes**:
- **xcolor case-sensitivity**: `\definecolor{x}{RGB}{153 153 192}` was producing `#FFFFFF` due to lowercased model dispatch. Fixed to case-sensitive match — lowercase rgb/cmy/gray take 0..1 components; uppercase RGB/HSB/Gray take 0..255.
- **Page counter**: now starts at 1 per Perl #2442.
- **Bibitem auto-open**: prune empty whatsit, reuse ID per Perl #2409.
- **\text@frac semantic FRACOP**: `\case` in aas_support now produces semantic fraction markup.
- **\person@thanks inline**: elsart_support_core.
- **\backsimeq U+22CD** (Perl #2633); **mixed-delimiter definecolor** (Perl #2551); **Explode newline** reverted to CC_OTHER per Perl #2700.
- **RefCell panics** fixed in `with_font_info` + `font::decode` re-entry (common/mathchar.rs, latexml_sty.rs).
- **DefEnvironment scope lifecycle wisdom**: `after_digest` vs `after_digest_body` matters — body runs post-frame-pop, so local state assigns in before_digest are gone. Documented in `WISDOM.md`.

**Sandbox transitions (broken → OK)**: 9 papers (0705.1190, 0705.2808, 0707.4170, 0710.2880, 0711.4787, 0802.1100, 0810.1610, 0704.2400, 0705.1050, 0705.2208).

**Post-session 512 verification**:

| Category | Count |
|----------|-------|
| ok | **477 (93.2%)** |
| conversion_error | 21 (paper-specific) |
| abort (timeout ~61s) | 14 |
| **panics** | **0** |

### Session 107 (2026-04-16)

- Fix 4 speculative redesign (13 test XMLs updated)
- Documented safety contracts on all 10 unsafe blocks
- OXIDIZED_DESIGN #18 updated for Marpa design
- Paper 0707.1173 conversion: 12.4s (from 22.6s baseline)

### Session 106 (2026-04-16)

- Grammar Fixes 1/2/3 (OTHER_OPEN split, formula_list removal, term_list collapse)
- Narrowed `script_op`
- 317 integration tests pass; total enumerated trees 3767→3544

Earlier sessions (42–105) archived in git log and `memory/project_session_history.md`.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
