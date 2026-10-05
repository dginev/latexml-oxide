# singletons — root-cause notes

## tocloft_toc_entries (RED, 62k)
tocloft has no binding (Perl or Rust), so under `rawstyles` its `\tableofcontents` (tocloft.sty:130-139) replaces
LaTeXML's: it prints its own heading and runs `\@starttoc{toc}`, which reads a `.toc` LaTeXML never writes, so the
`<TOC>` is gone and only "Contents" is left. SHARED (Perl `[rawstyles]` gives the same). Fix site: a tocloft binding
that keeps the kernel's `\tableofcontents`/`\listoffigures`/`\listoftables` (they generate the lists from the
document) while defining tocloft's `\cft…` parameters for classes that set them.
