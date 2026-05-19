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
  // Perl: standalone.cls.ltxml
  InputDefinitions!("standalone", noltxml => true,
    extension => Some(Cow::Borrowed("cls")));
  // standalone.cls's \standaloneenv{tikzpicture} wraps environments with
  // \preview + \sa@varwidth for cropping. This creates an expansion loop
  // in LaTeXML because the \expandafter\def\expandafter\tikzpicture pattern
  // produces massive token overhead when the original environment is a
  // DefEnvironment Constructor. Since LaTeXML doesn't need preview/crop
  // functionality, neutralize the wrapper.
  def_macro_noop("\\@standaloneenv{}")?;
});
