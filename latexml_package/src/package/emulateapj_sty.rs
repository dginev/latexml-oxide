use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("aastex");
  RequirePackage!("epsf");
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
