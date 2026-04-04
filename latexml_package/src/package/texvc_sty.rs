//! texvc.sty — MediaWiki texvc math command definitions
//! Perl: texvc.sty.ltxml — 183 lines (39 DefMath definitions)
//! Covers the custom math commands used by Wikipedia/MediaWiki's texvc filter.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");
  RequirePackage!("amsfonts");
  RequirePackage!("amssymb");

  // Math operators (Perl L39-55)
  DefMath!("\\sgn", None, "sgn", role => "OPFUNCTION", meaning => "sign");
  DefMath!("\\arccot", None, "arccot", role => "OPFUNCTION", meaning => "arccot");
  DefMath!("\\arcsec", None, "arcsec", role => "OPFUNCTION", meaning => "arcsec");
  DefMath!("\\arccsc", None, "arccsc", role => "OPFUNCTION", meaning => "arccsc");

  // Number sets (Perl L57-69)
  DefMath!("\\N", None, "\u{2115}", role => "ID", meaning => "natural-numbers");
  DefMath!("\\R", None, "\u{211D}", role => "ID", meaning => "real-numbers");
  DefMath!("\\Z", None, "\u{2124}", role => "ID", meaning => "integers");
  DefMath!("\\Q", None, "\u{211A}", role => "ID", meaning => "rationals");
  DefMath!("\\C", None, "\u{2102}", role => "ID", meaning => "complex-numbers");
  DefMath!("\\H", None, "\u{210D}", role => "ID", meaning => "quaternions");

  // Additional symbols (Perl L71-100)
  DefMath!("\\natnums", None, "\u{2115}", role => "ID", meaning => "natural-numbers");
  DefMath!("\\reals", None, "\u{211D}", role => "ID", meaning => "real-numbers");
  DefMath!("\\integers", None, "\u{2124}", role => "ID", meaning => "integers");
  DefMath!("\\rationals", None, "\u{211A}", role => "ID", meaning => "rationals");
  DefMath!("\\cnums", None, "\u{2102}", role => "ID", meaning => "complex-numbers");
  DefMath!("\\Complex", None, "\u{2102}", role => "ID", meaning => "complex-numbers");

  DefMath!("\\bull", None, "\u{2022}");
  DefMath!("\\plusmn", None, "\u{00B1}", role => "ADDOP", meaning => "plus-or-minus");
  DefMath!("\\sdot", None, "\u{22C5}", role => "MULOP", meaning => "times");
  DefMath!("\\sub", None, "\u{2282}", role => "RELOP", meaning => "subset");
  DefMath!("\\supe", None, "\u{2287}", role => "RELOP", meaning => "superset-of-or-equal-to");
  DefMath!("\\sube", None, "\u{2286}", role => "RELOP", meaning => "subset-of-or-equal-to");
  DefMath!("\\infin", None, "\u{221E}", role => "ID", meaning => "infinity");
  DefMath!("\\ang", None, "\u{2220}", role => "ID", meaning => "angle");
  DefMath!("\\darr", None, "\u{2193}", role => "ARROW", meaning => "downward-arrow");
  DefMath!("\\uarr", None, "\u{2191}", role => "ARROW", meaning => "upward-arrow");
  DefMath!("\\rarr", None, "\u{2192}", role => "ARROW", meaning => "rightward-arrow");
  DefMath!("\\larr", None, "\u{2190}", role => "ARROW", meaning => "leftward-arrow");
  DefMath!("\\lrarr", None, "\u{2194}", role => "ARROW", meaning => "left-right-arrow");
  DefMath!("\\harr", None, "\u{2194}", role => "ARROW", meaning => "left-right-arrow");
  DefMath!("\\Darr", None, "\u{21D3}", role => "ARROW", meaning => "downward-double-arrow");
  DefMath!("\\Uarr", None, "\u{21D1}", role => "ARROW", meaning => "upward-double-arrow");
  DefMath!("\\Rarr", None, "\u{21D2}", role => "ARROW", meaning => "rightward-double-arrow");
  DefMath!("\\Larr", None, "\u{21D0}", role => "ARROW", meaning => "leftward-double-arrow");
  DefMath!("\\Lrarr", None, "\u{21D4}", role => "ARROW", meaning => "left-right-double-arrow");
  DefMath!("\\Harr", None, "\u{21D4}", role => "ARROW", meaning => "left-right-double-arrow");

  // Text-mode stubs (Perl L112-120)
  DefMacro!("\\part{}", "\\par\\textbf{#1}\\par");
  DefMacro!("\\bold{}", "\\mathbf{#1}");

  // Color — Perl L155-183
  DefMacro!("\\pagecolor{}", "");
  DefMacro!("\\definecolor{}{}{}", "");
});
