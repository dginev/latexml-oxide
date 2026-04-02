//! txfonts.sty — TX fonts math symbols
//! Perl: txfonts.sty.ltxml — full symbol set
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amssymb");

  //======================================================================
  // Table 27 — Binary operators
  DefMath!("\\circledbar", "\u{29B6}");
  DefMath!("\\circledbslash", "\u{29B8}");
  DefMath!("\\circledvee", "\u{2228}\u{20DD}");
  DefMath!("\\circledwedge", "\u{2227}\u{20DD}");
  DefMath!("\\invamp", "\u{214B}");
  DefMath!("\\boxast", "\u{29C6}");
  DefMath!("\\boxbar", "\u{25EB}");
  DefMath!("\\boxbslash", "\u{29C4}");
  DefMath!("\\boxslash", "\u{29C5}");
  DefMath!("\\circleddot", "\u{2299}");
  DefMath!("\\circledminus", "\u{2296}");
  DefMath!("\\circledplus", "\u{2295}");
  DefMath!("\\circledslash", "\u{2298}");
  DefMath!("\\circledtimes", "\u{2297}");

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
  DefMath!("\\boxdotleft", "\u{2190}\u{22A1}", role => "RELOP");
  DefMath!("\\boxdotLeft", "\u{21D0}\u{22A1}", role => "RELOP");
  DefMath!("\\boxdotright", "\u{22A1}\u{2192}", role => "RELOP");
  DefMath!("\\boxdotRight", "\u{22A1}\u{21D2}", role => "RELOP");
  DefMath!("\\boxleft", "\u{2190}\u{25A1}", role => "RELOP");
  DefMath!("\\boxLeft", "\u{21D0}\u{25A1}", role => "RELOP");
  DefMath!("\\boxright", "\u{25A1}\u{2192}", role => "RELOP");
  DefMath!("\\boxRight", "\u{25A1}\u{21D2}", role => "RELOP");
  DefMath!("\\circleddotleft", "\u{2190}\u{2299}", role => "RELOP");
  DefMath!("\\circleddotright", "\u{2299}\u{2192}", role => "RELOP");
  DefMath!("\\circledgtr", "\u{29C1}", role => "RELOP");
  DefMath!("\\circledless", "\u{29C0}", role => "RELOP");
  DefMath!("\\circleleft", "\u{2190}\u{25CB}", role => "RELOP");
  DefMath!("\\circleright", "\u{25CB}\u{2192}", role => "RELOP");
  DefMath!("\\colonapprox", ":\u{2248}", role => "RELOP");
  DefMath!("\\Colonapprox", "::\u{2248}", role => "RELOP");
  DefMath!("\\coloneq", ":-", role => "RELOP");
  DefMath!("\\Coloneq", "::-", role => "RELOP");
  DefMath!("\\coloneqq", "\u{2254}", role => "RELOP");
  DefMath!("\\Coloneqq", "\u{2A74}", role => "RELOP");
  DefMath!("\\colonsim", ":\u{223C}", role => "RELOP");
  DefMath!("\\Colonsim", "::\u{223C}", role => "RELOP");
  DefMath!("\\Diamonddotleft", "\u{2190}\u{27D0}", role => "RELOP");
  DefMath!("\\DiamonddotLeft", "\u{21D0}\u{27D0}", role => "RELOP");
  DefMath!("\\Diamonddotright", "\u{27D0}\u{2192}", role => "RELOP");
  DefMath!("\\DiamonddotRight", "\u{27D0}\u{21D2}", role => "RELOP");
  DefMath!("\\Diamondleft", "\u{2190}\u{25C7}", role => "RELOP");
  DefMath!("\\DiamondLeft", "\u{21D0}\u{25C7}", role => "RELOP");
  DefMath!("\\Diamondright", "\u{25C7}\u{2192}", role => "RELOP");
  DefMath!("\\DiamondRight", "\u{25C7}\u{21D2}", role => "RELOP");
  DefMath!("\\Eqcolon", "-::", role => "RELOP");
  DefMath!("\\eqcolon", "-:", role => "RELOP");
  DefMath!("\\Eqqcolon", "=::", role => "RELOP");
  DefMath!("\\eqqcolon", "\u{2255}", role => "RELOP");
  DefMath!("\\eqsim", "\u{2242}", role => "RELOP");
  DefMath!("\\leftsquigarrow", "\u{21DC}", role => "RELOP");
  DefMath!("\\lJoin", "\u{22C9}", role => "RELOP");
  DefMath!("\\lrtimes", "\u{22C8}", role => "RELOP");
  DefMath!("\\Join", "\u{22C8}", role => "RELOP");
  DefMath!("\\lrJoin", "\u{22C8}", role => "RELOP");
  DefMath!("\\Mappedfromchar", "\u{2AE4}", role => "RELOP");
  DefMath!("\\mappedfromchar", "\u{2ADE}", role => "RELOP");
  DefMath!("\\mmapstochar", "\u{2AE3}", role => "RELOP");
  DefMath!("\\Mmapstochar", "\u{2AE5}", role => "RELOP");
  DefMath!("\\multimapboth", "\u{29DF}", role => "RELOP");
  DefMath!("\\multimapdotbothA", "\u{22B6}", role => "RELOP");
  DefMath!("\\multimapdotbothB", "\u{22B7}", role => "RELOP");
  DefMath!("\\multimapinv", "\u{27DC}", role => "RELOP");
  DefMath!("\\napproxeq", "\u{224A}\u{0338}", meaning => "not-approximately-equals",
    role => "RELOP");
  DefMath!("\\nasymp", "\u{226D}", meaning => "not-equivalent-to", role => "RELOP");
  DefMath!("\\nbacksim", "\u{223D}\u{0337}", role => "RELOP");
  DefMath!("\\nbacksimeq", "\u{22CD}\u{0338}", role => "RELOP");
  DefMath!("\\nBumpeq", "\u{224E}\u{0338}", role => "RELOP");
  DefMath!("\\nbumpeq", "\u{224F}\u{0338}", role => "RELOP");
  DefMath!("\\Nearrow", "\u{21D7}", role => "ARROW");
  DefMath!("\\nequiv", "\u{2262}", meaning => "not-equivalent-to", role => "RELOP");
  DefMath!("\\ngg", "\u{226B}\u{0338}", role => "RELOP");
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
  DefMath!("\\ntwoheadleftarrow", "\u{2B34}", role => "RELOP");
  DefMath!("\\ntwoheadrightarrow", "\u{2900}", role => "RELOP");
  DefMath!("\\nVdash", "\u{22AE}", role => "RELOP", meaning => "not-forces");
  DefMath!("\\Nwarrow", "\u{21D6}", role => "ARROW");
  DefMath!("\\Perp", "\u{2AEB}", role => "RELOP");
  DefMath!("\\preceqq", "\u{2AB3}", role => "RELOP", meaning => "precedes-or-equals");
  DefMath!("\\precneqq", "\u{2AB5}", role => "RELOP", meaning => "precedes-and-not-equals");
  DefMath!("\\rJoin", "\u{22CA}", role => "RELOP",
    meaning => "right-normal-factor-semidirect-product");
  DefMath!("\\Rrightarrow", "\u{21DB}", role => "RELOP");
  DefMath!("\\Searrow", "\u{21D8}", role => "ARROW");
  DefMath!("\\strictfi", "\u{297C}", role => "RELOP");
  DefMath!("\\strictif", "\u{297D}", role => "RELOP");
  DefMath!("\\strictiff", "\u{297C}\u{297D}", role => "RELOP");
  DefMath!("\\succeqq", "\u{2AB4}", role => "RELOP", meaning => "succeeds-or-equals");
  DefMath!("\\succneqq", "\u{2AB6}", role => "RELOP", meaning => "succeeds-and-not-equals");
  DefMath!("\\Swarrow", "\u{21D9}", role => "ARROW");
  DefMath!("\\varparallel", "\u{2AFD}", role => "RELOP");
  DefMath!("\\napprox", "\u{2249}", meaning => "not-approximately-equals", role => "RELOP");
  DefMath!("\\nsubset", "\u{2284}", meaning => "not-subset-of", role => "RELOP");
  DefMath!("\\nsupset", "\u{2285}", meaning => "not-superset-of", role => "RELOP");

  //======================================================================
  // Table 38 — Arrows
  DefMath!("\\Longmappedfrom", "\u{27FD}", role => "ARROW");
  DefMath!("\\Longmapsto", "\u{27FE}", role => "ARROW");
  DefMath!("\\Mappedfrom", "\u{2906}", role => "ARROW");
  DefMath!("\\Mapsto", "\u{2907}", role => "ARROW");

  //======================================================================
  // Table 43 — Upright Greek
  DefMath!("\\alphaup", "\u{03B1}", font => { shape => "upright", forceshape => true });
  DefMath!("\\betaup", "\u{03B2}", font => { shape => "upright", forceshape => true });
  DefMath!("\\gammaup", "\u{03B3}", font => { shape => "upright", forceshape => true });
  DefMath!("\\deltaup", "\u{03B4}", font => { shape => "upright", forceshape => true });
  DefMath!("\\epsilonup", "\u{03F5}", font => { shape => "upright", forceshape => true });
  DefMath!("\\varepsilonup", "\u{03B5}", font => { shape => "upright", forceshape => true });
  DefMath!("\\zetaup", "\u{03B6}", font => { shape => "upright", forceshape => true });
  DefMath!("\\etaup", "\u{03B7}", font => { shape => "upright", forceshape => true });
  DefMath!("\\thetaup", "\u{03B8}", font => { shape => "upright", forceshape => true });
  DefMath!("\\varthetaup", "\u{03D1}", font => { shape => "upright", forceshape => true });
  DefMath!("\\iotaup", "\u{03B9}", font => { shape => "upright", forceshape => true });
  DefMath!("\\kappaup", "\u{03BA}", font => { shape => "upright", forceshape => true });
  DefMath!("\\lambdaup", "\u{03BB}", font => { shape => "upright", forceshape => true });
  DefMath!("\\muup", "\u{03BC}", font => { shape => "upright", forceshape => true });
  DefMath!("\\nuup", "\u{03BD}", font => { shape => "upright", forceshape => true });
  DefMath!("\\xiup", "\u{03BE}", font => { shape => "upright", forceshape => true });
  DefMath!("\\piup", "\u{03C0}", font => { shape => "upright", forceshape => true });
  DefMath!("\\varpiup", "\u{03D6}", font => { shape => "upright", forceshape => true });
  DefMath!("\\rhoup", "\u{03C1}", font => { shape => "upright", forceshape => true });
  DefMath!("\\varrhoup", "\u{03F1}", font => { shape => "upright", forceshape => true });
  DefMath!("\\sigmaup", "\u{03C3}", font => { shape => "upright", forceshape => true });
  DefMath!("\\varsigmaup", "\u{03C2}", font => { shape => "upright", forceshape => true });
  DefMath!("\\tauup", "\u{03C4}", font => { shape => "upright", forceshape => true });
  DefMath!("\\upsilonup", "\u{03C5}", font => { shape => "upright", forceshape => true });
  DefMath!("\\phiup", "\u{03D5}", font => { shape => "upright", forceshape => true });
  DefMath!("\\varphiup", "\u{03C6}", font => { shape => "upright", forceshape => true });
  DefMath!("\\chiup", "\u{03C7}", font => { shape => "upright", forceshape => true });
  DefMath!("\\psiup", "\u{03C8}", font => { shape => "upright", forceshape => true });
  DefMath!("\\omegaup", "\u{03C9}", font => { shape => "upright", forceshape => true });

  //======================================================================
  // Table 44 — Variant letterforms
  DefMath!("\\varg", "\u{210A}");

  //======================================================================
  // Table 61 — Miscellaneous symbols
  DefMath!("\\Diamondblack", "\u{25C6}");
  DefMath!("\\Diamonddot", "\u{27D0}");
  DefMath!("\\mathcent", "\u{00A2}");
  DefMath!("\\mathsterling", "\u{00A3}");
  DefMath!("\\varclubsuit", "\u{2667}");
  DefMath!("\\vardiamondsuit", "\u{2666}");
  DefMath!("\\varheartsuit", "\u{2665}");
  DefMath!("\\varspadesuit", "\u{2664}");

  //======================================================================
  // Bracket symbols
  DefMath!("\\llbracket", "\u{27E6}", role => "OPEN");
  DefMath!("\\rrbracket", "\u{27E7}", role => "CLOSE");
});
