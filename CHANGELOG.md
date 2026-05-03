# Change Log

## [0.4.3] (round-19 — 100k canvas REAL-regression-free)

  - **100k canvas mission accomplished**. Staged 10 × 10k validation
    on the `100k_noproblem_sandbox` corpus: **99,774 OK / 100,000 =
    99.77% raw, 0 unfixed REAL_REGRESSION across all 100k papers**.
    Each stage cleared a zero-REAL_REGRESSION gate via
    `parity_check.sh` triage at TIMEOUT_SECS=120+. Per-stage detail
    archived in `docs/archive/round19_iteration_log.md`.
  - **Telemetry foundation complete**. End-to-end per-job phase
    instrumentation: `latexml_core::telemetry` records 17/17 phases
    (Bootstrap, Digest, Build, Rewrite, MathParse, PostXmlParse,
    PostScan, Bibliography, Crossref, Graphics, MathImages,
    MathmlPres, MathmlCont, Split, Xslt, Html5Fixups, Serialize)
    plus a per-formula `math_parse_buckets` histogram.
    `cortex_worker` emits `telemetry.json` into output ZIPs;
    `tools/benchmark_canvas.sh` aggregates to
    `telemetry.jsonl.gz`; `tools/perf_phase_summary.py` and
    `tools/perf_compare.py` consume. See `docs/TELEMETRY.md`.
  - **Cluster fixes** (recovers user-visible papers vs Perl):
    - `\lx@NBSP` / `\lx@nobreakspace` / `\nobreakspace` soft-expand
      inside `\csname...\endcsname` (commit `75a5a42877`) — recovers
      18 papers (Rust beats Perl, ~542 errors total).
    - `\@ifundefined` made globally available via Let to
      `\lx@ifundefined` (commit `5732f3c3b4`).
    - revtex3 `\setdec` / `\dec` no-op stubs (`fe6cbd3a53`) and
      `\CITE → \cite` Let (`0143ad5e59`) — covers ~23 revtex-era
      physics papers.
    - PiCTeX `\putrectangle` 4-numeric-arg gobble stub
      (`3e71dc3f7e`); `\setdots` / `\setdashes` Plain-TeX-compatible
      `\futurelet` dispatch (`0f8475b8a2`).
  - **Robustness / Perl parity**:
    - `MAX_ERRORS=100` default matches Perl's `Fatal('too_many_errors')`
      cap (commit `fc80907932`). Was 10000.
    - `Fatal:invalid:not_tex_source` PDF-magic guard in
      `find_main_tex` (commit `345ace6fb1`) — refuses to convert
      mis-named PDF files.
    - `tools/parity_check.sh` lax `Error:[a-z]+:` regex catches
      inline-error markers; `tools/benchmark_canvas.sh`
      retry-on-transient pass for SIGABRT/timeout under load.
  - **Performance**:
    - `mimalloc` global allocator in `cortex_worker` and
      `latexml_oxide` binaries — measured 3.4× speedup at 16 workers
      (glibc arena-mutex contention fix).
    - `latexml_post::graphics` deduplicates `convert` subprocess
      invocations across `<ltx:graphics>` nodes sharing
      `(source, page, options)` (commit `4a456dc8b0`); also fixes a
      latent layering bug where two distinct option-sets for the
      same source could overwrite each other's destination file.
  - **Cluster-regression integration test**
    (`latexml_oxide/tests/06_cluster_regressions.rs`): pins the
    surpass-Perl wins (NBSP-in-csname, `\@ifundefined`,
    `\setdec`/`\dec`, `\CITE`) as 0-error so future regressions
    fail CI before merge.
  - **Color regression resolved**: reverted the dvipsnames sRGB
    override (commit `66d61be6b7`) after first-principles audit
    found it diverged too far from xcolor's naive cmyk→rgb model
    (which most modern PDF viewers use). The c!p extrapolation fix
    is kept.
  - **Parity-discipline lesson**: documented in
    [`feedback_perl_parity_timeout_handling.md`](.claude/projects/-home-deyan-git-latexml-oxide/memory/feedback_perl_parity_timeout_handling.md):
    `parity_check.sh` 90s timeout can falsely flag REAL_REGRESSION
    when Perl's partial error count is below Rust's. Re-verify with
    `TIMEOUT_SECS=120+` before classifying. Concrete sample:
    0705.0102 reported as REAL at 90s (R=36 vs P-partial=30); at
    120s P=R=36 → SHARED-FAILURE / OUT-OF-SCOPE.

