use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


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
