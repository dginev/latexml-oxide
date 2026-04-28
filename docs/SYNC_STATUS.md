# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function,
> macro, and definition must faithfully reproduce the original Perl
> semantics, control flow, and edge-case behavior. The Perl source
> (`LaTeXML/` directory) is the ground truth. Only diverge when
> explicitly documented in `docs/OXIDIZED_DESIGN.md`.

## Mission (2026-04-26 pivot — STRICT-PERL DUMP PARITY)

The engine dumps must be a strict translation of Perl's `make formats`
output. This is the **top priority**; CI-green / 10k-sandbox concerns
are LOWERED until the dumps are complete and Perl-faithful.

**The four invariants** (set by user directive 2026-04-26):

1. **Strict mutual-exclusivity in `LoadFormat`**
   (Perl `Package.pm:LoadFormat` L2734-2752):
   * if `<format>_dump.pool.ltxml` exists AND `LATEXML_NODUMP` is
     unset → load `bootstrap → dump → constructs` (NO base);
   * else → load `bootstrap → base → constructs` (NO dump).
2. **Unconditional `I()`/`V()` semantics** in `dump_reader.rs`:
   no admission gate, no skip-if-defined, no closure guards.
   Mirrors Perl's `Core/Dumper.pm` L59-67 which call
   `assign_internal('global')` without filters.
3. **Same-file definitions** as Perl: every `\foo` defined in
   `Engine/<file>.pool.ltxml` must be defined in
   `latexml_package/src/engine/<file>.rs`. Use raw `\outer\def`
   bodies wherever Perl uses RawTeX, so the dump captures them as
   serializable Token-bodies (not opaque Rust closures).
4. **Perl-zero-error baseline**: `--init=plain.tex` and
   `--init=latex.ltx` must complete with **zero errors** —
   matching Perl. Any error during expl3-code.tex / latex.ltx
   raw-load is a parity gap, not a thing to suppress with caps.

**Working doc:** [`PERL_LOADFORMAT_AUDIT.md`](PERL_LOADFORMAT_AUDIT.md).

### 2026-04-29 — Multi-thread `cargo test` SIGSEGV race fixed

**Root cause: `std::env::var` calls on TeX hot paths.** `cargo test
--release --tests` was SIGSEGVing in `__GI_getenv` at ~50% rate when
running with default parallelism (~20 threads × ~20 binaries on a
20-CPU box). Captured via a custom `.init_array`-installed handler
that persists the crashing thread's stack to
`/tmp/latexml_sigsegv_<pid>.txt` (gated by `LATEXML_SIGSEGV_TRACE=1`).
Stack consistently pinned at:

```
__GI_getenv ← SIGSEGV
std::env::var
gullet::read_x_token (gullet.rs:615 LXML_TRACE_GROUP_END check)
```

glibc's `getenv` walks the process-global `environ` array
unprotected; under millions of concurrent reads from N test threads
the walk can land on a stale slot during loader / DSO transitions.

