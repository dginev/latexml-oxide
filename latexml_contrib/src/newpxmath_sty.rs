//! newpxmath.sty (Palatino math fonts). The font itself is presentational;
//! what documents reach is its SYMBOL SET — newpxmath.sty:187 loads the
//! `amssymbols` option by default and :1075-1258 `\re@DeclareMathSymbol`s
//! the AMS set (`\square`, `\blacksquare`, `\blacktriangleright`, `\nmid`
//! … uantwerpenexam-example2), which the amssymb binding already carries.
//! Guard: `perfect_kernel_batch56::font_symbol_packages_carry_amssymb`.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("amssymb");
  // Map newpxmath variant font macros to their standard equivalents.
  Let!("\\varmathbb", "\\mathbb");
  Let!("\\vmathbb", "\\mathbb");
  Let!("\\vvmathbb", "\\mathbb");
  Let!("\\vvarmathbb", "\\mathbb");
});
