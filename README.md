# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.7.0-orange.svg) 
[![ported tests](https://img.shields.io/badge/ported%20tests%20-%201454%2F0%2F0-%20%2332a852?style=flat)
](https://github.com/dginev/latexml-oxide/issues/30)

This project is in active **beta**, approaching mainline LaTeXML parity. The full
conversion pipeline already works end-to-end —
`latexml_oxide --format=html5 --dest=paper.html paper.tex` produces complete HTML
with cross-references, citations, MathML, and XSLT — but treat the output as
not-yet-production-grade until parity is declared.

**Status:** strict-Perl dump/format parity is complete; remaining work is
sandbox long-tail cleanup (see [`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md)).
Local verification: `cargo test --tests` **1454/0/0**; the latest 100k-paper
warning-subset sandbox run is **~99.4% OK**. Release-profile wall time on the
[1910.01256](https://arxiv.org/abs/1910.01256) mini-benchmark is **0.71 s**
(vs ~1.11 s pdflatex idle).

### Why?

The three main reasons:

  * LaTeXML is **too slow** for large-scale production use.
  * Perl 5's ecosystem and tooling have **aged out of the mainstream**.
  * LaTeXML is **urgently needed** for turning LaTeX sources into responsive,
    accessible web documents.

Design goals:

  * Faithfully rewrite the LaTeXML code base as-is, attempting to be as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring Perl idioms
  * Carefully address the newly required resource constraints

### Install (prebuilt binaries)

Every tagged release on the [Releases page](https://github.com/dginev/latexml-oxide/releases)
ships prebuilt binaries for **Linux x86-64** (`.deb` + portable tarball) and
**macOS Apple Silicon** (tarball). The binary is fully self-contained — all XSLT
stylesheets, CSS, JS, and RelaxNG schemas are embedded at build time and served
from memory, so no `resources/` tree is needed alongside it (a deliberate design
goal — see [docs/OXIDIZED_DESIGN.md](docs/OXIDIZED_DESIGN.md)). A working TeX Live
installation is still required at runtime for TeX package/class/font resolution.
Every asset has a `<name>.sha256` sidecar for integrity checking.

Set the version once (use the latest from the Releases page):

```
$ VERSION=0.7.0
```

#### Ubuntu / Debian

The `.deb` declares its runtime apt dependencies (libraries + TeX Live + the
graphics tools), so this is the recommended path. Built on Ubuntu 22.04
(glibc 2.35), so it runs on Ubuntu 22.04+ and Debian 12+.

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide_${VERSION}-1_amd64.deb
$ sudo apt install ./latexml-oxide_${VERSION}-1_amd64.deb
```

Prefer the portable tarball? Install the runtime dependencies yourself:

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-unknown-linux-gnu.tar.gz
$ tar xzf latexml-oxide-$VERSION-x86_64-unknown-linux-gnu.tar.gz
$ sudo cp latexml-oxide-$VERSION-x86_64-unknown-linux-gnu/latexml_oxide /usr/local/bin/
$ sudo apt install libxml2 libxslt1.1 libkpathsea6 imagemagick mupdf-tools \
                   texlive-latex-base texlive-latex-extra texlive-science
```

#### macOS (Apple Silicon / arm64 only)

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-aarch64-apple-darwin.tar.gz
$ tar xzf latexml-oxide-$VERSION-aarch64-apple-darwin.tar.gz
$ sudo cp latexml-oxide-$VERSION-aarch64-apple-darwin/latexml_oxide /usr/local/bin/
$ brew install libxml2 libxslt imagemagick mupdf-tools
$ brew install texlive          # or install MacTeX / BasicTeX
```

Homebrew's `texlive` ships `libkpathsea`; with MacTeX/BasicTeX the binary instead
resolves TeX files through your distribution's `kpsewhich` executable (ensure
`/Library/TeX/texbin` is on `PATH`). Intel Macs are not yet a published target —
see [docs/RELEASING.md](docs/RELEASING.md) for the platform roadmap.

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

Example for macOS (Apple Silicon / arm64; the full test suite runs on macOS CI):

```
$ brew install libxml2 libxslt texlive
$ export PKG_CONFIG_PATH="$(brew --prefix libxml2)/lib/pkgconfig:$(brew --prefix libxslt)/lib/pkgconfig"
$ cargo build --bin latexml_oxide
```

libxml2 and libxslt are keg-only in Homebrew, hence the
`PKG_CONFIG_PATH` export (put it in your shell profile for regular
work). Homebrew's `texlive` ships `libkpathsea` + `kpathsea.pc`, so the
build links it in-process — the fastest configuration.

**Using MacTeX/BasicTeX instead?** That also works: MacTeX ships *no*
`libkpathsea` at all (no header, no dylib, no `.pc`), so the build
prints a one-time `kpathsea_sys` notice and falls back to resolving
TeX files through your distribution's own `kpsewhich` executable
(`/Library/TeX/texbin` must be on PATH — the MacTeX installer sets
this up). Same conversions, slightly slower cold file lookups. You
still need `brew install libxml2 libxslt` either way.

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
| **`maxperf`** | distribution / published release artifact | Smallest, fastest binary: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, stripped. Slowest build; used by `tools/make_release.sh` for the shipped binaries. |

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
