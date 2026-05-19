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
  // Perl emulateapj.sty.ltxml L25-39: capture `\fig` before loading aastex
  // (which may redefine it) and reinstate the pre-existing definition
  // afterwards. Comment in Perl: "collides in arXiv:astro-ph/0002091".
  // Use lookup_definition_stored so the handle round-trips through
  // install_definition's `Into<Stored>` bound.
  let saved_fig = state::lookup_definition_stored(&T_CS!("\\fig")).ok().flatten();
  // Perl L28: RequirePackage('aastex', withoptions => 1)
  require_package_with_options("aastex")?;
  RequirePackage!("epsf");
  if let Some(def) = saved_fig {
    state::install_definition(def, Some(Scope::Global));
    AssignValue!("\\fig:locked" => 1i64, Some(Scope::Global));
  }
  def_macro_noop("\\LongTables")?;
  Let!("\\BeginEnvironment", "\\begin");
  Let!("\\EndEnvironment",   "\\end");
  def_macro_noop("\\BeforeBegin{}{}")?;
  def_macro_noop("\\BeforeEnd{}{}")?;
  def_macro_noop("\\AfterBegin{}{}")?;
  def_macro_noop("\\AfterEnd{}{}")?;
  DefMacro!("\\ApjSectionMarkInTitle{}",         "{#1.\\ }");
  DefMacro!("\\ApjSectionpenalty",               "0");
  DefMacro!("\\AppendixApjSectionMarkInTitle{}", "{#1.\\ }");
  def_macro_noop("\\NullCom{}")?;
  DefMacro!("\\apjsecfont",        "\\small");
  DefMacro!("\\lastfootnote",      "\\small");
  DefMacro!("\\lastpagefootnote",  "\\small");
  DefMacro!("\\lastpagefootnotes", "\\small");
  def_macro_noop("\\tableheadfrac{}")?;
  def_macro_noop("\\tabletypesize{}")?;
  Let!("\\tablefontsize", "\\tabletypesize");
  def_macro_noop("\\subtitle")?;
  DefMacro!("\\submitted{}",   "\\@add@frontmatter{ltx:date}[role=submitted]{#1}");
  DefMacro!("\\journalinfo{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\keywordsname", "Subject headings");
  DefMacro!("\\ulap{}", "\\hbox{#1}");
  DefMacro!("\\dlap{}", "\\hbox{#1}");
  Let!("\\tabcaption", "\\caption");
  DefMacro!("\\format@title@section{}",    "\\lx@tag[][.\\space]{\\thesection}#1");
  DefMacro!("\\format@title@subsection{}", "\\lx@tag[][.\\space]{\\thesubsection}#1");
  DefMacro!("\\format@title@figure{}",     "\\lx@tag[][.\\lx@emdash\\space]{\\lx@fnum@@{figure}}#1");
  DefMacro!("\\format@title@table{}",      "\\lx@tag{\\lx@fnum@@{table}}#1");
});
