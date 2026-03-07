# CLAUDE.md

## Project Overview

latexml-oxide is a Rust port of [LaTeXML](https://github.com/brucemiller/latexml), a Perl tool that converts LaTeX documents into accessible web documents (HTML/XML).

The `LaTeXML/` directory contains the legacy Perl source being ported. Do not modify it — it serves as the reference implementation.

## Workspace Structure

Cargo workspace with 6 crates:

- **latexml_core** — Core engine: tokenizer (mouth), macro expander (gullet), digester (stomach), document builder, state management, definitions, bindings
- **latexml_oxide** — Top-level crate with binary targets (`latexml_oxide`, `latexmlmath_oxide`) and integration tests
- **latexml_package** — TeX/LaTeX package system: compile-time macro engine, package loader, prelude
- **latexml_math_parser** — Math expression parser with Marpa-style grammar
- **latexml_codegen** — Proc macros for compile-time code generation (constructable, modelable, parametrizeable, testable, tokenizeable)
- **latexml_contrib** — User-contributed style packages

Supporting directories:
- `resources/` — CSS, DTD, JavaScript, RelaxNG schemas, XSLT, Profiles
- `tools/` — Utility scripts (e.g. `compile_metrics.pl`)
- `.githooks/` — Pre-push hook for quality checks

## Build & Test

Requires **Rust nightly** (v1.83+).

System dependencies (Ubuntu):
```bash
sudo apt install libxml2-dev libxslt1-dev libkpathsea-dev texlive-latex-base imagemagick
```

```bash
# Run all tests (use --release for realistic performance)
cargo test --release --tests

# Convert a formula
cargo run --release --bin latexmlmath_oxide '1+1=2'

# Convert a document
cargo run --release --bin latexml_oxide latexml_oxide/tests/hello/hello.tex

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

## Architecture Notes

**Important:** The one novelty in the Rust rewrite is the math parser engine, which now uses a highly ambiguous Marpa grammar.
  - the new goal is to be highly ambiguous in parsing, 
  - but aggressively prune in the semantics rules, so as to minimize the final parses
  - this is active ongoing research. So be very cautious when porting math tests, ideally do them after everything else is solid.

- **State** is a thread-local, global, mutable singleton (see CHANGELOG 0.3.2 decision)
- Uses a **string interner** for efficient symbol handling
- TeX macro definitions can be compiled at compile-time via proc macros in `latexml_codegen`
- The port aims to be **faithful to the Perl original** while using idiomatic Rust where possible
- Test files (`.t` extension) mirror the original LaTeXML Perl test suite; `.rs` files are the Rust equivalents
- most tests are regression-oriented. They contain a complete TeX input, and can experience failures in many different intermediate stages.
- we are interested in finding meaningful Rust types for the previously untyped Perl.

## Practical guidance

- When an adjacent `TODO` note is relevant to the current task, extend scope to complete the TODO as well.
- Stay as close as possible to the organization and abstractions of the original Perl, as we aim for parity of the rewrite.
- The Perl LaTeXML directory gets updated at times, as the original project is still active. Before doing new work, always revise the current Rust against the current Perl, and update the Rust when outdated.

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

## CI

GitHub Actions runs on push/PR: installs system deps, uses Rust nightly, runs `cargo test`.
