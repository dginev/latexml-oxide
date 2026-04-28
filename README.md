# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.5.0-orange.svg) 
[![ported tests 100%](https://img.shields.io/badge/ported%20tests%20-%20100%25%20(391%2F391)-%20%2332a852?style=flat)
](https://github.com/dginev/latexml-oxide/issues/30)

This project is in an **early beta** stage. Please avoid using it in any real world setting before mainline LaTeXML parity is reached.

**Current status (2026-04-26):** active strict-Perl dump-parity work
(see [`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md) "Mission"). The
engine dump is being aligned with Perl's `make formats` output;
test regressions during this work are expected and will be cleared
once the dumps stabilise. Full post-processing pipeline:
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

Three named profiles in `Cargo.toml`, each tuned for one purpose:

| Profile | When | Goal |
|---------|------|------|
| **`test`** (default for `cargo test`) | day-to-day development | Maximum debug info (`debug = "full"`, `debug-assertions`, `overflow-checks`), incremental rebuilds, `-O1` for tolerable test runtime. Use as much local RAM/CPU as needed. |
| **`ci`**   | GitHub Actions only      | Lowest possible RAM (16 GB runner budget) and fastest compile (`opt-level = 0`, `codegen-units = 256`, no LTO). Just enough to prove tests pass. |
| **`release`** | distribution / final perf measurement | Best practice release: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`. Slowest build, fastest runtime, smallest binary. |

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

CI runs `cargo test --profile ci --tests` automatically; you should never
need to invoke that profile by hand. For local performance benchmarking
or when comparing against Perl LaTeXML, always use `--release`.

### Development Tips

To enable linting quality control via rustfmt and clippy, you can activate the included hooks via:
```bash
$ rustup component add rust-analyzer --toolchain nightly
$ rustup component add rustfmt --toolchain nightly
$ rustup component add clippy --toolchain nightly
$ git config --local core.hooksPath .githooks/
```

To generate the project documentation locally, run:
```bash
$ cargo doc --workspace --no-deps --open
```

**IMPORTANT:** There is a compile-time plugin that collects the files in the test suite. 
This means that when adding a new test `[name].tex` and `[name].xml` pair of files, you may need to manually execute `cargo clean` to rediscover the entry.
