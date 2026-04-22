# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-22. **Open gaps & active TODOs only.** Completed work
lives in git log and `memory/project_session_history.md`.

**Test inventory:** 1097 tests pass (0 failures, 0 ignored) via `cargo test --release --tests` across 44 binaries.

**arxiv sandbox:** 101 papers in `arxiv-examples/`. **93+%** catalog OK.

**10k sandbox (session 128 post-idstore-fix):** retried 38 aborts
from the prior 7898-paper sweep with the idstore-rebuild-at-finalize
binary at -j 8 parallel: **15 of 38 now pass** (incl. 1605.08055
SIGSEGV, DUPID borderline timeouts 1505.03876 / 1506.09203 /
1511.07586, pgfkeys-library paper 1511.00722, math-parser paper
1403.4135, and several CPU-contention edges that fit in 60 s at
-j 8 vs -j 12). Projected aggregate: **7884/7898 = 99.82%**
exit=0 at -j 8. Remaining 14 aborts: 5 OOM (exit=137 — 1112.6246,
1203.5977, 1710.03688 babel french, 1711.10191, 1711.11576), 9 at
-60 s timeout (exit=134 — 1210.1891, 1407.5769, 1308.5727,
1611.04489, 1702.00409, 1707.01155, 1709.05096, 1710.11417,
1802.08782). Remaining abort classes: pgfkeys.code.tex port gap
(1611.04489), math-parser pathological-ambiguity (1407.5769,
1308.5727), preamble-heavy digestion timeouts (1210.1891), and a
handful of slow-convergence papers. Runner:
`tools/benchmark_10k.sh`; tool: `cortex_worker --standalone --timeout 60`.

**Engine definition coverage:** **99.9%** (2,455/2,457 Perl Engine definitions ported). Only `\directlua` (LuaTeX) and `\ASCII` (niche) missing by design.

**Package bindings:** 100% (all 406+ Perl bindings ported). Zero MISSING.

**Dump:** 25,172 entries serialized; 6,154 installed into state at load time. Add-only policy preserves engine semantics. Unified load order `bootstrap → _base → dump → _constructs`. `LATEXML_NODUMP=1` opts out.

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational.

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) | [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) | [`PERFORMANCE.md`](PERFORMANCE.md)

---

## Engine Files — Open Gaps

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`/`CommaList` ported (token-stream encoding since Rust has no Array type). `DigestUntil` and `SanitizedVerbatim` landed 2026-04-21. Parameterized `CommaList:Type` form still unported (no Perl users). |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics |
| tex_tables.rs | MINOR | Minor: padding CSS classes (XSLT concern) |

**Cross-cutting:** `FontDef` parameter type simplified to `FontToken` — blocks full `\fontdimen`, per-font `\hyphenchar` tracking.

**Unported:** `AmSTeX.pool.ltxml` (112 defs, ~30%, Plain TeX rare); `BibTeX.pool.ltxml` (956 defs, 0%, skipped via `--nobibtex`).

Active round-17 workstreams: Def*-parity audit (see "DP" section
below) + raw-TeX / expl3 kernel parity (D1-D2 + Long-horizon). Recent
completed landings in git log; this file tracks open work only.

## Tikz — Known Diffs (vs Perl output)

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. tikz matrix rendering uses `<svg:g class="ltx_tikzmatrix">` groups (Rust) vs inline-blocks (Perl)

**Permanent sandbox ignores:** ns1–ns5 (52_namespace, no DTD); 2402.03300, 2410.10068, 2511.03798 (Perl also fails).

**Perl-error-only papers** (excluded from parity target — Perl itself fails under the
same `--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` profile):

- `1207.6068` — Perl emits 30 errors (acknowledgements-only file, no `\documentclass`)
- `0909.3444` — Perl emits 2 errors (frenchb babel missing)

---

## Work Plan — Open TODOs

Phase D0 (2k-sandbox, 84/84) and the test-suite refactor (round 17) are
closed out; per-paper narration and session diaries for sessions ≤128
live in git log and `memory/project_session_history.md`. What remains:

### DP. Def*-parity audit (round 17, in progress)

**Tool:** `tools/audit_def_parity.py` — compares Perl `Def*` vs Rust
`Def*!` kinds pair-by-pair. Baselines tracked in `docs/def_parity_*.tsv`;
batch plan + per-batch progress in `docs/DEF_PARITY_AUDIT.md`.

**Progress (2026-04-22):** engine 52 → 19, package 234 → 232 (minor
false-positive cleanup), contrib 0 (clean). 6 commits on
`claude-round-17`: `e967af9c7`..`e075c4987`. 1097/0/0 throughout.

**Remaining 19 engine mismatches** (by file, with shape):
- `latex_constructs.rs` (11): picture primitives (`\line`/`\vector`/
  `\oval`/`\qbezier`/`\bezier`/`\lx@pic@bezier`, 6) — DefConstructor↔DefMacro
  reversal; `\tabular` DefKeyVal; `\abstract` DefEnvironmentI; + 3
  small.
- `plain_base.rs` (6): `#`/`&`/`%`/`$` mode-aware Box emission + `\mit`.
- `tex_math.rs` (3): `\left`/`\right`/`\mathchar`.
- `base_xmath.rs` (2): `\lx@cases@condition` / `@end@condition` —
  DefConstructorI → DefPrimitive.
