# CLAUDE.md

> **This is a Perl-to-Rust translation project.** Every translated entry must follow tightly the original semantics and nuances of the Perl source. Read the Perl source first, translate precisely; do not invent new abstractions, rename concepts, or simplify behavior unless documented as an intentional divergence in `docs/parity/OXIDIZED_DESIGN.md`. The Perl code is the ground truth.

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
   fleet converts the full ~2.8M-doc arXiv corpus (2026-07 rerun complete).
   Beyond-Perl levers: performance (17% math over-parse, tikz-cd digest —
   [`docs/performance/PERFORMANCE.md`](docs/performance/PERFORMANCE.md) /
   [`docs/performance/ARXIV_PERFORMANCE.md`](docs/performance/ARXIV_PERFORMANCE.md)), reliability
   ([`docs/performance/STABILITY_WITNESSES.md`](docs/performance/STABILITY_WITNESSES.md)), and the
   source-provenance showcase (issues #47/#92,
   [`docs/performance/SOURCE_PROVENANCE.md`](docs/performance/SOURCE_PROVENANCE.md)).

Current verification (tracked in `SYNC_STATUS.md`): `cargo test --tests` is
**1760 passing** (2026-07-29, 105 targets, on `main` @ `d5684f0bcf`; the two `latexml_post`
vector-SVG tests self-skip — silently, and *green* — unless `mutool` or
`pdftocairo` is on PATH, so a green local run does not by itself prove that
branch ran; CI installs poppler/mupdf). **A fully green suite still prints
`Error:` lines to stderr** — several tests deliberately raise diagnostics to
prove they get reported (the `graphics.rs` worker-thread fold emits
`failed_to_convert` for a nonexistent `w0.pdf`; the Rhai script-binding tests
emit `boom`). Judge a test run by its `test result:` lines and exit code, never
by grepping its output for `Error:` — that heuristic is for *conversion* logs
(below), and it inverts here. `cargo clippy --workspace --all-targets -- -D warnings` is clean
(policy in `[workspace.lints]`, gated by CI's `lint` job and the pre-push hook —
`latexml_oxide/build.rs` sets `core.hooksPath`). `cargo doc --workspace` is
**rustdoc-warning-clean** and gated on `-D warnings` in the same `lint` job (and
again in `rustdoc.yml` before publishing) — a broken intra-doc link renders as
dead text on the deployed site, so the warning is the only signal there is. The 2026-07 full-arXiv rerun
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
LaTeXML's CI; 2026 joined the window 2026-07-23 — upstream published
`:2026` on the same trixie/`libxml2.so.2` base, and the TL2026
`latex.ltx` init reached the zero-error gate). One kpathsea-UNLINKED
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

Cargo workspace with 8 crates — see `Cargo.toml` `[workspace] members` for the
list and each crate's own `lib.rs` docs for its role. The Perl→Rust crate map is
in "Key Concepts Mapping" below.

One directory whose purpose is not self-evident: `background/` — TeX
documentation and source, the original project generating PDF, which LaTeXML
emulates and adapts.

## Internal Documentation

All active docs live in `docs/`, grouped into themed subdirectories that mirror
the two mission targets. **[`docs/README.md`](docs/README.md)** is the single
index: a multi-level TOC over those subdirectories, followed by a **Per-file
detail** section that says what each doc is for and when to read it. Read it when
you need to find or place a doc — it used to be duplicated here, and the copy in
this file was retired 2026-07-28 so there is one index to keep current, not two.

**[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md) is the start-here worklist** for
both targets (ranked rows R1…R9 — take the top unblocked one). Labels there have
gone stale before: verify a status against its named guard test or `gh issue
view` before acting on it, and note that SHA-ancestry does not work here because
the repo squash-merges.

The placement rules below are policy, not navigation, so they stay in this file.

