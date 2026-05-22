# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.5.0-orange.svg) 
[![ported tests](https://img.shields.io/badge/ported%20tests%20-%201328%2F0%2F0-%20%2332a852?style=flat)
](https://github.com/dginev/latexml-oxide/issues/30)

This project is in an **early beta** stage. Please avoid using it in any real world setting before mainline LaTeXML parity is reached.

**Current status (2026-05-19):** active strict-Perl parity work at
the format/dump and package-loading boundary, followed by sandbox
long-tail cleanup (see [`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md)).
Current local verification is `cargo test --tests` **1328/0/0** and
the latest 100k-paper warning-subset sandbox result is **~99.4% OK**.
Full post-processing pipeline:
`latexml_oxide --format=html5 --dest=paper.html paper.tex`
produces complete HTML with cross-references, citations, MathML,
and XSLT. Release-profile wall time on the
[1910.01256](https://arxiv.org/abs/1910.01256) mini-benchmark is
**0.71 s** (vs ~1.11 s pdflatex idle).

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

### Releases

Prebuilt `x86_64-unknown-linux-gnu` binaries are attached to every
tagged release on the [Releases page](https://github.com/dginev/latexml-oxide/releases).
The binary is fully self-contained — all XSLT stylesheets, CSS, JS,
and RelaxNG schemas are embedded at build time, so no `resources/`
tree is needed alongside it. Tested on Ubuntu 22.04 LTS and later
(glibc ≥ 2.35) and Debian 12+.

**Debian / Ubuntu (`.deb`, declares runtime apt deps):**

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/latest/download/latexml-oxide_<VERSION>-1_amd64.deb
$ sudo apt install ./latexml-oxide_<VERSION>-1_amd64.deb
```

**Portable tarball:**

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/latest/download/latexml-oxide-<VERSION>-x86_64-unknown-linux-gnu.tar.gz
$ tar xzf latexml-oxide-<VERSION>-x86_64-unknown-linux-gnu.tar.gz
$ sudo cp latexml-oxide-<VERSION>-x86_64-unknown-linux-gnu/latexml_oxide /usr/local/bin/
```

Tarball users also need the runtime libraries:

```
$ sudo apt install libxml2 libxslt1.1 libkpathsea6 \
                   texlive-latex-base texlive-latex-extra texlive-science
```

Every asset has a SHA-256 sidecar file (`<name>.sha256`) for integrity
checking. Other platforms are not yet shipped — see
[docs/RELEASING.md](docs/RELEASING.md).

### Build from source

Requires a recent Rust `nightly` to compile.

We still need the non-perl OS dependencies from [get LaTeXML](https://math.nist.gov/~BMiller/LaTeXML/get.html),
but adapted for Rust bindings.

Example for Ubuntu:
```
$ sudo apt install libxml2-dev libxslt1-dev texlive-latex-base imagemagick libkpathsea-dev libkpathsea6 mold \
                   texlive texlive-latex-extra texlive-science \
                   texlive-bibtex-extra texlive-publishers poppler-utils
```

`poppler-utils` provides `pdftocairo`, used as the default fast PDF →
PNG/SVG rasterizer (≈25× faster than ImageMagick `convert` on
vector-heavy PDFs).

#### Optional but recommended: MuPDF tools (≈2× faster PDF conversion)

```
$ sudo apt install mupdf-tools
```

`mutool draw` (MuPDF) is tried **before** `pdftocairo` for PDF →
PNG/SVG conversion. Measured 2026-05-12: on a matplotlib scatter
PDF, mutool runs in 0.48 s vs pdftocairo's 0.86 s, and its SVG
output is ~4× more gzip-compressible. Falls through to `pdftocairo`
if `mutool` is not on PATH — install is optional.

#### Optional: inkscape fallback for vector-preserving PDF → SVG

For the opt-in `--graphics-svg-threshold-kb N` flag (see
[docs/SYNC_STATUS.md](docs/SYNC_STATUS.md) and upstream
[brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)):

```
$ sudo apt install inkscape
```

`inkscape` is used as the **last-resort** SVG converter when both
`mutool` and `pdftocairo` fail. The path is disabled by default; if
the flag is enabled but inkscape is missing at runtime, the pipeline
silently falls back to raster `convert`.

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
