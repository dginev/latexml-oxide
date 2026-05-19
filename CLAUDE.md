# CLAUDE.md

> **This is a Perl-to-Rust translation project.** Every translated entry must follow tightly the original semantics and nuances of the Perl source. Do not invent new abstractions, rename concepts, or simplify behavior unless explicitly marked as an intentional divergence. The Perl code is the ground truth.

## Active priority (refreshed 2026-05-19): strict-Perl parity

Strict Perl parity at the format/dump and package-loading boundary is
the current top priority, followed by sandbox long-tail cleanup.
Current local verification in `docs/SYNC_STATUS.md`: `cargo test
--tests` is **1328/0/0** and `cargo clippy --workspace --all-targets`
is **14 warnings (all in `latexml_math_parser` from the in-flight
ASF migration — collaborator's lane)**. The latest sandbox result for
the 100k `next_warning_papers` corpus is ~99.4% OK; the latest 10k
stage v3 ranges 97.4–99.5%. Working docs:
[`docs/PERL_LOADFORMAT_AUDIT.md`](docs/PERL_LOADFORMAT_AUDIT.md),
[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md),
[`docs/BIBTEX_PORT_PLAN.md`](docs/BIBTEX_PORT_PLAN.md).

Concretely:

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

**Distribution follow-up — LANDED 2026-05-15.** Per-TL-year dump
files (`resources/dumps/{plain,latex}.YYYY.dump.txt` +
`texlive.YYYY.version`) are committed to the repo and embedded into
the binary at build time via `include_str!`. Runtime resolves the
ambient year via `kpsewhich -var-value=SELFAUTOPARENT` with
`pdflatex --version` fallback (`kpsewhich --version` returns the
same kpathsea-library string on TL2023 and TL2025, so it's NOT a
reliable discriminator). TL2023 + TL2025 are bundled currently; add
new years via `tools/make_formats.sh`. Follow-up IA-record
consolidation (`81176ba689`) halved `latex.YYYY.dump.txt` size by
collapsing per-slot fontdimen V-records into per-(font,size) `IA`
records with RLE-encoded data.

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

Three key documents track porting progress and known issues:

- **[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md)** — Master tracking document: file-by-file Perl→Rust sync status, test suite counts, Rust error fixes, infrastructure gaps, package bindings status, and the 9-phase roadmap to full parity. **Start here** when resuming work.
- **[`docs/ORGANIZATION.md`](docs/ORGANIZATION.md)** — Maps Perl engine files (`LaTeXML/Engine/*.pool.ltxml`) to Rust files (`latexml_engine/src/*.rs`). Shows loading hierarchy and LaTeX chapter structure.
- **[`docs/KNOWN_PERL_ERRORS.md`](docs/KNOWN_PERL_ERRORS.md)** — Documents upstream Perl LaTeXML issues: `packParameters` alignment warning, `\fontname` format, per-font `\hyphenchar`, `specialize()` property reset, `readBalanced` `#`-ambiguity, `guessTableHeaders` heuristic. When investigating test failures, check here first to see if the issue is inherited from Perl.
- **[`docs/WISDOM.md`](docs/WISDOM.md)** — Tactical insights about system internals, discovered through specialized debugging. Covers: compile-time vs runtime token packing, Font::merge/specialize interaction, catcode CS vs ESCAPE, RegisterType PartialEq trap, at_letter restore. Check here to avoid re-introducing known bugs.
- **[`docs/OXIDIZED_DESIGN.md`](docs/OXIDIZED_DESIGN.md)** — Public-facing design document: architecture decisions, intentional Perl divergences, type system improvements, tactical insights. Read this file to check if a translation difference was marked as intentional.
- **[`docs/MATH_PARSER_AND_ASF.md`](docs/MATH_PARSER_AND_ASF.md)** — Rationalization of the math parser's three-stage ambiguity pipeline against the Marpa ASF (abstract syntax forest) traversal paradigm. Read before touching `latexml_math_parser/src/parser.rs::parse_string` or `semantics.rs::Actions`. Companion to [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md) on the fork's `asf-completion` branch.

**Rules for these docs:**
- `KNOWN_PERL_ERRORS.md` is for Perl-origin issues only. Include minimal trigger examples.
- `WISDOM.md` is for tactical system insights — record when specialized analysis leads to a correct patch.
- Rust-specific error fixes go in `SYNC_STATUS.md` under "Rust Error Fixes", referencing the KNOWN_PERL_ERRORS entry when applicable.
- When an upstream Perl error is identified, record it. Fix in Rust if simple; otherwise keep as-is.

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

**Distribution build** (shipping the binary to users): use `--profile maxperf --no-default-features` for the smallest, fastest artifact. Example: `cargo build --no-default-features --profile maxperf --bin latexml_oxide`. The `--no-default-features` flag drops the `test-utils` feature, removing `phf` + `glob` (and 4 transitive crates) from the binary. The `maxperf` profile uses `panic = "abort"` — production-only since canvas sweeps depend on `catch_unwind` for per-paper panic isolation.

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
# panic=abort, no test-utils feature)
cargo build --no-default-features --profile maxperf --bin latexml_oxide

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
- Test files (`.t` extension) mirror the original LaTeXML Perl test suite; `.rs` files are the Rust equivalents
- most tests are regression-oriented. They contain a complete TeX input, and can experience failures in many different intermediate stages.
- we are interested in finding meaningful Rust types for the previously untyped Perl.

## Intentional divergences from Perl

- **`%\n` not emitted**: Rust does not emit `%\n` (TeX comment-newline line-break separator) in `tex` attributes. When copying test XMLs from Perl, strip all `%&#10;` occurrences. This is a no-semantic-content formatting artifact.
- **`\cdots` role**: Uses `role="ELIDEOP"` (Perl uses `role="ID"`) for math parser grammar rules.
- **Color: visual equivalence**: Colors are compared by variant+values, not reference identity. `\color{black}` in a black context produces no `color="#000000"` attribute. See OXIDIZED_DESIGN #20.
- **No `tex=` on `<picture>`**: The `tex=` attribute on `<ltx:picture>` is suppressed by default. Enable with `LATEXML_SVG_TEX_ATTRIBUTE=true`. See OXIDIZED_DESIGN #21.

## Practical guidance

- When an adjacent `TODO` note is relevant to the current task, extend scope to complete the TODO as well.
- Stay as close as possible to the organization and abstractions of the original Perl, as we aim for parity of the rewrite.
- **Active work**: drive the strict-Perl dump-parity mission described above. Concrete sub-tasks are tracked in `docs/PERL_LOADFORMAT_AUDIT.md` and `docs/SYNC_STATUS.md`.
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
