//! txfonts.sty — TX fonts math symbols
//! Perl: txfonts.sty.ltxml — full symbol set
use crate::prelude::*;

/// Runtime helper for the trivial `DefMath!` shape used 100+ times in
/// txfonts (DEP-17, mirrors DEP-15 fontawesome approach). The macro
/// arm expands at compile time into ~1.1 KiB of `.text` per invocation
/// (parse_prototype + MathPrimitiveOptions builder); routing through
/// this single helper drops `load_definitions` size at the cost of a
/// runtime `parse_prototype` call per entry — paid once at engine
/// bootstrap.
fn def_math_sym(cs: &str, present: &str, role: Option<&str>, meaning: Option<&str>) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let mut opts = MathPrimitiveOptions::default();
  if let Some(r) = role { opts.role = Some(r.to_string()); }
  if let Some(m) = meaning { opts.meaning = Some(m.to_string()); }
  def_math(cs_tok, params, present.to_string(), opts)?;
  Ok(())
}

/// DEP-17 helper for the upright-Greek `DefMath!("\\xxxup", "char",
/// font => { shape => "upright", forceshape => true })` shape — 29
/// entries in txfonts (the lowercase + uppercase Greek `*up` family).
fn def_math_upright_greek(cs: &str, present: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let opts = MathPrimitiveOptions {
    font: Some(FontDirective::from(fontmap!(shape => "upright", forceshape => true))),
    ..MathPrimitiveOptions::default()
  };
  def_math(cs_tok, params, present.to_string(), opts)?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amssymb");

  //======================================================================
  // Table 27 — Binary operators
  def_math_sym("\\circledbar", "\u{29B6}", None, None)?;
  def_math_sym("\\circledbslash", "\u{29B8}", None, None)?;
  def_math_sym("\\circledvee", "\u{2228}\u{20DD}", None, None)?;
  def_math_sym("\\circledwedge", "\u{2227}\u{20DD}", None, None)?;
  def_math_sym("\\invamp", "\u{214B}", None, None)?;
  def_math_sym("\\boxast", "\u{29C6}", None, None)?;
  def_math_sym("\\boxbar", "\u{25EB}", None, None)?;
  def_math_sym("\\boxbslash", "\u{29C4}", None, None)?;
  def_math_sym("\\boxslash", "\u{29C5}", None, None)?;
  def_math_sym("\\circleddot", "\u{2299}", None, None)?;
  def_math_sym("\\circledminus", "\u{2296}", None, None)?;
  def_math_sym("\\circledplus", "\u{2295}", None, None)?;
  def_math_sym("\\circledslash", "\u{2298}", None, None)?;
  def_math_sym("\\circledtimes", "\u{2297}", None, None)?;

  //======================================================================
  // Table 28 — Variable-size operators (integrals)
  DefMath!("\\fint", "\u{2A0F}", meaning => "integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\fintop", "\u{2A0F}", meaning => "integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\idotsint", "\u{222B}\u{22EF}\u{222B}", meaning => "multiple-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\idotsintop", "\u{222B}\u{22EF}\u{222B}", meaning => "multiple-integral",
    role => "INTOP", scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\iint", "\u{222C}", meaning => "double-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\iintop", "\u{222C}", meaning => "double-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\iiint", "\u{222D}", meaning => "triple-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\iiintop", "\u{222D}", meaning => "triple-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\iiiint", "\u{2A0C}", meaning => "quadruple-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\iiiintop", "\u{2A0C}", meaning => "quadruple-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);

  // Clockwise/counter-clockwise contour integrals with combining overlays
  DefMath!("\\oiiintclockwise", "\u{222D}\u{20D9}",
    meaning => "triple-clockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\oiiintclockwiseop", "\u{222D}\u{20D9}",
    meaning => "triple-clockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\varoiiintclockwise", "\u{222D}\u{20D9}",
    meaning => "triple-clockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\varoiiintclockwiseop", "\u{222D}\u{20D9}",
    meaning => "triple-clockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\oiiintctrclockwise", "\u{222D}\u{20DA}",
    meaning => "triple-counterclockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\oiiintctrclockwiseop", "\u{222D}\u{20DA}",
    meaning => "triple-counterclockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\varoiiintctrclockwise", "\u{222D}\u{20DA}",
    meaning => "triple-counterclockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\varoiiintctrclockwiseop", "\u{222D}\u{20DA}",
    meaning => "triple-counterclockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\oiiint", "\u{2230}",
    meaning => "triple-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\oiiintop", "\u{2230}",
    meaning => "triple-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\oiintclockwise", "\u{222C}\u{20D9}",
    meaning => "double-clockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\oiintclockwiseop", "\u{222C}\u{20D9}",
    meaning => "double-clockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\varoiintclockwise", "\u{222C}\u{20D9}",
    meaning => "double-clockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\varoiintclockwiseop", "\u{222C}\u{20D9}",
    meaning => "double-clockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\oiintctrclockwise", "\u{222C}\u{20DA}",
    meaning => "double-counterclockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\oiintctrclockwiseop", "\u{222C}\u{20DA}",
    meaning => "double-counterclockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\varoiintctrclockwise", "\u{222C}\u{20DA}",
    meaning => "double-counterclockwise-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\varoiintctrclockwiseop", "\u{222C}\u{20DA}",
    meaning => "double-counterclockwise-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\oiint", "\u{222F}", meaning => "double-contour-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\oiintop", "\u{222F}", meaning => "double-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\ointclockwise", "\u{2232}", meaning => "clockwise-contour-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\ointclockwiseop", "\u{2232}", meaning => "clockwise-contour-integral",
    role => "INTOP", scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\ointctrclockwise", "\u{2233}", meaning => "counter-clockwise-contour-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\ointctrclockwiseop", "\u{2233}", meaning => "counter-clockwise-contour-integral",
    role => "INTOP", scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\varointclockwise", "\u{2232}", meaning => "clockwise-contour-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\varointclockwiseop", "\u{2232}", meaning => "clockwise-contour-integral",
    role => "INTOP", scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\varointctrclockwise", "\u{2233}", meaning => "counter-clockwise-contour-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\varointctrclockwiseop", "\u{2233}", meaning => "counter-clockwise-contour-integral",
    role => "INTOP", scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\sqint", "\u{2A16}", role => "INTOP", meaning => "square-contour-integral",
    dynamic_mathstyle => true);

  DefMath!("\\bigsqcap", None, "\u{2A05}", role => "SUMOP",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  //======================================================================
  // Table 34 — Relations
  def_math_sym("\\boxdotleft", "\u{2190}\u{22A1}", Some("RELOP"), None)?;
  def_math_sym("\\boxdotLeft", "\u{21D0}\u{22A1}", Some("RELOP"), None)?;
  def_math_sym("\\boxdotright", "\u{22A1}\u{2192}", Some("RELOP"), None)?;
  def_math_sym("\\boxdotRight", "\u{22A1}\u{21D2}", Some("RELOP"), None)?;
  def_math_sym("\\boxleft", "\u{2190}\u{25A1}", Some("RELOP"), None)?;
  def_math_sym("\\boxLeft", "\u{21D0}\u{25A1}", Some("RELOP"), None)?;
  def_math_sym("\\boxright", "\u{25A1}\u{2192}", Some("RELOP"), None)?;
  def_math_sym("\\boxRight", "\u{25A1}\u{21D2}", Some("RELOP"), None)?;
  def_math_sym("\\circleddotleft", "\u{2190}\u{2299}", Some("RELOP"), None)?;
  def_math_sym("\\circleddotright", "\u{2299}\u{2192}", Some("RELOP"), None)?;
  def_math_sym("\\circledgtr", "\u{29C1}", Some("RELOP"), None)?;
  def_math_sym("\\circledless", "\u{29C0}", Some("RELOP"), None)?;
  def_math_sym("\\circleleft", "\u{2190}\u{25CB}", Some("RELOP"), None)?;
  def_math_sym("\\circleright", "\u{25CB}\u{2192}", Some("RELOP"), None)?;
  def_math_sym("\\colonapprox", ":\u{2248}", Some("RELOP"), None)?;
  def_math_sym("\\Colonapprox", "::\u{2248}", Some("RELOP"), None)?;
  def_math_sym("\\coloneq", ":-", Some("RELOP"), None)?;
  def_math_sym("\\Coloneq", "::-", Some("RELOP"), None)?;
  def_math_sym("\\coloneqq", "\u{2254}", Some("RELOP"), None)?;
  def_math_sym("\\Coloneqq", "\u{2A74}", Some("RELOP"), None)?;
  def_math_sym("\\colonsim", ":\u{223C}", Some("RELOP"), None)?;
  def_math_sym("\\Colonsim", "::\u{223C}", Some("RELOP"), None)?;
  def_math_sym("\\Diamonddotleft", "\u{2190}\u{27D0}", Some("RELOP"), None)?;
  def_math_sym("\\DiamonddotLeft", "\u{21D0}\u{27D0}", Some("RELOP"), None)?;
  def_math_sym("\\Diamonddotright", "\u{27D0}\u{2192}", Some("RELOP"), None)?;
  def_math_sym("\\DiamonddotRight", "\u{27D0}\u{21D2}", Some("RELOP"), None)?;
  def_math_sym("\\Diamondleft", "\u{2190}\u{25C7}", Some("RELOP"), None)?;
  def_math_sym("\\DiamondLeft", "\u{21D0}\u{25C7}", Some("RELOP"), None)?;
  def_math_sym("\\Diamondright", "\u{25C7}\u{2192}", Some("RELOP"), None)?;
  def_math_sym("\\DiamondRight", "\u{25C7}\u{21D2}", Some("RELOP"), None)?;
  def_math_sym("\\Eqcolon", "-::", Some("RELOP"), None)?;
  def_math_sym("\\eqcolon", "-:", Some("RELOP"), None)?;
  def_math_sym("\\Eqqcolon", "=::", Some("RELOP"), None)?;
  def_math_sym("\\eqqcolon", "\u{2255}", Some("RELOP"), None)?;
  def_math_sym("\\eqsim", "\u{2242}", Some("RELOP"), None)?;
  def_math_sym("\\leftsquigarrow", "\u{21DC}", Some("RELOP"), None)?;
  def_math_sym("\\lJoin", "\u{22C9}", Some("RELOP"), None)?;
  def_math_sym("\\lrtimes", "\u{22C8}", Some("RELOP"), None)?;
  def_math_sym("\\Join", "\u{22C8}", Some("RELOP"), None)?;
  def_math_sym("\\lrJoin", "\u{22C8}", Some("RELOP"), None)?;
  def_math_sym("\\Mappedfromchar", "\u{2AE4}", Some("RELOP"), None)?;
  def_math_sym("\\mappedfromchar", "\u{2ADE}", Some("RELOP"), None)?;
  def_math_sym("\\mmapstochar", "\u{2AE3}", Some("RELOP"), None)?;
  def_math_sym("\\Mmapstochar", "\u{2AE5}", Some("RELOP"), None)?;
  def_math_sym("\\multimapboth", "\u{29DF}", Some("RELOP"), None)?;
  def_math_sym("\\multimapdotbothA", "\u{22B6}", Some("RELOP"), None)?;
  def_math_sym("\\multimapdotbothB", "\u{22B7}", Some("RELOP"), None)?;
  def_math_sym("\\multimapinv", "\u{27DC}", Some("RELOP"), None)?;
  DefMath!("\\napproxeq", "\u{224A}\u{0338}", meaning => "not-approximately-equals",
    role => "RELOP");
  DefMath!("\\nasymp", "\u{226D}", meaning => "not-equivalent-to", role => "RELOP");
  def_math_sym("\\nbacksim", "\u{223D}\u{0337}", Some("RELOP"), None)?;
  def_math_sym("\\nbacksimeq", "\u{22CD}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\nBumpeq", "\u{224E}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\nbumpeq", "\u{224F}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\Nearrow", "\u{21D7}", Some("ARROW"), None)?;
  DefMath!("\\nequiv", "\u{2262}", meaning => "not-equivalent-to", role => "RELOP");
  def_math_sym("\\ngg", "\u{226B}\u{0338}", Some("RELOP"), None)?;
  DefMath!("\\ngtrapprox", "\u{2A86}\u{0338}",
    meaning => "not-greater-than-nor-approximately-equals", role => "RELOP");
  DefMath!("\\ngtrless", "\u{2278}",
    meaning => "not-greater-than-nor-less-than", role => "RELOP");
  DefMath!("\\ngtrsim", "\u{2275}",
    meaning => "not-greater-than-nor-equivalent-to", role => "RELOP");
  DefMath!("\\nlessapprox", "\u{2A85}\u{0338}",
    meaning => "not-less-than-nor-approximately-equals", role => "RELOP");
  DefMath!("\\nlessgtr", "\u{2279}",
    meaning => "not-less-than-nor-greater-than", role => "RELOP");
  DefMath!("\\nlesssim", "\u{2274}",
    meaning => "not-less-than-nor-equivalent-to", role => "RELOP");
  DefMath!("\\nll", "\u{226A}\u{0338}",
    meaning => "not-much-less-than", role => "RELOP");
  DefMath!("\\notin", "\u{2209}", meaning => "not-element-of", role => "RELOP");
  DefMath!("\\notni", "\u{220C}", meaning => "not-contains", role => "RELOP");
  DefMath!("\\notowns", "\u{220C}", meaning => "not-contains", role => "RELOP");
  DefMath!("\\nprecapprox", "\u{2AB7}\u{0338}",
    meaning => "not-precedes-nor-approximately-equals", role => "RELOP");
  DefMath!("\\npreccurlyeq", "\u{22E0}",
    meaning => "not-precedes-nor-equals", role => "RELOP");
  DefMath!("\\npreceqq", "\u{2AB3}\u{0338}", role => "RELOP",
    meaning => "not-precedes-nor-equals");
  DefMath!("\\nprecsim", "\u{227E}\u{0338}", role => "RELOP",
    meaning => "not-precedes-nor-equivalent-to");
  DefMath!("\\nsimeq", "\u{2243}\u{0338}", role => "RELOP",
    meaning => "not-equivalent-to-nor-equals");
  DefMath!("\\nsqsubset", "\u{228F}\u{0338}", role => "RELOP",
    meaning => "not-square-image-of");
  DefMath!("\\nsqsubseteq", "\u{22E2}", role => "RELOP",
    meaning => "not-square-image-of-nor-equals");
  DefMath!("\\nsqsupset", "\u{2290}\u{0338}", role => "RELOP",
    meaning => "not-square-original-of");
  DefMath!("\\nsqsupseteq", "\u{22E3}", role => "RELOP",
    meaning => "not-square-original-of-nor-equals");
  DefMath!("\\nSubset", "\u{22D0}\u{0338}", role => "RELOP",
    meaning => "not-double-subset-of");
  DefMath!("\\nsubseteqq", "\u{2AC5}\u{0338}", role => "RELOP",
    meaning => "not-subset-nor-equals");
  DefMath!("\\nsuccapprox", "\u{2AB8}\u{0338}", role => "RELOP",
    meaning => "not-succeeds-nor-approximately-equals");
  DefMath!("\\nsucccurlyeq", "\u{22E1}", role => "RELOP",
    meaning => "not-succeeds-nor-equals");
  DefMath!("\\nsucceqq", "\u{2AB4}\u{0338}", role => "RELOP",
    meaning => "not-succeeds-nor-equals");
  DefMath!("\\nsuccsim", "\u{227F}\u{0338}", role => "RELOP",
    meaning => "not-succeeds-nor-equivalent-to");
  DefMath!("\\nSupset", "\u{22D1}\u{0338}", role => "RELOP",
    meaning => "not-double-superset-of");
  DefMath!("\\nthickapprox", "\u{2249}", role => "RELOP",
    meaning => "not-approximately-equals");
  def_math_sym("\\ntwoheadleftarrow", "\u{2B34}", Some("RELOP"), None)?;
  def_math_sym("\\ntwoheadrightarrow", "\u{2900}", Some("RELOP"), None)?;
  def_math_sym("\\nVdash", "\u{22AE}", Some("RELOP"), Some("not-forces"))?;
  def_math_sym("\\Nwarrow", "\u{21D6}", Some("ARROW"), None)?;
  def_math_sym("\\Perp", "\u{2AEB}", Some("RELOP"), None)?;
  def_math_sym("\\preceqq", "\u{2AB3}", Some("RELOP"), Some("precedes-or-equals"))?;
  def_math_sym("\\precneqq", "\u{2AB5}", Some("RELOP"), Some("precedes-and-not-equals"))?;
  DefMath!("\\rJoin", "\u{22CA}", role => "RELOP",
    meaning => "right-normal-factor-semidirect-product");
  def_math_sym("\\Rrightarrow", "\u{21DB}", Some("RELOP"), None)?;
  def_math_sym("\\Searrow", "\u{21D8}", Some("ARROW"), None)?;
  def_math_sym("\\strictfi", "\u{297C}", Some("RELOP"), None)?;
  def_math_sym("\\strictif", "\u{297D}", Some("RELOP"), None)?;
  def_math_sym("\\strictiff", "\u{297C}\u{297D}", Some("RELOP"), None)?;
  def_math_sym("\\succeqq", "\u{2AB4}", Some("RELOP"), Some("succeeds-or-equals"))?;
  def_math_sym("\\succneqq", "\u{2AB6}", Some("RELOP"), Some("succeeds-and-not-equals"))?;
  def_math_sym("\\Swarrow", "\u{21D9}", Some("ARROW"), None)?;
  def_math_sym("\\varparallel", "\u{2AFD}", Some("RELOP"), None)?;
  DefMath!("\\napprox", "\u{2249}", meaning => "not-approximately-equals", role => "RELOP");
  DefMath!("\\nsubset", "\u{2284}", meaning => "not-subset-of", role => "RELOP");
  DefMath!("\\nsupset", "\u{2285}", meaning => "not-superset-of", role => "RELOP");

  //======================================================================
  // Table 38 — Arrows
  def_math_sym("\\Longmappedfrom", "\u{27FD}", Some("ARROW"), None)?;
  def_math_sym("\\Longmapsto", "\u{27FE}", Some("ARROW"), None)?;
  def_math_sym("\\Mappedfrom", "\u{2906}", Some("ARROW"), None)?;
  def_math_sym("\\Mapsto", "\u{2907}", Some("ARROW"), None)?;

  //======================================================================
  // Table 43 — Upright Greek
  def_math_upright_greek("\\alphaup", "\u{03B1}")?;
  def_math_upright_greek("\\betaup", "\u{03B2}")?;
  def_math_upright_greek("\\gammaup", "\u{03B3}")?;
  def_math_upright_greek("\\deltaup", "\u{03B4}")?;
  def_math_upright_greek("\\epsilonup", "\u{03F5}")?;
  def_math_upright_greek("\\varepsilonup", "\u{03B5}")?;
  def_math_upright_greek("\\zetaup", "\u{03B6}")?;
  def_math_upright_greek("\\etaup", "\u{03B7}")?;
  def_math_upright_greek("\\thetaup", "\u{03B8}")?;
  def_math_upright_greek("\\varthetaup", "\u{03D1}")?;
  def_math_upright_greek("\\iotaup", "\u{03B9}")?;
  def_math_upright_greek("\\kappaup", "\u{03BA}")?;
  def_math_upright_greek("\\lambdaup", "\u{03BB}")?;
  def_math_upright_greek("\\muup", "\u{03BC}")?;
  def_math_upright_greek("\\nuup", "\u{03BD}")?;
  def_math_upright_greek("\\xiup", "\u{03BE}")?;
  def_math_upright_greek("\\piup", "\u{03C0}")?;
  def_math_upright_greek("\\varpiup", "\u{03D6}")?;
  def_math_upright_greek("\\rhoup", "\u{03C1}")?;
  def_math_upright_greek("\\varrhoup", "\u{03F1}")?;
  def_math_upright_greek("\\sigmaup", "\u{03C3}")?;
  def_math_upright_greek("\\varsigmaup", "\u{03C2}")?;
  def_math_upright_greek("\\tauup", "\u{03C4}")?;
  def_math_upright_greek("\\upsilonup", "\u{03C5}")?;
  def_math_upright_greek("\\phiup", "\u{03D5}")?;
  def_math_upright_greek("\\varphiup", "\u{03C6}")?;
  def_math_upright_greek("\\chiup", "\u{03C7}")?;
  def_math_upright_greek("\\psiup", "\u{03C8}")?;
  def_math_upright_greek("\\omegaup", "\u{03C9}")?;

  //======================================================================
  // Table 44 — Variant letterforms
  def_math_sym("\\varg", "\u{210A}", None, None)?;

  //======================================================================
  // Table 61 — Miscellaneous symbols
  def_math_sym("\\Diamondblack", "\u{25C6}", None, None)?;
  def_math_sym("\\Diamonddot", "\u{27D0}", None, None)?;
  def_math_sym("\\mathcent", "\u{00A2}", None, None)?;
  def_math_sym("\\mathsterling", "\u{00A3}", None, None)?;
  def_math_sym("\\varclubsuit", "\u{2667}", None, None)?;
  def_math_sym("\\vardiamondsuit", "\u{2666}", None, None)?;
  def_math_sym("\\varheartsuit", "\u{2665}", None, None)?;
  def_math_sym("\\varspadesuit", "\u{2664}", None, None)?;

  //======================================================================
  // Bracket symbols
  def_math_sym("\\llbracket", "\u{27E6}", Some("OPEN"), None)?;
  def_math_sym("\\rrbracket", "\u{27E7}", Some("CLOSE"), None)?;
});
