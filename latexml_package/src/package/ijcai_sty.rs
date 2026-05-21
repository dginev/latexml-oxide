use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("natbib");
  Let!("\\AND",      "\\and");
  Let!("\\And",      "\\and");
  Let!("\\leftcite", "\\cite");
  DefMacro!("\\pubnote{}", "\\@add@frontmatter{ltx:note}[role=pubnote]{#1}");
  def_macro_noop("\\affiliations")?;
  def_macro_noop("\\emails")?;
});
