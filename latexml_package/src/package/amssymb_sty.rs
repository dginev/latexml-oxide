use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsfonts");

  //======================================================================
  // Lowercase Greek letters
  DefMath!("\\digamma", "\u{03DD}"); // GREEK SMALL LETTER DIGAMMA
  DefMath!("\\varkappa", "\u{03F0}"); // GREEK KAPPA SYMBOL

  //======================================================================
  // Hebrew
  DefMath!("\\beth", "\u{2136}"); // BET SYMBOL
  DefMath!("\\daleth", "\u{2138}"); // DALET SYMBOL
  DefMath!("\\gimel", "\u{2137}"); // GIMEL SYMBOL

  //======================================================================
  // Miscellaneous
  // \hbar  in LaTeX
  DefMath!("\\hslash", "\u{210F}", role => "ID", meaning => "Planck-constant-over-2-pi");
  DefMath!("\\vartriangle", "\u{25B3}");
  DefMath!("\\triangledown", "\u{25BD}");
  // \square, \lozenge in amsfonts
  DefMath!("\\circledS", "\u{24C8}");
  // \angle in tex
  DefMath!("\\measuredangle", "\u{2221}");
  DefMath!("\\nexists", "\u{2204}", role => "FUNCTION", meaning => "not-exists");
  // \mho in latex
  DefMath!("\\Finv", "\u{2132}");
  DefMath!("\\Game", "\u{2141}");
  DefMath!("\\Bbbk", "\u{1D55C}");
  DefMath!("\\backprime", "\u{2035}");
  DefMath!("\\varnothing",        "\u{2205}", role => "ID", meaning => "empty-set");
  DefMath!("\\blacktriangle", "\u{25B2}");
  DefMath!("\\blacktriangledown", "\u{25BC}");
  DefMath!("\\blacksquare", "\u{25A0}");
  DefMath!("\\blacklozenge", "\u{25C6}");
  DefMath!("\\bigstar", "\u{2605}");
  DefMath!("\\sphericalangle", "\u{2222}");
  DefMath!("\\complement", "\u{2201}", meaning => "complement");
  DefMath!("\\eth", "\u{00F0}");
  DefMath!("\\diagup", "\u{2571}");
  DefMath!("\\diagdown", "\u{2572}");

  //======================================================================
  // Binary operators
  DefMath!("\\dotplus",        "\u{2214}", role => "ADDOP"); // DOT PLUS
  DefMath!("\\smallsetminus",  "\u{2216}", role => "ADDOP", meaning => "set-minus");
  DefMath!("\\Cap",            "\u{22D2}", role => "ADDOP", meaning => "double-intersection");
  DefMath!("\\doublecap",      "\u{22D2}", role => "ADDOP", meaning => "double-intersection");
  DefMath!("\\Cup",            "\u{22D3}", role => "ADDOP", meaning => "double-union");
  DefMath!("\\doublecup",      "\u{22D3}", role => "ADDOP", meaning => "double-union");
  DefMath!("\\barwedge",       "\u{22BC}", role => "ADDOP", meaning => "not-and");
  DefMath!("\\veebar",         "\u{22BB}", role => "ADDOP", meaning => "exclusive-or");
  DefMath!("\\doublebarwedge", "\u{2A5E}", role => "ADDOP");
  DefMath!("\\boxminus",      "\u{229F}", role => "ADDOP"); // SQUARED MINUS
  DefMath!("\\boxtimes",      "\u{22A0}", role => "MULOP"); // SQUARED TIMES
  DefMath!("\\boxdot",        "\u{22A1}", role => "MULOP"); // SQUARED DOT OPERATOR
  DefMath!("\\boxplus",       "\u{229E}", role => "ADDOP"); // SQUARED PLUS
  DefMath!("\\divideontimes", "\u{22C7}", role => "MULOP"); // DIVISION TIMES
  DefMath!("\\ltimes", "\u{22C9}", role => "MULOP", meaning => "left-normal-factor-semidirect-product");
  DefMath!("\\rtimes", "\u{22CA}", role => "MULOP", meaning => "right-normal-factor-semidirect-product");
  DefMath!("\\leftthreetimes",  "\u{22CB}", role => "MULOP", meaning => "left-semidirect-product");
  DefMath!("\\rightthreetimes", "\u{22CC}", role => "MULOP", meaning => "right-semidirect-product");
  DefMath!("\\curlywedge",      "\u{22CF}", role => "ADDOP", meaning => "and");
  DefMath!("\\curlyvee",        "\u{22CE}", role => "ADDOP", meaning => "or");
  DefMath!("\\circleddash", "\u{229D}", role => "ADDOP"); // CIRCLED DASH
  DefMath!("\\circledast",  "\u{229B}", role => "MULOP"); // CIRCLED ASTERISK OPERATOR
  DefMath!("\\circledcirc", "\u{229A}", role => "MULOP"); // CIRCLED RING OPERATOR
  DefMath!("\\centerdot",   "\u{2219}", role => "MULOP"); // CIRCLED DOT OPERATOR
  DefMath!("\\intercal",    "\u{22BA}", role => "ADDOP"); // INTERCALATE

  //======================================================================
  // Binary relations
  DefMath!("\\leqq", "\u{2266}", role => "RELOP",
  meaning => "less-than-or-equals");
  DefMath!("\\leqslant", "\u{2A7D}", role => "RELOP",
  meaning => "less-than-or-equals");
  DefMath!("\\eqslantless", "\u{2A95}", role => "RELOP",
  meaning => "less-than-or-equals");
  DefMath!("\\lesssim", "\u{2272}", role => "RELOP",
  meaning => "less-than-or-similar-to");
  DefMath!("\\lessapprox", "\u{2A85}", role => "RELOP",
  meaning => "less-than-or-approximately-equals");
  DefMath!("\\approxeq", "\u{224A}", role => "RELOP",
  meaning => "approximately-equals-or-equals");
  DefMath!("\\lessdot", "\u{22D6}", role => "RELOP"); // LESS-THAN WITH DOT
  DefMath!("\\lll", "\u{22D8}", role => "RELOP",
  meaning => "very-much-less-than"); // VERY MUCH LESS-THAN
  DefMath!("\\llless", "\u{22D8}", role => "RELOP",
  meaning => "very-much-less-than"); // VERY MUCH LESS-THAN
  DefMath!("\\lessgtr", "\u{2276}", role => "RELOP",
  meaning => "less-than-or-greater-than");
  DefMath!("\\lesseqgtr", "\u{22DA}", role => "RELOP",
  meaning => "less-than-or-equals-or-greater-than");
  DefMath!("\\lesseqqgtr", "\u{2A8B}", role => "RELOP",
  meaning => "less-than-or-equals-or-greater-than");
  DefMath!("\\doteqdot", "\u{2251}", role => "RELOP",
  meaning => "geometrically-equals");
  DefMath!("\\Doteq", "\u{2251}", role => "RELOP",
  meaning => "geometrically-equals");
  DefMath!("\\risingdotseq", "\u{2253}", role => "RELOP",
  meaning => "image-of-or-approximately-equals");
  DefMath!("\\fallingdotseq", "\u{2252}", role => "RELOP",
  meaning => "approximately-equals-or-image-of");
  DefMath!("\\backsim", "\u{223D}", role => "RELOP"); // REVERSED TILDE
  DefMath!("\\backsimeq", "\u{224C}", role => "RELOP"); // ALL EQUAL TO; Note: this has double rather than single bar!!!
  DefMath!("\\subseteqq", "\u{2AC5}", role => "RELOP",
  meaning => "subset-of-or-equals");
  DefMath!("\\Subset", "\u{22D0}", role => "RELOP",
  meaning => "double-subset-of");
  // \sqsubset in tex
  DefMath!("\\preccurlyeq", "\u{227C}", role => "RELOP",
  meaning => "precedes-or-equals");
  DefMath!("\\curlyeqprec", "\u{22DE}", role => "RELOP",
  meaning => "equals-or-preceeds");
  DefMath!("\\precsim", "\u{227E}", role => "RELOP",
  meaning => "precedes-or-equivalent-to");
  DefMath!("\\precapprox", "\u{2AB7}", role => "RELOP",
  meaning => "precedes-or-approximately-equals");
  // \vartriangleleft, trianglelefteq in amsfonts
  DefMath!("\\vDash",      "\u{22A8}", role => "RELOP"); // TRUE
  DefMath!("\\Vvdash",     "\u{22AA}", role => "RELOP"); // TRIPLE VERTICAL BAR RIGHT TURNSTILE
  DefMath!("\\smallsmile", "\u{2323}", role => "RELOP"); // SMILE (small ?)
  DefMath!("\\smallfrown", "\u{2322}", role => "RELOP"); // FROWN (small ?)
  DefMath!("\\bumpeq", "\u{224F}", role => "RELOP",
  meaning => "difference-between");
  DefMath!("\\Bumpeq", "\u{224E}", role => "RELOP",
  meaning => "geometrically-equals");
  DefMath!("\\geqq", "\u{2267}", role => "RELOP",
  meaning => "greater-than-or-equals");
  DefMath!("\\geqslant", "\u{2A7E}", role => "RELOP",
  meaning => "greater-than-or-equals");
  DefMath!("\\eqslantgtr", "\u{2A96}", role => "RELOP",
  meaning => "greater-than-or-equals");
  DefMath!("\\gtrsim", "\u{2273}", role => "RELOP",
  meaning => "greater-than-or-equivalent-to");
  DefMath!("\\gtrapprox", "\u{2A86}", role => "RELOP",
  meaning => "greater-than-or-approximately-equals");
  DefMath!("\\eqsim",  "\u{2242}", role => "RELOP"); // MINUS TILDE
  DefMath!("\\gtrdot", "\u{22D7}", role => "RELOP"); // GREATER-THAN WITH DOT
  DefMath!("\\ggg", "\u{22D9}", role => "RELOP",
  meaning => "very-much-greater-than");
  DefMath!("\\gggtr", "\u{22D9}", role => "RELOP",
  meaning => "very-much-greater-than");
  DefMath!("\\gtrless", "\u{2277}", role => "RELOP",
  meaning => "greater-than-or-less-than");
  DefMath!("\\gtreqless", "\u{22DB}", role => "RELOP",
  meaning => "greater-than-or-equals-or-less-than");
  DefMath!("\\gtreqqless", "\u{2A8C}", role => "RELOP",
  meaning => "greater-than-or-equals-or-less-than");
  DefMath!("\\eqcirc",    "\u{2256}", role => "RELOP"); // RING IN EQUAL TO
  DefMath!("\\circeq",    "\u{2257}", role => "RELOP"); // RING EQUAL TO
  DefMath!("\\triangleq", "\u{225C}", role => "RELOP"); // DELTA EQUAL TO
  DefMath!("\\thicksim",  "\u{223C}", role => "RELOP"); // TILDE OPERATOR; Not thick!!!
  DefMath!("\\thickapprox", "\u{2248}", role => "RELOP",
  meaning => "approximately-equals");
  DefMath!("\\supseteqq", "\u{2AC6}", role => "RELOP",
  meaning => "superset-of-or-equals");
  DefMath!("\\Supset", "\u{22D1}", role => "RELOP",
  meaning => "double-superset-of");
  // \sqsupset in TeX
  DefMath!("\\succcurlyeq", "\u{227D}", role => "RELOP",
  meaning => "succeeds-or-equals");
  DefMath!("\\curlyeqsucc", "\u{22DF}", role => "RELOP",
  meaning => "equals-or-succeeds");
  DefMath!("\\succsim", "\u{227F}", role => "RELOP",
  meaning => "succeeds-or-equivalent-to");
  DefMath!("\\succapprox", "\u{2AB8}", role => "RELOP",
  meaning => "succeeds-or-approximately-equals");
  // \vartriangleright, \trianglerighteq in amsfonts
  DefMath!("\\Vdash", "\u{22A9}", role => "RELOP",
  meaning => "forces");
  DefMath!("\\shortmid", "\u{2223}", role => "RELOP",
  meaning => "divides");
  DefMath!("\\shortparallel", "\u{2225}", role => "RELOP",
  meaning => "parallel-to");
  DefMath!("\\between", "\u{226C}", role => "RELOP",
  meaning => "between");
  DefMath!("\\pitchfork", "\u{22D4}", role => "RELOP",
  meaning => "proper-intersection");
  DefMath!("\\varpropto", "\u{221D}", role => "RELOP",
  meaning => "proportional-to");
  DefMath!("\\blacktriangleleft", "\u{25C0}", role => "RELOP"); // BLACK LEFT-POINTING TRIANGLE
  DefMath!("\\therefore", "\u{2234}", role => "METARELOP",
  meaning => "therefore");
  DefMath!("\\backepsilon",        "\u{03F6}", role => "RELOP"); // GREEK REVERSED LUNATE EPSILON SYMBOL
  DefMath!("\\blacktriangleright", "\u{25B6}", role => "RELOP"); // BLACK RIGHT-POINTING TRIANGLE
  DefMath!("\\because", "\u{2235}", role => "METARELOP",
  meaning => "because");

  //======================================================================
  // Negated relations
  // NOTE: There are several here that I couldn"t find, but all
  // were negations of other symbols. I"ve used 0338 COMBINING LONG SOLIDUS OVERLAY
  // to create them, but I don"t know if that"s right.

  DefMath!("\\nless", "\u{226E}", role => "RELOP",
  meaning => "not-less-than");
  DefMath!("\\nleq", "\u{2270}", role => "RELOP",
  meaning => "not-less-than-nor-greater-than");
  DefMath!("\\nleqslant", "\u{2A7D}\u{0338}", role => "RELOP",
  meaning => "not-less-than-nor-equals");
  DefMath!("\\nleqq", "\u{2266}\u{0338}", role => "RELOP",
  meaning => "not-less-than-nor-equals");
  DefMath!("\\lneq", "\u{2A87}", role => "RELOP",
  meaning => "less-than-and-not-equals");
  DefMath!("\\lneqq", "\u{2268}", role => "RELOP",
  meaning => "less-than-and-not-equals");
  DefMath!("\\lvertneqq", "\u{2268}", role => "RELOP",
  meaning => "less-than-and-not-equals");
  DefMath!("\\lnsim", "\u{22E6}", role => "RELOP",
  meaning => "less-than-and-not-equivalent-to");
  DefMath!("\\lnapprox", "\u{2A89}", role => "RELOP",
  meaning => "less-than-and-not-approximately-equals");
  DefMath!("\\nprec", "\u{2280}", role => "RELOP",
  meaning => "not-precedes");
  DefMath!("\\npreceq", "\u{22E0}", role => "RELOP",
  meaning => "not-precedes-nor-equals"); // Using slant equals?
  DefMath!("\\precneqq", "\u{2AB5}", role => "RELOP",
  meaning => "precedes-and-not-equals");
  DefMath!("\\precnsim", "\u{22E8}", role => "RELOP",
  meaning => "precedes-and-not-equivalent-to");
  DefMath!("\\precnapprox", "\u{2AB9}", role => "RELOP",
  meaning => "precedes-and-not-approximately-equals");
  DefMath!("\\nsim", "\u{2241}", role => "RELOP",
  meaning => "not-similar-to"); // NOTE TILDE
  DefMath!("\\nshortmid", "\u{2224}", role => "RELOP",
  meaning => "not-divides"); // DOES NOT DIVIDE; Note: not short!
  DefMath!("\\nmid", "\u{2224}", role => "RELOP",
  meaning => "not-divides"); // DOES NOT DIVIDE
  DefMath!("\\nvdash", "\u{22AC}", role => "RELOP",
  meaning => "not-proves");
  DefMath!("\\nVdash", "\u{22AE}", role => "RELOP",
  meaning => "not-forces");
  DefMath!("\\ntriangleleft", "\u{22EA}", role => "RELOP",
  meaning => "not-subgroup-of");
  DefMath!("\\ntrianglelefteq", "\u{22EC}", role => "RELOP",
  meaning => "not-subgroup-of-nor-equals");
  DefMath!("\\nsubseteq", "\u{2288}", role => "RELOP",
  meaning => "not-subset-of-nor-equals");
  DefMath!("\\nsubseteqq", "\u{2AC5}\u{0338}", role => "RELOP",
  meaning => "not-subset-of-nor-equals");
  DefMath!("\\subsetneq", "\u{228A}", role => "RELOP",
  meaning => "subset-of-and-not-equals");
  DefMath!("\\varsubsetneq", "\u{228A}", role => "RELOP",
  meaning => "subset-of-and-not-equals");
  DefMath!("\\subsetneqq", "\u{2ACB}", role => "RELOP",
  meaning => "subset-of-and-not-equals");
  DefMath!("\\varsubsetneqq", "\u{2ACB}", role => "RELOP",
  meaning => "subset-of-and-not-equals");
  DefMath!("\\supsetneq", "\u{228B}", role => "RELOP",
  meaning => "superset-of-and-not-equals");
  DefMath!("\\varsupsetneq", "\u{228B}", role => "RELOP",
  meaning => "superset-of-and-not-equals");
  DefMath!("\\supsetneqq", "\u{2ACC}", role => "RELOP",
  meaning => "superset-of-and-not-equals");
  DefMath!("\\varsupsetneqq", "\u{2ACC}", role => "RELOP",
  meaning => "superset-of-and-not-equals");

  DefMath!("\\ngtr", "\u{226F}", role => "RELOP",
  meaning => "not-greater-than");
  DefMath!("\\ngeq", "\u{2271}", role => "RELOP",
  meaning => "not-greater-than-nor-equals");
  DefMath!("\\ngeqslant", "\u{2A7E}\u{0338}", role => "RELOP",
  meaning => "not-greater-than-nor-equals");
  DefMath!("\\ngeqq", "\u{2267}\u{0338}", role => "RELOP",
  meaning => "not-greater-than-nor-equals");
  DefMath!("\\gneq", "\u{2A88}", role => "RELOP",
  meaning => "greater-than-and-not-equals");
  DefMath!("\\gneqq", "\u{2269}", role => "RELOP",
  meaning => "greater-than-and-not-equals");
  DefMath!("\\gvertneqq", "\u{2269}", role => "RELOP",
  meaning => "greater-than-and-not-equals");
  DefMath!("\\gnsim", "\u{22E7}", role => "RELOP",
  meaning => "greater-than-and-not-equivalent-to");
  DefMath!("\\gnapprox", "\u{2A8A}", role => "RELOP",
  meaning => "greater-than-and-not-approximately-equals");
  DefMath!("\\nsucc", "\u{2281}", role => "RELOP",
  meaning => "not-succeeds");
  DefMath!("\\nsucceq", "\u{22E1}", role => "RELOP",
  meaning => "not-succeeds-nor-equals");
  DefMath!("\\succneqq", "\u{2AB6}", role => "RELOP",
  meaning => "succeeds-and-not-equals");
  DefMath!("\\succnsim", "\u{22E9}", role => "RELOP",
  meaning => "succeeds-and-not-equivalent-to");
  DefMath!("\\succnapprox", "\u{2ABA}", role => "RELOP",
  meaning => "succeeds-and-not-approximately-equals");
  DefMath!("\\ncong", "\u{2247}", role => "RELOP",
  meaning => "not-approximately-equals");
  DefMath!("\\nshortparallel", "\u{2226}", role => "RELOP",
  meaning => "not-parallel-to");
  DefMath!("\\nparallel", "\u{2226}", role => "RELOP",
  meaning => "not-parallel-to");
  DefMath!("\\nvDash", "\u{22AD}", role => "RELOP"); // NOT TRUE
  DefMath!("\\nVDash", "\u{22AF}", role => "RELOP"); // NEGATED DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE
  DefMath!("\\ntriangleright", "\u{22EB}", role => "RELOP",
  meaning => "not-contains");
  DefMath!("\\ntrianglerighteq", "\u{22ED}", role => "RELOP",
  meaning => "not-contains-nor-equals");
  DefMath!("\\nsupseteq", "\u{2289}", role => "RELOP",
  meaning => "not-superset-of-nor-equals");
  DefMath!("\\nsupseteqq", "\u{2AC6}\u{0338}", role => "RELOP",
  meaning => "not-superset-of-nor-equals");

  //======================================================================
  // Arrows
  DefMath!("\\leftleftarrows",   "\u{21C7}", role => "ARROW"); // LEFTWARDS PAIRED ARROWS
  DefMath!("\\leftrightarrows",  "\u{21C6}", role => "ARROW"); // LEFTWARDS ARROW OVER RIGHTWARDS ARROW
  DefMath!("\\Lleftarrow",       "\u{21DA}", role => "ARROW"); // LEFTWARDS TRIPLE ARROW
  DefMath!("\\twoheadleftarrow", "\u{219E}", role => "ARROW"); // LEFTWARDS TWHO HEADED ARROW
  DefMath!("\\leftarrowtail",    "\u{21A2}", role => "ARROW"); // LEFTWARDS ARROW WITH TAIL
  DefMath!("\\looparrowleft",    "\u{21AB}", role => "ARROW"); // leftwards arrow with loop
  DefMath!("\\leftrightharpoons", "\u{21CB}", role => "ARROW"); // LEFTWARDS HARPOON OVER RIGHTWARDS HARPOON
  DefMath!("\\curvearrowleft",    "\u{21B6}", role => "ARROW"); // ANTICLOCKWISE TOP SEMICIRCLE ARROW
  DefMath!("\\circlearrowleft",   "\u{21BA}", role => "ARROW"); // ANTICLOCKWISE OPEN CIRCLE ARROW
  DefMath!("\\Lsh",               "\u{21B0}", role => "ARROW"); // UPWAARDS ARROW WITH TIP LEFTWARDS
  DefMath!("\\upuparrows",        "\u{21C8}", role => "ARROW"); // UPWARDS PAIRED ARROWS
  DefMath!("\\upharpoonleft",     "\u{21BF}", role => "ARROW"); // UPWARDS HARPOON WITH BARB LEFTWARDS
  DefMath!("\\rightrightarrows",  "\u{21C9}", role => "ARROW"); // RIGHTWARDS PAIRED ARROWS
  DefMath!("\\rightleftarrows",   "\u{21C4}", role => "ARROW"); // RIGHTWARDS ARROW OVER LEFTWARD ARROW
  DefMath!("\\Rrightarrow",       "\u{21DB}", role => "ARROW"); // RIGHTWARDS TRIPLE ARROW
  DefMath!("\\twoheadrightarrow", "\u{21A0}", role => "ARROW"); // RIGHTWARDS TWO HEADED ARROW
  DefMath!("\\rightarrowtail",    "\u{21A3}", role => "ARROW"); // RIGHTWARDS ARROW WITH TAIL
  DefMath!("\\looparrowright",    "\u{21AC}", role => "ARROW"); // RIGHTWARDS ARROW WITH LOOP

  // \rightleftharpoons  21CC # RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON ; in amsfonts

  DefMath!("\\curvearrowright",  "\u{21B7}", role => "ARROW"); // CLOCKWISE TOP SEMICIRCLE ARROW
  DefMath!("\\circlearrowright", "\u{21BB}", role => "ARROW"); // CLOCKWISE OPEN CIRCLE ARROW
  DefMath!("\\Rsh",              "\u{21B1}", role => "ARROW"); // UPWAARDS ARROW WITH TIP RIGHTWARDS
  DefMath!("\\downdownarrows",   "\u{21CA}", role => "ARROW"); // DOWNWARDS PAIRED ARROWS
  DefMath!("\\upharpoonright",   "\u{21BE}", role => "ARROW"); // UPWARDS HARPOON WITH BARB RIGHTWARDS
  DefMath!("\\restriction",      "\u{21BE}", role => "ARROW"); // UPWARDS HARPOON WITH BARB RIGHTWARDS
  // (same as \upharpoonright)
  DefMath!("\\downharpoonleft",  "\u{21C3}", role => "ARROW"); // DOWNWARDS HARPOON WITH BARB LEFTWARDS
  DefMath!("\\multimap",         "\u{22B8}", role => "ARROW"); // MULTIMAP
  DefMath!("\\leftrightsquigarrow", "\u{21AD}", role => "ARROW"); // LEFT RIGHT WAVE ARROW
  DefMath!("\\downharpoonright", "\u{21C2}", role => "ARROW"); // DOWNWARDS HARPOON WITH BARB RIGHTWARDS
  // \rightsquigarrow amsfonts

  //======================================================================
  // Negated arrows
  DefMath!("\\nleftarrow",      "\u{219A}", role => "ARROW"); // LEFTWARDS ARROW WITH STROKE
  DefMath!("\\nLeftarrow",      "\u{21CD}", role => "ARROW"); // LEFTWARDS DOUBLE ARROW WITH STROKE
  DefMath!("\\nleftrightarrow", "\u{21AE}", role => "ARROW"); // LEFT RIGHT ARROW WITH STROKE
  DefMath!("\\nrightarrow",     "\u{219B}", role => "ARROW"); // RIGHTWARDS ARROW WITH STROKE
  DefMath!("\\nRightarrow",     "\u{21CF}", role => "ARROW"); // LEFTWARDS DOUBLE ARROW WITH STROKE
  DefMath!("\\nLeftrightarrow", "\u{21CE}", role => "ARROW"); // LEFT RIGHT DOUBLE ARROW WITH STROKE

  //======================================================================
});
