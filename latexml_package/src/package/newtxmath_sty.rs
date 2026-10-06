//! newtxmath.sty — TX math fonts (delegates to other packages)
//! Perl: newtxmath.sty.ltxml
use super::txfonts_sty::def_math_upright_greek;
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
  RequirePackage!("amssymb");
  RequirePackage!("txfonts");
  // newtxmath.sty:695-706: its re-declaration helpers, which a class calls to replace a math symbol (vmsta2.cls:236
  // `\re@DeclareMathAccent`; 2609.02595). Perl's binding lacks them too (KNOWN_PERL_ERRORS #510).
  RawTeX!(
    r"\def\re@DeclareMathDelimiter#1#2#3#4#5#6{\let#1=\undefined\DeclareMathDelimiter{#1}{#2}{#3}{#4}{#5}{#6}}
\def\re@DeclareMathAccent#1#2#3#4{\let#1=\undefined\DeclareMathAccent{#1}{#2}{#3}{#4}}
\def\re@DeclareMathRadical#1#2#3#4{\let#1=\undefined\DeclareMathRadical{#1}{#2}{#3}{#4}}"
  );
  // newtxmath.sty:2466-2467 and :2577: its own `\varmathbb` (txfonts' above), `\vmathbb` the
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
});