- `tex_fonts.rs` (1): `DefLigature` (Rust-only kind; likely intentional
  divergence — mark explicitly).

**Next actionable step:** `plain_base.rs \mit` — simplest remaining
item (Perl `DefPrimitiveI(..., requireMath=>1, font=>{family=>italic})`
is a standard font-primitive shape that Rust `DefPrimitive!` already
supports — see `DefPrimitive!("\\OMX", None, font => { family => "cmex10" })`
at `latex_constructs.rs:5429`).

**After that:** `base_xmath.rs \lx@cases@condition` / `@end@condition`
(2-line DefConstructorI ports), then tackle the larger clusters
(picture primitives need SVG emission; `plain_base.rs` mode-dispatch
needs closure; `tex_math.rs` needs math-mode constructors).

**Package (232 mismatches, batches P1–P9):** deferred until engine is
clean. Top files: caption_sty.rs (32), texvc_sty.rs (30),
physics_sty.rs (22), pgfsys_latexml_def.rs (17), llncs_cls.rs (15),
babel_support_sty.rs (15). P8 cross-file (45 `DefKeyVal→DefMacro`)
requires a real `DefKeyVal!` port.

### D1–D2. Residual sandbox aborts (~30 papers, ~0.4% of 7898)

Three failure classes in the session-128 7898-paper sweep:

1. **pgfkeys / raw-TeX expl3-group leaks.** The
   `\usepackage{lipsum}\usepackage{tikz}` minimal reproducer went
   from 988 errors to 0 after `load_tex_definitions` started
   digesting `\ExplSyntaxOff` at EOF when a raw-.sty load left
   expl3 active (`b0b9852bd`). Expected to clear 1511.00722 /
   1611.04489 / 1612.08368 on next 10k sweep.
   Deeper cleanup tracked in "Deep expl3 kernel parity" under
   Long-horizon — goal: no `SUPPRESS_*_ERRORS`, no catcode safety-
   nets, no EOF-injection workaround. Per user round-17 directive:
   "we want lipsum to load natively and cleanly."
2. **Math-parser pathological-ambiguity timeouts** — 1403.4135,
   1407.5769. 500+-token formulas × 121 parse choices × ~500 ms
   each hits the 60 s wall.
3. **Preamble-heavy digestion timeouts** — e.g. 1210.1891 (hyperref
   → etoolbox → kvoptions → nameref chain).

