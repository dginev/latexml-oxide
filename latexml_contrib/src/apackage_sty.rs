use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Perl: apackage.sty.ltxml
  def_macro_noop("\\my@package@stuff")?;
  DeclareOption!(
    "acommonoption",
    "\\xdef\\my@package@stuff{\\my@package@stuff, acommonoption}"
  );
  DeclareOption!(
    "apackageoption",
    "\\xdef\\my@package@stuff{\\my@package@stuff, apackageoption}"
  );
  DeclareOption!(
    "anotherpackageoption",
    "\\xdef\\my@package@stuff{\\my@package@stuff, anotherpackageoption}"
  );
  ProcessOptions!();
  DefMacro!(
    "\\showpackagestuff",
    "\\par\\noindent Package options: \\my@package@stuff"
  );
});
