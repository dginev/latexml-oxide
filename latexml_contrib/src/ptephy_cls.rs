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

  // ptephy frontmatter.
  DefMacro!("\\preprintnumber[]{}", "");
  DefMacro!("\\subjectindex{}", "");

  // \ack — Acknowledgements section opener (used in OUP / PTEP class).
  // Render as a starred section to drop a heading without numbering.
  DefMacro!("\\ack", "\\section*{Acknowledgements}");
  DefMacro!("\\acknow{}", "\\section*{Acknowledgements}#1");
});
