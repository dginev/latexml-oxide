# A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/latexml-oxide/actions/workflows/CI.yml)
[![release](https://img.shields.io/github/v/release/dginev/latexml-oxide?color=orange)](https://github.com/dginev/latexml-oxide/releases)
[![license: CC0-1.0](https://img.shields.io/badge/license-CC0--1.0-blue.svg)](https://github.com/dginev/latexml-oxide/blob/main/LICENSE)
[![ar5iv: 98%](https://img.shields.io/badge/ar5iv-98%25-brightgreen.svg)](https://ar5iv.labs.arxiv.org)
[![arXiv](https://img.shields.io/badge/arXiv-2605.16562-b31b1b.svg)](https://arxiv.org/abs/2605.16562)

latexml-oxide turns LaTeX sources into accessible web documents: HTML5 with MathML, XML, and EPUB 3.
It powers the [ar5iv Lab](https://ar5iv.labs.arxiv.org) and the production
[arXiv HTML Papers](https://info.arxiv.org/about/accessible_HTML.html), converting 98% of arXiv's articles without an
error. The project is in public **beta**.

### Why?

  * LaTeXML is **too slow** for large-scale production use.
  * Perl 5's ecosystem and tooling have **aged out of the mainstream**.
  * LaTeXML is **urgently needed** for turning LaTeX sources into responsive, accessible web documents.

### Goals

  * Faithfully rewrite the LaTeXML code base, staying as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring Perl idioms.
  * Carefully address the newly required resource constraints.
  * Raise the coverage and fidelity of arXiv conversions beyond 98%.

## Usage

**Try it first, no install:** [latexml.rs/editor](https://latexml.rs/editor) — type LaTeX, see the result live.

```bash
# A document → HTML5 with MathML (the output format follows the extension)
$ latexml_oxide paper.tex --destination=paper.html

# … or an EPUB 3 book
$ latexml_oxide paper.tex --destination=paper.epub

# A book split into one page per chapter
$ latexml_oxide book.tex --splitat=chapter --destination=book/index.html
```

Packages and classes without a LaTeXML binding are read from their own `.sty`/`.cls` sources. `--help` lists every
option; [docs/USAGE.md](https://github.com/dginev/latexml-oxide/blob/main/docs/USAGE.md) explains them.

## Install

Self-contained binaries for Linux, macOS and Windows are on the
[Releases page](https://github.com/dginev/latexml-oxide/releases); Homebrew, a `.deb`, Docker and
`cargo install latexml` work too. Raw packages, `expl3` and fonts need a TeX installation (TeX Live or MiKTeX).
Steps per platform: [docs/INSTALL.md](https://github.com/dginev/latexml-oxide/blob/main/docs/INSTALL.md).

## Extending with Rhai

Released binaries read TeX bindings written in [Rhai](https://rhai.rs), with no Rust toolchain. Put a
`<pkg>.sty.rhai` next to your document and `\usepackage{<pkg>}` loads it:

```rust
// mypkg.sty.rhai (backslashes are doubled in Rhai strings)
DefMacro("\\greet{}", |gullet, name| "Hello, " + name + "!");
```

Start with the [Rhai guide](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/RHAI_BINDINGS_GUIDE.md); the
[Perl | Rust | Rhai cheat sheet](https://github.com/dginev/latexml-oxide/blob/main/latexml_contrib/src/script_bindings/CHEATSHEET.md)
maps each Perl LaTeXML call to its Rhai form.

## Documentation

[Installing](https://github.com/dginev/latexml-oxide/blob/main/docs/INSTALL.md) · [Usage](https://github.com/dginev/latexml-oxide/blob/main/docs/USAGE.md) · [Changes](https://github.com/dginev/latexml-oxide/blob/main/CHANGELOG.md) ·
[Rust API](https://dginev.github.io/latexml-oxide/latexml/) · [all docs, by theme](https://github.com/dginev/latexml-oxide/blob/main/docs/README.md)

## License

The source code and original resources are dedicated to the public domain under
[CC0 1.0 Universal](https://github.com/dginev/latexml-oxide/blob/main/LICENSE). The release binaries also embed or link third-party material under its own license:
TeX Live's format dumps (LaTeX kernel LPPL 1.3c; plain TeX, Knuth), libxml2/libxslt (MIT), libmarpa (MIT, with LGPL
parts), libkpathsea (LGPL-2.1), `ar5iv.css` (MIT) and, on Windows, GNU libiconv (LGPL-2.1+). That material is attributed in
[`THIRD-PARTY-NOTICES`](https://github.com/dginev/latexml-oxide/blob/main/THIRD-PARTY-NOTICES), which ships with every download, and itemised in
[`LICENSE_INVENTORY.md`](https://github.com/dginev/latexml-oxide/blob/main/docs/release/LICENSE_INVENTORY.md).
