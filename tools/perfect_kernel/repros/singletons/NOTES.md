# singletons — root-cause notes

## tocloft_toc_entries (RED 62k → GREEN 62n)
tocloft has no binding (Perl or Rust), so under `rawstyles` its `\tableofcontents` (tocloft.sty:130-139) replaces
LaTeXML's: it prints its own heading and runs `\@starttoc{toc}`, which reads a `.toc` LaTeXML never writes, so the
`<TOC>` is gone and only "Contents" is left. SHARED (Perl `[rawstyles]` gives the same). Fix site: a tocloft binding
that keeps the kernel's `\tableofcontents`/`\listoffigures`/`\listoftables` (they generate the lists from the
document) while defining tocloft's `\cft…` parameters for classes that set them.
GREEN since 62n: `latexml_contrib/src/tocloft_sty.rs` interprets tocloft raw and puts the kernel's three lists back
(`\lx@tableofcontents` & co., sect04.rs) under tocloft's own condition and at its own times. Guard
`perfect_kernel_batch61::tocloft_keeps_the_kernel_lists`.

`includepdf_places_its_pages` (78j, OXIDIZED_DESIGN_DIVERGENCES #846; beyond Perl): Perl's pdfpages.sty.ltxml:30-36
prints "See pages X of ⟨link⟩". pdfpages_sty.rs places each page the `pages` list names as `\includegraphics[page=N]`
in its own paragraph (an open range reads the PDF's page count); a PDF not found keeps the link.
`includepdf_page_past_the_last` (78j r2): a page past the file's last is an Error placing nothing, pdfpages.sty:909-920
`\AM@checkpagenumber` ("Page 13 of `example-image-a4-numbered.pdf' does not exist"); r1 placed the last page again.
