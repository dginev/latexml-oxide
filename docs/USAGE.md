# Usage

[← README](../README.md) · [Install](INSTALL.md) · [Building](BUILDING.md) · [docs index](README.md)

**Try it first — no install: [latexml.rs/editor](https://latexml.rs/editor)** — type LaTeX,
see the converted document live.

There are two binaries: `latexml_oxide` converts documents and ships in every prebuilt release; `latexmlmath_oxide`
converts a single formula and comes with `cargo +nightly install latexml` or a build from source. `--help` lists every
option. From a checkout, prefix a command with `cargo run --bin latexml_oxide --`.

```bash
# A document → HTML5, with MathML for the math (the format follows the extension)
$ latexml_oxide paper.tex --destination=paper.html

# The same document as an EPUB 3 book
$ latexml_oxide paper.tex --destination=paper.epub

# Experimental: add a UnicodeMath annotation beside Presentation MathML (or use it alone with --nopmml)
$ latexml_oxide paper.tex --unicodemath --destination=paper.html

# One formula → Presentation MathML (--cmml for Content MathML; experimental: --unicodemath)
$ latexmlmath_oxide 'a^2+b^2=c^2'
```

## Output formats

`--format` takes `html5` (or `html`), `xhtml`, `xml`, or `epub` (or `epub3`; a zipped EPUB 3). Without it, the format
follows the `--destination` extension: `.html`, `.xhtml`, `.xml`, `.epub`, and `.zip` (HTML5 packed into a zip
archive).

## Stylesheets

The page links `LaTeXML.css`, written beside it. For the [ar5iv](https://github.com/dginev/ar5iv-css) look, the
prebuilt release binaries and the CLI container image bundle `ar5iv.css`: `--css=ar5iv.css --nodefaultresources`
writes it beside the page in place of `LaTeXML.css` (most of its rules are layered, so `LaTeXML.css`, if kept,
overrides them). A build from source and the cortex-worker image lack it; pass the path to a local copy instead.

A stylesheet or script already in the destination is used as-is; only a missing or empty one is written from the
bundled copy. A file of that name next to the `.tex` (or on `--path`) is used instead of the bundled one and copied
over the destination's. So a reused output folder keeps the `LaTeXML.css`/`ar5iv.css` an older version wrote: delete
the file to get the new one.

## Packages, classes and limits

**Packages and classes without a LaTeXML binding are read raw** from the TeX tree (and the document's directory), as
pdflatex reads them; a binding always wins. To get Perl LaTeXML's behaviour instead — bindings only, a missing package
skipped with a warning, an unknown class served by OmniBus — pass `--noincludestyles` (or
`--preload='[norawstyles,norawclasses]latexml.sty'`).

Runaway input stops at these limits:

| Limit | Value | Set with |
|---|---|---|
| conditionals | 480,000,000 (ten times ar5iv's, as the count accumulates over a long document) | latexml.sty's `iflimit` |
| absorbed boxes | 1,299,999 | latexml.sty's `absorblimit` |
| pushback | 5,000,000 tokens | latexml.sty's `pushbacklimit` |
| tokens | 20G, raised with the source size | `--token-limit` (kept as given) |

`iflimit=0,absorblimit=0` lifts the two limits Perl leaves unset
([OXIDIZED_DESIGN_DIVERGENCES #1080](parity/OXIDIZED_DESIGN_DIVERGENCES.md)).

The first conversion on a TeX tree the binary has no precompiled format for builds the format once (about 10 s) and
caches it; see [INSTALL.md](INSTALL.md#prebuilt-binaries).

## Large documents

`--splitat=chapter` (or `section`, …) writes one page per unit. Every split run streams its pages from the converted
document on disk, and a run of 64 or more pages renders them on up to 8 worker processes, within `--max-memory`.
A conversion into a folder keeps the names given to its images in `LaTeXML-oxide.cache` there (`--nocache` keeps
none). [ENVIRONMENT.md](ENVIRONMENT.md) has the variables that tune both; [INTERMEDIATE_FILES.md](INTERMEDIATE_FILES.md)
lists every file a conversion writes.

## Your own bindings (`.rhai`)

A binding tells the converter what a package's or class's commands mean. Add your own without a recompile as a Rhai
script named after the file it binds plus `.rhai`: `\usepackage{mypkg}` loads `mypkg.sty.rhai`, a class
`myclass.cls.rhai`, found beside the document or in a `--path=DIR` directory, before any built-in binding of that name;
`--preload=mypkg.sty` loads it for every document. A failing binding is reported as `Error:script:<file>.rhai`, located
at its line in that file. Start with the guide, [`parity/RHAI_BINDINGS_GUIDE.md`](parity/RHAI_BINDINGS_GUIDE.md), and
the [Perl | Rust | Rhai cheat sheet](../latexml_contrib/src/script_bindings/CHEATSHEET.md).

A binding can run external programs (`Command`); each run is named by an `Info:script:Command` line. Set
`LATEXML_DISABLE_SHELL_ESCAPE` (any value) to refuse them — required where the input is untrusted, since a `.rhai`
shipped beside a source loads with it ([`release/SAFETY.md`](release/SAFETY.md)).

## Environment variables

Memory, dumps, graphics, the render workers, TeX Live paths and more: [ENVIRONMENT.md](ENVIRONMENT.md), also the
rustdoc page `latexml::environment`.
