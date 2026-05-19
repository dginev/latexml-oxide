use crate::prelude::*;

/// Runtime helper for the trivial `DefMath!` shape used 90+ times in
/// amssymb (DEP-17c). Mirrors DEP-17/DEP-17b: route the same shape
/// through one fn so each call site compiles to a short call instead
/// of ~1 KiB of inlined macro expansion. Engine bootstrap pays
/// `parse_prototype` once per entry.
fn def_math_sym(cs: &str, present: &str, role: Option<&str>, meaning: Option<&str>) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let mut opts = MathPrimitiveOptions::default();
  if let Some(r) = role { opts.role = Some(r.to_string()); }
  if let Some(m) = meaning { opts.meaning = Some(m.to_string()); }
  def_math(cs_tok, params, present.to_string(), opts)?;
  Ok(())
}

LoadDefinitions!({
  RequirePackage!("amsfonts");

  //======================================================================
  // Lowercase Greek letters
  def_math_sym("\\digamma", "\u{03DD}", None, None)?; // GREEK SMALL LETTER DIGAMMA
  def_math_sym("\\varkappa", "\u{03F0}", None, None)?; // GREEK KAPPA SYMBOL

  //======================================================================
  // Hebrew
  def_math_sym("\\beth", "\u{2136}", None, None)?; // BET SYMBOL
  def_math_sym("\\daleth", "\u{2138}", None, None)?; // DALET SYMBOL
  def_math_sym("\\gimel", "\u{2137}", None, None)?; // GIMEL SYMBOL

  //======================================================================
  // Miscellaneous
  // \hbar  in LaTeX
  def_math_sym("\\hslash", "\u{210F}", Some("ID"), Some("Planck-constant-over-2-pi"))?;
  def_math_sym("\\vartriangle", "\u{25B3}", None, None)?;
  def_math_sym("\\triangledown", "\u{25BD}", None, None)?;
  // \square, \lozenge in amsfonts
  def_math_sym("\\circledS", "\u{24C8}", None, None)?;
  // \angle in tex
  def_math_sym("\\measuredangle", "\u{2221}", None, None)?;
  def_math_sym("\\nexists", "\u{2204}", Some("FUNCTION"), Some("not-exists"))?;
  // \mho in latex
  def_math_sym("\\Finv", "\u{2132}", None, None)?;
  def_math_sym("\\Game", "\u{2141}", None, None)?;
  def_math_sym("\\Bbbk", "\u{1D55C}", None, None)?;
  def_math_sym("\\backprime", "\u{2035}", None, None)?;
  def_math_sym("\\varnothing", "\u{2205}", Some("ID"), Some("empty-set"))?;
  def_math_sym("\\blacktriangle", "\u{25B2}", None, None)?;
  def_math_sym("\\blacktriangledown", "\u{25BC}", None, None)?;
  def_math_sym("\\blacksquare", "\u{25A0}", None, None)?;
  def_math_sym("\\blacklozenge", "\u{25C6}", None, None)?;
  def_math_sym("\\bigstar", "\u{2605}", None, None)?;
  def_math_sym("\\sphericalangle", "\u{2222}", None, None)?;
  DefMath!("\\complement", "\u{2201}", meaning => "complement");
  def_math_sym("\\eth", "\u{00F0}", None, None)?;
  def_math_sym("\\diagup", "\u{2571}", None, None)?;
  def_math_sym("\\diagdown", "\u{2572}", None, None)?;

  //======================================================================
  // Binary operators
  def_math_sym("\\dotplus", "\u{2214}", Some("ADDOP"), None)?; // DOT PLUS
  def_math_sym("\\smallsetminus", "\u{2216}", Some("ADDOP"), Some("set-minus"))?;
  def_math_sym("\\Cap", "\u{22D2}", Some("ADDOP"), Some("double-intersection"))?;
  def_math_sym("\\doublecap", "\u{22D2}", Some("ADDOP"), Some("double-intersection"))?;
  def_math_sym("\\Cup", "\u{22D3}", Some("ADDOP"), Some("double-union"))?;
  def_math_sym("\\doublecup", "\u{22D3}", Some("ADDOP"), Some("double-union"))?;
  def_math_sym("\\barwedge", "\u{22BC}", Some("ADDOP"), Some("not-and"))?;
  def_math_sym("\\veebar", "\u{22BB}", Some("ADDOP"), Some("exclusive-or"))?;
  def_math_sym("\\doublebarwedge", "\u{2A5E}", Some("ADDOP"), None)?;
  def_math_sym("\\boxminus", "\u{229F}", Some("ADDOP"), None)?; // SQUARED MINUS
  def_math_sym("\\boxtimes", "\u{22A0}", Some("MULOP"), None)?; // SQUARED TIMES
  def_math_sym("\\boxdot", "\u{22A1}", Some("MULOP"), None)?; // SQUARED DOT OPERATOR
  def_math_sym("\\boxplus", "\u{229E}", Some("ADDOP"), None)?; // SQUARED PLUS
  def_math_sym("\\divideontimes", "\u{22C7}", Some("MULOP"), None)?; // DIVISION TIMES
  def_math_sym("\\ltimes", "\u{22C9}", Some("MULOP"), Some("left-normal-factor-semidirect-product"))?;
  def_math_sym("\\rtimes", "\u{22CA}", Some("MULOP"), Some("right-normal-factor-semidirect-product"))?;
  def_math_sym("\\leftthreetimes", "\u{22CB}", Some("MULOP"), Some("left-semidirect-product"))?;
  def_math_sym("\\rightthreetimes", "\u{22CC}", Some("MULOP"), Some("right-semidirect-product"))?;
  def_math_sym("\\curlywedge", "\u{22CF}", Some("ADDOP"), Some("and"))?;
  def_math_sym("\\curlyvee", "\u{22CE}", Some("ADDOP"), Some("or"))?;
  def_math_sym("\\circleddash", "\u{229D}", Some("ADDOP"), None)?; // CIRCLED DASH
  def_math_sym("\\circledast", "\u{229B}", Some("MULOP"), None)?; // CIRCLED ASTERISK OPERATOR
  def_math_sym("\\circledcirc", "\u{229A}", Some("MULOP"), None)?; // CIRCLED RING OPERATOR
  def_math_sym("\\centerdot", "\u{2219}", Some("MULOP"), None)?; // CIRCLED DOT OPERATOR
  def_math_sym("\\intercal", "\u{22BA}", Some("ADDOP"), None)?; // INTERCALATE

  //======================================================================
  // Binary relations
  def_math_sym("\\leqq", "\u{2266}", Some("RELOP"), Some("less-than-or-equals"))?;
  def_math_sym("\\leqslant", "\u{2A7D}", Some("RELOP"), Some("less-than-or-equals"))?;
  def_math_sym("\\eqslantless", "\u{2A95}", Some("RELOP"), Some("less-than-or-equals"))?;
  def_math_sym("\\lesssim", "\u{2272}", Some("RELOP"), Some("less-than-or-similar-to"))?;
  def_math_sym("\\lessapprox", "\u{2A85}", Some("RELOP"), Some("less-than-or-approximately-equals"))?;
  def_math_sym("\\approxeq", "\u{224A}", Some("RELOP"), Some("approximately-equals-or-equals"))?;
  def_math_sym("\\lessdot", "\u{22D6}", Some("RELOP"), None)?; // LESS-THAN WITH DOT
  def_math_sym("\\lll", "\u{22D8}", Some("RELOP"), Some("very-much-less-than"))?; // VERY MUCH LESS-THAN
  def_math_sym("\\llless", "\u{22D8}", Some("RELOP"), Some("very-much-less-than"))?; // VERY MUCH LESS-THAN
  def_math_sym("\\lessgtr", "\u{2276}", Some("RELOP"), Some("less-than-or-greater-than"))?;
  def_math_sym("\\lesseqgtr", "\u{22DA}", Some("RELOP"), Some("less-than-or-equals-or-greater-than"))?;
  def_math_sym("\\lesseqqgtr", "\u{2A8B}", Some("RELOP"), Some("less-than-or-equals-or-greater-than"))?;
  def_math_sym("\\doteqdot", "\u{2251}", Some("RELOP"), Some("geometrically-equals"))?;
  def_math_sym("\\Doteq", "\u{2251}", Some("RELOP"), Some("geometrically-equals"))?;
  def_math_sym("\\risingdotseq", "\u{2253}", Some("RELOP"), Some("image-of-or-approximately-equals"))?;
  def_math_sym("\\fallingdotseq", "\u{2252}", Some("RELOP"), Some("approximately-equals-or-image-of"))?;
  def_math_sym("\\backsim", "\u{223D}", Some("RELOP"), None)?; // REVERSED TILDE
  // Perl commit 93347f6c (#2633): \backsimeq -> U+22CD (REVERSED TILDE EQUALS),
  // not U+224C (ALL EQUAL TO); the former has a single bar matching LaTeX output.
  def_math_sym("\\backsimeq", "\u{22CD}", Some("RELOP"), None)?;
  def_math_sym("\\subseteqq", "\u{2AC5}", Some("RELOP"), Some("subset-of-or-equals"))?;
  def_math_sym("\\Subset", "\u{22D0}", Some("RELOP"), Some("double-subset-of"))?;
  // \sqsubset in tex
  def_math_sym("\\preccurlyeq", "\u{227C}", Some("RELOP"), Some("precedes-or-equals"))?;
  def_math_sym("\\curlyeqprec", "\u{22DE}", Some("RELOP"), Some("equals-or-preceeds"))?;
  def_math_sym("\\precsim", "\u{227E}", Some("RELOP"), Some("precedes-or-equivalent-to"))?;
  def_math_sym("\\precapprox", "\u{2AB7}", Some("RELOP"), Some("precedes-or-approximately-equals"))?;
  // \vartriangleleft, trianglelefteq in amsfonts
  def_math_sym("\\vDash", "\u{22A8}", Some("RELOP"), None)?; // TRUE
  def_math_sym("\\Vvdash", "\u{22AA}", Some("RELOP"), None)?; // TRIPLE VERTICAL BAR RIGHT TURNSTILE
  def_math_sym("\\smallsmile", "\u{2323}", Some("RELOP"), None)?; // SMILE (small ?)
  def_math_sym("\\smallfrown", "\u{2322}", Some("RELOP"), None)?; // FROWN (small ?)
  def_math_sym("\\bumpeq", "\u{224F}", Some("RELOP"), Some("difference-between"))?;
  def_math_sym("\\Bumpeq", "\u{224E}", Some("RELOP"), Some("geometrically-equals"))?;
  def_math_sym("\\geqq", "\u{2267}", Some("RELOP"), Some("greater-than-or-equals"))?;
  def_math_sym("\\geqslant", "\u{2A7E}", Some("RELOP"), Some("greater-than-or-equals"))?;
  def_math_sym("\\eqslantgtr", "\u{2A96}", Some("RELOP"), Some("greater-than-or-equals"))?;
  def_math_sym("\\gtrsim", "\u{2273}", Some("RELOP"), Some("greater-than-or-equivalent-to"))?;
  def_math_sym("\\gtrapprox", "\u{2A86}", Some("RELOP"), Some("greater-than-or-approximately-equals"))?;
  def_math_sym("\\eqsim", "\u{2242}", Some("RELOP"), None)?; // MINUS TILDE
  def_math_sym("\\gtrdot", "\u{22D7}", Some("RELOP"), None)?; // GREATER-THAN WITH DOT
  def_math_sym("\\ggg", "\u{22D9}", Some("RELOP"), Some("very-much-greater-than"))?;
  def_math_sym("\\gggtr", "\u{22D9}", Some("RELOP"), Some("very-much-greater-than"))?;
  def_math_sym("\\gtrless", "\u{2277}", Some("RELOP"), Some("greater-than-or-less-than"))?;
  def_math_sym("\\gtreqless", "\u{22DB}", Some("RELOP"), Some("greater-than-or-equals-or-less-than"))?;
  def_math_sym("\\gtreqqless", "\u{2A8C}", Some("RELOP"), Some("greater-than-or-equals-or-less-than"))?;
  def_math_sym("\\eqcirc", "\u{2256}", Some("RELOP"), None)?; // RING IN EQUAL TO
  def_math_sym("\\circeq", "\u{2257}", Some("RELOP"), None)?; // RING EQUAL TO
  def_math_sym("\\triangleq", "\u{225C}", Some("RELOP"), None)?; // DELTA EQUAL TO
  def_math_sym("\\thicksim", "\u{223C}", Some("RELOP"), None)?; // TILDE OPERATOR; Not thick!!!
  def_math_sym("\\thickapprox", "\u{2248}", Some("RELOP"), Some("approximately-equals"))?;
  def_math_sym("\\supseteqq", "\u{2AC6}", Some("RELOP"), Some("superset-of-or-equals"))?;
  def_math_sym("\\Supset", "\u{22D1}", Some("RELOP"), Some("double-superset-of"))?;
  // \sqsupset in TeX
  def_math_sym("\\succcurlyeq", "\u{227D}", Some("RELOP"), Some("succeeds-or-equals"))?;
  def_math_sym("\\curlyeqsucc", "\u{22DF}", Some("RELOP"), Some("equals-or-succeeds"))?;
  def_math_sym("\\succsim", "\u{227F}", Some("RELOP"), Some("succeeds-or-equivalent-to"))?;
  def_math_sym("\\succapprox", "\u{2AB8}", Some("RELOP"), Some("succeeds-or-approximately-equals"))?;
  // \vartriangleright, \trianglerighteq in amsfonts
  def_math_sym("\\Vdash", "\u{22A9}", Some("RELOP"), Some("forces"))?;
  def_math_sym("\\shortmid", "\u{2223}", Some("RELOP"), Some("divides"))?;
  def_math_sym("\\shortparallel", "\u{2225}", Some("RELOP"), Some("parallel-to"))?;
  def_math_sym("\\between", "\u{226C}", Some("RELOP"), Some("between"))?;
  def_math_sym("\\pitchfork", "\u{22D4}", Some("RELOP"), Some("proper-intersection"))?;
  def_math_sym("\\varpropto", "\u{221D}", Some("RELOP"), Some("proportional-to"))?;
  def_math_sym("\\blacktriangleleft", "\u{25C0}", Some("RELOP"), None)?; // BLACK LEFT-POINTING TRIANGLE
  def_math_sym("\\therefore", "\u{2234}", Some("METARELOP"), Some("therefore"))?;
  def_math_sym("\\backepsilon", "\u{03F6}", Some("RELOP"), None)?; // GREEK REVERSED LUNATE EPSILON SYMBOL
  def_math_sym("\\blacktriangleright", "\u{25B6}", Some("RELOP"), None)?; // BLACK RIGHT-POINTING TRIANGLE
  def_math_sym("\\because", "\u{2235}", Some("METARELOP"), Some("because"))?;

  //======================================================================
  // Negated relations
  // NOTE: There are several here that I couldn"t find, but all
  // were negations of other symbols. I"ve used 0338 COMBINING LONG SOLIDUS OVERLAY
  // to create them, but I don"t know if that"s right.

  def_math_sym("\\nless", "\u{226E}", Some("RELOP"), Some("not-less-than"))?;
  def_math_sym("\\nleq", "\u{2270}", Some("RELOP"), Some("not-less-than-nor-greater-than"))?;
  def_math_sym("\\nleqslant", "\u{2A7D}\u{0338}", Some("RELOP"), Some("not-less-than-nor-equals"))?;
  def_math_sym("\\nleqq", "\u{2266}\u{0338}", Some("RELOP"), Some("not-less-than-nor-equals"))?;
  def_math_sym("\\lneq", "\u{2A87}", Some("RELOP"), Some("less-than-and-not-equals"))?;
  def_math_sym("\\lneqq", "\u{2268}", Some("RELOP"), Some("less-than-and-not-equals"))?;
  def_math_sym("\\lvertneqq", "\u{2268}", Some("RELOP"), Some("less-than-and-not-equals"))?;
  def_math_sym("\\lnsim", "\u{22E6}", Some("RELOP"), Some("less-than-and-not-equivalent-to"))?;
  def_math_sym("\\lnapprox", "\u{2A89}", Some("RELOP"), Some("less-than-and-not-approximately-equals"))?;
  def_math_sym("\\nprec", "\u{2280}", Some("RELOP"), Some("not-precedes"))?;
  def_math_sym("\\npreceq", "\u{22E0}", Some("RELOP"), Some("not-precedes-nor-equals"))?; // Using slant equals?
  def_math_sym("\\precneqq", "\u{2AB5}", Some("RELOP"), Some("precedes-and-not-equals"))?;
  def_math_sym("\\precnsim", "\u{22E8}", Some("RELOP"), Some("precedes-and-not-equivalent-to"))?;
  def_math_sym("\\precnapprox", "\u{2AB9}", Some("RELOP"), Some("precedes-and-not-approximately-equals"))?;
  def_math_sym("\\nsim", "\u{2241}", Some("RELOP"), Some("not-similar-to"))?; // NOTE TILDE
  def_math_sym("\\nshortmid", "\u{2224}", Some("RELOP"), Some("not-divides"))?; // DOES NOT DIVIDE; Note: not short!
  def_math_sym("\\nmid", "\u{2224}", Some("RELOP"), Some("not-divides"))?; // DOES NOT DIVIDE
  def_math_sym("\\nvdash", "\u{22AC}", Some("RELOP"), Some("not-proves"))?;
  def_math_sym("\\nVdash", "\u{22AE}", Some("RELOP"), Some("not-forces"))?;
  def_math_sym("\\ntriangleleft", "\u{22EA}", Some("RELOP"), Some("not-subgroup-of"))?;
  def_math_sym("\\ntrianglelefteq", "\u{22EC}", Some("RELOP"), Some("not-subgroup-of-nor-equals"))?;
  def_math_sym("\\nsubseteq", "\u{2288}", Some("RELOP"), Some("not-subset-of-nor-equals"))?;
  def_math_sym("\\nsubseteqq", "\u{2AC5}\u{0338}", Some("RELOP"), Some("not-subset-of-nor-equals"))?;
  def_math_sym("\\subsetneq", "\u{228A}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\varsubsetneq", "\u{228A}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\subsetneqq", "\u{2ACB}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\varsubsetneqq", "\u{2ACB}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\supsetneq", "\u{228B}", Some("RELOP"), Some("superset-of-and-not-equals"))?;
  def_math_sym("\\varsupsetneq", "\u{228B}", Some("RELOP"), Some("superset-of-and-not-equals"))?;
  def_math_sym("\\supsetneqq", "\u{2ACC}", Some("RELOP"), Some("superset-of-and-not-equals"))?;
  def_math_sym("\\varsupsetneqq", "\u{2ACC}", Some("RELOP"), Some("superset-of-and-not-equals"))?;

  def_math_sym("\\ngtr", "\u{226F}", Some("RELOP"), Some("not-greater-than"))?;
  def_math_sym("\\ngeq", "\u{2271}", Some("RELOP"), Some("not-greater-than-nor-equals"))?;
  def_math_sym("\\ngeqslant", "\u{2A7E}\u{0338}", Some("RELOP"), Some("not-greater-than-nor-equals"))?;
  def_math_sym("\\ngeqq", "\u{2267}\u{0338}", Some("RELOP"), Some("not-greater-than-nor-equals"))?;
  def_math_sym("\\gneq", "\u{2A88}", Some("RELOP"), Some("greater-than-and-not-equals"))?;
  def_math_sym("\\gneqq", "\u{2269}", Some("RELOP"), Some("greater-than-and-not-equals"))?;
  def_math_sym("\\gvertneqq", "\u{2269}", Some("RELOP"), Some("greater-than-and-not-equals"))?;
  def_math_sym("\\gnsim", "\u{22E7}", Some("RELOP"), Some("greater-than-and-not-equivalent-to"))?;
  def_math_sym("\\gnapprox", "\u{2A8A}", Some("RELOP"), Some("greater-than-and-not-approximately-equals"))?;
  def_math_sym("\\nsucc", "\u{2281}", Some("RELOP"), Some("not-succeeds"))?;
  def_math_sym("\\nsucceq", "\u{22E1}", Some("RELOP"), Some("not-succeeds-nor-equals"))?;
  def_math_sym("\\succneqq", "\u{2AB6}", Some("RELOP"), Some("succeeds-and-not-equals"))?;
  def_math_sym("\\succnsim", "\u{22E9}", Some("RELOP"), Some("succeeds-and-not-equivalent-to"))?;
  def_math_sym("\\succnapprox", "\u{2ABA}", Some("RELOP"), Some("succeeds-and-not-approximately-equals"))?;
  def_math_sym("\\ncong", "\u{2247}", Some("RELOP"), Some("not-approximately-equals"))?;
  def_math_sym("\\nshortparallel", "\u{2226}", Some("RELOP"), Some("not-parallel-to"))?;
  def_math_sym("\\nparallel", "\u{2226}", Some("RELOP"), Some("not-parallel-to"))?;
  def_math_sym("\\nvDash", "\u{22AD}", Some("RELOP"), None)?; // NOT TRUE
  def_math_sym("\\nVDash", "\u{22AF}", Some("RELOP"), None)?; // NEGATED DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE
  def_math_sym("\\ntriangleright", "\u{22EB}", Some("RELOP"), Some("not-contains"))?;
  def_math_sym("\\ntrianglerighteq", "\u{22ED}", Some("RELOP"), Some("not-contains-nor-equals"))?;
  def_math_sym("\\nsupseteq", "\u{2289}", Some("RELOP"), Some("not-superset-of-nor-equals"))?;
  def_math_sym("\\nsupseteqq", "\u{2AC6}\u{0338}", Some("RELOP"), Some("not-superset-of-nor-equals"))?;

  //======================================================================
  // Arrows
  def_math_sym("\\leftleftarrows", "\u{21C7}", Some("ARROW"), None)?; // LEFTWARDS PAIRED ARROWS
  def_math_sym("\\leftrightarrows", "\u{21C6}", Some("ARROW"), None)?; // LEFTWARDS ARROW OVER RIGHTWARDS ARROW
  def_math_sym("\\Lleftarrow", "\u{21DA}", Some("ARROW"), None)?; // LEFTWARDS TRIPLE ARROW
  def_math_sym("\\twoheadleftarrow", "\u{219E}", Some("ARROW"), None)?; // LEFTWARDS TWHO HEADED ARROW
  def_math_sym("\\leftarrowtail", "\u{21A2}", Some("ARROW"), None)?; // LEFTWARDS ARROW WITH TAIL
  def_math_sym("\\looparrowleft", "\u{21AB}", Some("ARROW"), None)?; // leftwards arrow with loop
  def_math_sym("\\leftrightharpoons", "\u{21CB}", Some("ARROW"), None)?; // LEFTWARDS HARPOON OVER RIGHTWARDS HARPOON
  def_math_sym("\\curvearrowleft", "\u{21B6}", Some("ARROW"), None)?; // ANTICLOCKWISE TOP SEMICIRCLE ARROW
  def_math_sym("\\circlearrowleft", "\u{21BA}", Some("ARROW"), None)?; // ANTICLOCKWISE OPEN CIRCLE ARROW
  def_math_sym("\\Lsh", "\u{21B0}", Some("ARROW"), None)?; // UPWAARDS ARROW WITH TIP LEFTWARDS
  def_math_sym("\\upuparrows", "\u{21C8}", Some("ARROW"), None)?; // UPWARDS PAIRED ARROWS
  def_math_sym("\\upharpoonleft", "\u{21BF}", Some("ARROW"), None)?; // UPWARDS HARPOON WITH BARB LEFTWARDS
  def_math_sym("\\rightrightarrows", "\u{21C9}", Some("ARROW"), None)?; // RIGHTWARDS PAIRED ARROWS
  def_math_sym("\\rightleftarrows", "\u{21C4}", Some("ARROW"), None)?; // RIGHTWARDS ARROW OVER LEFTWARD ARROW
  def_math_sym("\\Rrightarrow", "\u{21DB}", Some("ARROW"), None)?; // RIGHTWARDS TRIPLE ARROW
  def_math_sym("\\twoheadrightarrow", "\u{21A0}", Some("ARROW"), None)?; // RIGHTWARDS TWO HEADED ARROW
  def_math_sym("\\rightarrowtail", "\u{21A3}", Some("ARROW"), None)?; // RIGHTWARDS ARROW WITH TAIL
  def_math_sym("\\looparrowright", "\u{21AC}", Some("ARROW"), None)?; // RIGHTWARDS ARROW WITH LOOP

  // \rightleftharpoons  21CC # RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON ; in amsfonts

  def_math_sym("\\curvearrowright", "\u{21B7}", Some("ARROW"), None)?; // CLOCKWISE TOP SEMICIRCLE ARROW
  def_math_sym("\\circlearrowright", "\u{21BB}", Some("ARROW"), None)?; // CLOCKWISE OPEN CIRCLE ARROW
  def_math_sym("\\Rsh", "\u{21B1}", Some("ARROW"), None)?; // UPWAARDS ARROW WITH TIP RIGHTWARDS
  def_math_sym("\\downdownarrows", "\u{21CA}", Some("ARROW"), None)?; // DOWNWARDS PAIRED ARROWS
  def_math_sym("\\upharpoonright", "\u{21BE}", Some("ARROW"), None)?; // UPWARDS HARPOON WITH BARB RIGHTWARDS
  def_math_sym("\\restriction", "\u{21BE}", Some("ARROW"), None)?; // UPWARDS HARPOON WITH BARB RIGHTWARDS
  // (same as \upharpoonright)
  def_math_sym("\\downharpoonleft", "\u{21C3}", Some("ARROW"), None)?; // DOWNWARDS HARPOON WITH BARB LEFTWARDS
  def_math_sym("\\multimap", "\u{22B8}", Some("ARROW"), None)?; // MULTIMAP
  def_math_sym("\\leftrightsquigarrow", "\u{21AD}", Some("ARROW"), None)?; // LEFT RIGHT WAVE ARROW
  def_math_sym("\\downharpoonright", "\u{21C2}", Some("ARROW"), None)?; // DOWNWARDS HARPOON WITH BARB RIGHTWARDS
  // \rightsquigarrow amsfonts

  //======================================================================
  // Negated arrows
  def_math_sym("\\nleftarrow", "\u{219A}", Some("ARROW"), None)?; // LEFTWARDS ARROW WITH STROKE
  def_math_sym("\\nLeftarrow", "\u{21CD}", Some("ARROW"), None)?; // LEFTWARDS DOUBLE ARROW WITH STROKE
  def_math_sym("\\nleftrightarrow", "\u{21AE}", Some("ARROW"), None)?; // LEFT RIGHT ARROW WITH STROKE
  def_math_sym("\\nrightarrow", "\u{219B}", Some("ARROW"), None)?; // RIGHTWARDS ARROW WITH STROKE
  def_math_sym("\\nRightarrow", "\u{21CF}", Some("ARROW"), None)?; // LEFTWARDS DOUBLE ARROW WITH STROKE
  def_math_sym("\\nLeftrightarrow", "\u{21CE}", Some("ARROW"), None)?; // LEFT RIGHT DOUBLE ARROW WITH STROKE

  //======================================================================
});
