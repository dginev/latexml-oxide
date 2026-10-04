use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsbook.cls.ltxml
  // Ignorable options (Perl L22-30); `oneside`/`twoside` go on to book's (KNOWN_PERL_ERRORS #469:
  // two-sided by default, amsbook.cls:103-104/:329-330).
  for option in ["a4paper", "letterpaper", "landscape", "portrait",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "draft", "final", "e-only",
    "titlepage", "notitlepage", "onecolumn", "twocolumn",
    "centertags", "tbtags",
    "openright", "openany",
    "makeidx", "nomath", "noamsfonts", "psamsfonts"].iter()
  {
    DeclareOption!(*option, None);
  }
  // Perl L31-34: default ltx_leqno => 1 (left equation numbers), then
  // `leqno` re-asserts, `reqno` clears, `fleqn` sets ltx_fleqn=1. Rust
  // previously declared leqno/reqno/fleqn as no-ops, so amsbook docs
  // with [reqno] still rendered left-numbered equations.
  // amsbook.cls:130-133: `leqno` and `reqno` also go to amsmath, whose own
  // switch sets the tags (DIVERGENCES #336). `leqno` is the default
  // (amsbook.cls:329).
  DeclareOption!("leqno", {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true);
    Digest!("\\PassOptionsToPackage{leqno}{amsmath}")?;
  });
  DeclareOption!("reqno", {
    assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", None::<bool>);
    Digest!("\\PassOptionsToPackage{reqno}{amsmath}")?;
  });
  DeclareOption!("fleqn", { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });
  execute_options(&["leqno"])?;
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{book}")?;
  });
  ProcessOptions!();
  LoadClass!("book");
  RequirePackage!("ams_support");
  ams_support_sty::amsart_author_storage()?;
  ams_support_sty::amsart_uppercase_nonmath()?;

  // Frontmatter/mainmatter/backmatter — Perl L46-56 (no-ops); amsbook.cls:944-948 switches the
  // page numbering, which `\thepage` shows (roman, then arabic).
  DefMacro!("\\frontmatter", "\\pagenumbering{roman}");
  DefMacro!("\\mainmatter", "\\pagenumbering{arabic}");
  def_primitive_noop("\\backmatter")?;

  // List formatting — Perl L58-72
  DefMacro!("\\@listI", "\\leftmargin\\leftmargini\\parsep 4.5\\p@ plus2\\p@ minus\\p@\\topsep 8.5\\p@ plus3\\p@ minus4\\p@\\itemsep4.5\\p@ plus2\\p@ minus\\p@");
  Let!("\\@listi", "\\@listI");
  DefMacro!("\\@listii", "\\leftmargin\\leftmarginii\\labelwidth\\leftmarginii\\advance\\labelwidth-\\labelsep\\topsep 4\\p@ plus2\\p@ minus\\p@\\parsep 2\\p@ plus\\p@ minus\\p@\\itemsep\\parsep");
  DefMacro!("\\@listiii", "\\leftmargin\\leftmarginiii\\labelwidth\\leftmarginiii\\advance\\labelwidth-\\labelsep\\topsep 2\\p@ plus\\p@ minus\\p@\\parsep\\z@\\partopsep\\p@ plus\\z@ minus\\p@\\itemsep\\topsep");
  DefMacro!("\\@listiv", "\\leftmargin\\leftmarginiv\\labelwidth\\leftmarginiv\\advance\\labelwidth-\\labelsep");
  DefMacro!("\\@listv", "\\leftmargin\\leftmarginv\\labelwidth\\leftmarginv\\advance\\labelwidth-\\labelsep");
  DefMacro!("\\@listvi", "\\leftmargin\\leftmarginvi\\labelwidth\\leftmarginvi\\advance\\labelwidth-\\labelsep");

  // Perl L64-66: description end alias and \upn = \textup
  Let!("\\enddescription", "\\endlist");
  Let!("\\upn", "\\textup");

  // amsbook.cls:303-313: the running-head mark builder the class's
  // `\ps@headings` gives `\chaptermark`/`\sectionmark`/`\partmark`
  // (`\@secmark\markboth\chapterrunhead{}`), and the section number it reads.
  // Marks have no XML form (`\@mkboth` is `\@gobbletwo`), but a class may call
  // its marks: my-thesis.cls:153-160 `\pagestyle{headings}\chaptermark{}` (TeX
  // Live class census 2026-09-24).
  RawTeX!(r"\long\def\@nilgobble#1\@nil{}
\let\@secnumber\@empty
\def\@secmark#1#2#3#4{%
  \begingroup \let\protect\@unexpandable@protect
  \edef\@tempa{\endgroup \toks@{\protect#2{#3}{\@secnumber}}}%
  \@tempa
  \toks@\@xp{\the\toks@{#4}}%
  \afterassignment\@nilgobble\@temptokena\@themark{}\@nil
  \edef\@tempa{\@nx\@mkboth{%
    \ifx\markright#1\the\@temptokena\else\the\toks@\fi}{\the\toks@}}%
  \@tempa}");
});
