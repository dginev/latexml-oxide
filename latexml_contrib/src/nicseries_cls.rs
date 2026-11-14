//! nic-series.cls — the "NIC Series" (John von Neumann Institute for Computing)
//! proceedings class (author-bundled; not raw-loaded). OmniBus renders the
//! `\author … \inst{…} \and …` / `\institute{…}` frontmatter fine; the only
//! stray is `\authortoc`, a table-of-contents-only short author list that leaks
//! as literal text (witness 2410.14397). Gobble it (it duplicates `\author`).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  def_macro_noop("\\authortoc{}")?;
  // nic-series.cls:74-78 `\newcommand\chapter`: the class has chapters (OXIDIZED_DESIGN #179 (a)), which OmniBus as
  // its base does not give it; LaTeXML's chapter (witness 0902.2652). The class defines no `\@chapter` and no chapter
  // counter: in TeX only `\chapter*` works, a numbered `\chapter` stops at the undefined `\@chapter` (0902.2652 calls
  // neither).
  DefMacro!("\\chapter", "\\@startsection{chapter}{0}{}{}{}{}");
});
