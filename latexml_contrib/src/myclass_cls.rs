use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Perl: myclass.cls.ltxml
  def_macro_noop("\\my@class@stuff")?;
  DeclareOption!(
    "acommonoption",
    "\\xdef\\my@class@stuff{\\my@class@stuff, acommonoption}"
  );
  DeclareOption!(
    "aclassoption",
    "\\xdef\\my@class@stuff{\\my@class@stuff, aclassoption}"
  );
  // Perl: DeclareOption(undef, sub { PassOptions('article','cls',...) })
  DeclareOption!(None, {
    let opt = stomach::digest(T_CS!("\\CurrentOption"))?.to_string();
    state::push_value(&s!("opt@article.cls"), arena::pin(&opt))?;
  });
  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  DefMacro!(
    "\\showclassstuff",
    "\\par\\noindent Class options: \\my@class@stuff"
  );
});
