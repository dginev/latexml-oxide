# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml)
[![release](https://img.shields.io/github/v/release/dginev/latexml-oxide?color=orange)](https://github.com/dginev/latexml-oxide/releases)
[![license: CC0-1.0](https://img.shields.io/badge/license-CC0--1.0-blue.svg)](LICENSE)
[![ported tests](https://img.shields.io/badge/ported%20tests-1533%2F0%2F0-32a852?style=flat)](https://github.com/dginev/latexml-oxide/issues/30)

This project is in active **beta**, approaching mainline LaTeXML parity. The full
conversion pipeline already works end-to-end —
`latexml_oxide --format=html5 --dest=paper.html paper.tex` produces complete HTML
with cross-references, citations, MathML, and XSLT — but treat the output as
not-yet-production-grade until parity is declared. Strict-Perl dump/format parity
is complete; remaining work is sandbox long-tail cleanup, tracked in
[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md).

### Why?

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
ships prebuilt binaries for **Linux** (x86-64 and aarch64/arm64, `.deb` +
portable tarball each), **macOS** (Apple Silicon and Intel tarballs), and
**Windows** (x86-64, a single self-contained `.exe` — from 0.7.4). The
binary is fully self-contained — all XSLT
stylesheets, CSS, JS, and RelaxNG schemas are embedded at build time and served
from memory, so no `resources/` tree is needed alongside it (a deliberate design
goal — see [docs/parity/OXIDIZED_DESIGN.md](docs/parity/OXIDIZED_DESIGN.md)). A working TeX
installation — **TeX Live or MiKTeX** — is still required at runtime for TeX
package/class/font resolution.
Every asset has a `<name>.sha256` sidecar for integrity checking.

Set the version once (use the latest from the Releases page):

```
$ VERSION=0.7.3
```

#### Ubuntu / Debian

The `.deb` declares its runtime apt dependencies (libraries + TeX Live + the
graphics tools), so this is the recommended path. Built on Ubuntu 22.04
(glibc 2.35), so it runs on Ubuntu 22.04+ and Debian 12+.

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide_${VERSION}-1_amd64.deb
$ sudo apt install ./latexml-oxide_${VERSION}-1_amd64.deb
```

Prefer the portable tarball? The binary is statically linked against
libxml2/libxslt/libkpathsea, so you only need the external tools + TeX Live:

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-unknown-linux-gnu.tar.gz
$ tar xzf latexml-oxide-$VERSION-x86_64-unknown-linux-gnu.tar.gz
$ sudo cp latexml-oxide-$VERSION-x86_64-unknown-linux-gnu/latexml_oxide /usr/local/bin/
$ sudo apt install imagemagick mupdf-tools poppler-utils ghostscript dvipng dvisvgm \
                   texlive-latex-base texlive-latex-extra texlive-science
```

On **64-bit ARM** (AWS Graviton, Ampere, Raspberry Pi OS 64-bit) swap `amd64` →
`arm64` in the `.deb` name and `x86_64-unknown-linux-gnu` → `aarch64-unknown-linux-gnu`
in the tarball name; everything else is identical. `uname -m` prints `x86_64` or
`aarch64`.

#### macOS (Apple Silicon / arm64 + Intel / x86_64)

**Recommended — Homebrew.** Installs the right binary for your Mac, pulls the
graphics tools it needs, and (because `brew` strips macOS's quarantine flag) runs
with no Gatekeeper "unidentified developer" warning:

```
$ brew install dginev/tap/latexml-oxide
```

You still need a TeX distribution (`brew info latexml-oxide` repeats this):

```
$ brew install texlive          # Homebrew's full TeX Live (~5 GB)
# …or MacTeX / BasicTeX — https://tug.org/mactex/  (put /Library/TeX/texbin on PATH)
```

**Alternative — download the tarball directly.** Pick the one matching your Mac
(`uname -m` prints `arm64` or `x86_64`):

```
# Apple Silicon
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-aarch64-apple-darwin.tar.gz
$ tar xzf latexml-oxide-$VERSION-aarch64-apple-darwin.tar.gz
$ sudo cp latexml-oxide-$VERSION-aarch64-apple-darwin/latexml_oxide /usr/local/bin/

# Intel (built with a macOS 10.13 deployment target, so it runs on older Intel Macs)
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-apple-darwin.tar.gz
$ tar xzf latexml-oxide-$VERSION-x86_64-apple-darwin.tar.gz
$ sudo cp latexml-oxide-$VERSION-x86_64-apple-darwin/latexml_oxide /usr/local/bin/

$ brew install imagemagick mupdf-tools poppler ghostscript dvisvgm
$ brew install texlive          # or install MacTeX / BasicTeX (provides dvipng)
```

Homebrew's `texlive` ships `libkpathsea`; with MacTeX/BasicTeX the binary instead
resolves TeX files through your distribution's `kpsewhich` executable (ensure
`/Library/TeX/texbin` is on `PATH`).

> **Gatekeeper / "unidentified developer" (tarball, browser downloads).** The
> `curl` + `tar xzf` install above is warning-free: terminal downloads and
> command-line `tar` don't set macOS's `com.apple.quarantine` flag. If you
> instead download the tarball in a **browser** and unpack it by double-clicking
> in Finder, macOS may refuse to run the binary as "from an unidentified
> developer" — the binaries are ad-hoc signed, not Apple-notarized. Clear it
> once, either way: unpack in Terminal with `tar xzf …`, **or** after copying the
> binary run `xattr -d com.apple.quarantine /usr/local/bin/latexml_oxide`
> (equivalently, right-click it in Finder → **Open**). The **Homebrew install
> above avoids this entirely**, since `brew` strips the quarantine flag.

#### Windows (x86_64)

*(From `0.7.4`.)* A single self-contained `.exe` — no installer, no runtime DLLs
(fully static: no VC++ redistributable needed). In PowerShell:

```
> $VERSION = "0.7.4"
> curl.exe -LO "https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-pc-windows-msvc.exe"
> .\latexml-oxide-$VERSION-x86_64-pc-windows-msvc.exe --version
```

Rename it to `latexml_oxide.exe` and put it on your `PATH`. A TeX distribution —
**TeX Live for Windows or MiKTeX** — must be on `PATH` for host TeX resolution;
the binary auto-selects the fast in-process backend on TeX Live and falls back to
subprocess `kpsewhich` on MiKTeX. ImageMagick (`magick`), Ghostscript
(`gswin64c` / MiKTeX `mgs`), and MuPDF (`mutool`) on `PATH` are optional, for
figure conversion.

#### Docker

A batteries-included image (`latexml_oxide` + a reproducible TeX Live + the
graphics tools) is published to the GitHub Container Registry for both amd64 and
arm64. No local TeX Live needed — bind-mount your document tree and convert:

```
$ docker run --rm -v "$PWD:/work" ghcr.io/dginev/latexml-oxide:$VERSION paper.tex
```

`:latest` tracks the most recent release. The container builds its own binary
against the image's TeX Live, so the embedded kernel dumps match the bundled
texmf tree exactly.

### System dependencies

The binary is self-contained (libxml2/libxslt/kpathsea are linked in), but at
runtime it **shells out** to external tools for graphics conversion and reads
TeX assets from your TeX Live tree. None are bundled — install the ones your
documents need. When a required tool is missing, the conversion log names it
**and the package to install**. The `.deb` declares all of these, so
`apt install ./latexml-oxide_*.deb` pulls them automatically.

| Tool (`command`) | apt package | Homebrew | Used for |
|---|---|---|---|
| `convert` | `imagemagick` | `imagemagick` | raster image conversion |
| `mutool` | `mupdf-tools` | `mupdf-tools` | primary PDF graphics (fast) |
| `pdftocairo` | `poppler-utils` | `poppler` | vector-SVG from PDF |
| `gs`, `ps2pdf` | `ghostscript` | `ghostscript` | PDF/PostScript conversion |
| `dvipng` | `dvipng` | TeX Live | raster LaTeX-image output |
| `dvisvgm` | `dvisvgm` | TeX Live | vector-SVG LaTeX-image output |
| `kpsewhich`, `latex`, `pdflatex`, `tftopl` | `texlive-latex-base` (+`-extra`, `-science`) | `texlive` / MacTeX | TeX package/class/font resolution |

### Build from source

Requires a recent Rust `nightly` to compile.

We still need the non-perl OS dependencies from [get LaTeXML](https://math.nist.gov/~BMiller/LaTeXML/get.html),
but adapted for Rust bindings.

Example for Ubuntu:
```
$ sudo apt install libxml2-dev libxslt1-dev texlive-latex-base imagemagick ghostscript libkpathsea-dev libkpathsea6 mold \
                   texlive texlive-latex-extra texlive-science \
                   texlive-bibtex-extra texlive-publishers poppler-utils
```

`ghostscript` (`gs`) is listed explicitly: it is the EPS/PS rasterizer and
ImageMagick's PS/EPS/PDF delegate, so every EPS/PS figure needs it. It is only a
*Recommends* of `imagemagick`, so a host configured with `APT::Install-Recommends
"false"` would silently omit it and fail every EPS/PS conversion.

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

`mutool` (MuPDF) and `pdftocairo` (poppler-utils) are optional but recommended:
they convert PDF figures faster than ImageMagick. latexml-oxide tries the
available delegates fastest-first and falls back through the chain, so a figure is
never lost when a tool is missing.

### Build profiles

`Cargo.toml` defines named profiles for day-to-day development (`test`), the
RAM-bounded CI runner (`ci`), local perf measurement (`release`), and the shipped
distribution binaries (`maxperf` / `maxperf-cortex`). Use the default profile
(`cargo build` / `cargo test`, no flag) for everyday work; see
[CLAUDE.md](CLAUDE.md) for the full profile table.

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

4. Generate a rustdoc-styled HTML5 site for a RelaxNG (`.rnc`) schema — see [docs/performance/SCHEMA_DOCUMENTATION.md](docs/performance/SCHEMA_DOCUMENTATION.md).

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

This workspace is heavy for rust-analyzer (large proc-macro bodies); the
checked-in `.vscode/settings.json` ships a stability profile that keeps the IDE
responsive. Terminal `cargo build` / `cargo test` are unaffected.

To generate the project documentation locally, run:
```bash
$ cargo doc --workspace --no-deps --open
```

**IMPORTANT:** There is a compile-time plugin that collects the files in the test suite. 
This means that when adding a new test `[name].tex` and `[name].xml` pair of files, you may need to manually execute `cargo clean` to rediscover the entry.

### License

latexml-oxide's source code and original resources are dedicated to the public
domain under [CC0 1.0 Universal](LICENSE).

The release binary also embeds and links third-party material that keeps its own
license — most notably compiled TeX format dumps derived from TeX Live (the
LaTeX kernel, LPPL 1.3c; plain TeX, Knuth) and the system libxml2/libxslt
libraries (MIT). These are attributed in [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES),
which is bundled with every release. The public-domain dedication above applies
to the latexml-oxide source and original resources, **not** to that embedded or
linked third-party material. Full breakdown: [`docs/release/LICENSE_INVENTORY.md`](docs/release/LICENSE_INVENTORY.md).