[ ] **Per-paper diagnosis method.** Run Perl with the same
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` flags;
if Perl errors on the same CS it's a shared bug (skip). Otherwise
the divergence is upstream of the named symptom — trace `.sty`/
`.cls` option/hook machinery. Keep 1097/0 green throughout.

### D3. Performance corpus

Tier A corpus + per-phase audit (`LATEXML_POST_AUDIT=1`) + parallel
Graphics worker landed round 17. Details in `PERFORMANCE.md`. Reproducer:
`tools/run_perf_corpus.sh`. See commits `8df9c7b53` (audit flag),
`aa3c7c1bb` (parallel graphics — 4-5.5× on image-heavy Tier A papers).

- [~] **Vector-preserving PDF/EPS → SVG via inkscape/pdf2svg**
  (tracks upstream [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)).
  Opt-in via `--graphics-svg-threshold-kb N` on `latexml_oxide`. Runtime
  falls back to ImageMagick `convert` when inkscape fails / is missing.
  Timeout: 15 s default, `LATEXML_INKSCAPE_TIMEOUT_SECS`. Benchmarked
  130× speedup on `fig8.pdf` (issue #902). CI installs inkscape.
  PERFORMANCE.md has the validation table. Remaining:
  - [ ] EPS support via the same path. **Blocked upstream**:
    Inkscape 1.x dropped direct EPS/PS reading (relied on
    ghostscript glue that was removed). `inkscape source.eps`
    reports "Failed to open". Workarounds would be either (a) pipe
    through `epstopdf source.eps stage.pdf && inkscape stage.pdf`,
    adding a conversion step + `epstopdf` dependency, or (b) try
    `pstoedit` which can emit SVG directly from PS/EPS. Neither is
    compelling given that EPS files are often already raster-wrapped
    (EPSF-of-a-TIFF style). Leaving EPS on the ImageMagick raster
    path for now.
  - [ ] Consider pdf2svg as a cheaper fallback when inkscape is
    absent but pdf2svg is available — smaller install, simpler
    flags. (Not blocking; inkscape covers the use cases.)


Specific slow-convergence follow-ups:

- [~] **1709.05096** — >90s wall under parallel load (digestion, not
  post-processing). `Info:undefined:… KV:vattach …` scanning pattern
  suggests a keyval loop paying per-row cost in a huge tabular.
- [~] **1710.03688** — OOM at ~19 GB RSS during babel french.ldf load;
  `\bbl@exp@aux` undefined (babel 3.x port gap).

### D3b. Stability — libxml2 node lifetimes

**Policy:** no direct `libxml::bindings::*` FFI calls from latexml. When
a safe API is missing, add it to `~/git/rust-libxml` and
vendor-patch/upstream. **Zero direct FFI call sites remaining** as of
round 17 commit (see below).

(Prior lifetime caveats moved to WISDOM.md #36–37.)

- [~] Rc `Can not mutably reference a shared Node "text"` — guard raised
  to 8192 (diagnostic). dcpic cluster converges. Follow-up: identify
  semantic cause of high "text"-node refcounts (2000–8000) on dcpic.
- [ ] **Optional refinement**: downgrade `rebuild_idstore_from_dom`
  belt-and-suspenders fallback at `finalize()` entry to a debug-only
  consistency probe, now that all audited paths are consistent.

### Dump — deferred alias retry (session 128)

- [ ] `\a → \@tabacckludge` — still hand-written `Let!` in
  `latex_constructs.rs`. The dump serializer captured `\a` as an
  Expandable `E` record (with `\@changed@cmd`-wrapped body), not a
  PA let-alias, so the deferred-alias retry pass added in
  `91c82d5a4` doesn't help. Either (a) teach the serializer to
  detect "let-aliases preserved under _constructs" and emit them as
  PA records, or (b) widen the outer M-gate to admit E records whose
  body is a specific safe `\@changed@cmd` pattern. Deferred for now
  — the hand-written `Let!` mirrors latex.ltx L10007 exactly and
  has no behavioral downside.

### D4. Performance — parallel scaling & allocations

**Baseline (session 105, paper 0707.1173):** 1-worker 22.6s → 16-worker
76.8s (29% per-worker efficiency). 14-core/20-thread machine. Peak RSS
570 MB/process.

- [~] Audit `.to_string()` (~1900 sites) — replace with `&str` /
  interned symbols where the value goes into `HashMap<String,String>`.
  **61 sites converted round 17** (commits `741809e6e` through
  `7a5433cd4`), across 21 files spanning core, math parser, package
  engine, and individual packages (amsmath, xcolor, listings,
  enumitem, thmtools, xy, pgfsys, amscd, hyperref, fontenc, etc.).
  Key tooling added to the core:
  - `Tokens::eq_text` / `Tokens::starts_with_text`
  - `ArgWrap::eq_text` / `ArgWrap::starts_with_text`
  - `Stored::eq_text` / `Stored::starts_with_text` / `Stored::ends_with_text`
  - `state::graphics_paths_contains` (zero-alloc membership)
  Measured payoff (PERFORMANCE.md round-17 second refresh):
  complex/si.tex −9%, several Tier A papers −2-7%, others flat.
  Remaining sites are dominated by `Vec<String>` collections, format!
  results, and other legitimate owned-allocations. Further cleanup
  should be callgrind-guided, not blanket. **Pivoting** — next
  perf work should target Stored-enum rationalisation (separate
  long-horizon task) or Tokens deep-copy reduction.
- [ ] Audit `String::from("...")` literals for interned conversions.
- [ ] Replace `HashMap<String,String>` with `SymHashMap<SymStr>` in hot paths.
- [~] Audit `.clone()` in `document.rs` (~73), `latex_constructs.rs` (~73), `font.rs` (~39).
  **Audit finding (round 17)**: the raw clone counts are misleading.
  - `font.rs` (~39): almost all clones are on
    `Option<Cow<'static, str>>` fields whose common-case variant is
    `Cow::Borrowed("serif")`-style static-literal pointers (3 pointer
    reads, no heap). The `.clone().or_else(|| other.clone())` pattern
    in `make_concrete` / `merge_ref` looks redundant but costs
    exactly one real clone either way.
  - `document.rs` (~71): dominated by `self.node.clone()` patterns
    (24+ sites). `libxml::tree::Node` is `Rc<RefCell<_Node>>` so
    `.clone()` is an Rc refcount bump, not a DOM copy.
  Real optimisation would require consuming-self API overloads and
  an Rc-vs-Arc refcount audit — both invasive. Deferred as
  low-ROI. The D4 allocation-hotspot work should instead chase
  `.to_string()` on interner symbols and `Tokens` deep-copies —
  those are where the real heap churn is.
