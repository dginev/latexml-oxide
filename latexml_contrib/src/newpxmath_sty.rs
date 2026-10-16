//! newpxmath.sty (Palatino math fonts). The font itself is presentational;
//! what documents reach is its SYMBOL SET — newpxmath.sty:187 loads the
//! `amssymbols` option by default and :1075-1258 `\re@DeclareMathSymbol`s
//! the AMS set (`\square`, `\blacksquare`, `\blacktriangleright`, `\nmid`
//! … uantwerpenexam-example2), which the amssymb binding already carries.
//! Guard: `perfect_kernel_batch56::font_symbol_packages_carry_amssymb`.
//!
//! newpxmath.sty is newtxmath.sty's Palatino twin: the same symbol names over the same symbol-font layout (its
//! `symbolsC` is pxfonts' `pxsyc`, txfonts' `txsyc`), so the binding carries newtxmath's: txfonts' symbols (`\nequiv`
//! :1374, `\fint` :967, `\mapsfrom` :1313, `\circleddot` :644, `\llbracket` :527, `\multimapinv` :1345) without loading
//! txfonts (newpxmath.sty:23-33 never does; `\@ifpackageloaded{txfonts}` stays false) and without the txfonts names it
//! does not declare (`\varv`, `\varw`, `\vary`, the clockwise double and triple contour integrals), then newtxmath's
//! own layer (`\coloneq` and `\eqcolon` :1541-1542 as ≔ and ≕, `\varg` :1858 as an italic g, `\uppartial` :1860,
//! `\transp` :570, `\Angstrom` :551, `\cdotB` :1646, `\vv` :833, `\laplac` :745, the upright Greek `\Gammaup`…
//! :1662-1701 and their `\upGamma`… names :1703-1743).
//! Witnesses 1709.08558, 1902.06558, 2001.06387, 2003.04827, 2003.05278, 2305.12589, 2508.05134, 2603.09013.
//! Guards: `perfect_kernel_batch68j::newpxmath_carries_its_symbol_set`, `::newpxmath_coloneq_over_mathtools`.
use latexml_package::{
  package::{newtxmath_sty::def_newtx_math_symbols, txfonts_sty::def_txfonts_symbols},
  prelude::*,
};

LoadDefinitions!({
  // newpxmath.sty:23-33 `\RequirePackage{amsmath}`, `{etoolbox}`, `{xkeyval}`, `{centernot}`: the binding shadows
  // the raw load, so a document that relies on them (`\text`, `\tfrac` via a style that loads only newpxmath —
  // arXiv 2605.27258) flooded undefined.
  RequirePackage!("amsmath");
  RequirePackage!("etoolbox");
  RequirePackage!("xkeyval");
  RequirePackage!("centernot");
  RequirePackage!("amssymb");
  // The symbol set, as the newtxmath binding defines it (txfonts' symbols, then newtxmath's own layer).
  def_txfonts_symbols()?;
  def_newtx_math_symbols()?;
  // newpxmath.sty:2118-2119: `\varmathbb` is a blackboard alphabet of its own (`\vmathbb` the
  // same), not a copy of `\mathbb`, which a document may have `\renewcommand`ed to `\varmathbb`
  // before the load (the two then expanded into each other; KPE #355).
  DefConstructor!("\\varmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
  Let!("\\vmathbb", "\\varmathbb");
  Let!("\\vvmathbb", "\\varmathbb");
  Let!("\\vvarmathbb", "\\varmathbb");
});