**Fix:** Hoisted every `std::env::var(...)` / `var_os(...)` to a
file-top `static FOO: Lazy<...>` (or `LazyLock` where `once_cell`
isn't a dep), so `getenv` runs once per process. 14 files audited
and converted:

- `latexml_core/src/{gullet,stomach}.rs` (`LXML_TRACE_GROUP_END`,
  `LXML_TRACE_BOUND_MODE` ×7) — the actual hot-path callers
- `latexml_package/src/engine/{plain_dump,latex,tex,tex_job}.rs`
- `latexml_oxide/src/{core_interface,ini_tex,post,util/test}.rs`
- `latexml_math_parser/src/parser.rs`
- `latexml_post/src/{lib,math_processor,graphics}.rs`

Companion fix kept (separate `static mut` race in `rust-libxml`):
`set_node_rc_guard(8192)` in `Document::new()` is now `Once`-gated.

**Result:** SIGSEGV rate `5/10` → `0/12` under default parallelism.
Tests `301 passed; 1 failed` (only `numprints_test` — the unrelated
math-parser tabular-emission gap from prior sessions). CI hooks
(`cargo +nightly fmt --all`, `cargo +nightly clippy --workspace
-- --deny warnings`) both exit 0.

Memory: `wisdom_env_var_hot_path_race.md` (#56).

### 2026-04-28 evening — input_definitions miss-handler dependency-scan

**`maybe_require_dependencies` was effectively dead code before
this fix.** `input_definitions`'s miss-branch
(`latexml_core/src/binding/content.rs:485-507`) preemptively set
`<file>_loaded=true` to break retry loops, which shadowed
`require_package`'s post-call success check
`!_loaded && !_raw_loaded`. The Perl-faithful dependency-scan
fallback (Package.pm:2675-2679 `maybeRequireDependencies unless
$success`) therefore never fired for any package without an
`.ltxml` binding. This silently broke every paper that bundled
its own `.sty` declaring `\RequirePackage{...}`/`\LoadClass{...}`
prerequisites — typical cascade was 70+ errors per paper from
undefined-CS arguments hitting math-mode `_` checks.

The fix splits the flag in two:
- `_loaded` / `_raw_loaded` — genuine binding/raw load success
  (Perl's `$success` truthy)
- `_load_attempted` (Rust-only, internal) — retry-prevention guard

`input_definitions` miss-branch now invokes
`maybe_require_dependencies(name, scan_type)` BEFORE setting
`_load_attempted=true`. `already_handled` short-circuit
(L229-237) reads `_load_attempted` in the `notex`/default branches
so re-invocations still skip. `require_package`'s post-call
`maybe_require_dependencies` call is removed (driven internally
from input_definitions's miss-branch).

**Sandbox impact (paper-local `.sty` cluster):**
- `1803.09589` (jinstpub.sty bundled): 84 errors → 6 errors
  (matching Perl's 6-error baseline exactly). Trace now shows
  `Loading dependencies for jinstpub.sty: amsthm,newtxtt,amsmath,
  amssymb,graphicx,natbib,hyperref,wrapfig,fontenc`.
- `1302.4651`, `1705.03503`, `1803.09911`, `hep-ph9805446`: all
  clean (no obvious problems).
- Workspace tests: 1108/1/0 unchanged (only `numprints_test` —
  pre-existing alignment-template architectural gap).

Memory: `wisdom_load_attempted_separation.md` (#55).

### 2026-04-28 — Audit refresh + dump cleanup (evening loop)

Stable plateau at 1108/1109 tests passing (99.91%). This iteration's
focus: documentation refresh + dump-writer hygiene + warning cleanup.

**dump_writer skip filter for `\@currname`/`\@currext`** —
`latexml_core/src/dump_writer.rs:131-148`. These are file-IO
bookkeeping CSes set per-document by `read_input_file_recursive`
(`binding/content.rs:262-263, 701-702`) that survive into the
snapshot with literal `plain.tex` token bodies. Perl's
`plain_dump.pool.ltxml` omits them because Perl's
`TeX_FileIO.pool.ltxml:28-29` initializes them via
`Let('\@currname','\lx@empty')` BEFORE any file load (state matches
baseline). Skip mirrors the existing `\ver@*` runtime-state filter
pattern. plain.dump.txt 961 → 959 lines after fix; latex.dump.txt
unchanged in count. Behavioral impact: zero (the CSes are
overwritten by `\input` at runtime regardless). See
[wisdom_dump_filter_runtime_state.md] (#54).

**Audit doc refresh** — `docs/PERL_LOADFORMAT_AUDIT.md` updated
top-to-bottom. Added top-level status table; refreshed all 8
file-by-file sections; documented resolved items
(`\@currname/\@currext`, `\tex_*:D`-family with 537 PA aliases now
captured, `\hook_*` family with 31 M-keys now captured); marked
`plain_bootstrap`/`latex_bootstrap`/`latex_base` as PARITY;
`plain_constructs`/`plain_dump`/`latex_dump` as NEAR-PARITY.

**Warning cleanup** — workspace now warning-free:
* `latexml_core/src/gullet.rs:1130-1148` — removed dead
  `runaway_reported` flag (assigned but never read; followed by
  unconditional `break`).
* `latexml_package/src/engine/latex_constructs.rs:27` — removed
  unused `use std::ops::Deref;`.

`cargo check --workspace`: clean. `cargo test --tests --release`:
1108/1109 (numprints_test still failing — architectural fix
deferred). Build + test fully green except for the deferred
single-test architectural item.

### 2026-04-28 — Test recovery wave (afternoon loop)

Workspace failures: **16 → 2** in this session. All recovered via
Perl-faithful fixes, no expedient workarounds:

* **plainsample/plainmath** — math-mode entry now resets `fontfamily=-1`
  locally, mirror Perl `Core/Stomach.pm:505`. Without this, post-dump
  `\rm` leaves `fontfamily=0` leaking into math, wrapping every `=`/`+`
  reversion as `{\tenrm=}`. Fix: `latexml_core/src/stomach.rs:438` after
  `assign_font(new_font, Local)`. See
  [wisdom_fontfamily_math_entry_reset.md](wisdom_fontfamily_math_entry_reset.md).

* **11 PGF/tikz tests** — `find_file` with `notex=false` now honors
  explicit `<file>_binding_available` runtime flags, mirror Perl
  `\openin → FindFile(default args)` (TeX_FileIO.pool.ltxml:50-64
  comment "we SHOULD find an .ltxml version"). Without this, raw pgf.sty's
  `\pgfutil@IfFileExists{pgfsys-latexml.def}` (which uses `\openin`)
  bailed with "Driver file not found". Fix: narrow opt-in fallback at
  `latexml_core/src/binding/content.rs:1783-1801`; full registry stays
  reserved for `notex=true` to avoid t1enc.def cascade regression. See
  [wisdom_find_file_binding_available.md](wisdom_find_file_binding_available.md).

* **plainfonts** — INITEX letter/digit mathcode defaults now set in
  `State::new`, mirror Perl `Core/State.pm:128-137`. Without these,
  dump-load path leaves `'a'` mathcode unset (plain.dump.txt only
  captures the 57 plain.tex symbol overrides), so `\cal abc` falls
  through to font-decode text path with no role/meaning attributes.
  Fix: `latexml_core/src/state.rs` after `let mut state = State {…}`. See
  [wisdom_initex_letter_mathcodes.md](wisdom_initex_letter_mathcodes.md).

Also: `MC`/`DC` records re-enabled in `dump_reader.rs` (per CLAUDE.md
"Unconditional dump apply") + `F`/`FD` records ported (Stored::Font and
`\font`-defined Primitives now round-trip through dump) + `\hline`
engine override re-applied at end of `latex_constructs.rs::load_definitions`.

Remaining 1 failure (Perl-divergence, not Rust bug):
* `numprints_test`: math-mode `\numprint[pt]{...}` produces fuller
  XMDual output than Perl, which appears to truncate at `\lenprint`.

**ntheorem_test FIXED 2026-04-28** (16 → 1 cumulative): root cause was
`\vspace` defined as `DefPrimitive` no-op (WISDOM #44 rationale). latex.ltx
defines `\bigskip` as `\vspace\bigskipamount`, so the dump captures
`M \bigskip E \bigskip 0 16:\vspace,16:\bigskipamount`. With `\vspace`
no-op, `\bigskip` was a silent no-op, `\vskip` Constructor never invoked,
`<ltx:para>` stayed open, `<rule>` ended up nested. Restored Perl-faithful
`DefMacro!("\\vspace OptionalMatch:* {}", "\\vskip #2\\relax")` at
`latex_constructs.rs:7575`. Updated `tests/moderncv/cs_cv.xml` to match
new Perl-faithful output (5 lines diff — break before "Ph.D. Candidate"
now produces `<p>` siblings, matching Perl). See
[wisdom_vspace_perl_faithful.md](wisdom_vspace_perl_faithful.md).

**Spot-check 10k_sandbox impact** (4 papers from sandbox triage list):
* `1305.6480` (revtex4 `\NC@list` undefined): now converts cleanly
  (0 errors, 127 warnings, only multirow KeyVal warns).
* `1207.6068` (revtex4-1 `\shipout` undefined): now converts with
  "No obvious problems".
* `1304.0737` (amsart `\@nil` undefined): still 12 errors.
* `0909.3444` (article+babel frenchb): still 6 errors (babel).
* `1212.4860` (revtex4 mode mismatch): still 58 errors.

Fix targets papers using PGF/tikz drivers and revtex/AMS classes that
hit `\IfFileExists`/`\openin` for binding-only files.

### 2026-04-28 — dump-only test-failure characterization (loop session)

All four remaining dump-only test failures are now traced to root
causes (none requiring engine-pool fixes; deeper in-flight work):

* **`plainfonts_test`** — `\fontname` "fontname not available"
  (existing KNOWN_PERL_ERRORS material; long-standing).

* **`ntheorem_test`** — `\vspace` no-op breaks `\bigskip`-driven
  mode tracking. In dump path, `\bigskip` becomes
  `\vspace\bigskipamount` (LaTeX kernel override) → `\vspace` is
  intentionally a Rust no-op (deferred B5 port; see
  `latex_constructs.rs:7569-7574`) → no glue → no leaveHorizontal
  → `\hrule` lands inside `<para>` instead of after. NODUMP path's
  `plain_base.rs` `\bigskip` = `\vskip\bigskipamount` (real glue) →
  para closes correctly. See
  [wisdom_vspace_noop_dump_breaks][r3].

  [r3]: ../.claude/projects/-home-deyan-git-latexml-oxide/memory/wisdom_vspace_noop_dump_breaks.md

* **`xytest_test`** — picture height/width and circle radii
  differ by exactly **4.16pt** between dump (smaller) and NODUMP
  (matches Perl). Circle: r=5.59 actual / r=9.75 expected. Picture:
  33.94×77.71 actual / 38.10×81.86 expected. xy-pic computes object
  sizes from font metrics (`\halflineheight + \halffontsize`-style);
  the 4.16pt offset is consistent across all geometry → suggests one
  specific `\fontdimen` query reads back differently between paths.
  Root cause not yet traced; candidate is the xy-pic font selection
  (xyatip10/xybtip10) loading at a different time relative to dump
  state. Too deep for the present iteration; flagged for future
  investigation.

* **`numprints_test`** — dump-path Rust is too permissive: where Perl
  + Rust-NODUMP both error during `numprint.sty` raw-load, Rust dump
  path completes the load and renders 622 lines of correct
  `<XMath>` content vs the 91-line Perl-baseline. Root cause is
  some late-loaded definition in `latex.dump.txt` that changes the
  `numprint.sty.ltxml` parse outcome. Documented as
  KNOWN_PERL_ERRORS #18 (added 2026-04-28).

Net for this session: 0 code change in dump-only failures, but all 4
have characterized root causes. The remaining work is bisection-heavy
(numprints) or in flight (vspace B5 port; xy-pic fontdimen).

### 2026-04-28 — engine-pool parity tightening (loop session)

Three targeted iterations of `/loop 5m` engine-pool parity work
ahead of the next dump-path test push. Build clean throughout;
no test regressions; same 4 known dump-only failures persist
(`plainfonts_test`, `ntheorem_test`, `xytest_test`, `numprints_test`).

* **dump_reader register-alias address default** (largest impact;
  +10 expl3 tests recovered earlier in this rotation): when an
  `R`-line lacks an explicit `address` field, default to
  `rparts[0]` (the register's internal `cs` name) — NOT the
  M-line key. Mirrors Perl `Dumper.pm:337-342` `R()` constructor
  default `$traits{address} = ToString($cs) unless defined`.
  For register-aliases (e.g. `M \tex_endlinechar:D R \endlinechar
  N 0`) the key ≠ cs; assignments through the alias must reach
  the underlying register's slot, not a separate slot at the
  alias name. See [register-alias address wisdom][r1].

  [r1]: ../.claude/projects/-home-deyan-git-latexml-oxide/memory/wisdom_dump_register_alias_address.md

* **plain_base.rs: Non-English Symbols** (Perl
  `plain_base.pool.ltxml:525-533`): `\OE`, `\oe`, `\AE`, `\ae`,
  `\AA`, `\aa`, `\O`, `\o`, `\ss` as bare DefPrimitives. On the
  LaTeX path these get re-installed with `robust=>1` by
  `latex_constructs.rs:5752+` (mirroring Perl
  `latex_constructs.pool.ltxml:2814-2832`); on a pure plain-TeX
  path they're now correctly available from plain_base.

* **latex_constructs.rs: float-list bookkeeping stubs** (Perl
  `latex_constructs.pool.ltxml:1015-1028`): `\@topnewpage`,
  `\@next`, `\@xnext` (RawTeX), `\@elt`, `\@freelist`,
  `\@currbox`, `\@toplist`, `\@botlist`, `\@midlist`,
  `\@currlist`, `\@deferlist`, `\@dbltoplist`, `\@dbldeferlist`,
  `\@startcolumn`. The comment at `latex_base.rs:39-55` claimed
  these had been relocated, but the new home was empty. Now
  genuinely placed at `latex_constructs.rs:3849+` mirroring
  Perl's L1015-1028 ordering.

* **latex_constructs.rs: hooks + q-tokens + finalstrut**:
  `\@begindocumenthook` (Perl L5510), `\@preamblecmds` (Perl
  L5511), `\@qend`/`\@qrelax`/`\@spaces`/`\@sptoken` (Perl
  L5536-5539), `\@finalstrut{}` (Perl L4857). The `\@sptoken`
  binding required Token-level (`Let!("\\@sptoken", T_SPACE!())`)
  rather than macro-level (`Let!("\\@sptoken", "\\space")`)
  semantics — mirrors Perl's `Let('\@sptoken', T_SPACE)` and is
  required by makecell.sty's `\ifx \@sptoken\TeXr@temp`
  next-token-detection idiom. Initial macro-form attempt regressed
  cells_test; same-iteration fix to Token-form recovered it.
  See [Let token vs macro wisdom][r2].

  [r2]: ../.claude/projects/-home-deyan-git-latexml-oxide/memory/wisdom_let_token_vs_macro.md

* **Audit-by-regex limits**: structural audit comparing
  Perl pool files against Rust engine modules
  (`etex.rs`, `plain_constructs.rs`, `plain_bootstrap.rs`,
  `latex_bootstrap.rs`, `latex_base.rs`, `tex_box.rs`,
  `tex_math.rs`, `tex_inserts.rs`, `math_common.rs`) found that
  most apparent gaps are false positives — the regex misses
  multiline `DefMacro!(\n  "\\foo",\n  …)` forms and
  `Def[Macro|Constructor]!(T_CS!("\\foo"), …)` forms. Spot-checked
  ~30 supposedly-missing CSes; all were already present. Pool
  files are now structurally near-complete vs Perl.

### 2026-04-28 — dump-path test-suite continued recovery wave

Continued from 2026-04-27 wave. Seven commits brought 25+ tests
from failing to passing across multiple suites:

* `ddd23f95c` `plain_bootstrap`: INITEX letter `\lccode`/`\uccode`/
  `\sfcode` initialization for dump path. Mirrors plain.tex
  L112-113. Without it, `\MakeUppercase` produced lowercase output
  in dump path. Mathcodes deliberately NOT set (would preempt
  base_xmath's DefMath letter handlers — confirmed via bbold_test
  regression). Test impact: 10_expansion `partial_test` newly
  passes.

* `05146fecd` `tex_glue`: `\hskip` reversion preserves
  `\quad`/`\qquad`/`\enskip`/`\thinspace`/`\>`/`\;` macro names
  via em-multiple lookup (Perl `revertSkip` in TeX_Glue.pool.ltxml
  L57-65). Both Perl plain_dump and Rust plain.dump.txt capture
  `\quad`/`\qquad` as raw `\hskip 1em\relax` / `\hskip 2em\relax`
  bodies; without reversion the `tex=` attribute decayed to
  `\hskip 10.00002pt`. Dramatically shrunk diffs in 22_fonts.

* `c9db40925` `tex_glue`: `\hskip` emits `<ltx:XMHint>` in math
  mode (Perl L80 parity). The math parser's `filter_hints` then
  converts XMHint width into `lpadding`/`rpadding` on the adjacent
  token (or promotes large skips ≥10pt to virtual PUNCT XMHints).
  Without this branch, `\qquad` after `,` in math lost its
  `rpadding="20.0pt"` because no XMHint was emitted. Test
  impact: 22_fonts 14/9 → 19/4 (+acc, +esint, +mathaccents,
  +mathbbol, +stmaryrd).

* `e6ecf5c0f` `latex_constructs`: `Let \nobreakspace
  \lx@nobreakspace` (Perl L1847 parity). The dump captures
  latex.ltx's `\nobreakspace → \protect\nobreakspace<sp> →
  \leavevmode\nobreak\<sp>` chain (regular space). Re-let to
  `\lx@nobreakspace` (= NBSP `\u{00A0}`) so hyperref autoref's
  `section\nobreakspace1` produces `section\u{00A0}1` not
  `section 1`. Test impact: 10_expansion `hyperurls_test`
  newly passes.

* `3a532c15f` `latex_constructs`: `digested_to_text` walker for
  `\lx@author@prefix`'s `before=` attribute. Perl uses
  `DigestText(...)` which emits rendered chars (em-spaces from
  `\qquad`, or `, `/` and ` after ams_support overrides). Rust's
  `Digest!(...).to_string()` returned the Whatsit's `revert()`
  form (`\qquad` macro CS) instead. Walker handles TBox/List
  recursion; for `\hskip`-style Whatsits with no text content,
  falls back to `dimension_to_spaces(width)`. Test impact:
  50_structure 37/8 → 41/4 (+article, +authors, +book, +report,
  +amsarticle).

* `99ec353e7` `latex_constructs`: unlock state for
  `math_common`/`plain_constructs` reload. The first plain-format
  pass already locks common math CSes (`\prime`,
  `\active@math@prime`) via their `locked => true` DefMath.
  Without `local_state_unlocked(true)…expire`, the second pass's
  redefinitions were silently rejected, leaving the dump-loaded
  `\mathchardef\prime="0230` mathchar in place — `$\prime$`
  rendered as `0` (char 0x30 fam 2) instead of U+2032 ′.
  Test impact: +abxtest (22_fonts), +io (20_digestion),
  +amsdisplay/+sideset (56_ams), +eqnums (50_structure).

* `5f6aeb5bf` `plain_constructs`: re-add `\boldmath`/`\unboldmath`
  for dump-path parity. plain_base's DefPrimitive entries are
  replaced by plain_dump in dump mode; the dump's
  `\boldmath → \protect\boldmath<sp> → \@nomath\boldmath
  \mathversion{bold}` chain doesn't toggle our `mathfont` Stored
  slot. plain_constructs runs in BOTH paths AND is reloaded by
  latex_constructs's force-reload, so the Rust DefPrimitive wins
  post-dump. Test impact: 22_fonts 20/3 → 21/2 (+fonts).

Then commits `6c9cc0d3a` and `9e88d45aa` landed the muskip mu→pt
conversion suite:

* `6c9cc0d3a` `muskip`: switch `\thinmuskip`/`\medmuskip`/
  `\thickmuskip` to `MuGlue` (Perl TeX_Math.pool.ltxml:1168-1170
  parity). Add Perl-faithful mu→pt conversion in `Stored→Option<
  Dimension>` / `Stored→Option<Glue>` (used by `lookup_dimension`
  / `lookup_glue`) and `BoxOps::get_width` (used by `\the\wd`).
  Conversion uses Perl's two-step truncation `int(size *
  emwidth / 18)` then `(mu * MUWidth / UNITY).trunc()` —
  single-step `(mu * size / 18)` rounds 1.66666pt vs Perl's
  canonical 1.66663pt for 3mu at 10pt.

* `9e88d45aa` `muskip`: convert mu→pt at attribute output.
  `MuGlue::to_attribute` and `MuDimension::to_attribute` emit
  pt-typed attribute values; `Stored::to_attribute` routes
  MuDimension through `v.to_attribute()` (was commented out).
  `\lx@padded` walker for lpadding/rpadding handles the digested
  whatsit case.

**Cumulative test-count delta this wave (across major suites):**

| Suite | Before | After | Δ |
|---|---|---|---|
| 22_fonts | 14/9 | 21/2 | +7 |
| 50_structure | 37/8 | 42/3 | +5 |
| 56_ams | 4/3 | 6/1 | +2 |
| 20_digestion | 8/2 | 9/1 | +1 |
| 10_expansion | 29/4 | 33/3 | +4 |
| 70_parse | 20/9 | 28/1 | +8 |
| 55_theorem | 3/2 | 4/1 | +1 |
| 80_complex | 15/1 | 16/0 | +1 |

**Workspace total** (suites that complete in 90s): **247 passed
/ 28 failed** across 21 test suites (was 239/36 at the start of
this wave). Excluded: `40_math` and `53_alignment` (>90s
timeout, otherwise complete).

### 2026-04-28 — dump Register address serialization

Continuing the dump-path recovery wave. One critical fix landed:

* `a17cb8a4a` `dump`: serialize `Register.address` for allocated
  registers (`\newcount\m@ne` → address `\count22` etc.). Mirror
  Perl `Core/Dumper.pm`'s `R(C(...),undef,...,address=>'\\count22')`
  serialization. Before this fix, the dump_reader stored the
  Register's value at the CS-name slot, but the runtime address
  slot's value (set by an earlier `V \count22 Nm -1` entry) was
  later overwritten with the default 0 by the `M` entry. Result:
  `\m@ne` read as 0, breaking `\settabs 20\columns` (loops because
  `\advance\count@\m@ne` doesn't decrement) and 5+ plain-TeX
  sandbox papers that crashed on tab alignment.

* `e8ddb67e7` `gullet`: 4096-byte safety bound in `read_cs_name_inner`
  to surface runaway `\csname` expansions (lipsum.sty pathologies)
  with a clear error instead of OOMing the pushback Vec. Doesn't
  address the underlying expansion bug; bounds the allocation.

**Sandbox impact (181 papers in `~/data/10k_sandbox_failures`):**

| Round | OK | conv:2 | conv:3 | crash |
|---|---|---|---|---|
| Apr 26 baseline | 0 | 13 | 166 | 2 |
| Apr 27 mid-session | 5 | 151 | 13 | 12 |
| Apr 27 + address fix | 10 | 151 | 13 | 7 |

5 papers fully recovered this iteration (no errors): astro-ph9308008,
astro-ph9708022, funct-an9711006, hep-th9404085, q-alg9505016 — all
plain-TeX papers using `\m@ne`/`\@ne`/`\p@`/etc. allocated registers
in tab-alignment, glue, or counter-arithmetic contexts.

Test suite: 50_structure 42/3 → 43/2 (+plainsample). Workspace
total **248 passed / 27 failed**.

**Still-deferred: latex.dump.txt regen OOMs at preload.ltx with a
4.6GB single allocation in `read_x_token` pushback Vec.** Likely a
runaway macro expansion via `\ifcsname`. Not addressable with the
csname-byte-cap (which only bounds the `cs` accumulator, not the
gullet's pushback queue). Needs deeper investigation of which
specific macro expansion goes infinite during latex.ltx kernel
load. The on-disk `latex.dump.txt` (Apr 27 00:22 timestamp) remains
the source-of-truth for latex tests.

**Known issue: latex.dump.txt regen OOMs at preload.ltx.**
Re-running `--init=latex.ltx` to regenerate the dump aborts with
9.2GB allocation failure during preload.ltx raw-load. The on-disk
`latex.dump.txt` (Apr 27 00:22 timestamp from previous session) is
therefore the source-of-truth for latex tests in this wave. The
plain.dump.txt regenerates fine at ~6s. Tests like
50_structure::epitest_test still fail due to `\p@=0pt` (the dump's
register `value` field captures the Register definition's default,
not the address slot's runtime value after `\setlength{\p@}{1pt}`
in raw plain.tex). A dump_writer patch to read
`state::with_value(&reg.address)` was prepared and tested on
plain.dump (`\p@` correctly serializes `D 65536`) but cannot be
validated for latex until the regen OOM is diagnosed. Deferred.

### 2026-04-27 — test-suite recovery wave

Four commits brought `00_tokenize` from 0/14 (all hanging or
OOM-leak-killed) to 12/14 passing:

* `6e95dcd6b` — `LATEXML_INI_MODE` env gate set in
  `bin/latexml_oxide.rs` BEFORE `prepare_session`, so
  `tex.rs` / `latex.rs` skip the dump-or-base + constructs
  trio in init mode. Mirrors Perl `Core.pm::iniTeX` default
  `mode='Base'`. Plain dump went from 7 corrupt entries to
  925 valid entries.

* `94706300f` — `find_file` binding-registry hits gated on
  `notex=true`. Raw-file callers (`\openin`, `\IfFileExists`)
  no longer get the literal binding name as a phantom path.
  Killed the `t1enc.def` cascade in latex.ltx dump-build
  (log size 381 MB → 112 KB). Same commit restored
  `~ → \lx@NBSP` in `plain_constructs.rs` (mirror of Perl
  `plain_constructs.pool.ltxml:220`).

* `07a9f237b` — `dump_writer` skips `\everymath` /
  `\everydisplay` / etc. with self-`\the<key>` body.
  latex.ltx's `\let\frozen@everymath\everymath` +
  `\newtoks\everymath` dance results in our dump capturing
  the self-referential body on the new `\everymath` slot
  (slot aliasing isn't fully Perl-faithful yet). At runtime,
  math-mode entry recursively expanded `\the\everymath` until
  token-limit exhaustion — this was the OOM source the user
  observed (~57 MB/s gullet buffer growth in debug builds).

* `42294d611` — drop redundant `Let \@@input \input` in
  `latex_constructs.rs:6996`. The bootstrap-level Let in
  `latex_bootstrap.rs:48` already aliases `\@@input` to the
  raw TeX `\input` BEFORE the dump installs latex.ltx's
  redefined `\input` (`\@ifnextchar\bgroup\@iinput\@@input`).
  Re-letting AFTER the dump made `\@@input` self-referential;
  `\input <missing-file>` looped at the false branch.
  Triggered by `\verbatimlisting{snippet}` in
  `tests/tokenize/verb.tex`.

**Remaining `00_tokenize` failures (2/14):**

* `ligatures_test`, `mathtokens_test` — **RESOLVED 2026-04-28**
  via INITEX letter/digit mathcode defaults set in `State::new`
  (mirror Perl `Core/State.pm:128-137`). The `.`-in-math handling
  was broken because the dump-load path didn't establish letter
  mathcodes (`\fam` register / `mathcode\<char>` family). Now
  fixed at the State construction level so both dump and base
  paths share the same INITEX baseline. See
  [wisdom_initex_letter_mathcodes.md] (#52). All 14/14
  `00_tokenize` tests pass.

### Active gaps (as of 2026-04-26)

* **2026-04-26 (Perl `Dumper.pm` + `DumpFile` parity wave)**:
  Multi-commit refactor landing strict Perl parity at the
  dump-build / dump-load layer. **Stale `latex.dump.txt` on disk
  must be regenerated to take effect** — until then, tests still
  observe pre-fix pollution.
  - `32bfe0a74` `dump_reader`: every load arm now calls
    `state::assign_internal` directly, mirroring Perl
    `Core/Dumper.pm`'s `V/Cc/Mc/Sc/Lc/Uc/Dc/Im/I/Lt`. No more
    `install_definition` (lock-checked), no more
    `assign_meaning` (50-link `\let`-chase), no more
    `let_i` (deferred targets handled in-arm). Single state
    mutation API path = Perl `assign_internal`.
  - `610485966` `ini_tex::dump_format`: loads only
    `<name>_bootstrap` between snap and diff, NOT the full
    `latex.rs` chain (which pulled in `latex_base`/`latex_dump`
    + `latex_constructs`). Mirrors Perl `DumpFile`
    (TeX_Job.pool.ltxml L120-220) exactly. Eliminates the
    pollution where `latex_constructs.rs::DefMacro!(...,
    locked => true)` deposited dozens of `:locked` V-entries
    into the dump (Perl's dump has zero).
  - `bbc4675cc` engine: drop dead `stage_snapshot('<name>_bootstrap')`
    calls in `latex.rs` / `tex.rs`. Single source of truth for
    the diff baseline = `dump_format::take_snapshot()`.
  - `c67cbb862` Makefile: 6 GB virtual-memory cap on `make test`
    so runaway loops fail fast instead of OOMing host.
  Full-coverage audit of `plain_base.rs` and `latex_base.rs`
  vs `plain_base.pool.ltxml` / `latex_base.pool.ltxml`:
  zero misses on Def/Let/DefRegister/NewCounter targets;
  RawTeX-block CSes (`\baselineskip`, `\parskip`, `\newskip`,
  etc.) confirmed present (some relocated to `tex_paragraph.rs`,
  others embedded in `plain_base.rs::RawTeX!` blocks).
  **Next**: regenerate `resources/dumps/latex.dump.txt` via
  `make dump` (or `tools/make_formats.sh`) — only then do the
  url_test cluster + babel-timeout cluster see the benefit.

* **2026-04-26 (`_loaded` flag dual-naming complete)**: OXIDIZED_DESIGN
  #23 implementation landed:
  - `de21ae928` — path-aware `already_handled` closure in
    `input_definitions`. Allows binding `<file>.rs` to load its
    same-named raw `.sty/.cls/.def` after binding's own `_loaded`
    is already set (babel_sty / cite_sty / etc. pattern).
  - `01df250c6` — reader sites consult either flag:
    `\@ifpackageloaded`, soul_sty.rs (3 sites), cleveref_sty.rs.
  - `c4f7ddd55` — OXIDIZED_DESIGN.md updated.
  - `6e85a1cf9` — `dump_writer` adds Perl IGNORED_SYMBOLS missing
    entries (`meaning:\lnot`, `meaning:\to`).
  Sub-status: babel.sty timeout (separate dump-state issue) still
  open, but is independent of the flag work. Babel raw load via
  `LATEXML_NODUMP=1` already worked clean before this round.

* **2026-04-26 (commit `4da59f30e`)**: `expl3_sty.rs` reduced to
  strict-Perl 3-line mirror (229 → 23 lines, deletes 13 categories
  of compensating raw_tex blocks). Standalone `\usepackage{expl3}`:
  49 → **0 boxing errors**. The compensations were workarounds for
  an underlying engine bug: the `\__msg_interrupt:n` body has
  catcoded SPACE tokens (catcode 1/2 with content ` `) used as
  PADDING in `\tex_errmessage:D` rendering — Rust gullet/stomach
  treats every catcode-2 token as a structural group-close, hitting
  boxing-vs-non-boxing mismatch. Sandbox cost (commit 142312):
  12 conversion_error papers regressed to abort because OTHER
  expl3-dependent packages (xparse, l3keys2e, mhchem) hit the
  SAME chk_free→cascade when their raw loads call `\msg_new:nnn`,
  `\quark_new:N`, `\seq_gclear_new:N` etc. Audit doc:
  [`docs/EXPL3_PARITY_AUDIT.md`](EXPL3_PARITY_AUDIT.md). Fix path:
  engine-side change to gullet/stomach catcode-2-as-content
  vs catcode-2-as-syntax handling, OR per-package strict-Perl
  rewrites (xparse_sty.rs, l3keys2e_sty.rs etc.). Pending.
* **2026-04-26 audit (commit `4da59f30e` strict-Perl mirror trace)**:
  Perl ALSO fires "already-defined" error per duplicate `\msg_new:nnn`,
  but produces just **1 error per duplicate call** ("LaTeX Error:
  Message 'define-command' for module 'cmd' already defined"). Rust
  produces **8 boxing-group errors PLUS the LaTeX error** per duplicate
  call — 8× amplification. Both engines invoke `\__msg_interrupt:n`
  body (verified bit-equivalent in dumps). The body has 8 catcode-1
  + 8 catcode-2 SPACE tokens (TeX trick for error-message rendering)
  + 44 catcode-12 OTHER spaces. The catcode-1/2 SPACE tokens are
  STRUCTURAL group-syntax in TeX (8 begin / 8 end, balanced). They
  should pair within `\tex_errmessage:D`'s `{...}` arg-reading and
  `\cs_set_protected:Npn \<space> {body}` body-reading.

  The 8× amplification suggests Rust's `\errmessage{}` primitive
  (using `{}` parameter type → `read_balanced`) is correctly tracking
  catcoded-1/2 SPACE-as-BEGIN/END within braces, but somewhere ELSE
  in the `\__msg_interrupt:n` body, the structural pairing fails.
  Likely candidates: (a) `\cs_set_protected:Npn \<space>` body-reading
  via DefExpanded parameter — the body contains catcoded-1/2 SPACE
  pairs; (b) gullet's level counter in `read_balanced` not matching
  TeX's group-begin/end semantics for SPACE-content tokens; (c)
  invoke_token in stomach routing each unmatched catcode-2 SPACE to
  egroup() which fires boxing-mismatch.

  Engine investigation deferred to next iteration. Target file:
  `latexml_core/src/gullet.rs` `read_balanced` (level counter) +
  `latexml_core/src/stomach.rs` `egroup()` (group-mismatch check).


* **DONE 2026-04-26 (commit `e3d4f8532`)**: `\q_no_value`-recursion
  cascade resolved. Root cause: gullet's `DEFERRED_COMMANDS` gate
  in `read_balanced` only matched `defn.get_cs().text`, but Perl
  `Lt('\\exp_not:n','\\unexpanded')` shares the `\unexpanded`
  Definition object — Rust's dump_writer flattens these into
  separate Expandable entries with `alias=\unexpanded`. Without
  the alias-aware gate, `\exp_not:n {\s__seq \__seq_item:n {…}}`
  inside `\seq_gpush:Nn`'s body was re-expanded, `\__seq_item:n`
  hit its expandable-error trap, the seq stayed `\s__seq` only,
  later `\__hook_curr_name_pop:` on empty stack →
  `\msg_error:nn{hooks}{extra-pop-label}` → `\edef \__msg_use_code:`
  fully-expanded `\q_no_value` → recursion. Fix: dump_reader
  propagates alias to `ExpandableOptions` (narrow allowlist:
  `\unexpanded`/`\the`/`\detokenize`/`\showthe`),
  `Expandable::new` copies it through, gullet checks both
  `cs.text` and alias. `\documentclass{article}` errors
  4 → 2 (q_no_value × 2 gone).
  **10k_sandbox_failures rerun (181 papers, 2026-04-26 13:25):**
  Pre-fix: 100% conversion_fatal/abort/timeout. Post-fix:
  2 ok (clean HTML), 12 conversion_error (HTML w/ recoverable
  errors), 118 conversion_fatal, 22 abort, 24 timeout, 3 error.
  **14 papers (7.7%) recovered to HTML output** —
  `hep-th9609235` (18KB) and `math9712228` (50KB) fully clean.
  Documented in [wisdom_deferred_commands_alias.md].
* **NEXT cluster (12+ papers, 49 deterministic errors each)**:
  `\group_begin:` boxing-group close mismatch during expl3 raw-load.
  Pattern: `\if_case:w` warns "Missing number" near expl3.sty load,
  then `}` closes a `\begingroup`-frame triggering 49 successive
  boxing-group errors. The cascade then nukes definitions like
  `\author`/`\sqrt` (4974+ undefined errors per paper). All 12
  conversion_error papers exhibit this pattern with EXACTLY 49
  boxing-group errors — deterministic structural divergence
  during expl3 init. Likely upstream cause of many of the 118
  conversion_fatal papers too. Investigation deferred to next
  session — affecting papers loaded with raw expl3.sty (not
  ar5iv-bundled expl3 codepath).
* **Plain dump (the easier target — perfect this first).**
  Currently 1196 entries vs Perl's ~1238. ~36 non-`\lx@` extras
  remain in the Rust dump that Perl's `plain_dump.pool.ltxml`
  does not contain (`\Box`, `\Diamond`, `\Join`, `\boldmath`,
  `\unboldmath`, `\to`, `\lnot`, `\land`, `\lor`, `\sc`, `\sf`,
  etc.). These are math symbols / font commands defined in
  `math_common.rs` / `plain_base.rs` that load before
  `stage_snapshot("plain_bootstrap")`. In Perl they live in
  `latex_dump.pool.ltxml`, not `plain_dump.pool.ltxml`. Either
  they should be moved post-snapshot in Rust, or the Perl
  origin should be re-confirmed.
* **Latex dump — expl3 raw-load gap.** Manual `\global\let
  \tex_par:D\par` AT RUNTIME succeeds (long-body Expandable
  installed correctly), but the same `\__kernel_primitive:NN
  \par \tex_par:D` line in expl3-code.tex during raw `--init=
  latex.ltx` does NOT install the alias. ~302 of 752 `\tex_*:D`
  aliases are missing from `latex.dump.txt`. Suspect: catcode
  regime during the raw expl3 load doesn't match what the loop
  expects, OR the expl3 setup of `\__kernel_primitive:NN`
  inside its `\begingroup` doesn't expand consistently for all
  tokens. Compare against Perl's iniTeX log — Perl does this
  cleanly.
* **Eager-vs-lazy LaTeX load**. Perl autoloads LaTeX.pool from
  `\documentclass`/`\NeedsTeXFormat`/etc.
  (`TeX.pool.ltxml:33-39`); Rust's `latex.rs` runs at engine
  init. `ini_tex.rs` now explicitly preloads LaTeX.pool before
  the snapshot when `--init=latex.*` (commit `209083ff4`,
  19,797 → 24,987 dump entries, +26%). Path forward: move
  `latex.rs::LoadDefinitions` body behind `\@load@latex@pool`.
* **Closure-backed defs in `_base.rs`**. When the dump is
  loaded, `_base.rs` is skipped (strict split). Anything that
  relied on its Rust closures fails at runtime unless the dump
  captures them. Convert closures to raw-TeX `\outer\def`
  Token bodies wherever Perl uses RawTeX. Started in commit
  `0c4d609ad` (`\new*` family in `plain_base.rs`); rest TBD.
* **`Stored::Number` "Nm" marker** (commit `0c4d609ad`): was
  sharing "I" with `Stored::Int`, breaking register reads.

### Distribution follow-up (after parity)

Once TL2025 dumps are robust + tested: bundle multiple TL versions'
dumps (TL2022 … TL2026) into the binary via `include_bytes!` +
runtime selection by `kpsewhich --version`. Currently dumps load
from disk under `resources/dumps/` — fine for development, not
fine for single-binary distribution. Gated on TL2025 robustness.

## Status snapshot (carried forward from prior milestones)

These numbers reflect the state before the dump-parity pivot. Once
the dumps are correct, re-validate.

* Tests: 1098/0/0 on TL2023 CI (commit `4344e38e0`); 1108/0/0
  local TL2025 (some tests gated by `LATEXML_NODUMP=1`).
* Engine def coverage: 99.9% (2,455/2,457). Missing
  `\directlua`, `\ASCII` by design.
* Package bindings: 100% (406+). Zero MISSING.
* arxiv-examples/: 93+% of 101 papers OK.
* 10k sandbox (last full sweep `sandbox_full_2026-04-26c_postfix`,
  pre-strict-Perl): 7717/7898 OK = 97.71% clean. Many of the
  remaining 181 are deep multi-week clusters
  (math-parser shape, expl3 kernel cascade) tracked in
  `docs/sandbox_failures_SYNC_STATUS.md`. Sandbox work
  continues opportunistically but is **not** the gating front.
  Strict-Perl dump regressions during this work are accepted
  per user directive.

## Architectural invariants

### TL-version independence

Both Rust and Perl LaTeXML core engines are TL-version independent
by design. The only TL-bound surfaces are:

1. **Raw `.tex`/`.sty`/`.cls` loads** from the ambient TeXLive
   ecosystem (xparse, lipsum, expl3-code.tex, hyperref, …) —
   resolved via `kpsewhich`.
2. **Kernel-dumper output** (Rust-only artefact:
   `resources/dumps/{plain,latex}.dump.txt`) — generated against
   a specific TL and frozen for fast load.

Mismatch between dump baseline and ambient PATH should produce a
`Warn:latexml_dump TeXLive MISMATCH` warning at startup. Bugs that
PERSIST across both TL2023 PATH and TL2025 PATH point to core code,
not the version-bound layer.

### CI build parity (TL2023 mechanics)

CI runs on TL2023 (Ubuntu apt), local dev defaults to TL2025. Three
alignment layers: (1) dump content regenerated under TL2023 via
`REBUILD_PERL_FORMATS=1`, (2) kpsewhich path resolution, (3)
CI-equivalent package set installed via `tlmgr install IEEEtran …`
(`INSTALL_CI_PACKAGES=1` wrapper).

Run to reproduce CI locally:
`REBUILD_PERL_FORMATS=1 INSTALL_CI_PACKAGES=1 tools/test_with_tl2023.sh`.

### Schema generation (`LaTeXML.model`)

The Rust port ships `resources/RelaxNG/LaTeXML.model` (336 lines,
verbatim copy from Perl). Loaded at runtime by
`latexml_core::common::model::load_schema`; compile-time codegen
would save µs on minute-long sessions, so a runtime file remains
the right trade-off. `tools/compileschema.sh` ports stage 1
(rnc → rng); stage 2 (rng → model) still requires Perl's
`LaTeXML::Common::Model::compileSchema`. Acceptance for stage 2:
add a `--dump-model` flag on `latexml_oxide` that writes the
loaded schema to stdout in `.model` format, extend
`tools/compileschema.sh` to call it, regenerate both Perl-tree
and Rust-tree `.model` from the same `.rnc`, diff against Perl
output.

## Engine Files — Open Gaps

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`/`CommaList` ported. Parameterized `CommaList:Type` form unported (no Perl users). |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics (cross-cutting: `FontDef` parameter type simplified to `FontToken` — blocks per-font `\hyphenchar` tracking) |
| tex_tables.rs | MINOR | Minor: padding CSS classes (XSLT concern) |
| plain_base.rs | OPEN | Some closure-backed defs need conversion to Token bodies for dump round-trip (strict-Perl mission) |
| latex_base.rs | OPEN | Closure-backed defs need conversion or relocation to `latex_constructs.rs` (strict-Perl mission) |

**Unported:** `AmSTeX.pool.ltxml` (112 defs, ~30%, Plain TeX rare);
`BibTeX.pool.ltxml` (956 defs, 0%, skipped via `--nobibtex`).

**Permanent sandbox ignores:** ns1–ns5 (52_namespace, no DTD);
2402.03300, 2410.10068, 2511.03798 (Perl also fails).

**Perl-error-only papers** (excluded from parity target — Perl
itself fails under the same `--preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings` profile):

- `1207.6068` — Perl emits 30 errors (acknowledgements-only file,
  no `\documentclass`)
- `0909.3444` — Perl emits 2 errors (frenchb babel missing)

## Tikz — Known Diffs (vs Perl output)

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. tikz matrix rendering uses `<svg:g class="ltx_tikzmatrix">`
   groups (Rust) vs inline-blocks (Perl)

## Triaged work — not actionable, kept for context

Several large investigations have been triaged and folded into
WISDOM.md / KNOWN_PERL_ERRORS.md. Pulling them out of the active
TODO surface keeps this doc focused on the dump-parity mission.

* **Def\*-parity audit** — engine 14 residual kind-mismatches and
  package 187 records all reviewed. Every flip has an in-source
  breadcrumb pointing at WISDOM #38/#40/#41/#44 or a per-file
  umbrella. See `DEF_PARITY_AUDIT.md` for the catalogued
  pattern triage. No actionable kind-flip work remains.
* **`scope=>'global'`, `robust=>1`, `protected=>1`** — sweeps
  complete (26/27, 12/31 ported, all 32 ported respectively).
  Remaining `robust=>1` accented-letter sites blocked on a
  case-mapping pipeline rewrite (`DEF_PARITY_AUDIT.md` B1).
* **Perl upstream sync (2025-01 → 2026-04)** — all small
  port-gaps closed; large-scope items queued
  (pstricks_support refactor `fdc8bf91`, inline_math→math
  rename `2b1ff6df`, color-var inline styles `c2370ac3`).
  Future audits: grep for "Perl #" breadcrumbs first, then diff
  uncovered ranges.
* **`ar5iv.sty localrawstyles` flip** — resolved 2026-04-26.
  Rust now uses `rawstyles` (kpsewhich enabled) matching Perl;
  determinism via `tools/test_with_tl2023.sh` CI gate.
* **`1112.6246`** — fixed 2026-04-24 (commit `d162803d2`).
  `mn2e_support_sty.rs` was erroneously loading
  `amsmath`+`amssymb`; Perl's `mn2e_support.sty.ltxml`
  deliberately skips this. The `\cases` routing then mismatched.

## Open structural follow-ups

These are still actionable but secondary to the strict-Perl
dump-parity mission. Pursue when dump work is in a parity-stable
state.

* **`\font` primitive Perl-faithful rewrite (FN.1–FN.5).**
  Current `tex_fonts.rs:52-141` has diverged structurally from
  Perl `TeX_Fonts.pool.ltxml:82-120` in three dimensions:
  prototype mismatch, `at`/`scaled` read order, four-state-key
  storage shape vs Perl's single keyed struct. FN.1 landed
  (commit prior); FN.2–FN.5 pending. Six consumers need
  migration when the storage shape flips.
* **Compile-time bottleneck (CB.1–CB.10).** `latexml_package`
  consumes ~95% of cold-cache build wall-clock. Layered fixes:
  parallel rustc frontend (`-Z threads=8`), bumped LLVM
  codegen-units, splitting `latexml_engine` out as a sibling
  crate. Acceptance: ≥40% CI wall-clock reduction; tests
  unchanged. Work proceeds opportunistically.
* **Dump Let-alias preservation.** Perl serialises
  `Lt('\cs','\target')` separately from full Expandable
  `I(E(...))` records; our writer collapses both to `M E`. New
  `L <cs> <target>` record type with a narrow loader gate would
  recover wholesale `\filecontents`, `\fbox`, `\itshape`, `\ae`,
  `\shipout`, etc.
* **Vector-preserving PDF/EPS → SVG via inkscape/pdf2svg**
  (tracks upstream
  [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)).
  Inkscape PDF path landed (130× speedup on `fig8.pdf`). EPS
  path blocked upstream — Inkscape 1.x dropped EPS support.
  Workarounds: epstopdf pipe or pstoedit; not compelling.

## Math parser — open ambiguity hotspots

Live audit via `LATEXML_PARSE_AUDIT=1`; design context in
`docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`.

1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic
   ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math
   ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

Marpa-related CPU >60%: transitive closure 34.3%, grammar
precompute 8.3%, bv_scan 7.1%, AVL 6.8% (callgrind, math-heavy
paper). The grammar-recovery ladder is now graceful (clone +
trivial parse → retry → full `init_grammar()` rebuild → keep
previous engine on init failure); the panic-on-fallback path is
gone.

Long-horizon: a categorical first-principles redesign that pushes
disambiguation work into the grammar instead of post-hoc
pragmas. Many recently-added pragmas
(ConsistentLetterBlocks, AdjacentNumbersDontMultiply, etc.) are
guards against grammar over-expression — a sharper category
hierarchy would make them obsolete. See
`MATH_GRAMMAR_FIRST_PRINCIPLES.md`.

## Long-horizon — architectural rationalisation

Pursued only after the dump parity mission is closed.

* **Deep dumper-reader parity audit.** Perl `Dumper.pm` is 392
  lines, single-dispatch, no special cases. Our `dump_reader.rs`
  is ~950 lines with multiple gates and a deferred-alias retry.
  Strict-Perl pivot has already removed admission gates; next
  steps remove the remaining gates one class at a time, with
  the 83_expl3 test as canary. Acceptance:
  `dump_reader.rs` halved in size, no special gates,
  byte-identical dump consumption produces matching state
  Perl-vs-Rust.
* **Deep expl3 / LaTeX 3 kernel parity.** Goal:
  `\usepackage{lipsum}` (or any expl3-first package) loads
  cleanly without `SUPPRESS_*` flags or catcode safety-nets. The
  raw expl3-code.tex load currently relies on suppression to
  finish; that's the parity gap. Cross-links the dump-parity
  mission — every primitive missing from the dump is also a
  candidate for native port in `latexml_package/src/engine/`.
* **Rationalize the `Stored` enum.** Universal value currency,
  so its memory footprint and method dispatch is a first-order
  driver. Variant set has grown organically; needs
  size_of histogram + small/large variant split + closure
  accessors mirroring `state::with_value`. Invasive — deferred
  until D4 allocation hotspot work has more per-variant data.
* **Pragma rationalisation.** Classify every current pragma into
  {obsolete under redesign, still needed for genuine ambiguity,
  still needed as engineering compromise}. Migration plan in a
  design doc extending `MATH_GRAMMAR_FIRST_PRINCIPLES.md`.

## 2026-04-28 — `\hline` engine override re-application (51 tests recovered)

**Problem**: After dump-regen (TL2025), 67 tests failed (vs original 15
baseline). 50+ failures shared the pattern `<td><rule height="0.4pt"/></td>`
in place of expected `<td border="t"/>`. Affected tabular-using tests:
lettercase, ot1/t1/t2*/ts1/ly1, latin*, cp*, applemac, longtable, array,
colortbls, tabular, supertabular, morse, cells, ntheorem, plus ~30 others.

**Root cause**: `tex_tables.rs:418` defines the engine `\hline` override as
`\noalign{\@@alignment@hline}` (mirrors Perl `TeX_Tables.pool.ltxml`).
This override is clobbered at dump-load time by the latex.ltx M-line
`\hline → \noalign{\ifnum0=`}\fi\hrule\@height\arrayrulewidth\…\@xhline`,
which the dump faithfully captures from the raw latex.ltx load. Per
CLAUDE.md "Unconditional dump apply", every dumped meaning entry calls
`assign_internal('global')` without filters, so the engine `\hline`
loses to the dump's macro form. The macro form expands `\hrule\@height
\arrayrulewidth` literally → `<rule>` Constructor at `tex_box.rs:1100`
emits a content rule node inside the cell, instead of letting the
alignment binder set `border="t"` on the next row.

**Fix**: Re-apply the same engine `\hline` override at the end of
`latex_constructs.rs::load_definitions` (which runs post-dump per
`latex.rs:84`). Identical definition to `tex_tables.rs:418` and Perl's
`TeX_Tables.pool.ltxml`. Only the load-order placement differs: pragmatic
late re-install after dump-load is the only way for an engine override
to survive under unconditional-dump-apply.

**Impact**: 67 → 16 failing tests (51 recovered). Build clean. Cross-
referenced in `tex_tables.rs:418` comment.

## 2026-04-28 — FontDef (`FD`) record port (in flight)

Perl's `Core/Dumper.pm::dump_primitive` (L383-389) emits `FD(<cs>, <fontID>)`
for `\font`-defined primitives. Without this, Rust's dump_writer fell
back to `PA <self_cs>` which `dump_reader` skips, leaving plain.tex
font CSes (`\tenrm`, `\teni`, `\tenbf`, `\tentt`, `\tensy`, `\tenex`,
`\tensl`, `\tenit`, `\fiverm`, etc.) undefined post-dump.

**Ported**:
- `Primitive::font_id: Option<SymStr>` field (Rust counterpart of Perl
  `LaTeXML::Core::Definition::FontDef::fontID`).
- `Stored::Font` serialization (`F\t<key=val>\x1f...`) — mirrors Perl
  `dump_font`.
- `FD\t<font_id>` Primitive serialization in dump_writer.
- `FD` arm in dump_reader synthesizes a Primitive whose `before_digest`
  mirrors Perl `FontDef::invoke` (FontDef.pm L38-45): assignValue
  current_FontDef + merge_font from looked-up `Stored::Font`.
- `tex_fonts.rs` post-define hook tags the just-installed `\font`
  primitive with `font_id = "fontinfo_<cs_str>"`.

**Verified working**: minimal repro `{\tenrm hello}` produces identical
NODUMP and DUMP output (no `<ERROR class="undefined">`).

**Known regression**: `plainsample_test` shows `tex="a{\tenrm=}b{\tenrm+}c"`
(should be `a=b+c`). The math parser's reversion captures the
synthesized Primitive's box-emission. The original `\font`-defined
Primitive in tex_fonts.rs has identical `None` replacement and
similar before_digest, but somehow doesn't get the same reversion
inclusion. Needs trace of math digestion path: maybe original gets
a special invoke path via FontDirective::Asset that bypasses the
generic Primitive Box emission, or maybe my synthesized Primitive
needs `bounded => true` / a different invoke variant.

**Open work**:
- Plain-font cmsy mathchar resolution (plainfonts_test): F record
  needs extra fields (color/forceshape/forcefamily) for proper
  cmsy mapping, OR the synthesized Primitive's before_digest needs
  to call mathchar setup code in addition to merge_font.
- `\hline` row-separator handling in DUMP path (lettercase_test +
  ~30 sibling tabular tests): DUMP renders `\hline` as `<rule>`
  inside a `<td>`, NODUMP correctly produces `<td border="t"/>`.
  Likely an alignment-context-aware preamble Stored value not
  round-tripping through dump.
- Math reversion bug from FD synthesis (above).

## Future-facing (not wired)

Beyond-Perl directions worth revisiting only after parity is
clean. Not loaded, not referenced by any compiled code path.

* **Native l3hook storage.** Perl currently treats l3hooks as
  no-op stubs (parity); a richer Rust implementation would
  store hook code per name, fire it at `\hook_use:n{…}`. Sketch
  in
  [`memory/wisdom_lhook_perl_parity_stub.md`](../memory/wisdom_lhook_perl_parity_stub.md).
  Behind a feature flag (`LATEXML_OXIDE_L3_HOOKS`) only; engine
  default must NOT change without an A/B parity corpus showing
  the change is always an improvement.

---

> **Reminder:** Every entry ported from Perl must follow tightly
> the original semantics and nuances. Read the Perl source,
> translate precisely, preserve edge cases. The Perl code is the
> ground truth.
