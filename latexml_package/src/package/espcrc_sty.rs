use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefConstructor!("\\@@@address{}",
    "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");
  def_macro_noop("\\addressmark")?;
  DefMacro!("\\addresstext{}{}", "#2");
  DefMacro!("\\filedate",    "24 November 1993");
  DefMacro!("\\fileversion", "v2.6");
  NewCounter!("address");
  DefMacro!("\\theaddress", "\\alph{address}");
});
