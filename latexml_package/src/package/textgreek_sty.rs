use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: textgreek.sty.ltxml — not present in Perl LaTeXML.
  // The TL textgreek.sty package (Leonard Michlmayr, 2010-2011) provides
  // text-mode Greek letters via `\DeclareTextGreekSymbol` over the LGR font
  // encoding. Our raw load doesn't pick them up (LGR encoding isn't
  // installed in our font stack), so the binding maps each `\text<greek>`
  // CS to its Unicode equivalent for clean XML output.
  //
  // Witnesses (stage-1..2 of 100k warning corpus):
  //   arXiv:2603.02703 — `\textsigma` undefined via cuted.sty cascade
  //   arXiv:2604.09141 — same pattern

  // Lowercase Greek
  DefPrimitive!("\\textalpha",   "\u{03B1}");
  DefPrimitive!("\\textbeta",    "\u{03B2}");
  DefPrimitive!("\\textgamma",   "\u{03B3}");
  DefPrimitive!("\\textdelta",   "\u{03B4}");
  DefPrimitive!("\\textepsilon", "\u{03B5}");
  DefPrimitive!("\\textzeta",    "\u{03B6}");
  DefPrimitive!("\\texteta",     "\u{03B7}");
  DefPrimitive!("\\texttheta",   "\u{03B8}");
  DefPrimitive!("\\textiota",    "\u{03B9}");
  DefPrimitive!("\\textkappa",   "\u{03BA}");
  DefPrimitive!("\\textlambda",  "\u{03BB}");
  DefPrimitive!("\\textmugreek", "\u{03BC}");
  DefPrimitive!("\\textnu",      "\u{03BD}");
  DefPrimitive!("\\textxi",      "\u{03BE}");
  DefPrimitive!("\\textomikron", "\u{03BF}");
  DefPrimitive!("\\textpi",      "\u{03C0}");
  DefPrimitive!("\\textrho",     "\u{03C1}");
  DefPrimitive!("\\textsigma",   "\u{03C3}");
  DefPrimitive!("\\textvarsigma","\u{03C2}");
  DefPrimitive!("\\texttau",     "\u{03C4}");
  DefPrimitive!("\\textupsilon", "\u{03C5}");
  DefPrimitive!("\\textphi",     "\u{03C6}");
  DefPrimitive!("\\textchi",     "\u{03C7}");
  DefPrimitive!("\\textpsi",     "\u{03C8}");
  DefPrimitive!("\\textomega",   "\u{03C9}");

  // Uppercase Greek
  DefPrimitive!("\\textAlpha",   "\u{0391}");
  DefPrimitive!("\\textBeta",    "\u{0392}");
  DefPrimitive!("\\textGamma",   "\u{0393}");
  DefPrimitive!("\\textDelta",   "\u{0394}");
  DefPrimitive!("\\textEpsilon", "\u{0395}");
  DefPrimitive!("\\textZeta",    "\u{0396}");
  DefPrimitive!("\\textEta",     "\u{0397}");
  DefPrimitive!("\\textTheta",   "\u{0398}");
  DefPrimitive!("\\textIota",    "\u{0399}");
  DefPrimitive!("\\textKappa",   "\u{039A}");
  DefPrimitive!("\\textLambda",  "\u{039B}");
  DefPrimitive!("\\textMu",      "\u{039C}");
  DefPrimitive!("\\textNu",      "\u{039D}");
  DefPrimitive!("\\textXi",      "\u{039E}");
  DefPrimitive!("\\textOmikron", "\u{039F}");
  DefPrimitive!("\\textPi",      "\u{03A0}");
  DefPrimitive!("\\textRho",     "\u{03A1}");
  DefPrimitive!("\\textSigma",   "\u{03A3}");
  DefPrimitive!("\\textTau",     "\u{03A4}");
  DefPrimitive!("\\textUpsilon", "\u{03A5}");
  DefPrimitive!("\\textPhi",     "\u{03A6}");
  DefPrimitive!("\\textChi",     "\u{03A7}");
  DefPrimitive!("\\textPsi",     "\u{03A8}");
  DefPrimitive!("\\textOmega",   "\u{03A9}");
});
