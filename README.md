# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml)
[![release](https://img.shields.io/github/v/release/dginev/latexml-oxide?color=orange)](https://github.com/dginev/latexml-oxide/releases)
[![license: CC0-1.0](https://img.shields.io/badge/license-CC0--1.0-blue.svg)](LICENSE)
[![ar5iv: 98%](https://img.shields.io/badge/ar5iv-98%25-brightgreen.svg)](https://ar5iv.labs.arxiv.org)
[![arXiv](https://img.shields.io/badge/arXiv-2605.16562-b31b1b.svg)](https://arxiv.org/abs/2605.16562)

latexml-oxide turns LaTeX sources into accessible web documents.
It is used in the [ar5iv Lab](https://ar5iv.labs.arxiv.org) and in the production
[arXiv HTML Papers](https://info.arxiv.org/about/accessible_HTML.html) pages, converting
98% of arXiv's articles without an error.
The project is in active public **beta** as of July 2026, approaching mainline LaTeXML parity.

### Why?

  * LaTeXML is **too slow** for large-scale production use.
  * Perl 5's ecosystem and tooling have **aged out of the mainstream**.
  * LaTeXML is **urgently needed** for turning LaTeX sources into responsive, accessible web documents.

### Goals

  * Faithfully rewrite the LaTeXML code base as-is, staying as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring Perl idioms.
  * Carefully address the newly required resource constraints.
  * Offer a platform for iteratively increasing coverage to 90% of all of arXiv.

## Usage

**Try it first — no install: [latexml.rs/editor](https://latexml.rs/editor)** — type LaTeX,
see the converted document live.

```bash
# A document → HTML5, with MathML for the math (format inferred from the extension)
$ latexml_oxide paper.tex --destination=paper.html

# One formula → Presentation MathML (the default; --cmml for Content MathML)
$ latexmlmath_oxide 'a^2+b^2=c^2'
```

More in [docs/USAGE.md](docs/USAGE.md); `--help` lists every option.

## Install

Prebuilt, self-contained binaries for Linux, macOS and Windows are on the
[Releases page](https://github.com/dginev/latexml-oxide/releases); Homebrew, a `.deb`, Docker and
`cargo install latexml` work too. A TeX installation (TeX Live or MiKTeX) is needed at runtime.
Per-platform steps and the system dependencies: [docs/INSTALL.md](docs/INSTALL.md).

## Extending: runtime bindings (Rhai)

Every released binary reads TeX bindings written in [Rhai](https://rhai.rs) — no Rust toolchain, no recompile.
Drop a file named `<pkg>.sty.rhai` (or `<class>.cls.rhai`) **next to your document**, and `\usepackage{<pkg>}` finds
and loads it:

```rust
// mypkg.sty.rhai  —  note: backslashes are doubled in Rhai strings
DefMacro("\\greet{}", |name| "Hello, " + name + "!");
DefConstructor("\\mytext{}", "<ltx:text class=\"rhai\">#1</ltx:text>");
```

Start with the [Rhai guide](docs/parity/RHAI_BINDINGS_GUIDE.md).

## Documentation

| Topic | Where |
|---|---|
| All internal docs, by theme | [docs/README.md](docs/README.md) |
| Installing (per platform, Docker, system dependencies, crates.io, as a library) | [docs/INSTALL.md](docs/INSTALL.md) |
| Usage and the command line | [docs/USAGE.md](docs/USAGE.md) |
| Building from source, build profiles, development tips, Cargo features | [docs/BUILDING.md](docs/BUILDING.md) |
| Writing a Rhai binding | [docs/parity/RHAI_BINDINGS_GUIDE.md](docs/parity/RHAI_BINDINGS_GUIDE.md), [worked examples](docs/examples/) |
| Rhai API reference · Perl \| Rust macro \| Rhai cheat sheet | [interface](https://dginev.github.io/latexml-oxide/latexml_contrib/script_bindings/interface/index.html) · [cheat sheet](https://dginev.github.io/latexml-oxide/latexml_contrib/script_bindings/cheatsheet/index.html) |
| Rust API (rustdoc) | [dginev.github.io/latexml-oxide/latexml/](https://dginev.github.io/latexml-oxide/latexml/) |
| Changes per release | [CHANGELOG.md](CHANGELOG.md) |
| How and where we differ from Perl LaTeXML | [OXIDIZED_DESIGN](docs/parity/OXIDIZED_DESIGN.md), [divergences](docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md), [known Perl errors](docs/parity/KNOWN_PERL_ERRORS.md), [fidelity charter](docs/parity/FIDELITY_CHARTER.md) |
| Performance | [docs/performance/PERFORMANCE.md](docs/performance/PERFORMANCE.md) |
| Releases and licensing | [RELEASING](docs/release/RELEASING.md), [THIRD-PARTY-NOTICES](THIRD-PARTY-NOTICES), [LICENSE_INVENTORY](docs/release/LICENSE_INVENTORY.md) |

## License

latexml-oxide's source code and original resources are dedicated to the public
domain under [CC0 1.0 Universal](LICENSE).

The release binary also embeds and links third-party material that keeps its own license:
most notably compiled TeX format dumps derived from TeX Live (the LaTeX kernel, LPPL 1.3c; plain TeX, Knuth),
the libxml2/libxslt libraries (MIT), the libmarpa parser engine (MIT, with LGPL parts)
and libkpathsea (LGPL-2.1).

These are attributed in [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES), published as an asset on every release
and bundled inside every download — the tarballs, the Windows `.zip`, the `.deb`s, and the container
images (under `/usr/local/share/doc/latexml-oxide`).
The public-domain dedication above applies to the latexml-oxide source and original resources,
**not** to that embedded or linked third-party material.
Full breakdown: [`docs/release/LICENSE_INVENTORY.md`](docs/release/LICENSE_INVENTORY.md).
