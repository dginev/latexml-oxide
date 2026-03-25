# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.4.0-orange.svg) 
[![ported tests 94%](https://img.shields.io/badge/ported%20tests%20-%2094%25%20(303%2F324)-%20%2332a852?style=flat)
](https://github.com/dginev/latexml-oxide/issues/30)

This project is in an **alpha** stage. Please avoid using it in any real world setting before test parity is reached.

**If the "ported tests" badge above isn't at `100%`, we aren't stable and we aren't ready**.

**Current status (2026-03-25):** 303/324 tests passing (93.5%), 21 ignored, 0 failing. 499 package bindings (408 core + 91 contrib). Full 28/28 math parse suite + mathtools + listing + picture pass.

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

Requires Rust `nightly` v1.83, and newer.

We still need the non-perl OS dependencies from [get LaTeXML](https://math.nist.gov/~BMiller/LaTeXML/get.html),
but adapted for Rust bindings.

Example for Ubuntu:
```
$ sudo apt install libxml2-dev libxslt1-dev texlive-latex-base imagemagick
```

### Sample use

1. Make sure the tests pass first, via
    ```bash
    $ cargo test --release --tests
    ```

2. convert an example formula:
    ```bash
    $ cargo run --release --bin latexmlmath_oxide '1+1=2'
    ```

3. convert an example document:
    ```bash
    $ cargo run --release --bin latexml_oxide latexml_oxide/tests/hello/hello.tex
    ```

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

**IMPORTANT:** There is a compile-time plugin that collects the files in the test suite. This means that when adding a new test `[name].tex` and `[name].xml` pair of files, you may need to manually execute `cargo clean` to rediscover the entry.
