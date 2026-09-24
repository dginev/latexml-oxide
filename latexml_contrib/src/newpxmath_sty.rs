//! newpxmath.sty (Palatino math fonts). The font itself is presentational;
//! what documents reach is its SYMBOL SET — newpxmath.sty:187 loads the
//! `amssymbols` option by default and :1075-1258 `\re@DeclareMathSymbol`s
//! the AMS set (`\square`, `\blacksquare`, `\blacktriangleright`, `\nmid`
//! … uantwerpenexam-example2), which the amssymb binding already carries.
//! Guard: `perfect_kernel_batch56::font_symbol_packages_carry_amssymb`.
use latexml_package::prelude::*;

LoadDefinitions!({
  // newpxmath.sty:23 `\RequirePackage{amsmath}`: the binding shadows the raw
  // load, so a document that relies on it (`\text`, `\tfrac` via a style
  // that loads only newpxmath — arXiv 2605.27258) flooded undefined.
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // Map newpxmath variant font macros to their standard equivalents.
  Let!("\\varmathbb", "\\mathbb");
  Let!("\\vmathbb", "\\mathbb");
  Let!("\\vvmathbb", "\\mathbb");
  Let!("\\vvarmathbb", "\\mathbb");
});
