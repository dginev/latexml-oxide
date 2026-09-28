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
  // newpxmath.sty:2118-2119: `\varmathbb` is a blackboard alphabet of its own (`\vmathbb` the
  // same), not a copy of `\mathbb`, which a document may have `\renewcommand`ed to `\varmathbb`
  // before the load (the two then expanded into each other; KPE #355).
  DefConstructor!("\\varmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
  Let!("\\vmathbb", "\\varmathbb");
  Let!("\\vvmathbb", "\\varmathbb");
  Let!("\\vvarmathbb", "\\varmathbb");
});
