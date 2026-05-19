use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  Warn!(
    "missing_file",
    "tabu.sty",
    "tabu.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("array");
  RequirePackage!("varwidth");
  RequirePackage!("longtable");
  DefMacro!("\\tabu", "\\tabular");
  DefMacro!("\\endtabu", "\\endtabular");
  DefMacro!("\\longtabu", "\\longtable");
  DefMacro!("\\endlongtabu", "\\endlongtable");
  // stubs
  def_macro_noop("\\savetabu{}")?;
  def_macro_noop("\\usetabu{}")?;
  def_macro_noop("\\preamble{}")?;
  def_macro_noop("\\tabulinestyle{}")?;
  def_macro_noop("\\newtabulinestyle{}")?;
  DefMacro!("\\tabucline[]{}", "\\hline");
  def_macro_noop("\\taburulecolor OptionalMatch:| OptionalUntil:| {}")?;
  def_macro_noop("\\taburowcolors[] Number {}")?;
  def_macro_noop("\\tabuphantomline")?;
  DefRegister!("\\tracingtabu" => Number::new(0));
  DefRegister!("\\tabulinesep" => Dimension::new(0));
  DefRegister!("\\abovetabulinesep" => Dimension::new(0));
  DefRegister!("\\belowtabulinesep" => Dimension::new(0));
});
