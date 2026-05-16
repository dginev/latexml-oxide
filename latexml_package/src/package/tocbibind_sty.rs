use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tocbibind.sty.ltxml
  // I'm inclined to think there's nothing to do here!
  for option in ["notbib", "notindex", "nottoc", "notlof", "notlot"].iter() {
    DeclareOption!(*option, None);
  }

  ProcessOptions!();

  // tocbibind.sty L57+101: minimal internals so user code (and
  // classes that include tocbibind) can probe these without errors.
  // We don't actually generate ToC entries in XML output, so the
  // conditionals' values are immaterial. Witness 2408.01486 (SciPost
  // probing `\if@dotoctoc` from its own tocbibind raw-load).
  DefConditional!("\\if@dotoctoc");
  DefMacro!("\\@tocextra", "section");
  DefMacro!("\\tocotherhead{}", "");
});
