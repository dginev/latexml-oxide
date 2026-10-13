//! fourier.sty — Utopia text/math fonts (presentational). fourier.sty:61
//! loads fourier-orns (`\lefthand`, `\decoone` … pgfornament's tikzrput
//! manual); the symbol set is what documents reach.
use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // fourier.sty:61 `\RequirePackage[noOTF]{fourier-orns}`: no binding, so a missing-file warning
  // and the ornaments undefined. Its raw file draws each ornament as a slot of the `futs` font,
  // which has no glyph map here (font.rs), so a raw load would print the slot characters ("1" for
  // `\warning`) — invented text; RED `fonts-nfss/fourier_orns_ornaments_are_their_glyphs`.
  RequirePackage!("fourier-orns");
  // fourier.sty:300-303: `\math@bb` is the fourier-bb blackboard alphabet, and `\mathbb` becomes
  // it at `\begin{document}`, over amsfonts' and over a preamble
  // `\renewcommand{\mathbb}{\varmathbb}` (2406.06884; fourier has no `\varmathbb`).
  DefConstructor!("\\math@bb{}", "#1", bounded => true, require_math => true, alias => "\\mathbb",
    font => { family => "blackboard", series => "medium", shape => "upright" });
  RawTeX!(r"\AtBeginDocument{\let\mathbb\math@bb}");
  // fourier.sty:139-169, 215-227, 271-274, 304-319: the math symbols fourier declares beyond Computer Modern's,
  // as their Unicode glyphs (`\llbracket`, `\widearc`, `\VERT` led the first errors of 163 run-340 papers;
  // witnesses 2008.00463, 2508.01413, 1708.05248). Names amssymb (or another package) defined first keep that
  // definition: the same glyph under its established role and meaning.
  if !is_defined("\\leqslant") { DefMath!("\\leqslant", None, "\u{2A7D}", role => "RELOP", meaning => "less-than-or-equals"); }
  if !is_defined("\\geqslant") { DefMath!("\\geqslant", None, "\u{2A7E}", role => "RELOP", meaning => "greater-than-or-equals"); }
  if !is_defined("\\nleqslant") { DefMath!("\\nleqslant", None, "\u{2A7D}\u{0338}", role => "RELOP"); }
  if !is_defined("\\ngeqslant") { DefMath!("\\ngeqslant", None, "\u{2A7E}\u{0338}", role => "RELOP"); }
  if !is_defined("\\parallelslant") { DefMath!("\\parallelslant", None, "\u{2AFD}", role => "RELOP"); }
  if !is_defined("\\nparallelslant") { DefMath!("\\nparallelslant", None, "\u{2AFD}\u{0338}", role => "RELOP"); }
  if !is_defined("\\nparallel") { DefMath!("\\nparallel", None, "\u{2226}", role => "RELOP"); }
  if !is_defined("\\vDash") { DefMath!("\\vDash", None, "\u{22A8}", role => "RELOP"); }
  if !is_defined("\\nvDash") { DefMath!("\\nvDash", None, "\u{22AD}", role => "RELOP"); }
  if !is_defined("\\blacktriangleleft") { DefMath!("\\blacktriangleleft", None, "\u{25C0}", role => "RELOP"); }
  if !is_defined("\\blacktriangleright") { DefMath!("\\blacktriangleright", None, "\u{25B6}", role => "RELOP"); }
  if !is_defined("\\intercal") { DefMath!("\\intercal", None, "\u{22BA}", role => "ADDOP"); }
  if !is_defined("\\hslash") { DefMath!("\\hslash", None, "\u{210F}"); }
  if !is_defined("\\nexists") { DefMath!("\\nexists", None, "\u{2204}", role => "FUNCTION", meaning => "not-exists"); }
  if !is_defined("\\complement") { DefMath!("\\complement", None, "\u{2201}"); }
  if !is_defined("\\varsubsetneq") { DefMath!("\\varsubsetneq", None, "\u{228A}", role => "RELOP"); }
  if !is_defined("\\notowns") { DefMath!("\\notowns", None, "\u{220C}", role => "RELOP"); }
  if !is_defined("\\smallsetminus") { DefMath!("\\smallsetminus", None, "\u{2216}", role => "ADDOP"); }
  if !is_defined("\\subsetneqq") { DefMath!("\\subsetneqq", None, "\u{2ACB}", role => "RELOP"); }
  if !is_defined("\\rightrightarrows") { DefMath!("\\rightrightarrows", None, "\u{21C9}", role => "ARROW"); }
  if !is_defined("\\leftleftarrows") { DefMath!("\\leftleftarrows", None, "\u{21C7}", role => "ARROW"); }
  if !is_defined("\\square") { DefMath!("\\square", None, "\u{25A1}"); }
  if !is_defined("\\blacksquare") { DefMath!("\\blacksquare", None, "\u{25A0}"); }
  if !is_defined("\\curvearrowleft") { DefMath!("\\curvearrowleft", None, "\u{21B6}", role => "ARROW"); }
  if !is_defined("\\curvearrowright") { DefMath!("\\curvearrowright", None, "\u{21B7}", role => "ARROW"); }
  if !is_defined("\\llbracket") { DefMath!("\\llbracket", None, "\u{27E6}", role => "OPEN"); }
  if !is_defined("\\rrbracket") { DefMath!("\\rrbracket", None, "\u{27E7}", role => "CLOSE"); }
  RawTeX!(r"\def\dblbrackleft{\llbracket}\def\dblbrackright{\rrbracket}");
  if !is_defined("\\VERT") { DefMath!("\\VERT", None, "\u{2980}", role => "VERTBAR"); }
  if !is_defined("\\wideparen") { DefMath!("\\wideparen Digested", "\u{23DC}", operator_role => "OVERACCENT"); }
  if !is_defined("\\widearc") { DefMath!("\\widearc Digested", "\u{23DC}", operator_role => "OVERACCENT"); }
  if !is_defined("\\iint") {
    DefMath!("\\iint", None, "\u{222C}", role => "INTOP", meaning => "double-integral", variablesize_op => true);
  }
  if !is_defined("\\iiint") {
    DefMath!("\\iiint", None, "\u{222D}", role => "INTOP", meaning => "triple-integral", variablesize_op => true);
  }
  if !is_defined("\\oiint") {
    DefMath!("\\oiint", None, "\u{222F}", role => "INTOP", meaning => "surface-integral", variablesize_op => true);
  }
  if !is_defined("\\oiiint") {
    DefMath!("\\oiiint", None, "\u{2230}", role => "INTOP", meaning => "volume-integral", variablesize_op => true);
  }
});
