use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\address[]{}", "\\lx@add@address{#2}");
  def_macro_noop("\\addressmark")?;
  DefMacro!("\\addresstext{}{}", "#2");
  DefMacro!("\\filedate",    "24 November 1993");
  DefMacro!("\\fileversion", "v2.6");
  NewCounter!("address");
  DefMacro!("\\theaddress", "\\alph{address}");
});
