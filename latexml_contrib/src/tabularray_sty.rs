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
    "tabularray.sty",
    "tabularray.sty is not implemented and will not be interpreted raw."
  );
  RequirePackage!("booktabs");
  DefMacro!("\\tblr", "\\tabular");
  DefMacro!("\\endtblr", "\\endtabular");
  DefMacro!("\\booktabs", "\\tabular");
  DefMacro!("\\endbooktabs", "\\endtabular");
  DefMacro!("\\UseTblrLibrary", "\\usepackage");
  def_macro_noop("\\SetCell[]{}")?;
  def_macro_noop("\\SetCells[]{}")?;
  // tabularray styling primitives — no-op stubs.
  // Witness 2406.00523 (\SetTblrInner).
  def_macro_noop("\\SetTblrInner[]{}")?;
  def_macro_noop("\\SetTblrOuter[]{}")?;
  def_macro_noop("\\SetTblrStyle{}{}")?;
  def_macro_noop("\\NewTblrEnviron{}")?;
  def_macro_noop("\\NewColumnType{}[]{}")?;
  def_macro_noop("\\NewTblrTheme{}{}")?;
  def_macro_noop("\\DefTblrTemplate{}{}{}")?;
  def_macro_noop("\\SetTblrTemplate{}{}")?;
});
