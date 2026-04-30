use crate::prelude::*;

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
  DefMacro!("\\LongTables", "");
  Let!("\\BeginEnvironment", "\\begin");
  Let!("\\EndEnvironment",   "\\end");
  DefMacro!("\\BeforeBegin{}{}", "");
  DefMacro!("\\BeforeEnd{}{}",   "");
  DefMacro!("\\AfterBegin{}{}",  "");
  DefMacro!("\\AfterEnd{}{}",    "");
  DefMacro!("\\ApjSectionMarkInTitle{}",         "{#1.\\ }");
  DefMacro!("\\ApjSectionpenalty",               "0");
  DefMacro!("\\AppendixApjSectionMarkInTitle{}", "{#1.\\ }");
  DefMacro!("\\NullCom{}", "");
  DefMacro!("\\apjsecfont",        "\\small");
  DefMacro!("\\lastfootnote",      "\\small");
  DefMacro!("\\lastpagefootnote",  "\\small");
  DefMacro!("\\lastpagefootnotes", "\\small");
  DefMacro!("\\tableheadfrac{}", "");
  DefMacro!("\\tabletypesize{}", "");
  Let!("\\tablefontsize", "\\tabletypesize");
  DefMacro!("\\subtitle", "");
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
