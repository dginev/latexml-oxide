# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-20. **Open gaps & active TODOs only.** Completed work
lives in git log and `memory/project_session_history.md`.

**Test inventory:** 423 tests pass (0 failures, 0 ignored) via `cargo test --release --tests`.

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
| base_parameter_types.rs | MINOR | `DirectoryList`/`CommaList`: no Array type in Rust (ported to `{d1}{d2}...` token-stream encoding) |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics |
| tex_tables.rs | MINOR | Minor: padding CSS classes (XSLT concern) |

**Cross-cutting:** `FontDef` parameter type simplified to `FontToken` — blocks full `\fontdimen`, per-font `\hyphenchar` tracking.

**Unported:** `AmSTeX.pool.ltxml` (112 defs, ~30%, Plain TeX rare); `BibTeX.pool.ltxml` (956 defs, 0%, skipped via `--nobibtex`).

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

### D1–D2. Residual sandbox aborts (~30 papers, ~0.4% of 7898)

Three failure classes in the session-128 7933-paper sweep, after the
6 DUPID aborts were addressed:

1. **pgfkeys.code.tex port gap** — 3+ papers (1511.00722, 1611.04489,
   1612.08368). `pgfkeyslibraryfiltered.code.tex` from TeXLive triggers
   `\pgfkeys@non@outer@newif` / `\pgfkeysalso` / `\ifpgfkeyssuccess`
   undefined-CS cascades that loop until the 60s wall-clock caps. Perl
   also loads the raw TeX (via `InputDefinitions('pgfkeys.code', type=>'tex')`)
   but succeeds — divergence is in how our raw-TeX processor handles
   specific pgfkeys idioms, not a simple stub gap.
2. **Math-parser pathological-ambiguity timeouts** — 2+ papers
   (1403.4135, 1407.5769). 500+-token formulas with 121 parse choices
   each, ~500ms per formula × hundreds of formulas = 60s timeout.
3. **Preamble-heavy digestion timeouts** — e.g. 1210.1891 stuck at
   hyperref → etoolbox → kvoptions → nameref chain load.

[ ] **Per-paper diagnosis method:**
1. Run Perl `latexml` with matching `--preload=ar5iv.sty --path=...`; capture log + error count.
2. If Perl errors with the *same* CS → shared document bug, skip.
3. Otherwise apply `wisdom_upstream_error_attribution`: the divergence is
   earlier than the named symptom. Trace `.sty`/`.cls` conditional /
   option / flag / deferred-hook machinery to find the branch Perl takes
   that Rust doesn't.
4. Ensure 423 tests still pass; mark the entry `[x]` with a one-line note.
5. Re-run 12-way parallel sweep after every landed fix to catch cascaded
   benefits and regressions.

### D3. Performance corpus

- [x] Capture Tier A (~10 papers) + `complex/si.tex` as a standing perf
  corpus in `docs/PERFORMANCE.md`. Regression trigger: wall-clock drift
  > 15% on any corpus entry between commits. Reproducer:
  `tools/run_perf_corpus.sh` (idle-serial, no parallelism). Round-17
  baseline is the dated table in `PERFORMANCE.md`.
- [x] **0911.4739** (5.04 s) and **1005.1610** (7.38 s) Tier A
  outliers — **root-caused**: both are Graphics-phase bound (PDF/EPS
  → PNG conversion via the external `convert` CLI from ImageMagick).
  Round-17 per-phase audit with a new `LATEXML_POST_AUDIT=1` env
  flag (commit `8df9c7b53`):

  | paper     | Graphics | MathML | XSLT | Scan | total post |
  |-----------|---------:|-------:|-----:|-----:|-----------:|
  | 1005.1610 | 6614 ms  |  23 ms | 76ms | 30ms |     ~6.8 s |
  | 0911.4739 | 4087 ms  |  29 ms |130ms | 40ms |     ~4.3 s |

  1005.1610 has 10 PDF figures totalling ~7.5 MB; 0911.4739 has 31
  mixed EPS/PDF. Each `convert` subprocess fork-execs ImageMagick,
  rasterises the PDF at density=150, and writes PNG. At 132-660 ms
  per image this is intrinsic rasterisation cost, not a Rust perf
  problem. MathML::Presentation itself is 15-29 ms total across
  365-771 nodes — trivial.

  Use `LATEXML_POST_AUDIT=1 latexml_oxide --post …` to reproduce the
  per-phase breakdown on any paper.