## [0.4.2] (in active development) — strict-Perl dump parity pivot

  - **Status refresh 2026-04-30**: local `cargo test --tests` is
    **1109/0/0**. Runtime dump resources are local/ignored files:
    `plain.dump.txt` 959 lines, `latex.dump.txt` 25,792 lines.
    Latest-row 7898-paper sandbox status is 7731 OK = 97.89%.
  - **rust-analyzer stability profile**: `.vscode/settings.json`
    disables RA proc-macro expansion/cache priming, limits RA worker
    threads, keeps RA output in `target/rust-analyzer`, and excludes
    large/generated trees from file watching.
  - **LaTeX 2.09 `\documentstyle` option-flow recovery**: the old
    shortcut body was replaced with strict-Perl three-branch semantics
    for `.sty` / `.cls` / OmniBus fallback, `@unusedoptionlist`
    handles both string and VecDeque storage, unused options probe the
    compiled binding registry, and class-name probes use version
    fallback.
  - **Strict-Perl `LoadFormat` mutual exclusivity** (commit
    `0c4d609ad`). `tex.rs` and `latex.rs` now mirror Perl
    `Package.pm:LoadFormat` L2734-2752 exactly: `bootstrap → dump
    → constructs` when the dump is on disk and `LATEXML_NODUMP` is
    unset; `bootstrap → base → constructs` otherwise. Replaces the
    older "always run all four" unified design that had been on
    the back burner since 2026-04-18.
  - **`dump_reader.rs` admission gates removed**. Mirrors Perl
    `Core/Dumper.pm` L59-67 — every record calls
    `assign_internal('global')` unconditionally, with no
    skip-if-defined and no `:`-named filtering. Dumps now overwrite
    any prior definition.
  - **`Stored::Number` "Nm" marker** in dump format. Was sharing
    "I" with `Stored::Int`, breaking register reads after the
    strict split skipped `_base.rs`.
  - **`plain.dump.txt` runtime loader** replaces the legacy
    compiled-Rust `plain_dump.rs` (via `dump_codegen`). Matches
    `latex_dump.rs` pattern; resolution paths: `LATEXML_NODUMP`,
    `LATEXML_PLAIN_DUMP_PATH`, `LATEXML_DUMP_DIR`, exe-relative,
    dev-tree.
  - **`ini_tex.rs` LaTeX.pool preload**. `--init=latex.ltx` now
    explicitly loads LaTeX.pool BEFORE the snapshot (commit
    `209083ff4`), mirroring Perl's `make formats` recipe.
    Eliminates the 10000-error abort during expl3-code.tex
    raw-load. `latex.dump.txt` 19,797 → 24,987 entries (+26%);
    zero undefined-CS errors during expl3 load.
  - **Plain dump pollution removed** (commit `1e04a96c8`).
    Autoload triggers (`\documentclass`, `\AtBeginDocument`,
    `\Bbb`, `\align`, …), file-bookkeeping CSes
    (`\@pushfilename`, `\@popfilename`), and early stubs are now
    defined before the init/dump bootstrap snapshot, so they enter
    the baseline and do NOT pollute the dump diff. Historical result:
    plain.dump.txt 1238 → 1196 entries; current local dump is 959
    lines after later cleanup.
  - **`plain_base.rs` `\new*` family** converted to raw `\outer\def`
    Token bodies (commit `0c4d609ad`), matching Perl
    `plain_base.pool.ltxml:207-218` RawTeX block. Required because
    Rust closures aren't serializable through the dump format —
    when the strict split skips `_base.rs`, only Token bodies
    survive in the dump.
  - **Historical active gaps from the Apr 26 pivot** are preserved in
    [`PERL_LOADFORMAT_AUDIT.md`](docs/PERL_LOADFORMAT_AUDIT.md), but
    must be re-audited before action. Several were superseded by the
    Apr 28-30 dump cleanup and package-loading fixes.

