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

**Try it first — no install: [latexml.rs/editor](https://latexml.rs/editor)** — type LaTeX,
see the converted document live.

```bash
# A document → HTML5, with MathML for the math (the format follows the extension)
$ latexml_oxide paper.tex --destination=paper.html

# The same document as an EPUB 3 book
$ latexml_oxide paper.tex --destination=paper.epub

# A book split into one page per chapter (streamed; 64 or more pages render in parallel)
$ latexml_oxide book.tex --splitat=chapter --destination=book/index.html

# One formula → Presentation MathML (--cmml for Content MathML)
$ latexmlmath_oxide 'a^2+b^2=c^2'
```

Packages and classes without a LaTeXML binding are read from their own `.sty`/`.cls` sources;
`--noincludestyles` restores Perl LaTeXML's bindings-only behaviour. The release binaries bundle the
[ar5iv](https://github.com/dginev/ar5iv-css) stylesheet: `--css=ar5iv.css --nodefaultresources` uses it in place of
`LaTeXML.css`. `latexmlmath_oxide` comes with `cargo +nightly install latexml` or a build from source.
More in [docs/USAGE.md](https://github.com/dginev/latexml-oxide/blob/main/docs/USAGE.md); `--help` lists every option.

## Install

Prebuilt, self-contained binaries for Linux, macOS and Windows are on the
[Releases page](https://github.com/dginev/latexml-oxide/releases); Homebrew, a `.deb`, Docker and
`cargo install latexml` work too. Basic documents using built-in bindings convert without TeX; raw packages and
classes, `expl3` and fonts need a TeX installation (TeX Live or MiKTeX). On a TeX tree the binary has no precompiled
format for, the first conversion builds one (about 10 s) and caches it.
Per-platform steps and the system dependencies: [docs/INSTALL.md](https://github.com/dginev/latexml-oxide/blob/main/docs/INSTALL.md).
The Windows binary is not code-signed; a Microsoft Store package, signed by Microsoft, is planned for a 0.8.x release
(see the [code signing policy](https://github.com/dginev/latexml-oxide/blob/main/docs/release/CODE_SIGNING_POLICY.md)).

## Extending: runtime bindings (Rhai)

Every released binary reads TeX bindings written in [Rhai](https://rhai.rs) — no Rust toolchain, no recompile.
Drop a file named `<pkg>.sty.rhai` (or `<class>.cls.rhai`) **next to your document**, and `\usepackage{<pkg>}` finds
and loads it:

```rust
// mypkg.sty.rhai  —  note: backslashes are doubled in Rhai strings
DefMacro("\\greet{}", |gullet, name| "Hello, " + name + "!");
DefConstructor("\\mytext{}", "<ltx:text class=\"rhai\">#1</ltx:text>", #{ enterHorizontal: true });
```

Start with the [Rhai guide](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/RHAI_BINDINGS_GUIDE.md); the
[Perl | Rust | Rhai cheat sheet](https://github.com/dginev/latexml-oxide/blob/main/latexml_contrib/src/script_bindings/CHEATSHEET.md)
maps each Perl LaTeXML call to its Rhai form.

## Documentation

[Installing](https://github.com/dginev/latexml-oxide/blob/main/docs/INSTALL.md) ·
[Usage](https://github.com/dginev/latexml-oxide/blob/main/docs/USAGE.md) ·
[Changes](https://github.com/dginev/latexml-oxide/blob/main/CHANGELOG.md) ·
[Rust API](https://dginev.github.io/latexml-oxide/latexml/)

Everything else (environment variables, the files a conversion writes, building from source, supported packages,
Rhai bindings, how we differ from Perl LaTeXML, performance, releases and licensing) is indexed by theme in
[docs/README.md](https://github.com/dginev/latexml-oxide/blob/main/docs/README.md).

## License

latexml-oxide's source code and original resources are dedicated to the public
domain under [CC0 1.0 Universal](https://github.com/dginev/latexml-oxide/blob/main/LICENSE).

The release binary also embeds and links third-party material that keeps its own license, most notably: compiled
TeX format dumps derived from TeX Live (the LaTeX kernel, LPPL 1.3c; plain TeX, Knuth), the libxml2/libxslt libraries
(MIT), the libmarpa parser engine (MIT, with LGPL parts), libkpathsea (LGPL-2.1), the bundled `ar5iv.css` (MIT), and,
on Windows only, GNU libiconv (LGPL-2.1+).

These are attributed in [`THIRD-PARTY-NOTICES`](https://github.com/dginev/latexml-oxide/blob/main/THIRD-PARTY-NOTICES),
published as an asset on every release and bundled inside every download: the tarballs, the Windows `.zip`, the
`.deb`s, and the container images (under `/usr/local/share/doc/latexml-oxide`).
The public-domain dedication above applies to the latexml-oxide source and original resources,
**not** to that embedded or linked third-party material.
Full breakdown: [`docs/release/LICENSE_INVENTORY.md`](https://github.com/dginev/latexml-oxide/blob/main/docs/release/LICENSE_INVENTORY.md).
