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

### Active gaps (as of 2026-04-26)

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
