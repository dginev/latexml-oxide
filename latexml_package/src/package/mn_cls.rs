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
  // Perl: mn.cls.ltxml

  // Generally ignorable options
  for option in [
    "letters", "landscape", "galley", "referee",
  ].iter() {
    DeclareOption!(*option, None);
  }

  DeclareOption!("usenatbib", {
    AssignValue!("@usenatbib" => 1i64);
  });
  DeclareOption!("usedcolum", {
    AssignValue!("@usedcolum" => 1i64);
  });
  DeclareOption!("usegraphicx", {
    AssignValue!("@usegraphicx" => 1i64);
  });

  // Anything else is for article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("mn2e_support");

  // And some stuff not in the later version...
  def_macro_noop("\\NewSymbolFont{}{}")?;
  def_macro_noop("\\NewMathSymbol{}{}{}{}")?;
  def_macro_noop("\\NewMathDelimiter{}{}{}{}{}{}")?;
  def_macro_noop("\\NewMathAlphabet{}{}{}")?;
  def_macro_noop("\\NewTextAlphabet{}{}{}")?;
  def_macro_noop("\\UseAMStwoboldmath")?;
  RawTeX!("\\newif\\ifnfssone\\newif\\ifnfsstwo\\newif\\ifoldfss");
  DefRegister!("\\realparindent" => Dimension!("18pt"));
  def_macro_noop("\\resetsizehook{}{}{}{}")?;
});
