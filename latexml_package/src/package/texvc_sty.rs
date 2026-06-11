//! texvc.sty — MediaWiki texvc math command definitions
//! Perl: texvc.sty.ltxml — 183 lines (39 DefMath definitions)
//! Covers the custom math commands used by Wikipedia/MediaWiki's texvc filter.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");
  RequirePackage!("amsfonts");
  RequirePackage!("amssymb");
  // Perl L34, L36, L37: texvc also depends on xcolor (with dvipsnames+usenames),
  // eurosym, cancel. Without these, MediaWiki/Wikipedia documents using texvc
  // hit undefined-CS for `\color{...}` (xcolor), `\euro` (eurosym), and
  // `\cancel{...}` (cancel) — a real warning cascade in arxiv corpus papers
  // that consume MediaWiki-flavored math.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "usenames".to_string()]);
  RequirePackage!("eurosym");
  RequirePackage!("cancel");

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

  // Uppercase Greek (not in standard TeX) — Perl L65-90
  DefMath!("\\Alpha", None, "\u{0391}");
  DefMath!("\\Beta", None, "\u{0392}");
  DefMath!("\\Epsilon", None, "\u{0395}");
  DefMath!("\\Zeta", None, "\u{0396}");
  DefMath!("\\Eta", None, "\u{0397}");
  DefMath!("\\Iota", None, "\u{0399}");
  DefMath!("\\Kappa", None, "\u{039A}");
  DefMath!("\\Mu", None, "\u{039C}");
  DefMath!("\\Nu", None, "\u{039D}");
  DefMath!("\\omicron", None, "\u{03BF}");
  DefMath!("\\Omicron", None, "\u{039F}");
  DefMath!("\\Rho", None, "\u{03A1}");
  DefMath!("\\Tau", None, "\u{03A4}");
  DefMath!("\\Chi", None, "\u{03A7}");

  // Archaic Greek — Perl L93-108
  DefMath!("\\Digamma", None, "\u{03DC}");
  DefMath!("\\digamma", None, "\u{03DD}");
  DefMath!("\\Coppa", None, "\u{03D8}");
  DefMath!("\\coppa", None, "\u{03D9}");
  DefMath!("\\Koppa", None, "\u{03DE}");
  DefMath!("\\koppa", None, "\u{03DF}");
  DefMath!("\\Stigma", None, "\u{03DA}");
  DefMath!("\\stigma", None, "\u{03DB}");
  DefMath!("\\Sampi", None, "\u{03E0}");
  DefMath!("\\sampi", None, "\u{03E1}");

  // Spanish sine — Perl L112
  DefMath!("\\sen", None, "sen", role => "TRIGFUNCTION", meaning => "sine");
  // Perl L99,105: variant archaic letters
  DefMath!("\\varcoppa", None, "\u{03D9}");
  DefMath!("\\varstigma", None, "\u{03DB}");

  // Aliases — Perl L122-181
  DefMacro!("\\dArr", "\\Downarrow");
  DefMacro!("\\uArr", "\\Uparrow");
  DefMacro!("\\rArr", "\\Rightarrow");
  DefMacro!("\\lArr", "\\Leftarrow");
  DefMacro!("\\hAar", "\\Leftrightarrow");
  DefMacro!("\\lrArr", "\\Leftrightarrow");
  DefMacro!("\\lang", "\\langle");
  DefMacro!("\\rang", "\\rangle");
  DefMacro!("\\alef", "\\aleph");
  DefMacro!("\\alefsym", "\\aleph");
  DefMacro!("\\clubs", "\\clubsuit");
  DefMacro!("\\Dagger", "\\ddagger");
  DefMacro!("\\diamonds", "\\diamondsuit");
  DefMacro!("\\Doteq", "\\doteqdot");
  DefMacro!("\\doublecap", "\\Cap");
  DefMacro!("\\empty", "\\emptyset");
  DefMacro!("\\exist", "\\exists");
  DefMacro!("\\hearts", "\\heartsuit");
  DefMacro!("\\image", "\\Im");
  DefMacro!("\\isin", "\\in");
  DefMacro!("\\ne", "\\neq");
  DefMacro!("\\O", "\\emptyset");
  DefMacro!("\\real", "\\Re");
  DefMacro!("\\Reals", "\\mathbb{R}");
  DefMacro!("\\sect", "\\S");
  DefMacro!("\\spades", "\\spadesuit");
  DefMacro!("\\thetasym", "\\vartheta");
  DefMacro!("\\weierp", "\\wp");
  DefMacro!("\\le", "\\leq");
  DefMacro!("\\ge", "\\geq");
  // Perl L41-42: \part → \partial, \and → \land (both are texvc
  // overrides of LaTeX sectioning / frontmatter CSes — in a MediaWiki
  // math context \part means \partial, not the sectioning command).
  Let!("\\part", "\\partial");
  Let!("\\and", "\\land");
  // Perl L47: texvc turns off equation group numbers
  Let!("\\@equationgroup@number", "\\nonumber");
  // Perl L53-61: \unicode{x00C5} for arbitrary unicode char insertion
  DefPrimitive!("\\unicode[][]{}", sub[(_opt1, _opt2, code)] {
    let code_str = code.to_string();
    let code_val = if let Some(rest) = code_str.strip_prefix('x') {
      u32::from_str_radix(rest, 16).unwrap_or(0)
    } else {
      code_str.parse::<u32>().unwrap_or(0)
    };
    if let Some(ch) = char::from_u32(code_val) {
      unread(Tokens!(Token { text: pin_char(ch), code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    }));
    }
  });

  // \bold is MediaWiki-specific math shorthand for \mathbf; not in Perl
  // texvc.sty (which routes via amssymb/mathtools). Kept as a defensive
  // stub so MediaWiki-flavored documents resolve it (MediaWiki users
  // write `\bold{x}` rather than `\mathbf{x}`).
  DefMacro!("\\bold{}", "\\mathbf{#1}");

  // Color — Perl L155-183
  def_macro_noop("\\pagecolor{}")?;
  def_macro_noop("\\definecolor{}{}{}")?;
});
