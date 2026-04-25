//! epic.sty / eepic.sty — picture-mode drawing extensions.
//!
//! No Perl binding upstream. The packages add `\drawline`, `\dottedline`,
//! `\dashline`, `\path`, `\filltype`, etc. for `picture` environments.
//! Our XML output doesn't carry low-level pictures, so all drawing CSes
//! emit nothing — the surrounding text already explains the figure.
//!
//! ~8 sandbox papers (1604.01366 … 1606.06102) hit `\drawline` undefined
//! when their preamble loads epic/eepic. Both ext map onto this module.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // epic.sty — line and curve drawing in picture environments.
  // Most CSes take "stroke parameter" or coordinate args and draw
  // in the surrounding picture; emit nothing.
  DefMacro!("\\drawline OptionalSemiverbatim Semiverbatim", "");
  DefMacro!("\\dottedline OptionalSemiverbatim Semiverbatim Semiverbatim", "");
  DefMacro!("\\dashline OptionalSemiverbatim Semiverbatim Semiverbatim Semiverbatim", "");
  DefMacro!("\\drawlinestretch{}", "");
  DefMacro!("\\dashlinestretch{}", "");
  DefMacro!("\\dashlinegap{}", "");
  DefMacro!("\\dashlinedash{}", "");
  DefMacro!("\\dottedlinegap{}", "");
  DefMacro!("\\jput Semiverbatim {}", "");
  DefMacro!("\\matrixput Semiverbatim Semiverbatim Number Semiverbatim Number {}", "");
  DefMacro!("\\multiputlist Semiverbatim Semiverbatim {}", "");
  DefMacro!("\\path Semiverbatim", "");
  DefMacro!("\\spline Semiverbatim", "");
  DefMacro!("\\filltype{}", "");
  DefMacro!("\\arc Semiverbatim {}", "");
  DefMacro!("\\bigcircle{}", "");
  DefMacro!("\\Thicklines", "");
  DefMacro!("\\thicklines", "");
  DefMacro!("\\thinlines", "");

  // eepic.sty layers on top — additional patterns.
  DefMacro!("\\allinethickness{}", "");
  DefMacro!("\\Numbersymbol{}", "");
  DefMacro!("\\drawvector OptionalSemiverbatim Semiverbatim", "");
  DefMacro!("\\dottedlinevector OptionalSemiverbatim Semiverbatim", "");
  DefMacro!("\\dashlinevector OptionalSemiverbatim Semiverbatim", "");
});
