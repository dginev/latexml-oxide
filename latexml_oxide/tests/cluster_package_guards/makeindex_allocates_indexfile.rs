//! OXIDIZED_DESIGN #163: `\makeindex` allocates the `\@indexfile` write
//! stream (real latex.ltx contract) while staying otherwise nooped, so raw
//! doc.sty/l3doc-style `\protected@write\@indexfile{…}` works instead of
//! erroring `undefined \@indexfile` (Perl noops it entirely and fatals on
//! l3kernel's own manuals). The semantic `\index` constructor must remain
//! in charge — \makeindex must NOT restore the kernel's raw `\index`.

const TEX: &str = "\\documentclass{article}\n\
    \\makeindex\n\
    \\begin{document}\n\
    body\\index{alpha}\n\
    \\makeatletter\n\
    \\ifdefined\\@indexfile STREAMDEFINED\\else STREAMMISSING\\fi\n\
    \\protected@write\\@indexfile{}{raw-write-payload}\n\
    \\makeatother\n\
    \\end{document}\n";

#[test]
fn stream_allocated_semantic_index_intact() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "\\makeindex + raw \\@indexfile write must be error-free:\n{stderr}",
  );
  assert!(
    xml.contains("STREAMDEFINED"),
    "\\@indexfile not allocated:\n{xml}"
  );
  // Semantic \index survived — an indexmark, and the raw payload is NOT
  // typeset into the document.
  assert!(
    xml.contains("indexmark") && !xml.contains("raw-write-payload"),
    "semantic \\index must stay in charge and raw writes must not leak:\n{xml}",
  );
}
