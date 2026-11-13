//! newtxmath.sty — TX math fonts (delegates to other packages)
//! Perl: newtxmath.sty.ltxml
use super::txfonts_sty::{
  def_math_upright_greek, def_txfonts_symbols, def_txfonts_symbols_newpx_lacks,
};

/// A Greek letter in italic whatever the alphabet around it, as `def_math_upright_greek` sets an upright one.
fn def_math_italic_greek(cs: &str, present: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let opts = MathPrimitiveOptions {
    font: Some(FontDirective::from(
      fontmap!(shape => "italic", forceshape => true),
    )),
    ..MathPrimitiveOptions::default()
  };
  def_math(cs_tok, params, present.to_string(), opts)?;
  Ok(())
}
use crate::prelude::*;

LoadDefinitions!({
  // Perl newtxmath.sty.ltxml L22-31: explicitly require ifxetex+ifluatex
  // (both set identical ifcond false stubs in Rust). Rust previously
  // consolidated to `iftex`, which is a superset — same conditionals
  // available — but drifts from Perl parity. Match the Perl chain exactly.
  RequirePackage!("amsmath");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("ifxetex");
  RequirePackage!("ifluatex");
  RequirePackage!("xkeyval");
  // newtxmath.sty:36 `\RequirePackage{centernot}`, which Perl's binding skips ("just a minor typesetting adjustment",
  // newtxmath.sty.ltxml:26-27, KNOWN_PERL_ERRORS #694): papers under a class that loads newtxmath (acmart) write
  // `\centernot` without loading it (2002.09451, 2103.01489, 2104.13315).
  RequirePackage!("centernot");
  RequirePackage!("amssymb");
  // Perl newtxmath.sty.ltxml:29 `RequirePackage('txfonts')`, but newtxmath.sty never loads txfonts.sty: it declares
  // txfonts' names itself, so the binding defines txfonts' symbol set without marking txfonts loaded
  // (`\@ifpackageloaded{txfonts}` false, as in TeX; KNOWN_PERL_ERRORS #825). The set is the one the binding has
  // always carried, the names TL 2025's newtxmath.sty dropped included.
  def_txfonts_symbols()?;
  def_txfonts_symbols_newpx_lacks()?;
  // newtxmath.sty:2466 `\varmathbb`, a blackboard alphabet of its own (as txfonts.sty:920's; KPE #355).
  DefConstructor!("\\varmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
  // newtxmath.sty:2344 `\widebar` (newpxmath.sty has none, so it stays out of the shared set below).
  if !is_defined("\\widebar") {
    DefMath!("\\widebar Digested", "\u{00AF}", operator_role => "OVERACCENT");
  }
  def_newtx_math_symbols()?;
  // newtxmath.sty:246-247 `uprightGreek` (the default) and `slantedGreek`, and :251, :354 `frenchmath` upright; :277
  // `\ProcessOptions` runs them in declaration order (latex.ltx `\ProcessOptions`, as `ProcessOptions!()` does:
  // `process_options(false, …)` walks `@declaredoptions`), so `slantedGreek` wins over `uprightGreek` whatever order
  // the user writes; :1015-1039 then re-declares the capital Greek letters from `letters` or `lettersA` (witness
  // 2508.05134, newpxmath `slantedGreek`).
  declare_newtx_greek_options()?;
  ProcessOptions!();
  newtx_capital_greek()?;
});

/// newtxmath.sty:246-247, :251 (newpxmath.sty:173-175): the Greek options its binding reads, recorded for
/// `newtx_capital_greek` after the options are processed. Any other option is not the binding's concern.
pub fn declare_newtx_greek_options() -> Result<()> {
  // In newtxmath.sty's order (:246 before :247; newpxmath.sty:173 before :174), which `\ProcessOptions` follows.
  DeclareOption!("uprightGreek", {
    AssignValue!("newtx_slanted_greek" => false);
  });
  DeclareOption!("slantedGreek", {
    AssignValue!("newtx_slanted_greek" => true);
  });
  DeclareOption!("frenchmath", {
    AssignValue!("newtx_french_math" => true);
  });
  DeclareOption!(None, {});
  Ok(())
}

/// newtxmath.sty:354 and :1015-1039 (newpxmath.sty:212, :595-619): `frenchmath` implies `uprightGreek`; under
/// `slantedGreek` the capital Greek letters are the `letters` italics, italic as a Latin letter unless a math alphabet
/// is in force (KNOWN_PERL_ERRORS #1147), else the upright ones of `lettersA` — over a class's italic ones (llncs).
pub fn newtx_capital_greek() -> Result<()> {
  let slanted = lookup_bool("newtx_slanted_greek") && !lookup_bool("newtx_french_math");
  declare_capital_greek("", slanted.then_some(LettersClass::Variable))
}

/// newtxmath's own symbol layer over txfonts (newtxmath.sty:695-706, :928-988, 1165-1167, 1216, 1399-1497, 1744,
/// 2072-2183, 2344, 2466-2577, 2866, 3191). newpxmath.sty is newtxmath's Palatino twin: the same helpers and the same
/// symbol names over the same symbol-font layout (newpxmath.sty:354-357, 527-570, 745, 833, 1646, 1655-1743, 1860), so
/// its binding defines them through this function too (`newpxmath_sty`).
pub fn def_newtx_math_symbols() -> Result<()> {
  // newtxmath.sty:695-706: its re-declaration helpers, which a class calls to replace a math symbol (vmsta2.cls:236
  // `\re@DeclareMathAccent`; 2609.02595). Perl's binding lacks them too (KNOWN_PERL_ERRORS #510).
  RawTeX!(
    r"\def\re@DeclareMathDelimiter#1#2#3#4#5#6{\let#1=\undefined\DeclareMathDelimiter{#1}{#2}{#3}{#4}{#5}{#6}}
\def\re@DeclareMathAccent#1#2#3#4{\let#1=\undefined\DeclareMathAccent{#1}{#2}{#3}{#4}}
\def\re@DeclareMathRadical#1#2#3#4{\let#1=\undefined\DeclareMathRadical{#1}{#2}{#3}{#4}}
\def\re@DeclareMathSymbol#1#2#3#4{\if\relax\noexpand#1\let#1=\undefined \fi \DeclareMathSymbol{#1}{#2}{#3}{#4}}"
  );
  // newtxmath's own symbols papers reach for (:928-988, 1216, 1399-1497, 1744, 2072, 2344, 2866, 3191; `\widebar`,
  // `\Angstrom` and `\bigtimes` led the first errors of 400 run-340 papers, witnesses 2008.07007, 2604.11906,
  // 2407.15192), as their glyphs. A name a paper or another package defined first keeps that definition, as
  // `\DeclareMathSymbol` refuses one (the reason for `\re@DeclareMathSymbol` above).
  if !is_defined("\\Angstrom") {
    DefMath!("\\Angstrom", None, "\u{212B}", font => { shape => "upright", forceshape => true });
  }
  if !is_defined("\\bigtimes") {
    DefMath!("\\bigtimes", "\u{2A09}", meaning => "nary-times", role => "BIGOP", scriptpos => "mid",
      variablesize_op => true);
  }
  RawTeX!(r"\let\varprod\bigtimes");
  if !is_defined("\\bigcupdot") {
    DefMath!("\\bigcupdot", "\u{2A03}", role => "BIGOP", scriptpos => "mid", variablesize_op => true);
  }
  if !is_defined("\\sumint") {
    DefMath!("\\sumint", "\u{2A0B}", role => "INTOP", meaning => "summation-integral", variablesize_op => true);
  }
  if !is_defined("\\smallsum") {
    DefMath!("\\smallsum", None, "\u{2211}", role => "SUMOP", meaning => "sum");
  }
  // :1216-1217 `\protected\def\vv{\@ifstar{\vvstar}{\txvec}}`, the arrow accent, and `\vvstar#1#2`, the arrow over
  // #1 with #2 as its subscript (as esvect.sty:44's).
  RawTeX!(
    r"\protected\def\vv{\@ifstar\vvstar\overrightarrow}
\def\vvstar#1#2{{\overrightarrow{#1}}_{\mkern-1mu\relax#2}}"
  );
  // newtxmath.sty:1973-1974 (newpxmath.sty:1541-1542) `\re@DeclareMathSymbol{\coloneq}{\mathrel}{lettersA}{58}` and
  // `{\eqcolon}…{59}`, whatever was there before — txfonts' ":-" and "-:", or mathtools' when it loaded first (pdftotext
  // U+2254 and U+2255; 2304.05713, 2103.00564, 2108.04582, 2212.01087, 2508.05134, 2603.09013) — and :2238
  // (newpxmath.sty:1858) `\varg`, the italic g of lettersA (txfonts' is ℊ).
  DefMath!("\\coloneq", "\u{2254}", role => "RELOP");
  DefMath!("\\eqcolon", "\u{2255}", role => "RELOP");
  DefMath!("\\varg", "g");
  if !is_defined("\\mapsfrom") {
    DefMath!("\\mapsfrom", None, "\u{21A4}", role => "ARROW");
  }
  RawTeX!(r"\let\mappedfrom\mapsfrom\let\lBrack\llbracket\let\rBrack\rrbracket");
  if !is_defined("\\transp") {
    DefMath!("\\transp", None, "\u{22BA}", meaning => "transpose");
  }
  if !is_defined("\\uppartial") {
    DefMath!("\\uppartial", None, "\u{2202}", font => { shape => "upright", forceshape => true });
  }
  if !is_defined("\\lambdabar") {
    DefMath!("\\lambdabar", None, "\u{019B}");
  }
  if !is_defined("\\cdotB") {
    DefMath!("\\cdotB", None, "\u{22C5}");
  }
  // :1166 (newpxmath.sty:745) `\DeclareMathSymbol{\laplac}{\mathord}{symbols}{218}`, "the unicode-math name for the
  // official d'Alembertian operator, U+29E0" (newtxdoc.tex:118, newpxdoc.tex:147; pdftotext reads U+29E0); 2603.09013.
  if !is_defined("\\laplac") {
    DefMath!("\\laplac", None, "\u{29E0}");
  }
  // newtxmath.sty:2466-2467 and :2577: its own `\varmathbb` (each binding's, above), `\vmathbb` the
  // same, and a third blackboard variant `\vvmathbb` (the same font here). Constructors of their
  // own, so the reversion keeps the author's name.
  DefConstructor!("\\vmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
  DefConstructor!("\\vvmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
  // newtxmath.sty:2087-2097: the upright capitals of its lettersA font and `\varkappaup` (txfonts, above, has the
  // lowercase `\alphaup` set), and :2138-2183 the `\up<letter>` names "for compatibility with other packages" —
  // `\upmu` was undefined (2609.07528). The style `\let`s each name to its `\<letter>up`; a symbol of its own here
  // keeps the author's name in the reversion, as upgreek's `\upmu` does.
  for (letter, ch) in [
    ("Gamma", "\u{0393}"),
    ("Delta", "\u{0394}"),
    ("Theta", "\u{0398}"),
    ("Lambda", "\u{039B}"),
    ("Xi", "\u{039E}"),
    ("Pi", "\u{03A0}"),
    ("Sigma", "\u{03A3}"),
    ("Upsilon", "\u{03A5}"),
    ("Phi", "\u{03A6}"),
    ("Psi", "\u{03A8}"),
    ("Omega", "\u{03A9}"),
    ("varkappa", "\u{03F0}"),
  ] {
    def_math_upright_greek(&format!("\\{letter}up"), ch)?;
  }
  for (letter, ch) in [
    ("Gamma", "\u{0393}"),
    ("Delta", "\u{0394}"),
    ("Theta", "\u{0398}"),
    ("Lambda", "\u{039B}"),
    ("Xi", "\u{039E}"),
    ("Pi", "\u{03A0}"),
    ("Sigma", "\u{03A3}"),
    ("Upsilon", "\u{03A5}"),
    ("Phi", "\u{03A6}"),
    ("Psi", "\u{03A8}"),
    ("Omega", "\u{03A9}"),
    ("alpha", "\u{03B1}"),
    ("beta", "\u{03B2}"),
    ("gamma", "\u{03B3}"),
    ("delta", "\u{03B4}"),
    ("epsilon", "\u{03F5}"),
    ("zeta", "\u{03B6}"),
    ("eta", "\u{03B7}"),
    ("theta", "\u{03B8}"),
    ("iota", "\u{03B9}"),
    ("kappa", "\u{03BA}"),
    ("lambda", "\u{03BB}"),
    ("mu", "\u{03BC}"),
    ("nu", "\u{03BD}"),
    ("xi", "\u{03BE}"),
    ("pi", "\u{03C0}"),
    ("rho", "\u{03C1}"),
    ("sigma", "\u{03C3}"),
    ("tau", "\u{03C4}"),
    ("upsilon", "\u{03C5}"),
    ("phi", "\u{03D5}"),
    ("chi", "\u{03C7}"),
    ("psi", "\u{03C8}"),
    ("omega", "\u{03C9}"),
    ("varepsilon", "\u{03B5}"),
    ("vartheta", "\u{03D1}"),
    ("varpi", "\u{03D6}"),
    ("varrho", "\u{03F1}"),
    ("varsigma", "\u{03C2}"),
    ("varphi", "\u{03C6}"),
    ("varkappa", "\u{03F0}"),
  ] {
    def_math_upright_greek(&format!("\\up{letter}"), ch)?;
  }
  // newtxmath.sty:2262-2301 (newpxmath.sty:1817-1856) the italic Greek `\<letter>it` of the letters font, and
  // :2085-2088 `\varkappait` (2508.05134 `\deltait`).
  for (letter, ch) in [
    ("Gamma", "\u{0393}"),
    ("Delta", "\u{0394}"),
    ("Theta", "\u{0398}"),
    ("Lambda", "\u{039B}"),
    ("Xi", "\u{039E}"),
    ("Pi", "\u{03A0}"),
    ("Sigma", "\u{03A3}"),
    ("Upsilon", "\u{03A5}"),
    ("Phi", "\u{03A6}"),
    ("Psi", "\u{03A8}"),
    ("Omega", "\u{03A9}"),
    ("alpha", "\u{03B1}"),
    ("beta", "\u{03B2}"),
    ("gamma", "\u{03B3}"),
    ("delta", "\u{03B4}"),
    ("epsilon", "\u{03F5}"),
    ("zeta", "\u{03B6}"),
    ("eta", "\u{03B7}"),
    ("theta", "\u{03B8}"),
    ("iota", "\u{03B9}"),
    ("kappa", "\u{03BA}"),
    ("lambda", "\u{03BB}"),
    ("mu", "\u{03BC}"),
    ("nu", "\u{03BD}"),
    ("xi", "\u{03BE}"),
    ("pi", "\u{03C0}"),
    ("rho", "\u{03C1}"),
    ("sigma", "\u{03C3}"),
    ("tau", "\u{03C4}"),
    ("upsilon", "\u{03C5}"),
    ("phi", "\u{03D5}"),
    ("chi", "\u{03C7}"),
    ("psi", "\u{03C8}"),
    ("omega", "\u{03C9}"),
    ("varepsilon", "\u{03B5}"),
    ("vartheta", "\u{03D1}"),
    ("varpi", "\u{03D6}"),
    ("varrho", "\u{03F1}"),
    ("varsigma", "\u{03C2}"),
    ("varphi", "\u{03C6}"),
    ("varkappa", "\u{03F0}"),
  ] {
    def_math_italic_greek(&format!("\\{letter}it"), ch)?;
  }
  Ok(())
}