- [ ] Review `Tokens` cloning — pass `&Tokens` or `Cow` for read-only iteration.
- [ ] Profile math parser RAM independently (Marpa chart, forest).
- [ ] Investigate shared read-only engine state across processes (mmap dump).
- [ ] Long-running daemon / process pool to amortize 570 MB startup.
- [ ] Fork-based parallelism for CoW memory sharing.
- [~] `lookup_value(key)` → `with_value(key, |v| …)` closure refactor
  — 248 initial sites, ~12 converted (mathchar, pin_char, defined_as).
  Remaining: `state.rs` (17), `binding/content.rs` (5), `keyval.rs` (4,
  tricky — return `Option<Stored>`), `binding/counter/dialect.rs` (3).

### D5. Math parser ambiguity

Callgrind (math-heavy paper, session 105): Marpa dominates — transitive
closure 34.3%, grammar precompute 8.3%, bv_scan 7.1%, AVL 6.8%.
Marpa-related >60% CPU.

- [~] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
  **Partial landing round 17.** The fallback path is still needed
  (`testscripts_test` fixture demonstrates grammar-corruption patterns
  that only clear after a fresh precompute), but the recovery ladder
  is now: (1) clone `self.grammar` + trivial parse, (2) retry once —
  covers transient state hiccups without reaching for init_grammar,
  (3) full `init_grammar()` rebuild if both clone attempts fail, (4)
  log + keep previous engine if init_grammar itself errors. Removed
  the `init_grammar().unwrap()` panic — subsequent formula parses
  fail cleanly (0 trees) rather than crashing the whole conversion.
  The "avoid" goal remains aspirational; the real win is graceful
  degradation, not elimination. Still-open refinement: instrument the
  fallback call count on the 10k sandbox and identify the triggering
  grammar cases to see if they are addressable at the grammar level.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.
Remaining semantic-ambiguity hotspots (see
`docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`; live audit via
`LATEXML_PARSE_AUDIT=1`):

1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

### Long-horizon — architectural rationalization

