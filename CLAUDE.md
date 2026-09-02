# CLAUDE.md

> **This is a Perl-to-Rust translation project.** Every translated entry must follow tightly the original semantics and nuances of the Perl source. Read the Perl source first, translate precisely; do not invent new abstractions, rename concepts, or simplify behavior unless documented as an intentional divergence in `docs/parity/OXIDIZED_DESIGN.md`. The Perl code is the ground truth.

## Active priorities: faithful parity + beyond-Perl arXiv runs

Two co-equal targets drive current work:

1. **Faithful translation of the original Perl LaTeXML.** Strict parity at the
   format/dump and package-loading boundary is maintained — the strict-`LoadFormat`
   dump-parity mission is **complete** (zero-error inits, dumps match Perl; audit
   archived at [`docs/archive/PERL_LOADFORMAT_AUDIT.md`](docs/archive/PERL_LOADFORMAT_AUDIT.md),
   the ~72-CS Perl-only long-tail residual tracked in `SYNC_STATUS.md` "Engine file
   open gaps"). Ongoing parity work is corpus-driven: mine fatal/error clusters from
   live runs, classify vs same-host Perl (`canvas-triage` skill), fix
   GENUINE-RUST-ONLY divergences faithfully.
2. **Beyond-Perl improvement runs over arXiv.** The production `cortex_worker` fleet
   converts the full ~2.8M-doc arXiv corpus. Beyond-Perl levers: performance
   ([`docs/performance/PERFORMANCE.md`](docs/performance/PERFORMANCE.md) /
   [`ARXIV_PERFORMANCE.md`](docs/performance/ARXIV_PERFORMANCE.md)), reliability
   ([`STABILITY_WITNESSES.md`](docs/performance/STABILITY_WITNESSES.md)), and the
   source-provenance showcase (issues #47/#92,
   [`SOURCE_PROVENANCE.md`](docs/performance/SOURCE_PROVENANCE.md)).

Test counts, gate status, and corpus throughput live in
[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md), which is commit-tracked. Do not copy
those numbers here — the copy drifts.

Two traps when reading a test run:

- **A fully green suite still prints `Error:` lines to stderr.** Several tests
  deliberately raise diagnostics to prove they get reported (the `graphics.rs`
  worker-thread fold emits `failed_to_convert` for a nonexistent `w0.pdf`; the Rhai
  script-binding tests emit `boom`). Judge a test run by its `test result:` lines and
  exit code, **never** by grepping its output for `Error:` — that heuristic belongs to
  *conversion* logs (below), and here it inverts.
- The two `latexml_post` vector-SVG tests **self-skip silently and green** unless
  `mutool` or `pdftocairo` is on PATH, so a green local run does not by itself prove
  that branch ran. CI installs poppler/mupdf.

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

**Distribution model.** Per-TL-year dump files
(`resources/dumps/{plain,latex}.YYYY.dump.txt`) are **not committed**; releases
generate a 5-year window inside pinned TL containers and embed it at build time.
**Dev/CI generate the ambient-year dump via `tools/make_formats.sh`** — run it once
after checkout, after a TL upgrade, or before test runs needing dumps. The `--init`
runs are gated on zero `Error:`/`Fatal:` and **their output is suppressed otherwise,
so naive grepping sees nothing**. Year resolution, the container matrix, and the
`kpsewhich --version` non-discriminator: [`docs/parity/DUMP_DESIGN.md`](docs/parity/DUMP_DESIGN.md).

## Project Overview

latexml-oxide is a Rust port of [LaTeXML](https://github.com/brucemiller/latexml), a Perl tool that converts LaTeX documents into accessible web documents (HTML/XML).

The `LaTeXML/` directory contains the legacy Perl source being ported. Do not modify it — it serves as the reference implementation.
Similarly, the test `.tex`, `.xml` and `.pdf` files often need to be copied from the Perl space to the Rust space.

## Workspace Structure

Cargo workspace with 8 crates — see `Cargo.toml` `[workspace] members` for the
list and each crate's own `lib.rs` docs for its role. The Perl→Rust crate map is
in "Key Concepts Mapping" below.

One directory whose purpose is not self-evident: `background/` — TeX
documentation and source, the original project generating PDF, which LaTeXML
emulates and adapts.

## Internal Documentation

All active docs live in `docs/`, grouped into themed subdirectories that mirror the
two mission targets. **[`docs/README.md`](docs/README.md)** is the single index: a
multi-level, themed table of contents saying what each doc is for and when to read
it. Read it when you need to find or place a doc.

**[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md) is the start-here worklist** for both
targets (ranked rows R1…R9 — take the top unblocked one). Labels there have gone
stale before: verify a status against its named guard test or `gh issue view` before
acting on it, and note that SHA-ancestry does not work here because the repo
squash-merges.

**Rules for these docs:**
- `docs/parity/KNOWN_PERL_ERRORS.md` is for Perl-origin issues only. Include minimal trigger examples.
- `docs/parity/WISDOM.md` is for tactical system insights — record when specialized analysis leads to a correct patch. A reusable *method* is a durable fact, not narrative.
- Rust-specific error fixes go in `SYNC_STATUS.md` under "Rust Error Fixes", referencing the KNOWN_PERL_ERRORS entry when applicable.
- When an upstream Perl error is identified, record it. Fix in Rust if simple; otherwise keep as-is.
- **Diagnostic-snapshot naming.** Docs capturing a point-in-time diagnostic — `*_TRIAGE`, `*_HOTSPOTS`, `*_AUDIT`, `*_ANALYSIS`, `*_BISECT` — **must carry a date in the filename** (`NAME_YYYY-MM-DD.md`), from their last commit, so a frozen study cannot masquerade as a live worklist. *Living* worklists are exempt even when the name reads like a diagnostic. When such a worklist's mission completes, date it, move it to `docs/archive/`, and lift any live residual into `SYNC_STATUS.md`.
- **Record the conclusion, not the play-by-play.** State the defect, its cause, the fix, and the guard test names — not how it was found or what was tried on which day. Keep what is expensive to re-derive: witness arXiv ids, `file:line` into the Perl source, minimal trigger examples, named guards, identifiers a reader would otherwise grep for, measured figures with their basis, and settled dead-ends (one line each, so they are not re-attempted). Cut connective tissue, not identifiers.
- Keep **[`docs/README.md`](docs/README.md)** current — its themed TOC tables — when adding, renaming, merging, or archiving a doc.

## Skills and agents (`.claude/skills/`, `.claude/agents/`)

`cluster-classify` (group a sweep's failures) → `canvas-triage` (genuine Rust bug
vs Perl parity) → `min-repro` (shrink it) → `perl-port` (faithful fix) is the
standard chain, wrapped by `resolve-issue` for a public GitHub issue.
`surpass-perl` governs the rare intentional divergence, `dump-debug` the
dump-vs-NODUMP branch, `perf-check` measurement, `next-release` shipping.

Delegate read-only root-causing of a witness to the `root-causer` agent (pinned
to Opus 4.8 at xhigh by user directive; up to ~5 in parallel on independent
witnesses) and log tallying to `log-scanner`. Edits, builds, and test runs stay
in the main session, which owns the tree. Brief agents with the main checkout,
not a worktree: `LaTeXML/` (the Perl oracle) is gitignored and absent from
every worktree, so a worktree agent greps nothing and reports "no gaps".

## Build & Test

Requires **Rust nightly**.

| Profile | Use | Tuned for |
|---------|-----|-----------|
| `test`  | `cargo test` / `cargo run` / `cargo build` (default = `dev`/`test`) | Maximum debug info, debug-assertions, overflow-checks, incremental rebuilds. **All local development and triage** — the only profile to use day-to-day. |
| `ci`    | `cargo test --profile ci` (only used in `.github/workflows/CI.yml`) | Lowest RAM (16 GB GitHub Actions runner) and fastest compile. `opt-level = 0`, `codegen-units = 256`. |
| `release` | `cargo build --release` / `cargo run --release` | Strong-optimized binary tuned for our 32 GB / 20-thread laptop. `opt-level = 3`, `lto = "thin"`, `codegen-units = 20`, `strip = "symbols"`. Used for **sandbox sweeps and Perl-parity measurements**, NOT distribution. |
| `maxperf` | `cargo build --profile maxperf` | **Distribution / publish-grade artifact**. Inherits release, plus `lto = "fat"`, `codegen-units = 1`. Slowest build, smallest + fastest binary. **Reserved for shipping a stable state.** |

CI is *not* what local dev should mimic; CI is RAM-bounded and stripped. For sandbox
runs, build `cortex_worker` in the default profile and pass it to
`tools/benchmark_canvas.sh` via `--worker-bin`; use `--release` only when you
specifically need a publish-grade canvas measurement or a Perl-parity baseline
(`docs/performance/PERFORMANCE.md`).

**Distribution build** ships `--profile maxperf --no-default-features --features
runtime-bindings`: `--no-default-features` drops the `test-utils` feature (removing
`phf` + `glob` and 4 transitive crates), while `runtime-bindings` keeps the runtime
contributed-bindings front-end (runtime opt-in, so default conversions are
unaffected). This is the recipe `tools/make_release.sh` uses. `maxperf` sets
`panic = "abort"` — production-only, since canvas sweeps depend on `catch_unwind` for
per-paper panic isolation.

```bash
# Triage a sandbox failure (test profile, full backtraces)
tools/triage_failure.sh <arxiv_id>

# Distribution build — smallest, fastest artifact (slow build, fat LTO,
# panic=abort, no test-utils; keeps runtime-bindings)
cargo build --no-default-features --features runtime-bindings --profile maxperf --bin latexml_oxide
```

**Important:** A compile-time plugin discovers test suite files. When adding a new `[name].tex` / `[name].xml` test pair, run `cargo clean` to force rediscovery.

**Run the suite with `cargo nextest run --workspace`.** `cargo test` runs each
test binary to completion before starting the next, parallelising only *within* a
binary, so its wall floor is the sum of them (measured ~398 s across the former
~122 binaries). nextest schedules every test as its own process across all cores;
measured back-to-back, same ~1835 tests, both green: **8:48 → 1:27 (6.1×)**.
Coverage is identical — nextest skips doctests and this workspace has none. `cargo
test --workspace --tests --no-fail-fast` remains valid and is still the way to read
a per-binary tally. **Test binaries are grouped by concern**: single/double-test
files were consolidated 2026-08 into `cluster_*.rs` binaries (one link unit per
subsystem — `cluster_cli`, `cluster_fontmap`, `cluster_xslt_split`,
`cluster_package_guards`, `cluster_sizing`, `cluster_frontmatter_classes`, plus the
existing `06_cluster_*`) to cut link steps. **Do NOT merge the fixture-sweep
binaries** (`tex_tests!` harness stubs, `114_streaming_*`): each is a separate
process on purpose — co-locating their many conversions in one `cargo test` process
accretes unreclaimable libxml2 residue past the RSS fuse (see `streaming_sweep/mod.rs`).

Gates: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo doc
--workspace` (rustdoc warnings are errors) are enforced by CI's `lint` job and the
pre-push hook. Rustdoc matters more than it looks — a broken intra-doc link renders as
dead text on the deployed site, so the warning is the only signal there is.

## Code Style

Formatting is configured in `rustfmt.toml` — run `cargo +nightly fmt --all`.

```bash
rustup component add rust-analyzer rustfmt clippy --toolchain nightly
```

`latexml_oxide/build.rs` points `core.hooksPath` at `.githooks/` on first build; no
manual `git config` step is needed.

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
- **Self-contained, portable binary** (design requirement): a conversion must not *read* latexml_oxide's *own* resources from disk during its main operation. Engine dumps, the RelaxNG schema, and XSLT/CSS/JS are embedded and served from memory (to re-verify: `strace` for the XSLT read, and rename the dev-tree `resources/dumps/` away — conversion must still work). *Writing* outputs — including auxiliary files — into the **destination** directory is fine. New code that adds a runtime read of an *owned* resource must instead embed it (`include_bytes!` / `include_str!`). The host **TeX Live ecosystem is out of scope**: reading `.sty`/`.cls`/`.tfm` from the user's texmf tree via `kpathsea` is allowed and expected. Rationale in [`docs/parity/OXIDIZED_DESIGN.md`](docs/parity/OXIDIZED_DESIGN.md) → Guiding Principles.
- Test files (`.t` extension) mirror the original LaTeXML Perl test suite; `.rs` files are the Rust equivalents
- most tests are regression-oriented. They contain a complete TeX input, and can experience failures in many different intermediate stages.
- we are interested in finding meaningful Rust types for the previously untyped Perl.

## Intentional divergences from Perl

- **`%\n` not emitted**: Rust does not emit `%\n` (TeX comment-newline line-break separator) in `tex` attributes. When copying test XMLs from Perl, strip all `%&#10;` occurrences. This is a no-semantic-content formatting artifact.
- **Source comments off by default**: `INCLUDE_COMMENTS` defaults to **false** in the Rust binary (Perl defaults true), so source `%` comments and the `%**** <file> Line N ****` progress markers are suppressed in the output. Deliberate — debugging noise, no semantic content. `--comments` restores Perl's behavior. When diffing vs Perl, use `--nocomments` (or ignore `<!-- … -->` lines). See OXIDIZED_DESIGN #2.
- **`\cdots` role**: Uses `role="ELIDEOP"` (Perl uses `role="ID"`) for math parser grammar rules.
- **Color: visual equivalence**: Colors are compared by variant+values, not reference identity. `\color{black}` in a black context produces no `color="#000000"` attribute. See OXIDIZED_DESIGN #20.
- **No `tex=` on `<picture>`**: The `tex=` attribute on `<ltx:picture>` is suppressed **unconditionally**. The `LATEXML_SVG_TEX_ATTRIBUTE=true` escape hatch was designed but never implemented — the name appears in no source file, so don't go looking for it. See OXIDIZED_DESIGN #21.

## MathML output targets MathML Core

**We emit modern MathML Core, which has a deliberately reduced element set.** When
a construct can be expressed with a Core element or with one Core removed, use the
Core one — even if Perl emits the removed element, and even if the removed element
reads as more "semantic". Browsers implement Core; anything outside it is dead
markup that renders as a fallback at best.

Removed by Core, and what to emit instead:

| Removed | Emit instead |
|---|---|
| `<none/>` (absent `mmultiscripts` slot) | empty `<mrow/>` — the accepted placeholder for an omitted subtree |
| `<mfenced>` | `<mrow>` with explicit `<mo>` fences |
| `<mlabeledtr>` | `<mtr>`, label handled outside the table |
| `<maligngroup>`, `<malignmark>`, `<mglyph>` | nothing — drop |
| `<mstack>`, `<mlongdiv>`, `<msline>`, `<mscarries>`, `<mscarry>`, `<msgroup>`, `<msrow>` | nothing — elementary-math layout is out of scope |

Do **not** reason from "MathML 3 defines an element for exactly this purpose" —
that argument once produced a wrong divergence entry justifying `<m:none/>`
(OXIDIZED_DESIGN #86, since removed). MathML 3 defining it is not evidence Core
kept it; check Core.

Known residual: **`<m:menclose>` is still emitted** for `\cancel` / `\boxed`.
Core removed it and there is no mechanical replacement, so it is a rendering +
golden change needing its own branch — tracked as **`SYNC_STATUS.md` R3b**,
deferred by user directive 2026-07-30. Don't "fix" it incidentally.

## Practical guidance

- **Canvas signal integrity — fail toward flagging errors.** A failure to parse a
  conversion log must NEVER be silently treated as success; false positives (flagging
  a clean conversion) are acceptable, false negatives hide regressions. Two ANSI-free
  signals are canonical, and both beat grepping stderr: (1) **cortex's status code**,
  `Status:conversion:N` — written to the `status` member of the output zip and to
  stdout, where **3 = fatal, 2 = error**, lower = OK/warnings; (2) the on-disk
  `.latexml.log`, captured via the ANSI-stripped `LOG_BUFFER`. If you must grep stderr,
  `sed 's/\x1b\[[0-9;]*m//g'` first (the logger TTY-gates color now —
  `logger.rs::stderr_use_color`, `is_terminal() && NO_COLOR` unset — but older binaries
  emit `\x1b[31mError:`, against which a naive `grep -c '^Error:'` returns **zero**;
  this exact bug once masked 2002.05958=654, 1808.04050=441, 1705.10306=293 and
  1910.06783=859 errors as "fixed"), and gate on cortex's own `Processing content`
  file — multi-file papers ship decoy `\begin{document}` stubs. In-tree sweep harnesses
  are `tools/benchmark_canvas.sh` and `tools/parity_check.sh`; `canvas/run_one.sh` is
  out-of-tree, so don't go looking for it here.
- **Never delete a witness article (arXiv id) from a code comment; carry existing
  witnesses into any new/edited comment and add the new one.** Witnesses are the
  concrete reproducer a past decision hinged on. Before landing a change to a construct
  whose comment names a witness, **re-convert that witness and confirm it still
  succeeds** — the test suite can miss it. The `\hphantom` comment's `2004.10048`
  witness caught a lateral regression (a naive quantikz2 fix dropped that paper's
  bibliography) that no test guarded.
- When an adjacent `TODO` note is relevant to the current task, extend scope to complete the TODO as well.

## Working cadence

Chain work rather than idling: when something lands, take the next unblocked row from
`docs/SYNC_STATUS.md`. Don't stop with tests red, a worklist row half-finished, or a
TODO you opened still open. What you cannot finish is `docs/` itself — it is a standing
multi-target worklist, not a session-completion bar.

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
