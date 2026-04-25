//! harvard.sty — author-year (Harvard) citation style.
//!
//! No Perl binding upstream. ~2 sandbox papers (1901.01008, 1901.08800)
//! hit `\harvarditem` undefined. The package's `.bbl` form is
//! `\harvarditem[short]{long}{year}{key}` followed by the entry body
//! — semantically a `\bibitem` with a short-citation alias.
//!
//! Approximation: route to `\bibitem[#2, #3]{#4}`. The short alias is
//! discarded (Rust ports of natbib citation already pull author+year
//! from bibcite metadata).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\harvarditem [] {}{}{}", "\\bibitem[#2, #3]{#4}");
  // \citeasnoun{key}, \citename{name}, \citeyear{key} — author-year forms.
  DefMacro!("\\citeasnoun{}",   "\\cite{#1}");
  DefMacro!("\\possessivecite{}", "\\cite{#1}");
  DefMacro!("\\citeaffixed{}{}",  "\\cite[#2]{#1}");
  DefMacro!("\\citename{}",     "#1");
  DefMacro!("\\citeyear{}",     "\\cite{#1}");
  DefMacro!("\\citeyearpar{}",  "\\cite{#1}");
  DefMacro!("\\harvardyearleft", "(");
  DefMacro!("\\harvardyearright", ")");
  // (other macros from harvard.sty stubbed only as needed)
});
