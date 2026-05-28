use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tocbibind.sty.ltxml
  // I'm inclined to think there's nothing to do here!
  for option in ["notbib", "notindex", "nottoc", "notlof", "notlot"].iter() {
    DeclareOption!(*option, None);
  }

  ProcessOptions!();

  // tocbibind.sty L55-59: minimal internals so user code (and
  // classes that include tocbibind) can probe these without errors.
  // We don't actually generate ToC entries in XML output, so the
  // conditionals' values are immaterial. Witnesses: 2408.01486 (SciPost
  // probing `\if@dotoctoc`); 2003.02382 (CONVERR_2 on
  // `\if@dotoclof`/`\if@dotoclot`, paper uses
  // `\usepackage[nottoc]{tocbibind}` and downstream `\@dotoclof*`
  // conditional probes).
  DefConditional!("\\if@dotocbib");
  DefConditional!("\\if@dotocind");
  DefConditional!("\\if@dotoctoc");
  DefConditional!("\\if@dotoclot");
  DefConditional!("\\if@dotoclof");
  DefMacro!("\\@tocextra", "section");
  def_macro_noop("\\tocotherhead{}")?;
});
