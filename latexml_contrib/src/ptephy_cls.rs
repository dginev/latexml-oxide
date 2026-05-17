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
  // Used as `\ack <paragraph>` (no body) — keep as starred section to
  // open a heading; the following paragraph is the natural body.
  DefMacro!("\\ack", "\\section*{Acknowledgements}");
  // \acknow{body} — bracketed form. Emit as structural
  // ltx:acknowledgements (post-processors map to canonical role/styling).
  DefConstructor!("\\acknow{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>");
});
