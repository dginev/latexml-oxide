# CLAUDE.md

> **This is a Perl-to-Rust translation project.** Every translated entry must follow tightly the original semantics and nuances of the Perl source. Do not invent new abstractions, rename concepts, or simplify behavior unless explicitly marked as an intentional divergence. The Perl code is the ground truth.

## Active priorities (refreshed 2026-07-02): faithful parity + beyond-Perl arXiv runs

Two co-equal targets drive current work:

1. **Faithful translation of the original Perl LaTeXML.** Strict parity at
   the format/dump and package-loading boundary is maintained (the
   strict-`LoadFormat` dump-parity mission is **complete** — zero-error
   inits, dumps match Perl; audit archived at
   [`docs/archive/PERL_LOADFORMAT_AUDIT.md`](docs/archive/PERL_LOADFORMAT_AUDIT.md),
   the ~72-CS Perl-only long-tail residual tracked in `SYNC_STATUS.md`
   "Engine file open gaps"). Ongoing parity work = corpus-driven: mine
   fatal/error clusters from live runs, classify vs same-host Perl
   (`canvas-triage` skill), fix GENUINE-RUST-ONLY divergences faithfully.
   Worklist: [`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md).
2. **Beyond-Perl improvement runs over arXiv.** The production `cortex_worker`
   fleet converts the full ~2.8M-doc arXiv corpus (2026-07 rerun in flight;
   next: the July-5 run, built from a fresh binary with the week's fixes).
   Beyond-Perl levers: performance (17% math over-parse, tikz-cd digest —
   [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) /
   [`docs/ARXIV_PERFORMANCE.md`](docs/ARXIV_PERFORMANCE.md)), reliability
   ([`docs/STABILITY_WITNESSES.md`](docs/STABILITY_WITNESSES.md)), and the
   source-provenance showcase (issues #47/#92,
   [`docs/SOURCE_PROVENANCE.md`](docs/SOURCE_PROVENANCE.md)).

Current verification (tracked in `SYNC_STATUS.md`): `cargo test --tests` is
**1505/0/0**; `cargo clippy --workspace --all-targets -- -D warnings` is clean
(policy in `[workspace.lints]`, gated by CI's `lint` job and the pre-push hook —
`latexml_oxide/build.rs` sets `core.hooksPath`). The 2026-07 full-arXiv rerun
runs at ~44k docs/hr, avg 4.06 s/doc, fatal rate 0.78%.

Durable parity rules (the dump/format boundary):

1. **Strict `LoadFormat` mutual exclusivity** (Perl
   `Package.pm:LoadFormat` L2734-2752). `tex.rs` and `latex.rs`
   take exactly one branch:
   * `bootstrap → dump → constructs` if `<format>.dump.txt` is on
     disk and `LATEXML_NODUMP` is unset, OR
   * `bootstrap → base → constructs` otherwise.
   Never both.
2. **Unconditional dump apply** in `dump_reader.rs`. Mirrors Perl
   `Core/Dumper.pm` L59-67: every record calls
   `assign_internal('global')`. No admission gate, no
   skip-if-defined, no closure guards. The dump WILL overwrite
   any prior definition.
3. **Same-file definitions** as Perl. Every `\foo` defined in
   `LaTeXML/blib/lib/LaTeXML/Engine/<file>.pool.ltxml` must be
   defined in `latexml_engine/src/<file>.rs`. Use raw
   `\outer\def`-style Token bodies wherever Perl uses `RawTeX`,
   so the dump captures them as serializable Token-bodies, not
   opaque Rust closures.
4. **Perl-zero-error parity target**: `--init=plain.tex` and
   `--init=latex.ltx` must complete with **zero errors**, matching
   Perl. Any error during expl3-code.tex / latex.ltx raw-load is
   a parity gap, not a thing to suppress with caps.

The plain dump is the easier target — keep it perfect first, then
tackle latex. Historical test regressions during the dump pivot are
recorded in `SYNC_STATUS.md`; do not assume they are current without
re-running the relevant test or dump-generation command.

**Distribution model — REDESIGNED 2026-06-07 (was: committed dumps,
landed 2026-05-15).** Per-TL-year dump files
(`resources/dumps/{plain,latex}.YYYY.dump.txt` + `texlive.YYYY.version`)
are **NOT committed to the repo**. They are generated at release time by
`.github/workflows/release-dumps.yml` (called from `release.yml` on tag
push, dispatchable standalone): a 5-year moving TL window — currently
2022–2026 — each generated inside a pinned TL-year container
(`ghcr.io/tkw1536/texlive-docker:YYYY`, the image family behind Perl
LaTeXML's CI; 2026 interim: islandoftex `latest` until
tkw1536/historic-texlive-docker#1 publishes). One kpathsea-UNLINKED
dumper binary (subprocess-`kpsewhich` backend) serves all containers.
Each `--init` runs under `LATEXML_INIT_DEBUG=1` with a strict
zero-`Error:`/`Fatal:` gate (init output is suppressed otherwise —
naive grepping sees nothing). The release build then embeds the whole
window at build time (gzip, DEP-12; `latexml_engine/build.rs` scans
`resources/dumps/`). **Dev/CI generate their ambient-year dump via
`tools/make_formats.sh`** — run it once after checkout, after a TL
upgrade, or before test runs needing dumps (CI.yml does). Runtime
resolves the ambient year via `kpsewhich -var-value=SELFAUTOPARENT`
(leading-digit parse, so MacTeX's `2026basic` works) with
`pdflatex --version` fallback (`kpsewhich --version` returns the same
kpathsea-library string on TL2023 and TL2025, so it's NOT a reliable
discriminator). Earlier IA-record consolidation (`81176ba689`) halved
`latex.YYYY.dump.txt` size by collapsing per-slot fontdimen V-records
into per-(font,size) `IA` records with RLE-encoded data.

## Project Overview

latexml-oxide is a Rust port of [LaTeXML](https://github.com/brucemiller/latexml), a Perl tool that converts LaTeX documents into accessible web documents (HTML/XML).

The `LaTeXML/` directory contains the legacy Perl source being ported. Do not modify it — it serves as the reference implementation.
Similarly, the test `.tex`, `.xml` and `.pdf` files often need to be copied from the Perl space to the Rust space.

## Workspace Structure

Cargo workspace with 8 crates:

- **latexml_core** — Core engine: tokenizer (mouth), macro expander (gullet), digester (stomach), document builder, state management, definitions, bindings
- **latexml_engine** — TeX/LaTeX engine modules and kernel state
- **latexml_oxide** — Top-level crate with binary targets (`latexml_oxide`, `latexmlmath_oxide`) and integration tests
- **latexml_package** — TeX/LaTeX package system: compile-time macro engine, package loader, prelude
- **latexml_math_parser** — Math expression parser with Marpa-style grammar
- **latexml_codegen** — Proc macros for compile-time code generation (constructable, modelable, parametrizeable, testable, tokenizeable)
- **latexml_contrib** — User-contributed style packages
- **latexml_post** - post-processing functionality to HTML/MathML/ePub/JATS/... following the core XML generation phase

Supporting directories:
- `resources/` — CSS, JavaScript, RelaxNG schemas, XSLT, Profiles
- `tools/` — Utility scripts (e.g. `compile_metrics.pl`)
- `.githooks/` — Pre-push hook for quality checks
- `docs/` — Internal project documentation (see below)
- `background/` — TeX documentation and code, the original project generating PDF, which LaTeXML emulates and adapts

## Internal Documentation

All active docs live in `docs/`. This is the authoritative index — keep it
current when adding, renaming, merging, or archiving a doc. Grouped by the two
mission targets (docs serving both come first):

**Worklists & contracts (start here when resuming work):**
- **[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md)** — The BRIEF ACTIONABLE worklist for both targets: current status, the in-flight session entry, open tasks, deferred families, stable reference. Completed session logs are lifted to `docs/archive/SYNC_SESSIONS_*.md`. **Start here.**
- **[`docs/RELEASE_CRITERIA.md`](docs/RELEASE_CRITERIA.md)** — The "what must be true before a public 1.0" contract: release gates, binary-size budget, portability staging, license/public-domain audit, distribution safety profile, tail-latency/RSS signals, surpass-Perl policy.
- **[`docs/ISSUE_AUDIT.md`](docs/ISSUE_AUDIT.md)** — Local mirror of open GitHub issues with status + interpretation (refreshed 2026-07-02; 8 open). **Refresh before milestone planning.** (Issue numbers are GitHub-tracker numbers — they do **not** correspond to any internal `#N` in `WISDOM.md`.)

**Target 1 — faithful Perl translation (parity):**
- **[`docs/OXIDIZED_DESIGN.md`](docs/OXIDIZED_DESIGN.md)** — Public-facing design document: architecture decisions, **intentional Perl divergences**, type-system improvements. Read this to check if a translation difference was a marked intentional divergence.
- **[`docs/ORGANIZATION.md`](docs/ORGANIZATION.md)** — Maps Perl engine files (`LaTeXML/Engine/*.pool.ltxml`) to Rust files (`latexml_engine/src/*.rs`). Loading hierarchy and LaTeX chapter structure.
- **[`docs/WISDOM.md`](docs/WISDOM.md)** — Tactical insights about system internals from specialized debugging. Check here to avoid re-introducing known bugs.
- **[`docs/KNOWN_PERL_ERRORS.md`](docs/KNOWN_PERL_ERRORS.md)** — Upstream Perl LaTeXML issues (40 entries). Check here first when investigating a test failure; when a shared bug is simple, fix in Rust and record it here (candidate to upstream).
- **[`docs/DUMP_DESIGN.md`](docs/DUMP_DESIGN.md)** — Design record for the kernel dump precompilation (strict-Perl LoadFormat mutual exclusivity, unconditional apply) — the live architecture behind the per-TL-year release dumps. NOTE the format-layering nuance: the latex format sits on the REAL-plain.tex layer (Perl's is hand-curated), so plain-only macros can leak into latex sessions (the `\+` class, retracted at the `latex.rs` seam; audit in SYNC_STATUS 2026-07-02).
- **[`docs/BINDING_DSL_ARCHITECTURE.md`](docs/BINDING_DSL_ARCHITECTURE.md)** — Decision record for the binding-definition DSL: one shared `ConstructorBuilder` lowering spine, compile-time `macro_rules!` + runtime Rhai front-ends. Subsumes closed issues #93/#171.
- **[`docs/script_bindings_plan.md`](docs/script_bindings_plan.md)** — The runtime (Rhai) `script-bindings` front-end reference. OFF by default (`script-bindings` feature).

**Target 2 — beyond-Perl improvement runs over arXiv:**
- **[`docs/ARXIV_PERFORMANCE.md`](docs/ARXIV_PERFORMANCE.md)** — Living empirical performance campaign over arXiv: slowest-100 testbed, corpus-wide profiles, phase rollups, optimization log; records settled dead-ends.
- **[`docs/PERFORMANCE.md`](docs/PERFORMANCE.md)** — Timeless optimization principles, open/closed lever state, and the dated **Audit log** of periodic perf passes.
- **[`docs/STABILITY_WITNESSES.md`](docs/STABILITY_WITNESSES.md)** — Living worklist of reliability witness papers (timeout/OOM/peak-RSS/hang) with current-binary + Perl baselines. Distinct from `SYNC_STATUS.md` (correctness errors).
- **[`docs/CORTEX_WORKER_HARNESS.md`](docs/CORTEX_WORKER_HARNESS.md)** — `cortex_worker --harness` fleet orchestration: one-conversion-per-process, five-layer memory guards, crash-loop backoff, production deployment recommendation. Companion to pericortex `docs/HARNESS.md` and CorTeX `MANUAL.md` §7.
- **[`docs/TELEMETRY.md`](docs/TELEMETRY.md)** — Per-job structured telemetry schema for `cortex_worker` runs.
- **[`docs/SOURCE_PROVENANCE.md`](docs/SOURCE_PROVENANCE.md)** — Design for the prioritized beyond-Perl showcase: live source ↔ preview over a shared locator substrate (ar5iv-editor + VSCode clients), accurate linting (#47) and Rust-grade author errors (#92). Locators opt-in (`--source-map`). (The landed-but-deprioritized `--server` LSP docs: [`docs/archive/LSP_SERVER.md`](docs/archive/LSP_SERVER.md), [`docs/archive/LSP_MULTIFILE_PLAN.md`](docs/archive/LSP_MULTIFILE_PLAN.md); smoke `tools/lsp_smoke.py`.)
- **[`docs/RELEASING.md`](docs/RELEASING.md)** — Tag-driven release procedure; the self-contained-binary requirement.
- **[`docs/SAFETY.md`](docs/SAFETY.md)** — Threat model and `unsafe` inventory (distribution posture in `RELEASE_CRITERIA.md` §6).
- **[`docs/SCHEMA_DOCUMENTATION.md`](docs/SCHEMA_DOCUMENTATION.md)** — RelaxNG Compact schema → rustdoc-styled HTML doc site (supported the closed #199 HTML-dialect schema).

**Math parser (serves both targets):**
- **[`docs/MATH_PARSER_AND_ASF.md`](docs/MATH_PARSER_AND_ASF.md)** — Canonical: the three-stage ambiguity pipeline vs the Marpa ASF traversal paradigm. Read before touching `latexml_math_parser/src/parser.rs::parse_string` or `semantics.rs::Actions`. Companion to [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md).
- **[`docs/MATH_PARSER_ASF_TIEBREAKING.md`](docs/MATH_PARSER_ASF_TIEBREAKING.md)** — ASF tie-breaking rules detail.
- **[`docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`](docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md)** — Design rationale for the Marpa grammar.
- **[`docs/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`](docs/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md)** — Measured and-node counts per ambiguity pattern; ranked open levers (`f(x)` apply-vs-multiply, bare-`|x|` pre-lexer, integral Step 2). The top `math_parse` lever for the arXiv runs; supersedes the archived 2026-05-21 ambiguity audit.

**Open dated diagnostics** (point-in-time studies with pending halves — see naming rule):
- **[`docs/ARXIV_FORK_AUDIT_2026-07-03.md`](docs/ARXIV_FORK_AUDIT_2026-07-03.md)** — due-diligence survey of the arXiv/LaTeXML "velocity fork" (57 commits): what's already translated (pgfmath core, usebbl, calc params, hyperxmp, acmart ARIA) vs the ranked residual candidates (abstract-in-TOC inlist, pgfmath string results, graphicx actualtext).
- **[`docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`](docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md)** — `expected:id` dangling-XMRef cluster: container-id half landed; content-branch/MathFork reconciliation still pending.
- **[`docs/EXPL3_CATCODE_GAP_2026-06-08.md`](docs/EXPL3_CATCODE_GAP_2026-06-08.md)** — expl3 catcode-gap study: still OPEN (deep); records four attempted fixes that all regressed and were reverted.

Completed/superseded snapshots live in `docs/archive/` (see
[`docs/archive/README.md`](docs/archive/README.md) — most recently, the
2026-07-02 consolidation archived the 2026-06 session logs, the BibTeX port
plan, the 2026-05-21 ambiguity audit + sandbox-triage workflow, the 3-sandbox
fatal analysis, and the startup-cost study). Single-paper reproducers /
out-of-scope cases live in `docs/reproducers/`, `docs/out-of-scope/`,
`docs/known_crashes/`.

**Rules for these docs:**
- `KNOWN_PERL_ERRORS.md` is for Perl-origin issues only. Include minimal trigger examples.
- `WISDOM.md` is for tactical system insights — record when specialized analysis leads to a correct patch.
- Rust-specific error fixes go in `SYNC_STATUS.md` under "Rust Error Fixes", referencing the KNOWN_PERL_ERRORS entry when applicable.
- When an upstream Perl error is identified, record it. Fix in Rust if simple; otherwise keep as-is.
- **Diagnostic-snapshot naming.** Docs that capture a point-in-time technical diagnostic — `*_TRIAGE`, `*_HOTSPOTS`, `*_AUDIT`, `*_ANALYSIS`, `*_BISECT`, and similar — **must carry a date in the filename** (`NAME_YYYY-MM-DD.md`), using the date of their last commit. This keeps a study from masquerading as a live worklist. *Living* worklists are exempt even when their name reads like a diagnostic — date only what is a frozen snapshot. (When such a worklist's mission *completes*, date it and move it to `docs/archive/`, lifting any live residual into `SYNC_STATUS.md` — as was done for the LoadFormat audit.)
- Keep this index current. When a diagnostic snapshot is superseded, archive it under `docs/archive/` rather than leaving it orphaned at the top level.

## Skills (`.claude/skills/`)

Reusable workflows that encode this project's hard-won judgement — invoke with
`/<name>` or let them surface by description. They wrap the `tools/` scripts with
the *rules* (what verdict to trust, what trap to avoid):

- **`canvas-triage`** — decide if a failing paper is a genuine Rust bug or parity
  with Perl. Encodes the non-negotiables: never downgrade errors to cheat,
  fail-safe toward flagging failure, classify Perl with **verbose not `--quiet`**,
  ANSI-strip before grepping, same-host Perl only, and "the cortex DB is a screen,
  not ground truth."
- **`min-repro`** — reduce a confirmed failure to a minimal reproducer/fixture.
- **`perl-port`** — faithfully translate/fix a binding: read the Perl source
  first, use `ORGANIZATION.md` for placement, check `WISDOM.md` /
  `KNOWN_PERL_ERRORS.md` / `OXIDIZED_DESIGN.md`, and obey the divergence policy.
- **`perf-check`** — measure correctly, pick the right profile, and don't
  re-litigate the settled dead-ends (PGO/target-cpu = no gain; startup dump-parse
  lever = declined).

## Build & Test

Requires **Rust nightly**.

We follow Rust best practice with four named profiles in `Cargo.toml`:

| Profile | Use | Tuned for |
|---------|-----|-----------|
| `test`  | `cargo test` / `cargo run` / `cargo build` (default = `dev`/`test`) | Maximum debug info, debug-assertions, overflow-checks, incremental rebuilds. **All local development and triage** — the only profile to use day-to-day. |
| `ci`    | `cargo test --profile ci` (only used in `.github/workflows/CI.yml`) | Lowest RAM (16 GB GitHub Actions runner) and fastest compile. `opt-level = 0`, `codegen-units = 256`. |
| `release` | `cargo build --release` / `cargo run --release` | Strong-optimized binary tuned for our 32 GB / 20-thread laptop. `opt-level = 3`, `lto = "thin"`, `codegen-units = 20`, `strip = "symbols"`. Used for **sandbox sweeps and Perl-parity measurements**, NOT distribution. |
| `maxperf` | `cargo build --profile maxperf` | **Distribution / publish-grade artifact**. Inherits release, plus `lto = "fat"`, `codegen-units = 1`. Slowest build, smallest + fastest binary. **Reserved for shipping a stable state.** |

**Day-to-day development**: use the default `test` profile via `cargo test` / `cargo run` / `cargo build` (no flag). Full debug info, line-table backtraces, debug-assertions, overflow-checks. Best diagnosability when something fails. CI is *not* what local dev should mimic; CI is RAM-bounded and stripped.

**Sandbox runs**: build `cortex_worker` in the default profile and pass that path to `tools/benchmark_canvas.sh` via `--worker-bin`, OR build with `--release` once if you specifically need a publish-grade canvas measurement.

**Publish-grade measurement** (matching against Perl LaTeXML, baseline updates in `docs/PERFORMANCE.md`): use `--release`. The CI profile is for the GitHub runner only.

**Distribution build** (shipping the binary to users): use `--profile maxperf --no-default-features --features runtime-bindings` for the smallest, fastest artifact that still ships the Rhai script-bindings capability. Example: `cargo build --no-default-features --features runtime-bindings --profile maxperf --bin latexml_oxide`. The `--no-default-features` flag drops the `test-utils` feature (removing `phf` + `glob` and 4 transitive crates), while `--features runtime-bindings` keeps the runtime contributed-bindings front-end — runtime opt-in, so default conversions are unaffected (this is the recipe `tools/make_release.sh` uses). The `maxperf` profile uses `panic = "abort"` — production-only since canvas sweeps depend on `catch_unwind` for per-paper panic isolation.

```bash
# Run all tests (default test profile)
RUST_BACKTRACE=1 cargo test --tests -- --nocapture

# Convert a formula (default test profile, fast incremental rebuild)
cargo run --bin latexmlmath_oxide -- '1+1=2'

# Convert a document (default test profile)
cargo run --bin latexml_oxide -- latexml_oxide/tests/hello/hello.tex

# Triage a sandbox failure (test profile, full backtraces)
tools/triage_failure.sh <arxiv_id>

# Publish-grade measurement build (sandbox sweeps, Perl-parity)
cargo build --release --bin latexml_oxide

# Distribution build — smallest, fastest artifact (slow build, fat LTO,
# panic=abort, no test-utils; keeps runtime-bindings)
cargo build --no-default-features --features runtime-bindings --profile maxperf --bin latexml_oxide

# Generate docs
cargo doc --workspace --no-deps --open
```

**Important:** A compile-time plugin discovers test suite files. When adding a new `[name].tex` / `[name].xml` test pair, run `cargo clean` to force rediscovery.

## Code Style

Formatting is configured in `rustfmt.toml`:
- 2-space indentation (`tab_spaces = 2`)
- Max line width: 100
- Edition 2021, style edition 2024

Enable linting hooks:
```bash
rustup component add rust-analyzer rustfmt clippy --toolchain nightly
git config --local core.hooksPath .githooks/
```

Rust-analyzer stability: this workspace's `latexml_codegen` proc
macros can make RA loop and allocate large amounts of RAM. The
checked-in `.vscode/settings.json` intentionally disables RA proc-macro
expansion/cache priming and excludes `target/`, `LaTeXML/`, generated
HTML, sample corpora, and dumps. Keep terminal `cargo` as the source of
truth for macro-expanded diagnostics.

## Architecture Notes

**Math parser:** The Rust rewrite uses a highly ambiguous Marpa grammar (replacing Perl's Parse::RecDescent).
  - The new goal is to be highly ambiguous in parsing, but aggressively prune in the semantics rules, so as to minimize the final parses.
  - Math-related details (XMDual, delimited expressions, etc.) should be translated faithfully, keeping in mind the difference between Parse::RecDescent and the Marpa approach.

- **State** is a thread-local, global, mutable singleton (see CHANGELOG 0.3.2 decision)
- Uses a **string interner** for efficient symbol handling
- TeX macro definitions can be compiled at compile-time via proc macros in `latexml_codegen`
- **No DTD support** — the Rust port only supports RelaxNG schemas. DTD-based document tests (namespace ns1–ns5, xii) are permanently ignored. The `DocType!` macro has been removed; `RegisterDocumentNamespaces!` handles namespace registration only.
- The port aims to be **faithful to the Perl original** while using idiomatic Rust where possible
- **Self-contained, portable binary** (design requirement): a conversion must not *read* latexml_oxide's *own* resources from disk during its main operation. Engine dumps, the RelaxNG schema, and XSLT/CSS/JS are embedded and served from memory (verified: XSLT via `strace`, dumps by renaming the dev-tree `resources/dumps/` away and still converting). *Writing* outputs — including auxiliary files — into the **destination** directory is fine. New code that adds a runtime read of an *owned* resource must instead embed it (`include_bytes!` / `include_str!`). The host **TeX Live ecosystem is out of scope**: reading `.sty`/`.cls`/`.tfm` from the user's texmf tree via `kpathsea` is allowed and expected. Official releases ship the `maxperf` binary as a GitHub Release Asset, runnable with no `resources/` tree. Full rationale in [`docs/OXIDIZED_DESIGN.md`](docs/OXIDIZED_DESIGN.md) → Guiding Principles.
- Test files (`.t` extension) mirror the original LaTeXML Perl test suite; `.rs` files are the Rust equivalents
- most tests are regression-oriented. They contain a complete TeX input, and can experience failures in many different intermediate stages.
- we are interested in finding meaningful Rust types for the previously untyped Perl.

## Intentional divergences from Perl

- **`%\n` not emitted**: Rust does not emit `%\n` (TeX comment-newline line-break separator) in `tex` attributes. When copying test XMLs from Perl, strip all `%&#10;` occurrences. This is a no-semantic-content formatting artifact.
- **Source comments off by default**: `INCLUDE_COMMENTS` defaults to **false** in the Rust binary (Perl defaults true), so source `%` comments and the `%**** <file> Line N ****` progress markers (emitted every 25 lines) are suppressed in the output. Deliberate — debugging noise, no semantic content. `--comments` restores Perl's behavior. When diffing vs Perl, use `--nocomments` (or ignore `<!-- … -->` lines). See OXIDIZED_DESIGN #2.
- **`\cdots` role**: Uses `role="ELIDEOP"` (Perl uses `role="ID"`) for math parser grammar rules.
- **Color: visual equivalence**: Colors are compared by variant+values, not reference identity. `\color{black}` in a black context produces no `color="#000000"` attribute. See OXIDIZED_DESIGN #20.
- **No `tex=` on `<picture>`**: The `tex=` attribute on `<ltx:picture>` is suppressed by default. Enable with `LATEXML_SVG_TEX_ATTRIBUTE=true`. See OXIDIZED_DESIGN #21.

## Practical guidance

- **Canvas signal integrity — robust log parsing is the #1 method (fail toward flagging errors).**
  In the large-canvas auto-upgrade path, the single most important thing for a trustworthy
  signal is **robust parsing of the conversion log so that EVERY `Error:` and `Fatal:` message
  is captured.** The bias must be **fail-safe toward detecting failure**: it is acceptable to
  produce **false positives** (flag a clean conversion as an error), but a **failure to parse
  the log must NEVER be silently treated as a success** — that is a false negative, and it
  hides real regressions. Concretely: latexml_oxide/cortex emit **ANSI-colored** logs
  (`\x1b[31mError:`), so a naive `grep -c '^Error:'` matches **zero** and silently reports
  "0 errors / fixed" on a paper that actually has hundreds (this exact bug masked
  2002.05958=654, 1808.04050=441, 1705.10306=293, 1910.06783=859 as "fixed" — see
  `docs/SYNC_STATUS.md`).
  **Two reliable, ANSI-free signals exist — prefer them over grepping colored stderr:**
  (1) **cortex's status code** — `Status:conversion:N` (written to the `status` member of the
  output zip and to stdout), where **3 = fatal, 2 = error**, lower = OK/warnings; this integer
  is the canonical pass/fail. (2) **the on-disk `.latexml.log`** — captured via the
  ANSI-stripped `LOG_BUFFER`, so it is color-free by construction.
  **As of 2026-06-01 the logger also TTY-gates stderr colors** (`logger.rs::stderr_use_color`,
  `is_terminal() && NO_COLOR unset`), so **redirected stderr is now ANSI-free too** — a naive
  `grep '^Error:'` works on `cortex ... > log.txt 2>&1`. Still, defensively `sed
  's/\x1b\[[0-9;]*m//g'` before `grep -acE '^(Error|Fatal):'` (logs from older binaries carry
  ANSI), and gate on **cortex's own `Processing content` file** (multi-file papers ship decoy
  `\begin{document}` stubs). `canvas/run_one.sh` was HARDENED 2026-06-01 to **strip ANSI before
  the `^Error:`/`^Fatal:` count** — behaviour-preserving on the current ANSI-emitting release
  binary AND future-proof for an ANSI-free one (so the old landmine, where rebuilding release
  with the TTY-gate fix would zero-out run_one.sh's `$'^\x1b[31mError:'` grep and mark every
  paper a false "OK", is DEFUSED; release may now be rebuilt safely). Validated against ground
  truth on a 100-undefined-macro + recursion article: 101 errors / 1 fatal, identical counts on
  both the ANSI and ANSI-free binaries, matching `Status:conversion:3`. When in doubt, count it
  as a failure to investigate, not a pass.
- When an adjacent `TODO` note is relevant to the current task, extend scope to complete the TODO as well.
- Stay as close as possible to the organization and abstractions of the original Perl, as we aim for parity of the rewrite.
- **Active work**: the two targets in "Active priorities" above — corpus-driven parity mining plus the beyond-Perl arXiv-run levers. The actionable list lives in `docs/SYNC_STATUS.md`; completed missions (dump parity, upstream-sync U1–U11, …) are archived under `docs/archive/`.
- When a test failure traces to an upstream Perl issue, document it in `docs/KNOWN_PERL_ERRORS.md`.

When a **session is completed**: continue working, until:
- all tests pass
- the plans in docs/ are fully completed
- all edge cases are explored
- no obvious improvements remain

Do **not** stop early.

## Key Concepts Mapping (Perl → Rust)

| LaTeXML Perl | latexml-oxide |
|---|---|
| `LaTeXML::Core::Mouth` | `latexml_core::mouth` — tokenizer/reader |
| `LaTeXML::Core::Gullet` | `latexml_core::gullet` — macro expansion |
| `LaTeXML::Core::Stomach` | `latexml_core::stomach` — digestion |
| `LaTeXML::Core::Document` | `latexml_core::document` — XML construction |
| `LaTeXML::Core::State` | `latexml_core::state` — global state |
| `LaTeXML::Core::Definition` | `latexml_core::definition` — macro/command defs |
| `LaTeXML::Package` | `latexml_package` — package loading |

---

> **Reminder:** This is a faithful Perl-to-Rust translation. When porting any Perl code, preserve the original semantics, control flow, edge cases, and naming conventions. Read the Perl source first, translate precisely, and only diverge when documented in `docs/OXIDIZED_DESIGN.md`.