## [0.4.1] (in active development)

  - **D0 d.1 complete — dump / `_base` closure-only gap closed from
    32 → 1 CSes** (the single holdout `\wlog` is defined by
    `plain_base.rs` as a closure before the snapshot). Three landings:
    (1) `Expandable::get_num_args` override so E-entries record correct
    nargs; (2) `serialize_stored` handles `None`-body Expandables as
    empty E-entries; (3) `ini_tex.rs` surgically preloads `latex_base`
    after the bootstrap snapshot so its `_base`-only CSes enter state
    before the raw-load.
  - **Dump E-format v2** (new 5th field): full parameter prototype
    serialized per entry via `Parameters::stringify()` so DefToken /
    Optional / Until / Match types round-trip instead of being
    flattened to Plain. Reader gracefully falls back to
    `"{}".repeat(nargs)` when proto fails to parse.
  - **Latent dump-pipeline bug fixes**: (a) `parse_and_load`'s
    `line.trim()` stripped trailing tabs from empty-body E-entries,
    causing `splitn(4)` to report 3 fields and reject the entry;
    (b) `dump_reader`, `dump_loader`, `dump_codegen`, and
    `latex_constructs::\DeclareTextFontCommand` all called
    `parse_parameters(..., false)` which leaves declared Parameters
    with the mock reader ("Missing argument {}" at first use) — now
    all pass `init_flag=true` for runtime paths.
  - **Perl parity sweep** (commits back to 2025):
    #2771 if_count/absorb_count control-counter filter on dump writer;
    #2777 KeyVal empty-macroprefix fallback + empty-keyset skip;
    #2698 aastex revtex4 option is a no-op;
    #2697 DecodeColor Warn on unresolvable name;
    #4e3d1b8d filecontents header prepend "from source" line;
    #aaacdba2 nominal Locator on dump-loaded Expandables + Registers.
  - **archive/TRANSLATION_GAPS.md audit + ports**: verified every section
    against current Rust source with line citations. Three small
    Box.pm helpers (`is_math`, `set_properties`, `total_height`) and
    `fracSizer` from TeX_Math.pool ported. Seven pdfTeX primitives
    added: no-op stubs for `\pdfsavepos`, `\pdfstartthread`,
    `\pdfendthread`, `\pdfnoligatures`, `\pdfsetrandomseed`, `\lpfcode`,
    `\rpfcode`; plus `OpenAnnotSpecification` parameter type +
    `\pdfannot` + `\pdfobj` + `\pdfcolorstack` with full OptionalMatch
    parameter parsing. Section 9 (pdfTeX) now has zero Perl-defined
    gaps remaining.
  - **dump_reader perf**: five-commit sequence cuts allocations across
    the hot dump-load path — unused `_cs_name` decodes in E/R arms,
    no-`%` fast path in `url_decode`, no-`%` fast path in
    `parse_token`, Cow-wrapping the per-line key. Hundreds of thousands
    of Strings avoided per dump load.
  - **Babel parity**: reduced `babel_sty.rs` from 384 → 62 lines (85%) after
    closing the `@currname` leakage bug in our `input_definitions` path
    (plain `\input` now locally saves/restores `@currname`/`@currext`,
    unblocking babel's two-phase `\ProcessOptions*` pipeline). Three
    long-standing D0 items formally closed as a result:
    `\openin`-based `.ini` loading, `\initiate@active@char` active-char
    lifecycle, and AtBeginDocument hook chain ordering.
  - Dump staleness warning at runtime: compares the dump's
    `texlive.version` stamp against ambient `kpsewhich --version` and
    logs a loud warning on mismatch (opt-out via
    `LATEXML_SKIP_DUMP_STAMP_CHECK=1`).
  - `make fresh-test` target regenerates the kernel dump from ambient
    TeX Live before running tests; canonical path for CI.
  - Reduced `todo!()` panics from ~15 to 3 (all deliberate invariant
    asserts on unreachable branches).
  - All clippy warnings fixed; `STAGED_SNAPSHOTS` nested generic type
    factored into named aliases.

## [0.4.0] 2024-09-10
  - The project was refactored to indicate an official `latexml` clone with an `-oxide` suffix.

## [0.3.2] 2024-15-07
  - Handover release, at the end of NIST's sponsorship for this project.
  - Many of the supported internals have been updated to the mainline LaTeXML v0.8.8 logic
  - Passing a lot more tests in `tokenize`, `structure`, `digestion`
  - added compile-time TeX macros
  - Decision: thread-local, global, mutable, singleton `State`
  - more TeX.pool coverage
  - math parsing executable was 

## [0.3.1] 2023-31-05
  - Rudimentary alignment support
  - refactored to use a string-interner

## [0.3.0] 2023-13-03
  - The `expansion` test suite is now passing.

## [0.2.0] 2022-20-04
  - update to 03.2022 state of the mainline LaTeXML test suite
  - unblock math parsing with the inclusion of a Marpa grammar
  - pass most of `tokenize` and `grouping` tests
  - `DefParameter` has an `untokenized` flag that acts as a type designator. Unrealistic ergonomics in Rust. Instead, augment the `reader` paradigm with an optional follow-up closure called `reader_predigest`, which has access to the stomach and can be ran immediately after a `read` is completed. One can still use an `reader_predigest => undigested!()` macro call to allow arguments to pass through digestion untouched.
  - Note: "SEARCHPATHS" no longer needs to be looked up, it's in `state.search_paths`



## [0.1.7] 2018-24-12
  - pass `tokenize/percent` and `tokenize/url` test
  - Much improved `Def*` macro ergonomics since 0.1.4
  - Fleshed out more coverage, cleared some porting bugs in tokenization,
  - in particular `url.sty` and related bits of tex and latex pool files

## [0.1.4] 2018-27-08
  - First optimization release
