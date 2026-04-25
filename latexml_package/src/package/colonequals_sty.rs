//! colonequals.sty (Donald Arseneau) — colon-based assignment glyphs.
//!
//! No Perl binding upstream. ~5 sandbox papers (1108.3241 … 1204.2526)
//! fail on `\colonequals` undefined when the raw .sty isn't on the
//! texmf path. The CSes are math-mode glyphs combining colon with =,
//! ==, -, etc.; emit the canonical Unicode equivalents.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Single colon variants.
  DefMath!("\\colonequals",   "\u{2254}", role => "RELOP"); // ≔
  DefMath!("\\equalscolon",   "\u{2255}", role => "RELOP"); // ≕
  // Double colon variants.
  DefMath!("\\coloncolonequals", "\u{2A74}", role => "RELOP"); // ⩴ ::=
  // Compound forms (mathtools-style — no native Unicode, fall back to text).
  DefMath!("\\colonminus",      ":-",  role => "RELOP");
  DefMath!("\\minuscolon",      "-:",  role => "RELOP");
  DefMath!("\\colonapprox",     ":\u{2248}", role => "RELOP"); // :≈
  DefMath!("\\approxcolon",     "\u{2248}:", role => "RELOP");
  DefMath!("\\colonsim",        ":\u{223C}", role => "RELOP"); // :∼
  DefMath!("\\simcolon",        "\u{223C}:", role => "RELOP");
  DefMath!("\\coloncolon",      "::",  role => "RELOP");
  DefMath!("\\coloncolonminus", "::-", role => "RELOP");
  DefMath!("\\minuscoloncolon", "-::", role => "RELOP");
});
