//! AlegreyaSans.sty — the Alegreya Sans family as the sans-serif font (NFSS).
//!
//! Perl LaTeXML ships no `AlegreyaSans.sty.ltxml`; at the default `notex`
//! setting it does not raw-load the real file either, so `\usepackage{…}` is
//! parity. This contrib binding raw-loads the genuine TL `AlegreyaSans.sty`
//! where the font is installed (a classic NFSS font package). Font choice has
//! no effect on the LaTeXML XML tree; the binding exists so the package is
//! recognised and — on a host that ships the font — its `\…family` switches
//! resolve. On a host without the font installed the raw-load is a no-op
//! (kpathsea miss).
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("AlegreyaSans", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
