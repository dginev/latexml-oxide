use crate::prelude::*;

// substr.sty — substring extraction utility (Harald Harders, 2009).
// 117-line plain-TeX `\def`/`\loop`; no expl3, no xparse.
//
// Perl LaTeXML has no `substr.sty.ltxml`; raw-loads the TL .sty
// directly. Already auto-loads transitively from
// `datatool-base.sty`'s `\RequirePackage{substr}` via the
// `INTERPRETING_DEFINITIONS` flag — this shim closes the gap when
// a paper does `\usepackage{substr}` standalone.

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("substr", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