(l3hooks Perl-parity stub caveat moved to WISDOM.md #38.)

- [ ] **Kernel-first discipline for dumper widening** (user directive,
  round 17). Cross-links the dumper audit and the expl3 kernel
  parity tasks. Each failed staircase step in the dumper widening
  is a signal that a specific runtime primitive is missing:

  - Step 4 (1-CS body E records) regressed because the referenced
    CS targets — often `\hook_*`, `\__regex_*`, `\__cmd_*` — are
    not defined by our engine, so the admitted wrapper cascades
    on expansion.
  - Step 5 (all :-named E) regressed for the same root cause at
    scale.

  **Discipline going forward**: every dumper widening step that
  fails must be paired with a targeted Rust port in
  `latexml_package/src/engine/` (or `latexml_core/` if truly
  kernel-level) of the underlying primitive family. The dumper
  and the engine advance together — the dumper can't admit
  records whose bodies call primitives the engine doesn't execute.

  Immediate primitive candidates from step 4/5 body analysis:
  - ~~`\hook_*:n` family~~ — **resolved via Perl-parity stubs**,
    see the completed l3hooks entry above. Not a native port;
    Perl itself doesn't emulate hook storage.
  - `\group_begin:`, `\group_end:` (already aliased to
    `\begingroup`/`\endgroup` via PA, but blocked from the dump
    under the `:`-key gate — see the "Known antipattern" below
    for why they can't be admitted solo)
  - `\exp_args:N*` family (expansion control)
  - `\__kernel_*` internals used by the kernel setup
  - `\cs_new_eq:NN`, `\cs_gset_protected:Npn`, etc. (cs-aliasing
    and definition primitives)

  Targeted port suggestion: after the hook-stub parity fix, retry
  the step-4/5 widening to re-profile which primitive family now
  dominates the cascade. The hook fix is expected to unblock a
  large fraction of PA/MPA records whose bodies called
  `\hook_gput_code:nnn`; what remains should point to the next
  family to port.

- [ ] **Dump writer ↔ reader simplicity audit** (user directive,
  round 17). Separate question from widening: are both the Rust
  writer and reader as simple and universal as the Perl
  equivalents?

  Current state:
  - Rust writer (`dump_writer.rs`, 525 lines): emits tab-separated
    records from `(TableName, SymStr, Stored)` tuples. Schema
    documented in the file header (M/E, M/PA, M/T, M/R, M/N, V,
    C, LC/UC/SC/MC/DC). All state types covered.
  - Rust reader (`dump_reader.rs`, 950 lines): text parser with
    gates, deferred-alias retry pass, type dispatch.
  - Perl writer+reader (`LaTeXML::Core::Dumper.pm`, 392 lines):
    emits **Perl source code**. The dump is a `.pl` file that's
    `require`-d. Each record becomes one sub call — `I(defn)`,
    `V(key, value)`, `Lt(key, target)`, `N(...)`, etc. No parser,
    no gates. Writer and reader share one object model with the
    runtime.

  Coverage parity exists. **Conceptual simplicity parity doesn't.**
  Perl's dump is "code, not data"; ours is "data, not code". The
  data choice forces a reader with gates because the reader can't
  execute what it can't safely parse.

  **Proposed direction** (to evaluate):
  - Option A — keep text format but tighten semantics so the reader
    becomes a flat dispatch table with no gates. Each record's
    safety is the writer's responsibility (don't emit what can't
    load); the reader just applies each record to the state.
  - Option B — swap the text format for generated Rust source
    (compiled via build.rs into a kernel_dump.rs module). Zero
    runtime parsing, structural typing of records. Mirrors Perl's
    code-not-data philosophy; downside is dump regen requires a
    build.
  - Option C — hybrid: keep text for arxiv-sandbox dumps (portable
    across developer machines), compile to Rust for the kernel
    dump that's embedded in the binary.

  Cross-ref: the Deep Dumper-Reader Parity task below focuses on
  runtime behavior (which records load cleanly); this task focuses
  on architectural simplicity of the pair itself.

- [ ] **Deep dumper-reader parity audit** (user directive round 17).
  Parallel to the expl3 task below. Perl's `LaTeXML::Core::Dumper`
  is 392 lines, single-dispatch, no special cases: each dump record
  just calls `assign_internal($STATE, $table, $key, $value, 'global')`.
  Our `dump_reader.rs` is 950 lines with three gates
  (`is_at_internal`, `is_public_register`, `is_safe_let_alias`), a
  deferred-alias retry pass, and explicit rejection of `:`-named
  entries. The gap isn't in the data — both sides speak the same
  tab-separated format. The gap is in the **runtime semantics
  downstream of the reader**: every gate exists because enabling it
  triggered a cascade (undefined-CS recovery loop, infinite expl3
  expansion, etc.). Systematic path to parity:

  **Step 1 — enumerate and classify the current gates**:
  - `is_at_internal` — `@`-named CS, no `:` in key. Admits
    `\@tabacckludge`-style aliases. Should always be safe.
  - `is_public_register` — data starts with `R\t…` (CharDef /
    Register). No hook/cascade risk; always safe.
  - `is_safe_let_alias` — PA/MPA where neither key nor target
    contains `:`. Recovers ~170 plain-LaTeX public aliases.
  - Everything with `:` in the key — blocked. Includes ~8,914 M
    entries and ~156 PA entries in our current latex.dump.txt.

  **Step 2 — attempt narrow widenings, one class at a time**,
  with the 83_expl3 test as the canary. Round-17 confirmed that
  widening PA-with-colon-name-to-non-colon-target alone regresses
  the test into an infinite loop because expl3.sty's own guards
  misfire. PA and `:`-style M entries must be widened together,
  and the expl3_sty.rs short-circuit must know about the new
  coverage.

  **Step 3 — for each regression, identify the specific runtime
  feature missing in core/engine/package** and port it properly:
  - If the symptom is "undefined CS" on a `:`-named expl3
    primitive, port that primitive in `latexml_package/src/engine/`.
  - If the symptom is a hook cascade, understand the hook
    mechanism and port the hook primitives (`\hook_gput_code:nnn`,
    `\hook_use:n`, etc.) natively.
  - If the symptom is expansion looping, trace which forward-ref
    closes the loop in Perl but not in Rust.

  **Step 4 — when a gate is demonstrably not needed (all records
  of that class load safely), remove it**. Target end state:
  `dump_reader.rs` down to ~400 lines, no special gates, single
  uniform dispatch matching Perl's streamlined pattern.

  **Acceptance**:
  1. `dump_reader.rs` line count halved (950 → ~400-500).
  2. All three custom gates removed.
  3. Deferred-alias retry pass no longer needed.
  4. Full dump loads without error; every M/PA/C/LC/UC/SC/MC/DC
     record round-trips cleanly.
  5. Byte-identical latex.dump.txt consumption produces matching
     state between Perl's `assign_internal` path and Rust's.

  Tracks alongside the expl3 kernel parity task — the two share a
  common root (faithful runtime semantics for everything the dump
  can contain). Work here often directly unblocks work there.

- [ ] **Deep expl3 / LaTeX 3 kernel parity** (round 17 directive,
  `58617b6b6` diagnostic). Goal: `\usepackage{lipsum}` — or any
  other expl3-first package — loads cleanly without error
  suppression, catcode safety-nets, or dump-loader gate
  exceptions. Subsequent package loads (tikz, etc.) should
  compose cleanly.

  **Evidence of current gaps**:
  - `expl3_sty.rs` sets `SUPPRESS_UNDEFINED_ERRORS`,
    `SUPPRESS_UNEXPECTED_ERRORS`, and `set_suppress_log_output`
    around the raw `expl3-code.tex` load. Forward-ref errors are
    being swallowed rather than avoided.
  - `expl3_sty.rs`, `xparse_sty.rs`, `l3keys2e_sty.rs` all have
    catcode restoration safety-nets after raw-file loading (space,
    underscore, at, colon). If `\ExplSyntaxOff` worked correctly
    these wouldn't be needed.
  - `dump_reader.rs` blocks every `:`-named PA alias from loading,
    even trivial ones like `\group_begin: PA \begingroup` — reflects
    low trust in the downstream expl3 machinery.
  - 1611.04489's failure cascade (`\group_begin:` on the stack
    persisting through pgfutil-common.tex) is a *symptom* of the
    partial port, not an isolated pgfkeys bug.

  **Acceptance criteria**:
  1. `\usepackage{lipsum}` loads with 0 errors, 0 warnings,
     without the SUPPRESS_* flags or catcode safety-nets.
  2. `\usepackage{lipsum}\usepackage{tikz}` loads both cleanly
     and neither leaves state residue.
  3. `dump_reader.rs` gate can admit `:`-named aliases that point
     at safe targets (trivial primitives, not expl3 hooks) without
     the current "both-non-colon" guard.
  4. arxiv:1611.04489 / 1511.00722 / 1612.08368 convert with the
     sandbox-expected error profile (which matches Perl's — for
     1611.04489 that's 2 warnings, exit=0, 25 s).
  5. Broader sandbox: any paper currently aborting with `\pgfkeys`
     / `\pgfmath` undefined-CS cascades recovers. Session 128
     counted these collectively as one of the three dominant
     remaining abort classes (~8 papers in the 14-abort
     residual).

  Deferred — deep work. Requires a careful expl3-code.tex
  run-through, identifying each forward-ref / hook / cascade that
  currently breaks, and porting enough kernel primitives that the
  raw loading succeeds on its own merit.

  **Known antipattern to avoid** (round-17 experiment):
  widening the `dump_reader.rs` gate to admit `:`-named PA aliases
  like `\tex_let:D PA \let` IN ISOLATION (without also admitting
  `:`-style M entries and adjusting the `expl3_sty.rs`
  short-circuit) regresses the 83_expl3 test into an infinite loop
  / 60 s timeout. The mechanism: `\tex_let:D` gets let-aliased to
  `\let` → `expl3.sty`'s own guard thinks expl3 is "loaded" →
  skips raw `\input expl3-code.tex` → post-guard code hits
  `\__kernel_dependency_version_check:Nn`, `\ProcessOptions`,
  `\keys_define:nn { sys }` which our gate still doesn't define
  → undefined-CS recovery loop. PA widening and M widening must
  land **together**, coordinated with the expl3_sty.rs
  short-circuit logic.



- [ ] **Rationalize pragma / semantics / grammar categories from first
  principles.** Observation from round 17: many of the recently-added
  pragmas (ConsistentLetterBlocks, AdjacentNumbersDontMultiply,
  FencedLettersAreFunctionArguments, HigherOrderInvisibleOpsAreExceptions,
  FlattenSimpleInvisibleTimesChains, …) are downstream *guards* that
  correct for grammar over-expression — they exist because the Marpa
  grammar admits parses that would not even be theoretically
  reachable under a better-factored categorical hierarchy of
  mathematical notation. `xy` as "function application vs
  multiplication of two bare letters", `(x)` as "fenced expression vs
  single-arg call", `f × f` as a flat binary product etc. are not
  genuine mathematical ambiguities — they are artefacts of conflating
  role-bearing categories (operator, function, coefficient, variable)
  at the same lexical surface.

  Near-term (done): enable the pragmas that already encode the correct
  preferences so the forest is narrowed before post-processing.

  Long-term: sit down with `docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md` and
  redraw the boundary between (a) what the grammar produces, (b) what
  semantic enrichment (role, fences, NUMBER/ID/OPERATOR distinctions)
  adds, and (c) what pragmas legitimately prune. The goal is to push
  work *earlier*: if a phenomenon can be made unreachable by a
  sharper category (e.g. "bare letters cannot be operators unless
  lexically marked as such"), the pragma that guards against it
  becomes obsolete and can be deleted. Pragmas should only remain for
  *genuine* semantic ambiguities that notation cannot disambiguate
  — the four remaining hotspots listed above are candidates
  (`\sin[XY]`, `tr ρ`, `FGHa`, `a|a|+b|b|+c|c|`).

  Deliverable: a design doc (probably extending
  `MATH_GRAMMAR_FIRST_PRINCIPLES.md`) that classifies every current
  pragma into {obsolete under redesign, still needed for genuine
  ambiguity, still needed as engineering compromise}, with a migration
  plan for each. Not scheduled — tracked here so the categorical
  rethink isn't forgotten when the near-term pragma list stabilises.

- [ ] **Rationalize the `Stored` enum** (`latexml_core::common::store::Stored`).
  Nearly every step in the core business logic assigns or looks up a
  `Stored` value through the state API — macro bodies, counters,
  graphics paths, keyval stores, registers, font specs, the lot.
  Because the enum is the universal value currency, its memory
  footprint and method-dispatch cost is a first-order driver of
  overall conversion speed and RSS.

  Round-17 observations supporting this:
  - `arena::to_string(sym)` heap-allocates each time — motivated the
    zero-alloc `state::graphics_paths_contains` helper landed today
    (`<round-17>`), but the same waste pattern recurs across every
    caller of `get_graphics_paths` / `get_search_paths` / etc.
  - `Stored::clone()` shows up in callgrind as a non-trivial band
    (session 116-117 already cut it from 1.02% → 0.17% by routing
    hot paths through `state::with_meaning`, but the remaining 0.17%
    is still a per-token cost at conversion scale).
  - The variant set has grown organically (`String`, `Strings`,
    `VecDequeStored`, `Number`, `Dimension`, `Glue`, `MuGlue`,
    `RegisterValue`, `Tokens`, `Expandable`, `Meaning`, …) with
    ad-hoc packing; likely overlap we could coalesce.

  Proposed investigation:
  1. Measure `size_of::<Stored>()` and the histogram of live-instance
    variants on a representative paper (complex/si.tex and a
    math-heavy paper).
  2. Split hot variants into `Copy`-able small forms vs heap-carrying
    forms; consider an enum-with-small-variants pattern
    (`SmallStored` fitting in a `u64`/`u128`).
  3. Add `with_*`-style closure accessors for every variant that
    currently hands out owned data — mirror the `state::with_value`
    refactor done for `lookup_value` (D4 tracks that separately).
  4. Audit method dispatch: many `match`-on-variant call sites can be
    pushed into inherent methods on `Stored` (`as_string()`,
    `as_int()`, `as_tokens()`) with inlined fast-paths.

  This is a high-payoff but invasive refactor — deferred until D4
  allocation hotspot work has more per-variant data to guide the
  redesign. Not scheduled.

### Other structural follow-ups

- [ ] **Dump Let-alias preservation.** Perl serialises
  `Lt('\cs','\target')` (Let alias) separately from full Expandable
  `I(E(...))` records; our Rust dump collapses both to `M E`, which
  forces the loader's safety gate at `dump_reader.rs:177-191` to admit
  *all* public-CS M records (cascades expl3/hook bodies) or *none*
  (misses plain Let aliases latex.ltx relies on, e.g. `\let\a=\@tabacckludge`
  at L10007). Workaround: explicit `Let!("\\a", ...)` in
  `latex_constructs.rs`. Proper fix: new `L <cs> <target>` record type
  with a narrow loader gate. Would recover `\filecontents`, `\fbox`,
  `\itshape`, `\ae`, `\shipout`, etc. wholesale.

### Future-facing / not-wired exploration

The following designs are **intentionally kept out of the active
engine wiring** — they describe beyond-Perl directions worth
revisiting once the parity baseline is cleaner. Not loaded, not
referenced by any compiled code path.

- [ ] **Native l3hook storage** (post-parity beyond-Perl direction).
  Perl's bindings handle l3hooks as no-op stubs (see the completed
  "l3hooks — Perl-parity stub port" entry above). A richer Rust
  implementation would actually store hook code per name, fire it at
  `\hook_use:n{…}`, and let `\AtBeginDocument` / `\AtEndDocument` /
  `\AtBeginEnvironment` route through it. Sketch:

  1. `\hook_new:n{name}` — declare a hook (lazy: first `gput` creates).
  2. `\hook_gput_code:nnn{name}{label}{code}` — `state::push_value("@l3hook:{name}", code)`.
  3. `\hook_use:n{name}` — `state::with_vecdeque` + digest each.
  4. `\hook_if_exist:nTF{name}{T}{F}`.
  5. Parallel: `\AddToHook` → maps to `\hook_gput_code:nnn`, etc.

  **Why not now**: Perl doesn't do this, so adopting it inside the
  core engine risks divergent render output whenever a package
  registers code into a hook that Perl silently drops. Any pursuit
  should be behind a feature flag (e.g. `LATEXML_OXIDE_L3_HOOKS`)
  and validated with a dedicated test corpus, not the parity test
  suite.

  **Placement when pursued**: a new standalone crate or a
  `latexml_package/src/engine/latex_lthooks.rs` module behind a
  `#[cfg(feature = "l3hooks")]` flag. Engine wiring must NOT be
  added to the default path.

  **Prerequisites before any of this is reasonable**:
  - Dumper staircase complete / dump_reader simplified (this
    unblocks the ambient test surface).
  - A purpose-built hook test corpus with Perl/Rust A/B parity to
    show the cases where "store and fire" changes output vs
    "silently drop" — and confirm the changes are always
    improvements, never regressions.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