- [x] **Parallelise Graphics phase** — landed `aa3c7c1bb`. 3-phase
  refactor (serial DOM read / parallel subprocess / serial DOM
  write) via `std::thread::scope`, worker cap
  `min(available_parallelism, 8)`. Measured: 1005.1610 Graphics
  6614 → 1665 ms (4×), 0911.4739 4087 → 742 ms (5.5×). Full-pipeline
  wall-clock −66% / −65% on the two outliers respectively.

- [~] **Vector-preserving PDF/EPS → SVG via inkscape/pdf2svg**
  (tracks upstream [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)).

  **Landed round 17** (opt-in, default off):
  - `--graphics-svg-threshold-kb N` CLI flag on `latexml_oxide`
  - `Graphics::with_svg_threshold_kb()` builder on the post processor
  - `convert_image_svg` shells out to `inkscape --export-type=svg
    --export-plain-svg` inside the round-17 parallel worker slot
  - `read_svg_dimensions` parses the root `<svg viewBox="…">` for
    width/height
  - `should_try_svg_path` — file-size heuristic (< N KB, PDF only)
  - Runtime falls back to ImageMagick `convert` when inkscape fails
    / is missing (silent fallback; no hard dependency)
  - README adds `inkscape` to the "Optional" apt install
  - CI (`.github/workflows/CI.yml`) installs inkscape so the branch
    is always covered on PRs
  - `latexml_post/tests/fixtures/cifar10_vector.pdf` (41 KB,
    matplotlib vector plot from issue #902) — the canonical test PDF
  - New `test_vector_svg_graphics_path` integration test: runs the
    Graphics processor on the fixture, asserts an `.svg` file is
    emitted with a `<svg>` root and under 2 MB (silently skipped if
    inkscape is absent — e.g. on bare dev laptops)

  **Empirical classification data (round 17)**:
  - CEP.pdf (30 KB vector): inkscape 0.43 s / 519 KB SVG; convert
    1.4 s / 61 KB PNG — SVG wins on fidelity, PNG on size
  - cifar10_vector.pdf (41 KB vector): inkscape 0.21 s / 60 KB SVG;
    convert 0.20 s / 149 KB PNG — SVG wins on both
  - Fade.pdf (1.7 MB raster-disguised-vector): inkscape 46 s / 102 MB
    (!!); convert 1.4 s / 61 KB — classic case the threshold must
    exclude; the 200 KB threshold we suggest does

  **Still to do**:
  - [x] Timeout on the inkscape subprocess — landed. Default 15 s
    (overridable via `LATEXML_INKSCAPE_TIMEOUT_SECS`). Rust-side
    spawn + poll + SIGKILL in `Graphics::run_with_timeout`; 6 new
    unit tests lock in the timeout/kill semantics, the file-size
    heuristic, and SVG viewBox parsing (also fixed a bug where the
    XML-prolog `?>` was mis-matched as the root-tag close).
  - [x] Benchmark the pathological-convert PDFs from issue #902 —
    landed with `fig8.pdf` fixture from arxiv:1807.01606 and a new
    `test_vector_svg_pathological_convert_case` regression test.
    Measured **130× speedup** end-to-end (32.4 s → 0.29 s via
    inkscape on a minimal doc containing just
    `\includegraphics{fig8.pdf}`). PERFORMANCE.md records the
    validation table.
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
vendor-patch/upstream. Current direct `libxml::bindings::*` call sites:
`latexml_core/src/lib.rs`, `latexml_core/src/document.rs` — 5 total.

- [ ] Route libxml node lifetimes through guardian forbidding unlink without cache invalidation.
- [ ] Replace unsafe-over-FFI with safe wrappers where practical.
- [ ] Migrate the remaining `libxml::bindings::*` callers to high-level
  `rust-libxml` methods; upstream new methods as needed.
- [~] Rc `Can not mutably reference a shared Node "text"` cluster — guard
  raised to 8192 (diagnostic, not safety). dcpic cluster 0805.2376 /
  1007.2309 / 1108.3241 / 1204.5278 all converge now. Lower-priority
  follow-up: identify the semantic cause of high `"text"`-node ref counts
  on dcpic diagrams (2000–8000 range).
- [x] **1605.08055 Finalizing-phase SIGSEGV** — FIXED (session
  128, commit `337c1ef52`). Root cause was a dangling-Node entry
  in `idstore`: upstream passes (math-parser `replace_tree`,
  various `unbind_node()` sites) drop xml:id-bearing subtrees
  without calling `unrecord_id`, so `mark_xmnode_visibility` later
  dereferences a freed libxml2 `Node` when it recurses through an
  `XMRef` whose idref resolves via the cache. Fix: new
  `rebuild_idstore_from_dom()` (clear + fresh DOM walk) called at
  the top of `finalize()` before `prune_xmduals`. 1605.08055:
  SIGSEGV → exit=0 in 0.8 s. No regressions on adjacent papers.
  **Do NOT also call `rebuild_idstore_from_dom` at the start of
  the Rewriting phase** — tried in session 128, broke split_test.
  When the DOM has duplicate xml:ids (rare but possible during
  math-parse), `findnodes` visits in document order so the
  FIRST-OCCURRENCE node wins the cache entry, but the prior
  idstore state may have had the LAST-OCCURRENCE node — which
  some rewrites depend on. Finalize is late enough that those
  rewrites have already fired, so the rebuild there is safe; at
  Rewriting entry it isn't.
  The full audit of every unlink-without-unrecord path remains
  open — this is a belt-and-suspenders fix that makes the
  downstream passes robust to stale idstore entries regardless of
  upstream behavior.

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

- [ ] Audit `.to_string()` (~1900 sites) — replace with `&str` / interned symbols where the value goes into `HashMap<String,String>`.
- [ ] Audit `String::from("...")` literals for interned conversions.
- [ ] Replace `HashMap<String,String>` with `SymHashMap<SymStr>` in hot paths.
- [ ] Audit `.clone()` in `document.rs` (~73), `latex_constructs.rs` (~73), `font.rs` (~39).
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

- [ ] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.
- [x] **Latent no-op pragmas — audit complete (round 17)**. Seven sites
  in `pragmatics.rs` previously matched only `XM::Lexeme("x.invisible_operator", …)`
  for the invisible-times operator head, but `apply_invisible_times`
  produces `XM::Token { role: MULOP, meaning: "times" }`, so they
  silently never fired on real parses.

  Landed:
  - `pragma_consistency_via_key` (session 128, `dfc0f263a`)
  - `pragma_fenced_letters_are_function_arguments` (`b786d85d4`) — 7 tests
  - `pragma_higher_order_invisible_ops_are_exceptions` (`c0c0720b6`) — 4 tests
  - `pragma_adjacent_numbers_dont_use_invisible_times` (`c0c0720b6`) — 3 tests
  - `check_invisible_times_recursive`, `is_invisible_times_apply`,
    `all_simple_identifiers` (`282870c9d`) — 6 tests; also extracted
    module-private `is_invisible_times_op` helper and DRY-replaced
    four earlier inlined match blocks (incl. both
    `pragma_functions_prefer_wider_absorption` sites).

  The MULOP-contains-RELOP check in `pragma_relops_are_outermost` at
  ~L1240 still has a `name == "x.invisible_operator"` OR-fallback, but
  the primary `name.starts_with("MULOP")` predicate already fires on
  the Token shape via `base_operator_name()`, so that branch is
  harmless dead code — left in place. Full workspace: 1090 tests, 0
  fail, 0 ignored.

Remaining semantic-ambiguity hotspots (see
`docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`; live audit via
`LATEXML_PARSE_AUDIT=1`):

1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

### Long-horizon — architectural rationalization

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

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
