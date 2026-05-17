//! Stub for ptephy.cls (Progress of Theoretical and Experimental Physics).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");

  // ptephy frontmatter — preserve as ltx:note (content-preserving).
  // Both args carry author-typed data: a preprint identifier and a
  // PTEP subject-area code (used for indexing). Silent gobble would
  // lose both.
  DefMacro!("\\preprintnumber[]{}",
    "\\@add@frontmatter{ltx:note}[role=preprintnumber]{#2}");
  DefMacro!("\\subjectindex{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PTEP-subject]{#1}");

  // \ack — Acknowledgements section opener (used in OUP / PTEP class).
  // Used as `\ack <paragraph>` (no body) — keep as starred section to
  // open a heading; the following paragraph is the natural body.
  DefMacro!("\\ack", "\\section*{Acknowledgements}");
  // \acknow{body} — bracketed form. Emit as structural
  // ltx:acknowledgements (post-processors map to canonical role/styling).
  DefConstructor!("\\acknow{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>");
});
