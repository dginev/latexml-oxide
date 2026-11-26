//! edpsmath.cls (EDP Sciences' class for the ESAIM and MMNP journals, v2 2002/12/01, over amsart; shipped with the
//! paper, not in TeX Live): the class is loaded raw, and its keywords go to the frontmatter API.
//!
//! The class's `\keywords` sets them in a box (edpsmath.cls:212-215 `\global\setbox\@b@kwrds\vtop{…\textit{\keysname.~}
//! #1…}`) that only its `\@maketitle` prints, in a footnote (`\@setkeywords`): they were a footnote of the body
//! (1901.04804, `[mmnp]`), or lost where the journal option prints none (1110.0287, `[proc]`: edpsproc.clo:8
//! `\def\@setkeywords{}`), as the author's text the frontmatter keeps (FIDELITY_CHARTER §2). Here they are the
//! frontmatter's keywords, named by the class's label, and the title page prints no footnote of them. Perl has no
//! binding: it reports the class missing and loads its dependency amsart, whose `\keywords` keeps them.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("edpsmath", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  DefMacro!("\\keywords{}", "\\lx@add@keywords[name={\\keysname.~}]{#1}");
  DefMacro!("\\@setkeywords", "");
});
