use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // Specials  (matha/mathb)
  // These are intended to overlay to show negation,
  // but they're not going to work well for that.
  DefMath!("\\notsign",    "|", role => "OPERATOR", meaning => "not");
  DefMath!("\\varnotsign", "/", role => "OPERATOR", meaning => "not");
  // \changenotsign — Perl mathabx.sty.ltxml L24-26 is a DefPrimitive that
  // emits an Info('unexpected', '\changenotsign', ...,
  // "The \changenotsign operation of mathabx is not implemented.").
  // Rust had the stub silently swallow the CS, so documents invoking it
  // got no feedback. Switch to DefPrimitive that emits the same info
  // warning as Perl — helps authors understand why overlay-style negation
  // isn't working.
  DefPrimitive!("\\changenotsign", sub[_args] {
    Info!("unexpected", "\\changenotsign",
      "The \\changenotsign operation of mathabx is not implemented.");
    Ok(Vec::new())
  });
  // \cdotp

  //======================================================================
  // Usual binary operators (matha)
  //   +, -
  //   \times, \div
  //   \cdot, \circ
  //   *, \ast
  DefMath!("\\asterisk", "\u{2217}", role => "MULOP");
  // DefMath('\coasterisk',Tokens());
  DefMath!("\\ltimes", "\u{22C9}", role => "MULOP", meaning => "left-normal-factor-semidirect-product");
  DefMath!("\\rtimes", "\u{22CA}", role => "MULOP", meaning => "right-normal-factor-semidirect-product");
  //   \diamond, \bullet
  //   \star
  DefMath!("\\varstar", None, "\u{2736}", role => "MULOP");
  // Next two probably text style or small size?
  DefMath!("\\ssum",  None, "\u{2211}", role => "SUMOP", meaning => "sum");
  DefMath!("\\sprod", None, "\u{220F}", role => "SUMOP", meaning => "product");
  //   \amalg

  //======================================================================
  // Unusual binary operators (mathb)
  DefMath!("\\dotplus",  "\u{2214}", role => "ADDOP");
  DefMath!("\\dotdiv",   "\u{2238}", role => "MULOP");
  DefMath!("\\dottimes", "\u{2A30}", role => "MULOP");
  DefMath!("\\divdot",   "\u{2A2A}", role => "MULOP");
  DefMath!("\\udot",     "\u{22C5}", role => "MULOP");    // Same as \cdot, but should shift to left
  DefMath!("\\square",   "\u{25A1}", role => "MULOP");
  DefMath!("\\Asterisk", "\u{273D}", role => "MULOP");
  DefMath!("\\bigast",   "\u{273D}", role => "MULOP");
  // DefMath('\coAsterisk',Tokens());
  // DefMath('\bigcoast',Tokens());
  DefMath!("\\circplus",      "\u{2A22}", role => "MULOP");
  DefMath!("\\pluscirc",      "\u{2295}", role => "MULOP");    // Not quite right glyph
  DefMath!("\\convolution",   "\u{2733}", role => "MULOP");
  DefMath!("\\divideontimes", "\u{22C7}", role => "MULOP");
  DefMath!("\\blackdiamond",  "\u{25C6}", role => "MULOP");
  DefMath!("\\sqbullet",      "\u{2BC0}", role => "MULOP");
  DefMath!("\\bigstar",       "\u{1F7CA}", role => "MULOP");
  DefMath!("\\bigvarstar",    "\u{1F7CC}", role => "MULOP");

  //======================================================================
  // Usual relations (matha)
  //   =, \equiv
  //   \sim, \approx
  //   \simeq, \cong
  //   \asymp
  DefMath!("\\divides", "\u{2223}", role => "RELOP");
  //   \neq, \ne,
  DefMath!("\\nequiv", "\u{2262}", meaning => "not-equivalent-to", role => "RELOP");
  Let!("\\notequiv", "\\nequiv");
  DefMath!("\\nsim", "\u{2241}", role => "RELOP", meaning => "not-similar-to");
  DefMath!("\\napprox", "\u{2249}", meaning => "not-approximately-equals", role => "RELOP");
  DefMath!("\\nsimeq", "\u{2243}\u{0338}", role => "RELOP",
    meaning => "not-equivalent-to-nor-equals");
  DefMath!("\\ncong", "\u{2247}", role => "RELOP",
    meaning => "not-approximately-equals");
  DefMath!("\\notasymp",   "\u{226D}", meaning => "not-equivalent-to",   role => "RELOP");
  DefMath!("\\notdivides", "\u{2224}", role    => "RELOP", meaning => "does-not-divide");

  //======================================================================
  // Unusual relations (mathb)
  DefMath!("\\topdoteq", "=\u{0307}",  role => "RELOP");    // = combining dot
  DefMath!("\\botdoteq", "\u{2A66}",   role => "RELOP");
  DefMath!("\\doteqdot", "\u{2251}",   role => "RELOP", meaning => "geometrically-equals");
  Let!("\\dotseq", "\\doteqdot");
  Let!("\\Doteq",  "\\doteqdot");
  DefMath!("\\risingdotseq",  "\u{2253}", role => "RELOP", meaning => "image-of-or-approximately-equals");
  DefMath!("\\fallingdotseq", "\u{2252}", role => "RELOP", meaning => "approximately-equals-or-image-of");
  DefMath!("\\coloneq",       "\u{2254}", role => "RELOP");
  DefMath!("\\eqcolon",       "\u{2255}", role => "RELOP");
  DefMath!("\\bumpedeq",      "\u{224F}", role => "RELOP", meaning => "difference-between");
  // DefMath('\eqbumped',Tokens());
  DefMath!("\\Bumpedeq",    "\u{224E}", role => "RELOP", meaning => "geometrically-equals");
  DefMath!("\\circeq",      "\u{2257}", role => "RELOP");
  DefMath!("\\eqcirc",      "\u{2256}", role => "RELOP");
  DefMath!("\\triangleq",   "\u{225C}", role => "RELOP");
  DefMath!("\\corresponds", "\u{2258}", role => "RELOP", meaning => "corresponds-to");

  //======================================================================
  // Miscellaneous (matha)
  //   \neq, \lnot, \ll
  //   \gg,
  DefMath!("\\hash", "#", role => "RELOP");
  //  \vdash, \dashv
  DefMath!("\\nvdash", "\u{22AC}",         role => "RELOP");
  DefMath!("\\ndashv", "\u{22A3}\u{0338}", role => "RELOP");
  DefMath!("\\vDash",  "\u{22A8}",         role => "RELOP");
  DefMath!("\\Dashv",  "\u{2AE4}",         role => "RELOP");
  DefMath!("\\nvDash", "\u{22AD}",         role => "RELOP");
  DefMath!("\\nDashv", "\u{2AE4}\u{0338}", role => "RELOP");
  DefMath!("\\Vdash",  "\u{22A9}",         role => "RELOP", meaning => "forces");
  DefMath!("\\dashV",  "\u{2AE3}",         role => "RELOP");
  DefMath!("\\nVdash", "\u{22AE}",         role => "RELOP", meaning => "not-forces");
  DefMath!("\\ndashV", "\u{2AE3}\u{0338}", role => "RELOP");
  DefMath!("\\degree", "\u{00B0}",         role => "RELOP");
  //   \prime
  DefMath!("\\second", "\u{02BA}", role => "RELOP");
  DefMath!("\\third",  "\u{2034}", role => "RELOP");
  DefMath!("\\fourth", "\u{2057}", role => "RELOP");
  //   \flat
  //   \natural, \sharp
  //   \infty, \propto
  //   \dagger, \ddagger

  //======================================================================
  // Miscellaneous (mathb)
  DefMath!("\\between", "\u{226C}", role => "RELOP", meaning => "between");
  //   \smile
  //   \frown
  DefMath!("\\varhash",         "#",        role => "RELOP");
  DefMath!("\\leftthreetimes",  "\u{22CB}", role => "MULOP", meaning => "left-semidirect-product");
  DefMath!("\\rightthreetimes", "\u{22CC}", role => "MULOP", meaning => "right-semidirect-product");
  DefMath!("\\pitchfork",       "\u{22D4}", role => "RELOP", meaning => "proper-intersection");
  //  \bowtie, \Join
  DefMath!("\\VDash",  "\u{22AB}",         role => "RELOP");
  DefMath!("\\DashV",  "\u{2AE5}",         role => "RELOP");
  DefMath!("\\nVDash", "\u{22AF}",         role => "RELOP");
  DefMath!("\\nDashV", "\u{2AE5}\u{0338}", role => "RELOP");
  DefMath!("\\Vvdash", "\u{22AA}",         role => "RELOP");
  // Note that the above can be mirrored, but that doesn't quite help \dashVv!
  DefMath!("\\nVvash", "\u{22AA}\u{0338}", role => "RELOP");
  // DefMath('\ndashVv',Tokens());
  DefMath!("\\therefore", "\u{2234}", role => "METARELOP", meaning => "therefore");
  DefMath!("\\because",   "\u{2235}", role => "METARELOP", meaning => "because");
  DefMath!("\\ring{}", "\u{030A}", operator_role => "OVERACCENT");
  //   \dot
  //   \ddot,
  DefMath!("\\dddot{}",  "\u{02D9}\u{02D9}\u{02D9}",         operator_role => "OVERACCENT");
  DefMath!("\\ddddot{}", "\u{02D9}\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");
  //   \angle
  DefMath!("\\measuredangle",  "\u{2221}");
  DefMath!("\\sphericalangle", "\u{2222}");
  DefMath!("\\rip",            "\u{26FC}");    // Not quite the right glyph

  //======================================================================
  // Delimiters as symbols (matha)
  // (,)
  // [,]
  // \setminus, /
  // |, \mid

  //======================================================================
  // Delimiters as symbols (mathb)
  DefMath!("\\ulcorner", "\u{231C}");
  DefMath!("\\urcorner", "\u{231D}");
  DefMath!("\\llcorner", "\u{231E}");
  DefMath!("\\lrcorner", "\u{231F}");

  //======================================================================
  // Astronomical Symbols (mathbb)
  DefPrimitive!("\\Sun",         "\u{2609}");
  DefPrimitive!("\\Mercury",     "\u{263F}");
  DefPrimitive!("\\Venus",       "\u{2640}");
  DefPrimitive!("\\Earth",       "\u{2641}");    // wants circled + ???
  DefPrimitive!("\\Mars",        "\u{2642}");
  DefPrimitive!("\\Jupiter",     "\u{2643}");
  DefPrimitive!("\\Saturn",      "\u{2644}");
  DefPrimitive!("\\Uranus",      "\u{2645}");
  DefPrimitive!("\\Neptune",     "\u{2646}");
  DefPrimitive!("\\Pluto",       "\u{2647}");
  DefPrimitive!("\\varEarth",    "\u{2641}");
  DefPrimitive!("\\leftmoon",    "\u{263E}");
  DefPrimitive!("\\rightmoon",   "\u{263D}");
  DefPrimitive!("\\fullmoon",    "\u{25CB}");    // actually just white circle
  DefPrimitive!("\\newmoon",     "\u{25CF}");    // actually just black circle
  DefPrimitive!("\\Aries",       "\u{2648}");
  DefPrimitive!("\\Taurus",      "\u{2649}");
  DefPrimitive!("\\Gemini",      "\u{264A}");
  DefPrimitive!("\\Cancer",      "\u{264B}");
  DefPrimitive!("\\Leo",         "\u{264C}");
  DefPrimitive!("\\Virgo",       "\u{264D}");
  DefPrimitive!("\\Libra",       "\u{264E}");
  DefPrimitive!("\\Scorpio",     "\u{264F}");
  DefPrimitive!("\\Sagittarius", "\u{2650}");
  DefPrimitive!("\\Capricorn",   "\u{2651}");
  DefPrimitive!("\\Aquarius",    "\u{2652}");
  DefPrimitive!("\\Pisces",      "\u{2653}");

  //======================================================================
  // Letter-like symbols (matha)
  //  \forall,
  DefMath!("\\complement", "\u{2201}", meaning => "complement");
  //  \partial
  DefMath!("\\partialslash", "\u{2202}\u{0338}", role => "OPERATOR");
  //  \exists,
  DefMath!("\\nexists", "\u{2204}", role => "FUNCTION", meaning => "not-exists");
  DefMath!("\\Finv",    "\u{2132}");
  DefMath!("\\Game",    "\u{2141}");
  //   \emptyset,
  DefMath!("\\diameter", "\u{2300}");
  //   \top, \bot
  //   \perp,
  DefMath!("\\nottop",     "\u{22A4}\u{0338}", role => "ADDOP", meaning => "not-top");
  DefMath!("\\notbot",     "\u{22A5}\u{0338}", role => "ADDOP", meaning => "not-bottom");
  DefMath!("\\notperp",    "\u{27C2}\u{0338}", role => "RELOP", meaning => "not-perpendicular-to");
  DefMath!("\\curlywedge", "\u{22CF}",         role => "ADDOP", meaning => "and");
  DefMath!("\\curlyvee",   "\u{22CE}",         role => "ADDOP", meaning => "or");
  //   \in, \owns
  //   \notin
  DefMath!("\\notowner", "\u{220C}", meaning => "not-contains", role => "RELOP");
  Let!("\\notni",       "\\notowner");
  Let!("\\notowns",     "\\notowner");
  Let!("\\varnotin",    "\\notin");
  Let!("\\varnotowner", "\\notowner");
  DefMath!("\\barin",   "\u{22F6}", role => "ADDOP", meaning => "element-of-with-overbar");
  DefMath!("\\ownsbar", "\u{22F8}", role => "ADDOP", meaning => "element-of-with-underbar");
  //  \cap, \cup
  //  \uplus, \sqcap
  //  \sqcup, \squplus
  //  \wedge, \and, \vee, \lor

  //======================================================================
  // Letter-like symbols (mathb)
  DefMath!("\\barwedge",       "\u{22BC}", role => "ADDOP", meaning => "not-and");
  DefMath!("\\veebar",         "\u{22BB}", role => "ADDOP", meaning => "exclusive-or");
  DefMath!("\\doublebarwedge", "\u{2A5E}", role => "ADDOP");
  DefMath!("\\veedoublebar",   "\u{2A63}", role => "ADDOP");
  DefMath!("\\doublecap",      "\u{22D2}", role => "ADDOP", meaning => "double-intersection");
  DefMath!("\\doublecup",      "\u{22D3}", role => "ADDOP", meaning => "double-union");
  DefMath!("\\sqdoublecap",    "\u{2A4E}", role => "ADDOP", meaning => "double-square-intersection");
  DefMath!("\\sqdoublecup",    "\u{2A4F}", role => "ADDOP", meaning => "double-square-union");

  //======================================================================
  //  Subset's and superset's signs (matha)
  //  \subset, \supset
  DefMath!("\\nsubset", "\u{2284}", meaning => "not-subset-of",   role => "RELOP");
  DefMath!("\\nsupset", "\u{2285}", meaning => "not-superset-of", role => "RELOP");
  //  \subseteq, \supseteq
  DefMath!("\\nsubseteq",    "\u{2288}", role => "RELOP", meaning => "not-subset-of-nor-equals");
  DefMath!("\\nsupseteq",    "\u{2289}", role => "RELOP", meaning => "not-superset-of-nor-equals");
  DefMath!("\\subsetneq",    "\u{228A}", role => "RELOP", meaning => "subset-of-and-not-equals");
  DefMath!("\\supsetneq",    "\u{228B}", role => "RELOP", meaning => "superset-of-and-not-equals");
  DefMath!("\\varsubsetneq", "\u{228A}", role => "RELOP", meaning => "subset-of-and-not-equals");
  DefMath!("\\varsupsetneq", "\u{228B}", role => "RELOP", meaning => "subset-of-and-not-equals");
  DefMath!("\\subseteqq",    "\u{2AC5}", role => "RELOP", meaning => "subset-of-or-equals");
  DefMath!("\\supseteqq",    "\u{2AC6}", role => "RELOP", meaning => "superset-of-or-equals");
  DefMath!("\\nsubseteqq", "\u{2AC5}\u{0338}", role => "RELOP", meaning => "not-subset-of-nor-equals");
  DefMath!("\\nsupseteqq", "\u{2AC6}\u{0338}", role => "RELOP", meaning => "not-superset-of-nor-equals");
  DefMath!("\\subsetneqq",    "\u{2ACB}", role => "RELOP", meaning => "subset-of-and-not-equals");
  DefMath!("\\supsetneqq",    "\u{2ACC}", role => "RELOP", meaning => "superset-of-and-not-equals");
  DefMath!("\\varsubsetneqq", "\u{2ACB}", role => "RELOP", meaning => "subset-of-and-not-equals");
  DefMath!("\\varsupsetneqq", "\u{2ACC}", role => "RELOP", meaning => "superset-of-and-not-equals");
  DefMath!("\\Subset",        "\u{22D0}", role => "RELOP", meaning => "double-subset-of");
  DefMath!("\\Supset",        "\u{22D1}", role => "RELOP", meaning => "double-superset-of");
  DefMath!("\\nSubset",  "\u{22D0}\u{0338}", role => "RELOP", meaning => "not-double-subset-of");
  DefMath!("\\nSupset",  "\u{22D1}\u{0338}", role => "RELOP", meaning => "not-double-superset-of");

  //======================================================================
  // Square Subset's and superset's signs (mathb)
  //  \sqsubset, \sqsupset
  DefMath!("\\nsqsubset", "\u{228F}\u{0338}", role => "RELOP", meaning => "not-square-image-of");
  DefMath!("\\nsqsupset", "\u{2290}\u{0338}", role => "RELOP", meaning => "not-square-original-of");
  //  \sqsubseteq, \sqsupseteq
  DefMath!("\\nsqsubseteq", "\u{22E2}", role => "RELOP", meaning => "not-square-image-of-nor-equals");
  DefMath!("\\nsqsupseteq", "\u{22E3}", role => "RELOP", meaning => "not-square-original-of-nor-equals");
  DefMath!("\\sqsubsetneq", "\u{22E4}", role => "RELOP", meaning => "square-image-of-or-not-equals");
  DefMath!("\\sqsupsetneq", "\u{22E5}", role => "RELOP", meaning => "square-original-of-or-not-equals");
  Let!("\\varsqsubsetneq", "\\sqsubsetneq");
  Let!("\\varsqsupsetneq", "\\sqsupsetneq");
  // Pretty crummy, using underline
  DefMath!("\\sqsubseteqq",  "\u{228F}\u{0333}", role => "RELOP", meaning => "square-image-of-or-equals");
  DefMath!("\\sqsupseteqq",  "\u{2290}\u{0333}", role => "RELOP", meaning => "square-original-of-or-equals");
  DefMath!("\\nsqsubseteqq", "\u{228F}\u{0333}\u{0338}", role => "RELOP", meaning => "not-square-image-of-nor-equals");
  DefMath!("\\nsqsupseteqq", "\u{2290}\u{0333}\u{0338}", role => "RELOP", meaning => "not-square-original-of-nor-equals");

  //======================================================================
  // Triangles as relations (matha)
  //  \triangleleft,
  DefMath!("\\vartriangleleft",  "\u{22B2}");    // NORMAL SUBGROUP OF (\lhd)
  // \triangleright
  DefMath!("\\vartriangleright", "\u{22B3}");    // CONTAINS AS NORMAL SUBGROUP (\rhd)
  DefMath!("\\ntriangleleft",    "\u{22EA}", role => "RELOP", meaning => "not-subgroup-of");
  DefMath!("\\ntriangleright",   "\u{22EB}", role => "RELOP", meaning => "not-contains");
  DefMath!("\\trianglelefteq",   "\u{22B4}");    // NORMAL SUBGROUP OF OR EQUAL TO (\unlhd)
  DefMath!("\\trianglerighteq",  "\u{22B5}");    // CONTAINS AS NORMAL SUBGROUP OR EQUAL TO (\unrhd)
  DefMath!("\\ntrianglelefteq",  "\u{22EC}", role => "RELOP", meaning => "not-subgroup-of-nor-equals");
  DefMath!("\\ntrianglerighteq", "\u{22ED}", role => "RELOP", meaning => "not-contains-nor-equals");

  //======================================================================
  // Triangles as binary operators (mathb)
  DefMath!("\\smalltriangleup",    "\u{25B5}", role => "RELOP");
  DefMath!("\\smalltriangledown",  "\u{25BF}", role => "RELOP");
  DefMath!("\\smalltriangleleft",  "\u{25C3}", role => "RELOP");
  DefMath!("\\smalltriangleright", "\u{25B9}", role => "RELOP");
  DefMath!("\\blacktriangleup",    "\u{25B4}", role => "RELOP");
  DefMath!("\\blacktriangledown",  "\u{25BE}", role => "RELOP");
  DefMath!("\\blacktriangleleft",  "\u{25C2}", role => "RELOP");
  DefMath!("\\blacktriangleright", "\u{25B8}", role => "RELOP");

  //======================================================================
  // Inequalities (matha)
  //  <, >
  DefMath!("\\nless", "\u{226E}", role => "RELOP", meaning => "not-less-than");
  DefMath!("\\ngtr",  "\u{226F}", role => "RELOP", meaning => "not-greater-than");
  //   \leq, \geq (\leqslant, \qeqslant)
  DefMath!("\\nleq", "\u{2270}", role => "RELOP", meaning => "not-less-than-nor-greater-than");
  DefMath!("\\ngeq", "\u{2271}", role => "RELOP", meaning => "not-greater-than-nor-equals");
  Let!("\\varleq",  "\\leq");
  Let!("\\vargeq",  "\\geq");
  Let!("\\nvarleq", "\\nleq");
  Let!("\\nvargeq", "\\ngeq");
  DefMath!("\\lneq",  "\u{2A87}",         role => "RELOP", meaning => "less-than-and-not-equals");
  DefMath!("\\gneq",  "\u{2A88}",         role => "RELOP", meaning => "greater-than-and-not-equals");
  DefMath!("\\leqq",  "\u{2266}",         role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\geqq",  "\u{2267}",         role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\nleqq", "\u{2266}\u{0338}", role => "RELOP", meaning => "not-less-than-nor-equals");
  DefMath!("\\ngeqq", "\u{2267}\u{0338}", role => "RELOP", meaning => "not-greater-than-nor-equals");
  DefMath!("\\lneqq", "\u{2268}",         role => "RELOP", meaning => "less-than-and-not-equals");
  DefMath!("\\gneqq", "\u{2269}",         role => "RELOP", meaning => "greater-than-and-not-equals");
  DefMath!("\\lvertneqq",    "\u{2268}",  role => "RELOP", meaning => "less-than-and-not-equals");
  DefMath!("\\gvertneqq",    "\u{2269}",  role => "RELOP", meaning => "greater-than-and-not-equals");
  DefMath!("\\eqslantless",  "\u{2A95}",  role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\eqslantgtr",   "\u{2A96}",  role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\neqslantless", "\u{2A95}\u{0338}", role => "RELOP", meaning => "not-less-than-nor-equals");
  DefMath!("\\neqslantgtr",  "\u{2A96}\u{0338}", role => "RELOP", meaning => "not-greater-than-nor-equals");
  DefMath!("\\lessgtr",     "\u{2276}", role => "RELOP", meaning => "less-than-or-greater-than");
  DefMath!("\\gtrless",     "\u{2277}", role => "RELOP", meaning => "greater-than-or-less-than");
  DefMath!("\\lesseqgtr",   "\u{22DA}", role => "RELOP", meaning => "less-than-or-equals-or-greater-than");
  DefMath!("\\gtreqless",   "\u{22DB}", role => "RELOP", meaning => "greater-than-or-equals-or-less-than");
  DefMath!("\\lesseqqgtr",  "\u{2A8B}", role => "RELOP", meaning => "less-than-or-equals-or-greater-than");
  DefMath!("\\gtreqqless",  "\u{2A8C}", role => "RELOP", meaning => "greater-than-or-equals-or-less-than");
  DefMath!("\\lesssim",     "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  DefMath!("\\gtrsim",      "\u{2273}", role => "RELOP", meaning => "greater-than-or-equivalent-to");
  DefMath!("\\nlesssim",  "\u{2272}\u{0338}", role => "RELOP", meaning => "not-less-than-nor-similar-to");
  DefMath!("\\ngtrsim",   "\u{2273}\u{0338}", role => "RELOP", meaning => "not-greater-than-nor-equivalent-to");
  DefMath!("\\lnsim",     "\u{22E6}", role => "RELOP", meaning => "less-than-and-not-equivalent-to");
  DefMath!("\\gnsim",     "\u{22E7}", role => "RELOP", meaning => "greater-than-and-not-equivalent-to");
  DefMath!("\\lessapprox", "\u{2A85}", role => "RELOP", meaning => "less-than-or-approximately-equals");
  DefMath!("\\gtrapprox",  "\u{2A86}", role => "RELOP", meaning => "greater-than-or-approximately-equals");
  DefMath!("\\nlessapprox", "\u{2A85}\u{0338}", role => "RELOP", meaning => "not-less-than-nor-approximately-equals");
  DefMath!("\\ngtrapprox",  "\u{2A86}\u{0338}", role => "RELOP", meaning => "not-greater-than-nor-approximately-equals");
  DefMath!("\\lnapprox", "\u{2A89}", role => "RELOP", meaning => "less-than-and-not-approximately-equals");
  DefMath!("\\gnapprox", "\u{2A8A}", role => "RELOP", meaning => "greater-than-and-not-approximately-equals");
  DefMath!("\\lessdot", "\u{22D6}", role => "RELOP");
  DefMath!("\\gtrdot",  "\u{22D7}", role => "RELOP");
  DefMath!("\\lll",     "\u{22D8}", role => "RELOP", meaning => "very-much-less-than");
  DefMath!("\\ggg",     "\u{22D9}", role => "RELOP", meaning => "very-much-greater-than");
  DefMath!("\\precdot", "\u{22D6}", role => "RELOP");    // glyph is for less with dot!
  DefMath!("\\succdot", "\u{22D7}", role => "RELOP");    // gtr with dot!

  //======================================================================
  // Inequalities (mathb)
  // Sometimes using \x{0338} to negate (which is slash, but should use vertical?)
  //  \prec, \succ
  DefMath!("\\nprec",        "\u{2280}",         role => "RELOP", meaning => "not-precedes");
  DefMath!("\\nsucc",        "\u{2281}",         role => "RELOP", meaning => "not-succeeds");
  DefMath!("\\preccurlyeq",  "\u{227C}",         role => "RELOP", meaning => "precedes-or-equals");
  DefMath!("\\succcurlyeq",  "\u{227D}",         role => "RELOP", meaning => "succeeds-or-equals");
  DefMath!("\\npreccurlyeq", "\u{227C}\u{0338}", role => "RELOP", meaning => "not-precedes-nor-equals");
  DefMath!("\\nsucccurlyeq", "\u{227D}\u{0338}", role => "RELOP", meaning => "not-succeeds-nor-equals");
  //  \preceq, succeq
  DefMath!("\\npreceq",      "\u{22E0}",         role => "RELOP", meaning => "not-precedes-nor-equals");
  DefMath!("\\nsucceq",      "\u{22E1}",         role => "RELOP", meaning => "not-succeeds-nor-equals");
  DefMath!("\\precneq",      "\u{22E8}",         role => "RELOP", meaning => "precedes-not-equals");
  DefMath!("\\succneq",      "\u{22E9}",         role => "RELOP", meaning => "succeeds-not-equals");
  DefMath!("\\curlyeqprec",  "\u{22DE}",         role => "RELOP", meaning => "equals-or-preceeds");
  DefMath!("\\curlyeqsucc",  "\u{22DF}",         role => "RELOP", meaning => "equals-or-succeeds");
  DefMath!("\\ncurlyeqprec", "\u{22DE}\u{0338}", role => "RELOP", meaning => "not-equals-nor-preceeds");
  DefMath!("\\ncurlyeqsucc", "\u{22DF}\u{0338}", role => "RELOP", meaning => "not-equals-nor-succeeds");
  DefMath!("\\precsim",      "\u{227E}",         role => "RELOP", meaning => "precedes-or-equivalent-to");
  DefMath!("\\succsim",      "\u{227F}",         role => "RELOP", meaning => "succeeds-or-equivalent-to");
  DefMath!("\\nprecsim", "\u{227E}\u{0338}", role => "RELOP", meaning => "not-precedes-nor-equivalent-to");
  DefMath!("\\nsuccsim", "\u{227F}\u{0338}", role => "RELOP", meaning => "not-succeeds-nor-equivalent-to");
  DefMath!("\\precnsim",   "\u{22E8}", role => "RELOP", meaning => "precedes-and-not-equivalent-to");
  DefMath!("\\succnsim",   "\u{22E9}", role => "RELOP", meaning => "succeeds-and-not-equivalent-to");
  DefMath!("\\precapprox", "\u{2AB7}", role => "RELOP", meaning => "precedes-or-approximately-equals");
  DefMath!("\\succapprox", "\u{2AB8}", role => "RELOP", meaning => "succeeds-or-approximately-equals");
  DefMath!("\\nprecapprox", "\u{2AB7}\u{0338}", meaning => "not-precedes-nor-approximately-equals", role => "RELOP");
  DefMath!("\\nsuccapprox", "\u{2AB8}\u{0338}", role => "RELOP", meaning => "not-succeeds-nor-approximately-equals");
  DefMath!("\\precnapprox", "\u{2AB9}", role => "RELOP", meaning => "precedes-and-not-approximately-equals");
  DefMath!("\\succnapprox", "\u{2ABA}", role => "RELOP", meaning => "succeeds-and-not-approximately-equals");
  DefMath!("\\llcurly", "\u{2ABB}", role => "RELOP", meaning => "double-precedes");
  DefMath!("\\ggcurly", "\u{2ABC}", role => "RELOP", meaning => "double-succeeds");

  //======================================================================
  // Arrows and Harpoons (matha)
  //  \leftarrow, \gets \rightarrow, \to
  //  \nwarrow, \nearrow
  //  \swarrow, \searrow
  //  \leftrightarrow
  DefMath!("\\nleftarrow",      "\u{219A}", role => "ARROW");
  DefMath!("\\nrightarrow",     "\u{219B}", role => "ARROW");
  DefMath!("\\nleftrightarrow", "\u{21AE}", role => "ARROW");    // LEFT RIGHT ARROW WITH STROKE
  //  \relbar
  //  \mapstochar
  DefMath!("\\mapsfromchar", "|", role => "RELOP");
  //  \leftharpoonup
  //  \rightharpoonup, \leftharpoondown
  //  \rightharpoondown,
  DefMath!("\\upharpoonleft",     "\u{21BF}", role => "ARROW");
  DefMath!("\\downharpoonleft",   "\u{21C3}", role => "ARROW");
  DefMath!("\\upharpoonright",    "\u{21BE}", role => "ARROW");
  DefMath!("\\restriction",       "\u{21BE}", role => "ARROW");
  DefMath!("\\downharpoonright",  "\u{21C2}", role => "ARROW");
  DefMath!("\\leftrightharpoons", "\u{21CB}", role => "ARROW");
  //  \rightleftharpoons
  DefMath!("\\updownharpoons", "\u{296E}", role => "ARROW");
  DefMath!("\\downupharpoons", "\u{296F}", role => "ARROW");
  //  \Leftarrow, \Rightarrow
  //  \Leftrightarrow,
  DefMath!("\\nLeftarrow",      "\u{21CD}", role => "ARROW");
  DefMath!("\\nRightarrow",     "\u{21CF}", role => "ARROW");
  DefMath!("\\nLeftrightarrow", "\u{21CE}", role => "ARROW");
  //  \Relbar
  DefMath!("\\Mapstochar",   "|", role => "RELOP");
  DefMath!("\\Mapsfromchar", "|", role => "RELOP");

  //======================================================================
  // Arrows and Harpoons (mathb)
  DefMath!("\\leftleftarrows",     "\u{21C7}", role => "ARROW");
  DefMath!("\\rightrightarrows",   "\u{21C9}", role => "ARROW");
  DefMath!("\\upuparrows",         "\u{21C8}", role => "ARROW");
  DefMath!("\\downdownarrows",     "\u{21CA}", role => "ARROW");
  DefMath!("\\leftrightarrows",    "\u{21C6}", role => "ARROW");
  DefMath!("\\rightleftarrows",    "\u{21C4}", role => "ARROW");
  DefMath!("\\updownarrows",       "\u{21C5}", role => "ARROW");
  DefMath!("\\downuparrows",       "\u{21F5}", role => "ARROW");
  DefMath!("\\leftleftharpoons",   "\u{2962}", role => "ARROW");
  DefMath!("\\rightrightharpoons", "\u{2964}", role => "ARROW");
  DefMath!("\\upupharpoons",       "\u{2963}", role => "ARROW");
  DefMath!("\\downdownharpoons",   "\u{2965}", role => "ARROW");
  DefMath!("\\leftbarharpoon",     "\u{296A}", role => "ARROW");
  DefMath!("\\rightbarharpoon",    "\u{296C}", role => "ARROW");
  DefMath!("\\barleftharpoon",     "\u{296B}", role => "ARROW");
  DefMath!("\\barrightharpoon",    "\u{296D}", role => "ARROW");
  DefMath!("\\leftrightharpoon",   "\u{294A}", role => "ARROW");
  DefMath!("\\rightleftharpoon",   "\u{294B}", role => "ARROW");
  //  \rhook, \lhook
  DefMath!("\\diagup",         "\u{2571}");
  DefMath!("\\diagdown",       "\u{2572}");
  DefMath!("\\Lsh",            "\u{21B0}", role => "ARROW");
  DefMath!("\\Rsh",            "\u{21B1}", role => "ARROW");
  DefMath!("\\dlsh",           "\u{21B2}", role => "ARROW");
  DefMath!("\\drsh",           "\u{21B3}", role => "ARROW");
  DefMath!("\\looparrowleft",  "\u{21AB}", role => "ARROW");
  DefMath!("\\looparrowright", "\u{21AC}", role => "ARROW");
  DefMath!("\\curvearrowleft",  "\u{21B6}", role => "ARROW");
  DefMath!("\\curvearrowright", "\u{21B7}", role => "ARROW");
  DefMath!("\\curvearrowbotright", "\u{293B}", role => "ARROW");
  DefMath!("\\circlearrowleft",     "\u{21BA}", role => "ARROW");
  DefMath!("\\circlearrowright",    "\u{21BB}", role => "ARROW");
  DefMath!("\\leftsquigarrow",      "\u{21DC}", role => "RELOP");
  DefMath!("\\rightsquigarrow",     "\u{219D}", role => "ARROW");
  DefMath!("\\leftrightsquigarrow", "\u{21AD}", role => "ARROW");
  DefMath!("\\lefttorightarrow",    "\u{2B8E}", role => "ARROW");
  DefMath!("\\righttoleftarrow",    "\u{2B8C}", role => "ARROW");
  DefMath!("\\uptodownarrow",       "\u{2B8F}", role => "ARROW");
  DefMath!("\\downtouparrow",       "\u{2B8D}", role => "ARROW");

  //======================================================================
  // Circles (matha)
  //   Using combining circle \x{20DD} for missing cases, but positioning is bad
  //  \oplus, \ominus (\circleddash)
  //  \otimes
  DefMath!("\\odiv", "\u{00F7}\u{20DD}", role => "ADDOP");
  //  \odot
  DefMath!("\\ocirc",      "\u{229A}");
  DefMath!("\\oasterisk",  "\u{229B}", role => "MULOP");
  // DefMath('\ocoasterisk',Tokens());
  DefMath!("\\oleft",  "\u{22A3}\u{20DD}", role => "ADDOP");
  DefMath!("\\oright", "\u{22A2}\u{20DD}", role => "ADDOP");
  DefMath!("\\otop",   "\u{22A4}\u{20DD}", role => "ADDOP");
  DefMath!("\\obot",   "\u{29BA}");
  DefMath!("\\ovoid",  "\u{25CB}");
  //  \oslash
  DefMath!("\\obackslash",  "\u{29B8}");
  DefMath!("\\otriangleup", "\u{25B3}\u{20DD}", role => "ADDOP");

  //======================================================================
  // Boxes (mathb)
  //   Using combining square \x{20DE} for missing cases, but positioning is bad
  DefMath!("\\boxplus",     "\u{229E}",             role => "ADDOP");
  DefMath!("\\boxminus",    "\u{229F}",             role => "ADDOP");
  DefMath!("\\boxtimes",    "\u{22A0}",             role => "MULOP");
  DefMath!("\\boxdiv",      "\u{00F7}\u{20DE}",     role => "ADDOP");
  DefMath!("\\boxdot",      "\u{22A1}",             role => "MULOP");
  DefMath!("\\boxcirc",     "\u{2218}\u{20DE}",     role => "ADDOP");
  DefMath!("\\boxasterisk", "\u{29C6}");
  // DefMath('\boxcoasterisk',Tokens());
  DefMath!("\\boxleft",  "\u{22A3}\u{20DE}", role => "ADDOP");
  DefMath!("\\boxright", "\u{22A2}\u{20DE}", role => "ADDOP");
  DefMath!("\\boxtop",   "\u{22A4}\u{20DE}", role => "ADDOP");
  DefMath!("\\boxbot",   "\u{22A5}\u{20DE}", role => "ADDOP");
  DefMath!("\\boxvoid",  "\u{25A1}");
  //  \Box
  DefMath!("\\boxslash",      "\u{29C5}");
  DefMath!("\\boxbackslash",  "\u{29C4}");
  DefMath!("\\boxtriangleup", "\u{25B3}\u{20DE}", role => "ADDOP");

  //======================================================================
  // Mayan numerals

  //======================================================================
  // Large operators (mathx)
  //  \sum, \prod
  //  \coprod, \intop
  DefMath!("\\iintop", "\u{222C}", meaning => "double-integral", role => "INTOP",
    dynamic_mathstyle => true);
  DefMath!("\\iiintop", "\u{222D}", meaning => "triple-integral", role => "INTOP",
    dynamic_mathstyle => true);
  //  \ointop, \oint
  DefMath!("\\oiintop", "\u{222F}", meaning => "double-contour-integral", role => "INTOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\bigplus", "+",
    font => { scale => 1.2 },
    meaning => "nary-plus", role => "BIGOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\bigtimes", "\u{2A09}",
    meaning => "nary-times", role => "BIGOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  DefMath!("\\bigcomplementop", "\u{2201}",
    meaning => "nary-complement", role => "BIGOP",
    scriptpos => "mid", dynamic_mathstyle => true);
  //  \bigcap
  //  \bigcup, \biguplus
  DefMath!("\\bigsqcap", None, "\u{2A05}",
    role => "SUMOP",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  //  \bigsqcup
  //  \bigwedge
  //  \bigvee
  DefMath!("\\bigcurlywedge", None, "\u{22CF}",
    font => { scale => 1.6 },
    role => "SUMOP",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigcurlyvee", None, "\u{22CE}",
    font => { scale => 1.6 },
    role => "SUMOP",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  //======================================================================
  // Big circles (mathx)
  //  \bigoplus
  //  \bigotimes
  DefMath!("\\bigominus", "\u{2296}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigodiv", "\u{00F7}\u{20DD}", role => "ADDOP",
    font => { scale => 1.2 });
  //  \bigodot
  DefMath!("\\bigocirc", "\u{229A}",
    font => { scale => 1.2 });
  DefMath!("\\bigoasterisk", "\u{229B}", role => "MULOP",
    font => { scale => 1.2 });
  // DefMath('\ocoasterisk',Tokens());
  DefMath!("\\bigoleft", "\u{22A3}\u{20DD}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigoright", "\u{22A2}\u{20DD}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigotop", "\u{22A4}\u{20DD}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigobot", "\u{29BA}",
    font => { scale => 1.2 });
  DefMath!("\\bigovoid", "\u{25CB}",
    font => { scale => 1.2 });
  DefMath!("\\bigoslash", "\u{2298}", role => "MULOP",
    font => { scale => 1.2 });
  DefMath!("\\bigobackslash", "\u{29B8}",
    font => { scale => 1.2 });
  DefMath!("\\bigotriangleup", "\u{25B3}\u{20DD}", role => "ADDOP",
    font => { scale => 1.2 });

  //======================================================================
  // Big boxes (mathx)
  DefMath!("\\bigboxplus", "\u{229E}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxminus", "\u{229F}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxtimes", "\u{22A0}", role => "MULOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxdiv", "\u{00F7}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxdot", "\u{22A1}", role => "MULOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxcirc", "\u{2218}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxasterisk", "\u{29C6}",
    font => { scale => 1.2 });
  // DefMath('\boxcoasterisk',Tokens());
  DefMath!("\\bigboxleft", "\u{22A3}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxright", "\u{22A2}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxtop", "\u{22A4}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxbot", "\u{22A5}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });
  DefMath!("\\bigboxvoid", "\u{25A1}",
    font => { scale => 1.2 });
  //  \Box
  DefMath!("\\bigboxslash", "\u{29C5}",
    font => { scale => 1.2 });
  DefMath!("\\bigboxbackslash", "\u{29C4}",
    font => { scale => 1.2 });
  DefMath!("\\bigboxtriangleup", "\u{25B3}\u{20DE}", role => "ADDOP",
    font => { scale => 1.2 });

  //======================================================================
  // Delimiters (matha/mathx)
  // (,)
  // [,]
  // \lbrace, \{, \rbrace,\}
  DefMath!("\\ldbrack", "\u{27E6}", role => "OPEN",  stretchy => false);
  DefMath!("\\rdbrack", "\u{27E7}", role => "CLOSE", stretchy => false);
  //  \langle, \rangle
  //  \backslash, /
  // \vert, |
  // \Vert
  DefMath!("\\vvvert", "\u{2980}", role => "MID", stretchy => false);
  // \uparrow, \downarrow
  // \updownarrow, \Uparrow
  // \Downarrow, \Updownarrow

  //======================================================================
  // Delimiters (mathb/mathx)
  //  \lgroup, \rgroup
  //  \lceil, \rceil
  //  \lfloor, \rfloor
  DefMath!("\\thickvert", "\u{2759}", role => "MID", stretchy => false);

  //======================================================================
  // Extensible accents (mathx)

  // The way these are defined recognizes Digested style parameter type
  //  \widehat
  DefMath!("\\widecheck Digested", "\u{02C7}", operator_role => "OVERACCENT");
  // \widetilde
  DefMath!("\\widebar Digested",   "\u{00AF}", operator_role => "OVERACCENT");
  DefMath!("\\widearrow Digested", "\u{2192}", operator_role => "OVERACCENT");
  DefMath!("\\wideparen Digested", "\u{23DC}", operator_role => "OVERACCENT");
  DefMath!("\\ring Digested",      "\u{030A}", operator_role => "OVERACCENT");

  // The remaining macros in this group only accept traditional style {} argument
  //  \overbrace, \underbrace
  DefMath!("\\overgroup{}",  "\u{23DC}", operator_role => "OVERACCENT");
  DefMath!("\\undergroup{}", "\u{23DD}", operator_role => "UNDERACCENT");

  //   \overrightarrow, \overleftarrow
  DefMath!("\\overleftrightarrow{}",  "\u{2194}", operator_role => "OVERACCENT");
  DefMath!("\\underrightarrow{}",     "\u{2192}", operator_role => "UNDERACCENT");
  DefMath!("\\underleftarrow{}",      "\u{2190}", operator_role => "UNDERACCENT");
  DefMath!("\\underleftrightarrow{}", "\u{2194}", operator_role => "UNDERACCENT");
  DefMath!("\\overRightarrow{}",      "\u{21D2}", operator_role => "OVERACCENT");
  DefMath!("\\overLeftarrow{}",       "\u{21D0}", operator_role => "OVERACCENT");
  DefMath!("\\overLeftRightarrow{}",  "\u{21D4}", operator_role => "OVERACCENT");
  DefMath!("\\underRightarrow{}",     "\u{21D2}", operator_role => "UNDERACCENT");
  DefMath!("\\underLeftarrow{}",      "\u{21D0}", operator_role => "UNDERACCENT");
  DefMath!("\\underLeftRightarrow{}", "\u{21D4}", operator_role => "UNDERACCENT");
  DefMacro!("\\widering{}",   "\\ring{\\wideparen{#1}}");
  DefMacro!("\\widedot{}",    "\\dot{\\wideparen{#1}}");
  DefMacro!("\\wideddot{}",   "\\ddot{\\wideparen{#1}}");
  DefMacro!("\\widedddot{}",  "\\dddot{\\wideparen{#1}}");
  DefMacro!("\\wideddddot{}", "\\ddddot{\\wideparen{#1}}");
});
