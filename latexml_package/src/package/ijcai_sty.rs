use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("natbib");
  Let!("\\AND",      "\\and");
  Let!("\\And",      "\\and");
  Let!("\\leftcite", "\\cite");
  DefMacro!("\\pubnote{}", "\\@add@frontmatter{ltx:note}[role=pubnote]{#1}");
  DefMacro!("\\affiliations", "");
  DefMacro!("\\emails",       "");
});
