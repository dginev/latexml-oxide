# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.4.0-orange.svg) 
[![ported tests 26%](https://img.shields.io/badge/ported%20tests%20-%2026%25%20-%20%23a89932?style=flat)
](https://github.com/dginev/latexml-oxide/issues/30)

This project is in a **pre-alpha** stage! Please avoid using it in any real world setting before test parity is reached. 

**If the "ported tests" badge above isn't at `100%`, we aren't stable and we aren't ready**.

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

To enable linting quality control via rustfmt and clippy, you can activate the inlcuded hooks via:
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
