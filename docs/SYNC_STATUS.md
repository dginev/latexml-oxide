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

* `ligatures_test`, `mathtokens_test` — both diff on math
  number ligature: `12345.67890` becomes
  `<NUMBER>12345</NUMBER><METARELOP>colon</METARELOP><NUMBER>67890</NUMBER>`
  instead of one `<NUMBER>12345.67890</NUMBER>`. Both pass
  with `LATEXML_NODUMP=1` (raw `latex_base` path); the dump
  path's `.`-in-math handling is broken regardless of dump
  file content (even an empty-body `latex.dump.txt` triggers
  the failure — file existence alone routes to `latex_dump`
  instead of `latex_base`). Likely a missing
  math-character-state initialization that `latex_base` does
  and the dump capture misses. **Deferred to a separate
  investigation.**

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
