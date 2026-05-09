# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.5.0-orange.svg) 
[![ported tests 100%](https://img.shields.io/badge/ported%20tests%20-%20100%25%20(391%2F391)-%20%2332a852?style=flat)
](https://github.com/dginev/latexml-oxide/issues/30)

This project is in an **early beta** stage. Please avoid using it in any real world setting before mainline LaTeXML parity is reached.

**Current status (2026-04-30):** active strict-Perl parity work at
the format/dump and package-loading boundary, followed by sandbox
long-tail cleanup (see [`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md)).
Current local verification is `cargo test --tests` **1109/0/0** and
the latest-row 7898-paper sandbox result is **7731 OK = 97.89%**.
Full post-processing pipeline:
`latexml_oxide --format=html5 --dest=paper.html paper.tex`
produces complete HTML with cross-references, citations, MathML,
and XSLT.

### Why?

The three main reasons:

  * LaTeXML is **too slow** for large-scale production use.
  * Perl 5 has **no street cred** anymore.
  * LaTeXML is **urgently needed**
    - for turning LaTeX sources into responsive, accessible web documents.

Design goals:

  * Faithfully rewrite the LaTeXML code base as-is, attempting to be as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring Perl idioms
  * Carefully address the newly required resource constraints

### Installation

Requires a recent Rust `nightly` to compile.

We still need the non-perl OS dependencies from [get LaTeXML](https://math.nist.gov/~BMiller/LaTeXML/get.html),
but adapted for Rust bindings.

Example for Ubuntu:
```
$ sudo apt install libxml2-dev libxslt1-dev texlive-latex-base imagemagick libkpathsea-dev libkpathsea6 \
                   texlive texlive-latex-extra texlive-science
```

#### Optional: Vector-preserving PDF → SVG

For the opt-in `--graphics-svg-threshold-kb N` flag (see
[docs/SYNC_STATUS.md](docs/SYNC_STATUS.md) and upstream
[brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)):

```
$ sudo apt install inkscape
```

`inkscape` is used to convert small vector-authored PDFs into vector SVG
instead of rasterising them via ImageMagick `convert`. The path is
disabled by default; if the flag is enabled but inkscape is missing at
runtime, the pipeline silently falls back to `convert`.

### Build profiles (Rust best practice)

Four named profiles in `Cargo.toml`, each tuned for one purpose:

| Profile | When | Goal |
|---------|------|------|
| **`test`** (default for `cargo test`) | day-to-day development | Maximum debug info (`debug = "full"`, `debug-assertions`, `overflow-checks`), incremental rebuilds, `-O1` for tolerable test runtime. Use as much local RAM/CPU as needed. |
| **`ci`**   | GitHub Actions only      | Lowest possible RAM (16 GB runner budget) and fastest compile (`opt-level = 0`, `codegen-units = 256`, no LTO). Just enough to prove tests pass. |
| **`release`** | local sandbox canvas / perf measurement | Laptop-throughput release: `opt-level = 3`, `lto = "thin"`, `codegen-units = 20`, `strip = "symbols"`. Strong runtime optimization while using the 20-thread local machine during release builds. |
| **`maxperf`** | one-off absolute runtime build | Preserves the old maximum optimizer scope: `lto = "fat"`, `codegen-units = 1`. Slower and less parallel, but available when build time is irrelevant. |

### Sample use

1. Make sure the tests pass first (uses the `test` profile automatically — no flag):
    ```bash
    $ cargo test --tests
    ```

2. Convert an example formula (release-grade binary for performance work):
    ```bash
    $ cargo run --release --bin latexmlmath_oxide '1+1=2'
    ```

3. Convert an example document:
    ```bash
    $ cargo run --release --bin latexml_oxide latexml_oxide/tests/structure/article.tex
    ```

4. Generate a rustdoc-styled HTML5 site for a RelaxNG (`.rnc`) schema — see [docs/SCHEMA_DOCUMENTATION.md](docs/SCHEMA_DOCUMENTATION.md).

CI runs `cargo test --profile ci --tests` automatically; you should never
need to invoke that profile by hand. For local performance benchmarking
or when comparing against Perl LaTeXML, always use `--release`.

### Development Tips

To enable linting quality control via rustfmt and clippy, you can activate the included hooks via:
```bash
$ rustup component add rustfmt --toolchain nightly
$ rustup component add clippy --toolchain nightly
$ git config --local core.hooksPath .githooks/
```

This workspace is heavy for rust-analyzer because of large proc-macro
definition bodies. The checked-in `.vscode/settings.json` uses a
stability profile: proc-macro expansion and cache priming are disabled,
RA uses `target/rust-analyzer`, and large/generated directories are
excluded from file watching. Terminal `cargo build` / `cargo test`
still compile proc macros normally.

To generate the project documentation locally, run:
```bash
$ cargo doc --workspace --no-deps --open
```

**IMPORTANT:** There is a compile-time plugin that collects the files in the test suite. 
This means that when adding a new test `[name].tex` and `[name].xml` pair of files, you may need to manually execute `cargo clean` to rediscover the entry.