**Rules for these docs:**
- `KNOWN_PERL_ERRORS.md` is for Perl-origin issues only. Include minimal trigger examples.
- `WISDOM.md` is for tactical system insights — record when specialized analysis leads to a correct patch. A reusable *method* is a durable fact, not narrative.
- Rust-specific error fixes go in `SYNC_STATUS.md` under "Rust Error Fixes", referencing the KNOWN_PERL_ERRORS entry when applicable.
- When an upstream Perl error is identified, record it. Fix in Rust if simple; otherwise keep as-is.
- **Diagnostic-snapshot naming.** Docs that capture a point-in-time technical diagnostic — `*_TRIAGE`, `*_HOTSPOTS`, `*_AUDIT`, `*_ANALYSIS`, `*_BISECT`, and similar — **must carry a date in the filename** (`NAME_YYYY-MM-DD.md`), using the date of their last commit. This keeps a study from masquerading as a live worklist. *Living* worklists are exempt even when their name reads like a diagnostic — date only what is a frozen snapshot. (When such a worklist's mission *completes*, date it and move it to `docs/archive/`, lifting any live residual into `SYNC_STATUS.md` — as was done for the LoadFormat audit.)
- **Record the conclusion, not the play-by-play.** State the defect, its cause, the fix, and the guard test names — not the narrative of how it was found or what was tried on which day. Keep what is expensive to re-derive: witness arXiv ids, `file:line` into the Perl source, minimal trigger examples, named guards, identifiers a reader would otherwise grep for, measured figures with their basis, and settled dead-ends (one line each, so they are not re-attempted). Cut connective tissue, not identifiers. A table cell is not an essay: in `ISSUE_AUDIT.md` and similar, a few sentences, then point at `KNOWN_PERL_ERRORS`/`OXIDIZED_DESIGN` for the mechanism.
- Keep **[`docs/README.md`](docs/README.md)** current — both its TOC table and its **Per-file detail** section — when adding, renaming, merging, or archiving a doc. When a diagnostic snapshot is superseded, archive it under `docs/archive/` rather than leaving it orphaned at the top level.

## Skills (`.claude/skills/`)

Reusable workflows encoding this project's hard-won judgement — the *rules*
around the `tools/` scripts (what verdict to trust, what trap to avoid). Each is
listed with its description in every session and loads on invocation, so they are
not restated here; read the `SKILL.md` for the one you need.

`canvas-triage` (genuine Rust bug vs Perl parity) → `min-repro` (shrink it) →
`perl-port` (faithful fix) is the standard chain, wrapped by `resolve-issue` for
a public GitHub issue. `perf-check` governs measurement.

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

**Publish-grade measurement** (matching against Perl LaTeXML, baseline updates in `docs/performance/PERFORMANCE.md`): use `--release`. The CI profile is for the GitHub runner only.

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

Formatting is configured in `rustfmt.toml` — run `cargo +nightly fmt --all`
rather than matching its settings by hand.

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
- **Self-contained, portable binary** (design requirement): a conversion must not *read* latexml_oxide's *own* resources from disk during its main operation. Engine dumps, the RelaxNG schema, and XSLT/CSS/JS are embedded and served from memory (verified: XSLT via `strace`, dumps by renaming the dev-tree `resources/dumps/` away and still converting). *Writing* outputs — including auxiliary files — into the **destination** directory is fine. New code that adds a runtime read of an *owned* resource must instead embed it (`include_bytes!` / `include_str!`). The host **TeX Live ecosystem is out of scope**: reading `.sty`/`.cls`/`.tfm` from the user's texmf tree via `kpathsea` is allowed and expected. Official releases ship the `maxperf` binary as a GitHub Release Asset, runnable with no `resources/` tree. Full rationale in [`docs/parity/OXIDIZED_DESIGN.md`](docs/parity/OXIDIZED_DESIGN.md) → Guiding Principles.
- Test files (`.t` extension) mirror the original LaTeXML Perl test suite; `.rs` files are the Rust equivalents
- most tests are regression-oriented. They contain a complete TeX input, and can experience failures in many different intermediate stages.
- we are interested in finding meaningful Rust types for the previously untyped Perl.

## Intentional divergences from Perl

- **`%\n` not emitted**: Rust does not emit `%\n` (TeX comment-newline line-break separator) in `tex` attributes. When copying test XMLs from Perl, strip all `%&#10;` occurrences. This is a no-semantic-content formatting artifact.
- **Source comments off by default**: `INCLUDE_COMMENTS` defaults to **false** in the Rust binary (Perl defaults true), so source `%` comments and the `%**** <file> Line N ****` progress markers (emitted every 25 lines) are suppressed in the output. Deliberate — debugging noise, no semantic content. `--comments` restores Perl's behavior. When diffing vs Perl, use `--nocomments` (or ignore `<!-- … -->` lines). See OXIDIZED_DESIGN #2.
- **`\cdots` role**: Uses `role="ELIDEOP"` (Perl uses `role="ID"`) for math parser grammar rules.
- **Color: visual equivalence**: Colors are compared by variant+values, not reference identity. `\color{black}` in a black context produces no `color="#000000"` attribute. See OXIDIZED_DESIGN #20.
- **No `tex=` on `<picture>`**: The `tex=` attribute on `<ltx:picture>` is suppressed **unconditionally**. (An `LATEXML_SVG_TEX_ATTRIBUTE=true` escape hatch was designed but never implemented — the name appears in no source file. Verified 2026-07-20.) See OXIDIZED_DESIGN #21.

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
  `\begin{document}` stubs). `canvas/run_one.sh` (an out-of-tree sweep harness — it is NOT in
  this repo, so don't go looking; the in-tree equivalents are `tools/benchmark_canvas.sh` and
  `tools/parity_check.sh`) already ANSI-strips before its `^Error:`/`^Fatal:` count. When in
  doubt, count it as a failure to investigate, not a pass.
- When an adjacent `TODO` note is relevant to the current task, extend scope to complete the TODO as well.
- **Never delete a witness article (arXiv id) from a code comment; always carry existing witnesses into any new/edited comment and add the new one.** Witnesses are the concrete reproducer a past decision hinged on — they are very valuable. Before landing a change to a construct whose comment names a witness, **re-convert that witness and confirm it still succeeds** (the test suite can miss it). Example: the `\hphantom` comment's `2004.10048` witness caught a lateral regression (the naive quantikz2 fix dropped 2004.10048's bibliography) that no test guarded.

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
