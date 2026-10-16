//! cimento.cls (Il Nuovo Cimento, Società Italiana di Fisica): the papers ship the class (it is not in TeX Live) —
//! or a renamed copy (`cimento_modif.cls`, 2203.13736), which the prefix fallback answers with this binding — and it is
//! read raw, with what the kernel's frontmatter needs of it. A paper whose source lacks the class falls to OmniBus.
use latexml_package::prelude::*;

LoadDefinitions!({
  let Some(class) = crate::shipped_class::requested_shipped_class("cimento")? else {
    LoadClass!("OmniBus");
    return Ok(());
  };
  InputDefinitions!(&class, noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // cimento.cls:746 `\def\ETC{\unskip\unscrunch|}`, inside its `\maketitle`: in an author list (or a title) `\ETC` marks
  // where the running head cuts the list short with "etc." (:766 `\def\ETC{\unskip\ etc.\scrunch|}`), and on the title
  // page it sets the whole list (`\unscrunch|#1|{#1}` over the rest of `\@author@`, which `\author` ends with `|`,
  // :680). The kernel's `\author` and `\title` keep the whole text, so there it is the `\unskip`. A variant's own
  // `\ETC` (:1361, :1679) stands. Witnesses astro-ph0502331 (`\author{K.-I.~Nishikawa\from{ins:x}\ETC, …}`),
  // astro-ph0506233, 1804.04677 (`\title{… physics \ETC\ from …}`).
  RawTeX!(r"\providecommand{\ETC}{\unskip}");
});
