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
