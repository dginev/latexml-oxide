//! tgpagella.sty — TeX Gyre Pagella (a URW Palladio / Palatino clone) as the
//! roman text font, plus its NFSS encoding declarations.
//!
//! Perl LaTeXML ships no `tgpagella.sty.ltxml`; at the default `notex` setting
//! it does not raw-load the real file either, so `\usepackage{tgpagella}` is
//! parity. This contrib binding raw-loads the genuine TL `tgpagella.sty`: it is
//! a classic NFSS font package (`\renewcommand{\rmdefault}{…}` + encoding
//! setup, no fontspec on the pdflatex path), so it loads cleanly. Font choice
//! carries no semantics in the LaTeXML XML tree, but raw-loading keeps the
//! encoding/`\DeclareTextSymbol` declarations available for any text-symbol
//! lookups and silences the missing-file warning.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("tgpagella", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
