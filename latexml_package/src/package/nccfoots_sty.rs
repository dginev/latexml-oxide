//! nccfoots.sty (ncctools, loaded by manyfoot), which Perl LaTeXML does not bind: `\Footnotetext{<mark>}{<text>}` sets
//! `\@thefnmark` to the mark given (none keeps the last) and sets the text marked with it (nccfoots.sty:28-38), here the
//! kernel's `\footnotetext[<mark>]`; `\Footnote` is `\Footnotemark` and that text (:36), whose `\@footnotetext` the
//! kernel marks with its own counter.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("nccfoots", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r"\renewcommand*{\Footnotetext}[1]{\NCC@makefnmark{#1}\expandafter\lx@ncc@text\expandafter{\@thefnmark}}
\def\lx@ncc@text#1{\footnotetext[#1]}
\renewcommand*{\Footnote}[1]{\Footnotemark{#1}\Footnotetext{}}");
});
