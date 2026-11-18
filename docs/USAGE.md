# Usage

[← README](../README.md) · [Install](INSTALL.md) · [Building](BUILDING.md) · [docs index](README.md)

**Try it first — no install: [latexml.rs/editor](https://latexml.rs/editor)** — type LaTeX,
see the converted document live.

Two binaries: `latexml_oxide` for documents, `latexmlmath_oxide` for a single formula.

```bash
# A document → HTML5, with MathML for the math (format inferred from the extension)
$ latexml_oxide paper.tex --destination=paper.html

# One formula → Presentation MathML (the default; --cmml for Content MathML)
$ latexmlmath_oxide 'a^2+b^2=c^2'
```

`--format` takes `html5`, `html`, `xhtml`, `xml`, or `epub` (a zipped EPUB 3) — inferred from the
`--destination` extension when omitted. `--help` lists every option.
Working from a checkout instead of an installed binary? Prefix with
`cargo run --bin latexml_oxide --`.

## Your own bindings (`.rhai`)

A binding tells the converter what a package's or class's commands mean. Add your own without a recompile as a Rhai
script named after the file it binds plus `.rhai`: `\usepackage{mypkg}` loads `mypkg.sty.rhai`, a class
`myclass.cls.rhai`, found beside the document or in a `--path=DIR` directory, before any built-in binding of that name;
`--preload=mypkg.sty` loads it for every document. A failing binding is reported as `Error:script:<file>.rhai`, located
at its line in that file. The guide: [`parity/RHAI_BINDINGS_GUIDE.md`](parity/RHAI_BINDINGS_GUIDE.md).

A binding can run external programs (`Command`); each run is named by an `Info:script:Command` line. Set
`LATEXML_DISABLE_SHELL_ESCAPE` (any value) to refuse them — required where the input is untrusted, since a `.rhai`
shipped beside a source loads with it ([`release/SAFETY.md`](release/SAFETY.md)).

**Packages and classes without a LaTeXML binding are read raw** from the TeX tree (and the document's directory), as
pdflatex reads them; a binding always wins. The runaway limits are ar5iv's: 48,000,000 conditionals and 1,299,999
absorbed boxes, a 5,000,000-token pushback, and a token budget of 20G raised with the source size (`--token-limit`).
To get Perl LaTeXML's behaviour — bindings only, a missing package skipped with a warning, an unknown class served
by OmniBus — pass `--noincludestyles` (or `--preload='[norawstyles,norawclasses]latexml.sty'`); latexml.sty's `iflimit`, `absorblimit` and
`pushbacklimit` options set those limits, and `iflimit=0,absorblimit=0` lifts the two Perl leaves
unset (OXIDIZED_DESIGN_DIVERGENCES #1080).
