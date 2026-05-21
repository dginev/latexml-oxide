use crate::prelude::*;

/// Runtime helper for the trivial `DefMath!` shape used 170+ times in
/// mathabx (DEP-17b, mirrors DEP-15/DEP-17 approach). The macro arm
/// expands at compile time into ~1.1 KiB of `.text` per invocation;
/// routing through this helper drops `load_definitions` size at the
/// cost of a runtime `parse_prototype` call per entry — paid once at
/// engine bootstrap.
fn def_math_sym(cs: &str, present: &str, role: Option<&str>, meaning: Option<&str>) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let mut opts = MathPrimitiveOptions::default();
  if let Some(r) = role { opts.role = Some(r.to_string()); }
  if let Some(m) = meaning { opts.meaning = Some(m.to_string()); }
  def_math(cs_tok, params, present.to_string(), opts)?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // Specials  (matha/mathb)
  // These are intended to overlay to show negation,
  // but they're not going to work well for that.
  def_math_sym("\\notsign", "|", Some("OPERATOR"), Some("not"))?;
  def_math_sym("\\varnotsign", "/", Some("OPERATOR"), Some("not"))?;
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
  def_math_sym("\\asterisk", "\u{2217}", Some("MULOP"), None)?;
  // DefMath('\coasterisk',Tokens());
  def_math_sym("\\ltimes", "\u{22C9}", Some("MULOP"), Some("left-normal-factor-semidirect-product"))?;
  def_math_sym("\\rtimes", "\u{22CA}", Some("MULOP"), Some("right-normal-factor-semidirect-product"))?;
  //   \diamond, \bullet
  //   \star
  DefMath!("\\varstar", None, "\u{2736}", role => "MULOP");
  // Next two probably text style or small size?
  DefMath!("\\ssum",  None, "\u{2211}", role => "SUMOP", meaning => "sum");
  DefMath!("\\sprod", None, "\u{220F}", role => "SUMOP", meaning => "product");
  //   \amalg

  //======================================================================
  // Unusual binary operators (mathb)
  def_math_sym("\\dotplus", "\u{2214}", Some("ADDOP"), None)?;
  def_math_sym("\\dotdiv", "\u{2238}", Some("MULOP"), None)?;
  def_math_sym("\\dottimes", "\u{2A30}", Some("MULOP"), None)?;
  def_math_sym("\\divdot", "\u{2A2A}", Some("MULOP"), None)?;
  def_math_sym("\\udot", "\u{22C5}", Some("MULOP"), None)?;    // Same as \cdot, but should shift to left
  def_math_sym("\\square", "\u{25A1}", Some("MULOP"), None)?;
  def_math_sym("\\Asterisk", "\u{273D}", Some("MULOP"), None)?;
  def_math_sym("\\bigast", "\u{273D}", Some("MULOP"), None)?;
  // DefMath('\coAsterisk',Tokens());
  // DefMath('\bigcoast',Tokens());
  def_math_sym("\\circplus", "\u{2A22}", Some("MULOP"), None)?;
  def_math_sym("\\pluscirc", "\u{2295}", Some("MULOP"), None)?;    // Not quite right glyph
  def_math_sym("\\convolution", "\u{2733}", Some("MULOP"), None)?;
  def_math_sym("\\divideontimes", "\u{22C7}", Some("MULOP"), None)?;
  def_math_sym("\\blackdiamond", "\u{25C6}", Some("MULOP"), None)?;
  def_math_sym("\\sqbullet", "\u{2BC0}", Some("MULOP"), None)?;
  def_math_sym("\\bigstar", "\u{1F7CA}", Some("MULOP"), None)?;
  def_math_sym("\\bigvarstar", "\u{1F7CC}", Some("MULOP"), None)?;

  //======================================================================
  // Usual relations (matha)
  //   =, \equiv
  //   \sim, \approx
  //   \simeq, \cong
  //   \asymp
  def_math_sym("\\divides", "\u{2223}", Some("RELOP"), None)?;
  //   \neq, \ne,
  def_math_sym("\\nequiv", "\u{2262}", Some("RELOP"), Some("not-equivalent-to"))?;
  Let!("\\notequiv", "\\nequiv");
  def_math_sym("\\nsim", "\u{2241}", Some("RELOP"), Some("not-similar-to"))?;
  def_math_sym("\\napprox", "\u{2249}", Some("RELOP"), Some("not-approximately-equals"))?;
  def_math_sym("\\nsimeq", "\u{2243}\u{0338}", Some("RELOP"), Some("not-equivalent-to-nor-equals"))?;
  def_math_sym("\\ncong", "\u{2247}", Some("RELOP"), Some("not-approximately-equals"))?;
  def_math_sym("\\notasymp", "\u{226D}", Some("RELOP"), Some("not-equivalent-to"))?;
  def_math_sym("\\notdivides", "\u{2224}", Some("RELOP"), Some("does-not-divide"))?;

  //======================================================================
  // Unusual relations (mathb)
  def_math_sym("\\topdoteq", "=\u{0307}", Some("RELOP"), None)?;    // = combining dot
  def_math_sym("\\botdoteq", "\u{2A66}", Some("RELOP"), None)?;
  def_math_sym("\\doteqdot", "\u{2251}", Some("RELOP"), Some("geometrically-equals"))?;
  Let!("\\dotseq", "\\doteqdot");
  Let!("\\Doteq",  "\\doteqdot");
  def_math_sym("\\risingdotseq", "\u{2253}", Some("RELOP"), Some("image-of-or-approximately-equals"))?;
  def_math_sym("\\fallingdotseq", "\u{2252}", Some("RELOP"), Some("approximately-equals-or-image-of"))?;
  def_math_sym("\\coloneq", "\u{2254}", Some("RELOP"), None)?;
  def_math_sym("\\eqcolon", "\u{2255}", Some("RELOP"), None)?;
  def_math_sym("\\bumpedeq", "\u{224F}", Some("RELOP"), Some("difference-between"))?;
  // DefMath('\eqbumped',Tokens());
  def_math_sym("\\Bumpedeq", "\u{224E}", Some("RELOP"), Some("geometrically-equals"))?;
  def_math_sym("\\circeq", "\u{2257}", Some("RELOP"), None)?;
  def_math_sym("\\eqcirc", "\u{2256}", Some("RELOP"), None)?;
  def_math_sym("\\triangleq", "\u{225C}", Some("RELOP"), None)?;
  def_math_sym("\\corresponds", "\u{2258}", Some("RELOP"), Some("corresponds-to"))?;

  //======================================================================
  // Miscellaneous (matha)
  //   \neq, \lnot, \ll
  //   \gg,
  def_math_sym("\\hash", "#", Some("RELOP"), None)?;
  //  \vdash, \dashv
  def_math_sym("\\nvdash", "\u{22AC}", Some("RELOP"), None)?;
  def_math_sym("\\ndashv", "\u{22A3}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\vDash", "\u{22A8}", Some("RELOP"), None)?;
  def_math_sym("\\Dashv", "\u{2AE4}", Some("RELOP"), None)?;
  def_math_sym("\\nvDash", "\u{22AD}", Some("RELOP"), None)?;
  def_math_sym("\\nDashv", "\u{2AE4}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\Vdash", "\u{22A9}", Some("RELOP"), Some("forces"))?;
  def_math_sym("\\dashV", "\u{2AE3}", Some("RELOP"), None)?;
  def_math_sym("\\nVdash", "\u{22AE}", Some("RELOP"), Some("not-forces"))?;
  def_math_sym("\\ndashV", "\u{2AE3}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\degree", "\u{00B0}", Some("RELOP"), None)?;
  //   \prime
  def_math_sym("\\second", "\u{02BA}", Some("RELOP"), None)?;
  def_math_sym("\\third", "\u{2034}", Some("RELOP"), None)?;
  def_math_sym("\\fourth", "\u{2057}", Some("RELOP"), None)?;
  //   \flat
  //   \natural, \sharp
  //   \infty, \propto
  //   \dagger, \ddagger

  //======================================================================
  // Miscellaneous (mathb)
  def_math_sym("\\between", "\u{226C}", Some("RELOP"), Some("between"))?;
  //   \smile
  //   \frown
  def_math_sym("\\varhash", "#", Some("RELOP"), None)?;
  def_math_sym("\\leftthreetimes", "\u{22CB}", Some("MULOP"), Some("left-semidirect-product"))?;
  def_math_sym("\\rightthreetimes", "\u{22CC}", Some("MULOP"), Some("right-semidirect-product"))?;
  def_math_sym("\\pitchfork", "\u{22D4}", Some("RELOP"), Some("proper-intersection"))?;
  //  \bowtie, \Join
  def_math_sym("\\VDash", "\u{22AB}", Some("RELOP"), None)?;
  def_math_sym("\\DashV", "\u{2AE5}", Some("RELOP"), None)?;
  def_math_sym("\\nVDash", "\u{22AF}", Some("RELOP"), None)?;
  def_math_sym("\\nDashV", "\u{2AE5}\u{0338}", Some("RELOP"), None)?;
  def_math_sym("\\Vvdash", "\u{22AA}", Some("RELOP"), None)?;
  // Note that the above can be mirrored, but that doesn't quite help \dashVv!
  def_math_sym("\\nVvash", "\u{22AA}\u{0338}", Some("RELOP"), None)?;
  // DefMath('\ndashVv',Tokens());
  def_math_sym("\\therefore", "\u{2234}", Some("METARELOP"), Some("therefore"))?;
  def_math_sym("\\because", "\u{2235}", Some("METARELOP"), Some("because"))?;
  DefMath!("\\ring{}", "\u{030A}", operator_role => "OVERACCENT");
  //   \dot
  //   \ddot,
  DefMath!("\\dddot{}",  "\u{02D9}\u{02D9}\u{02D9}",         operator_role => "OVERACCENT");
  DefMath!("\\ddddot{}", "\u{02D9}\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");
  //   \angle
  DefMath!("\\measuredangle",  "\u{2221}");
  def_math_sym("\\sphericalangle", "\u{2222}", None, None)?;
  DefMath!("\\rip",            "\u{26FC}");    // Not quite the right glyph

  //======================================================================
  // Delimiters as symbols (matha)
  // (,)
  // [,]
  // \setminus, /
  // |, \mid

  //======================================================================
  // Delimiters as symbols (mathb)
  def_math_sym("\\ulcorner", "\u{231C}", None, None)?;
  def_math_sym("\\urcorner", "\u{231D}", None, None)?;
  def_math_sym("\\llcorner", "\u{231E}", None, None)?;
  def_math_sym("\\lrcorner", "\u{231F}", None, None)?;

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
  def_math_sym("\\partialslash", "\u{2202}\u{0338}", Some("OPERATOR"), None)?;
  //  \exists,
  def_math_sym("\\nexists", "\u{2204}", Some("FUNCTION"), Some("not-exists"))?;
  DefMath!("\\Finv",    "\u{2132}");
  DefMath!("\\Game",    "\u{2141}");
  //   \emptyset,
  def_math_sym("\\diameter", "\u{2300}", None, None)?;
  //   \top, \bot
  //   \perp,
  def_math_sym("\\nottop", "\u{22A4}\u{0338}", Some("ADDOP"), Some("not-top"))?;
  def_math_sym("\\notbot", "\u{22A5}\u{0338}", Some("ADDOP"), Some("not-bottom"))?;
  def_math_sym("\\notperp", "\u{27C2}\u{0338}", Some("RELOP"), Some("not-perpendicular-to"))?;
  def_math_sym("\\curlywedge", "\u{22CF}", Some("ADDOP"), Some("and"))?;
  def_math_sym("\\curlyvee", "\u{22CE}", Some("ADDOP"), Some("or"))?;
  //   \in, \owns
  //   \notin
  def_math_sym("\\notowner", "\u{220C}", Some("RELOP"), Some("not-contains"))?;
  Let!("\\notni",       "\\notowner");
  Let!("\\notowns",     "\\notowner");
  Let!("\\varnotin",    "\\notin");
  Let!("\\varnotowner", "\\notowner");
  def_math_sym("\\barin", "\u{22F6}", Some("ADDOP"), Some("element-of-with-overbar"))?;
  def_math_sym("\\ownsbar", "\u{22F8}", Some("ADDOP"), Some("element-of-with-underbar"))?;
  //  \cap, \cup
  //  \uplus, \sqcap
  //  \sqcup, \squplus
  //  \wedge, \and, \vee, \lor

  //======================================================================
  // Letter-like symbols (mathb)
  def_math_sym("\\barwedge", "\u{22BC}", Some("ADDOP"), Some("not-and"))?;
  def_math_sym("\\veebar", "\u{22BB}", Some("ADDOP"), Some("exclusive-or"))?;
  def_math_sym("\\doublebarwedge", "\u{2A5E}", Some("ADDOP"), None)?;
  def_math_sym("\\veedoublebar", "\u{2A63}", Some("ADDOP"), None)?;
  def_math_sym("\\doublecap", "\u{22D2}", Some("ADDOP"), Some("double-intersection"))?;
  def_math_sym("\\doublecup", "\u{22D3}", Some("ADDOP"), Some("double-union"))?;
  def_math_sym("\\sqdoublecap", "\u{2A4E}", Some("ADDOP"), Some("double-square-intersection"))?;
  def_math_sym("\\sqdoublecup", "\u{2A4F}", Some("ADDOP"), Some("double-square-union"))?;

  //======================================================================
  //  Subset's and superset's signs (matha)
  //  \subset, \supset
  def_math_sym("\\nsubset", "\u{2284}", Some("RELOP"), Some("not-subset-of"))?;
  def_math_sym("\\nsupset", "\u{2285}", Some("RELOP"), Some("not-superset-of"))?;
  //  \subseteq, \supseteq
  def_math_sym("\\nsubseteq", "\u{2288}", Some("RELOP"), Some("not-subset-of-nor-equals"))?;
  def_math_sym("\\nsupseteq", "\u{2289}", Some("RELOP"), Some("not-superset-of-nor-equals"))?;
  def_math_sym("\\subsetneq", "\u{228A}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\supsetneq", "\u{228B}", Some("RELOP"), Some("superset-of-and-not-equals"))?;
  def_math_sym("\\varsubsetneq", "\u{228A}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\varsupsetneq", "\u{228B}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\subseteqq", "\u{2AC5}", Some("RELOP"), Some("subset-of-or-equals"))?;
  def_math_sym("\\supseteqq", "\u{2AC6}", Some("RELOP"), Some("superset-of-or-equals"))?;
  def_math_sym("\\nsubseteqq", "\u{2AC5}\u{0338}", Some("RELOP"), Some("not-subset-of-nor-equals"))?;
  def_math_sym("\\nsupseteqq", "\u{2AC6}\u{0338}", Some("RELOP"), Some("not-superset-of-nor-equals"))?;
  def_math_sym("\\subsetneqq", "\u{2ACB}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\supsetneqq", "\u{2ACC}", Some("RELOP"), Some("superset-of-and-not-equals"))?;
  def_math_sym("\\varsubsetneqq", "\u{2ACB}", Some("RELOP"), Some("subset-of-and-not-equals"))?;
  def_math_sym("\\varsupsetneqq", "\u{2ACC}", Some("RELOP"), Some("superset-of-and-not-equals"))?;
  def_math_sym("\\Subset", "\u{22D0}", Some("RELOP"), Some("double-subset-of"))?;
  def_math_sym("\\Supset", "\u{22D1}", Some("RELOP"), Some("double-superset-of"))?;
  def_math_sym("\\nSubset", "\u{22D0}\u{0338}", Some("RELOP"), Some("not-double-subset-of"))?;
  def_math_sym("\\nSupset", "\u{22D1}\u{0338}", Some("RELOP"), Some("not-double-superset-of"))?;

  //======================================================================
  // Square Subset's and superset's signs (mathb)
  //  \sqsubset, \sqsupset
  def_math_sym("\\nsqsubset", "\u{228F}\u{0338}", Some("RELOP"), Some("not-square-image-of"))?;
  def_math_sym("\\nsqsupset", "\u{2290}\u{0338}", Some("RELOP"), Some("not-square-original-of"))?;
  //  \sqsubseteq, \sqsupseteq
  def_math_sym("\\nsqsubseteq", "\u{22E2}", Some("RELOP"), Some("not-square-image-of-nor-equals"))?;
  def_math_sym("\\nsqsupseteq", "\u{22E3}", Some("RELOP"), Some("not-square-original-of-nor-equals"))?;
  def_math_sym("\\sqsubsetneq", "\u{22E4}", Some("RELOP"), Some("square-image-of-or-not-equals"))?;
  def_math_sym("\\sqsupsetneq", "\u{22E5}", Some("RELOP"), Some("square-original-of-or-not-equals"))?;
  Let!("\\varsqsubsetneq", "\\sqsubsetneq");
  Let!("\\varsqsupsetneq", "\\sqsupsetneq");
  // Pretty crummy, using underline
  def_math_sym("\\sqsubseteqq", "\u{228F}\u{0333}", Some("RELOP"), Some("square-image-of-or-equals"))?;
  def_math_sym("\\sqsupseteqq", "\u{2290}\u{0333}", Some("RELOP"), Some("square-original-of-or-equals"))?;
  def_math_sym("\\nsqsubseteqq", "\u{228F}\u{0333}\u{0338}", Some("RELOP"), Some("not-square-image-of-nor-equals"))?;
  def_math_sym("\\nsqsupseteqq", "\u{2290}\u{0333}\u{0338}", Some("RELOP"), Some("not-square-original-of-nor-equals"))?;

  //======================================================================
  // Triangles as relations (matha)
  //  \triangleleft,
  DefMath!("\\vartriangleleft",  "\u{22B2}");    // NORMAL SUBGROUP OF (\lhd)
  // \triangleright
  def_math_sym("\\vartriangleright", "\u{22B3}", None, None)?;    // CONTAINS AS NORMAL SUBGROUP (\rhd)
  def_math_sym("\\ntriangleleft", "\u{22EA}", Some("RELOP"), Some("not-subgroup-of"))?;
  def_math_sym("\\ntriangleright", "\u{22EB}", Some("RELOP"), Some("not-contains"))?;
  DefMath!("\\trianglelefteq",   "\u{22B4}");    // NORMAL SUBGROUP OF OR EQUAL TO (\unlhd)
  DefMath!("\\trianglerighteq",  "\u{22B5}");    // CONTAINS AS NORMAL SUBGROUP OR EQUAL TO (\unrhd)
  def_math_sym("\\ntrianglelefteq", "\u{22EC}", Some("RELOP"), Some("not-subgroup-of-nor-equals"))?;
  def_math_sym("\\ntrianglerighteq", "\u{22ED}", Some("RELOP"), Some("not-contains-nor-equals"))?;

  //======================================================================
  // Triangles as binary operators (mathb)
  def_math_sym("\\smalltriangleup", "\u{25B5}", Some("RELOP"), None)?;
  def_math_sym("\\smalltriangledown", "\u{25BF}", Some("RELOP"), None)?;
  def_math_sym("\\smalltriangleleft", "\u{25C3}", Some("RELOP"), None)?;
  def_math_sym("\\smalltriangleright", "\u{25B9}", Some("RELOP"), None)?;
  def_math_sym("\\blacktriangleup", "\u{25B4}", Some("RELOP"), None)?;
  def_math_sym("\\blacktriangledown", "\u{25BE}", Some("RELOP"), None)?;
  def_math_sym("\\blacktriangleleft", "\u{25C2}", Some("RELOP"), None)?;
  def_math_sym("\\blacktriangleright", "\u{25B8}", Some("RELOP"), None)?;

  //======================================================================
  // Inequalities (matha)
  //  <, >
  def_math_sym("\\nless", "\u{226E}", Some("RELOP"), Some("not-less-than"))?;
  def_math_sym("\\ngtr", "\u{226F}", Some("RELOP"), Some("not-greater-than"))?;
  //   \leq, \geq (\leqslant, \qeqslant)
  def_math_sym("\\nleq", "\u{2270}", Some("RELOP"), Some("not-less-than-nor-greater-than"))?;
  def_math_sym("\\ngeq", "\u{2271}", Some("RELOP"), Some("not-greater-than-nor-equals"))?;
  Let!("\\varleq",  "\\leq");
  Let!("\\vargeq",  "\\geq");
  Let!("\\nvarleq", "\\nleq");
  Let!("\\nvargeq", "\\ngeq");
  def_math_sym("\\lneq", "\u{2A87}", Some("RELOP"), Some("less-than-and-not-equals"))?;
  def_math_sym("\\gneq", "\u{2A88}", Some("RELOP"), Some("greater-than-and-not-equals"))?;
  def_math_sym("\\leqq", "\u{2266}", Some("RELOP"), Some("less-than-or-equals"))?;
  def_math_sym("\\geqq", "\u{2267}", Some("RELOP"), Some("greater-than-or-equals"))?;
  def_math_sym("\\nleqq", "\u{2266}\u{0338}", Some("RELOP"), Some("not-less-than-nor-equals"))?;
  def_math_sym("\\ngeqq", "\u{2267}\u{0338}", Some("RELOP"), Some("not-greater-than-nor-equals"))?;
  def_math_sym("\\lneqq", "\u{2268}", Some("RELOP"), Some("less-than-and-not-equals"))?;
  def_math_sym("\\gneqq", "\u{2269}", Some("RELOP"), Some("greater-than-and-not-equals"))?;
  def_math_sym("\\lvertneqq", "\u{2268}", Some("RELOP"), Some("less-than-and-not-equals"))?;
  def_math_sym("\\gvertneqq", "\u{2269}", Some("RELOP"), Some("greater-than-and-not-equals"))?;
  def_math_sym("\\eqslantless", "\u{2A95}", Some("RELOP"), Some("less-than-or-equals"))?;
  def_math_sym("\\eqslantgtr", "\u{2A96}", Some("RELOP"), Some("greater-than-or-equals"))?;
  def_math_sym("\\neqslantless", "\u{2A95}\u{0338}", Some("RELOP"), Some("not-less-than-nor-equals"))?;
  def_math_sym("\\neqslantgtr", "\u{2A96}\u{0338}", Some("RELOP"), Some("not-greater-than-nor-equals"))?;
  def_math_sym("\\lessgtr", "\u{2276}", Some("RELOP"), Some("less-than-or-greater-than"))?;
  def_math_sym("\\gtrless", "\u{2277}", Some("RELOP"), Some("greater-than-or-less-than"))?;
  def_math_sym("\\lesseqgtr", "\u{22DA}", Some("RELOP"), Some("less-than-or-equals-or-greater-than"))?;
  def_math_sym("\\gtreqless", "\u{22DB}", Some("RELOP"), Some("greater-than-or-equals-or-less-than"))?;
  def_math_sym("\\lesseqqgtr", "\u{2A8B}", Some("RELOP"), Some("less-than-or-equals-or-greater-than"))?;
  def_math_sym("\\gtreqqless", "\u{2A8C}", Some("RELOP"), Some("greater-than-or-equals-or-less-than"))?;
  def_math_sym("\\lesssim", "\u{2272}", Some("RELOP"), Some("less-than-or-similar-to"))?;
  def_math_sym("\\gtrsim", "\u{2273}", Some("RELOP"), Some("greater-than-or-equivalent-to"))?;
  def_math_sym("\\nlesssim", "\u{2272}\u{0338}", Some("RELOP"), Some("not-less-than-nor-similar-to"))?;
  def_math_sym("\\ngtrsim", "\u{2273}\u{0338}", Some("RELOP"), Some("not-greater-than-nor-equivalent-to"))?;
  def_math_sym("\\lnsim", "\u{22E6}", Some("RELOP"), Some("less-than-and-not-equivalent-to"))?;
  def_math_sym("\\gnsim", "\u{22E7}", Some("RELOP"), Some("greater-than-and-not-equivalent-to"))?;
  def_math_sym("\\lessapprox", "\u{2A85}", Some("RELOP"), Some("less-than-or-approximately-equals"))?;
  def_math_sym("\\gtrapprox", "\u{2A86}", Some("RELOP"), Some("greater-than-or-approximately-equals"))?;
  def_math_sym("\\nlessapprox", "\u{2A85}\u{0338}", Some("RELOP"), Some("not-less-than-nor-approximately-equals"))?;
  def_math_sym("\\ngtrapprox", "\u{2A86}\u{0338}", Some("RELOP"), Some("not-greater-than-nor-approximately-equals"))?;
  def_math_sym("\\lnapprox", "\u{2A89}", Some("RELOP"), Some("less-than-and-not-approximately-equals"))?;
  def_math_sym("\\gnapprox", "\u{2A8A}", Some("RELOP"), Some("greater-than-and-not-approximately-equals"))?;
  def_math_sym("\\lessdot", "\u{22D6}", Some("RELOP"), None)?;
  def_math_sym("\\gtrdot", "\u{22D7}", Some("RELOP"), None)?;
  def_math_sym("\\lll", "\u{22D8}", Some("RELOP"), Some("very-much-less-than"))?;
  def_math_sym("\\ggg", "\u{22D9}", Some("RELOP"), Some("very-much-greater-than"))?;
  def_math_sym("\\precdot", "\u{22D6}", Some("RELOP"), None)?;    // glyph is for less with dot!
  def_math_sym("\\succdot", "\u{22D7}", Some("RELOP"), None)?;    // gtr with dot!

  //======================================================================
  // Inequalities (mathb)
  // Sometimes using \x{0338} to negate (which is slash, but should use vertical?)
  //  \prec, \succ
  def_math_sym("\\nprec", "\u{2280}", Some("RELOP"), Some("not-precedes"))?;
  def_math_sym("\\nsucc", "\u{2281}", Some("RELOP"), Some("not-succeeds"))?;
  def_math_sym("\\preccurlyeq", "\u{227C}", Some("RELOP"), Some("precedes-or-equals"))?;
  def_math_sym("\\succcurlyeq", "\u{227D}", Some("RELOP"), Some("succeeds-or-equals"))?;
  def_math_sym("\\npreccurlyeq", "\u{227C}\u{0338}", Some("RELOP"), Some("not-precedes-nor-equals"))?;
  def_math_sym("\\nsucccurlyeq", "\u{227D}\u{0338}", Some("RELOP"), Some("not-succeeds-nor-equals"))?;
  //  \preceq, succeq
  def_math_sym("\\npreceq", "\u{22E0}", Some("RELOP"), Some("not-precedes-nor-equals"))?;
  def_math_sym("\\nsucceq", "\u{22E1}", Some("RELOP"), Some("not-succeeds-nor-equals"))?;
  def_math_sym("\\precneq", "\u{22E8}", Some("RELOP"), Some("precedes-not-equals"))?;
  def_math_sym("\\succneq", "\u{22E9}", Some("RELOP"), Some("succeeds-not-equals"))?;
  def_math_sym("\\curlyeqprec", "\u{22DE}", Some("RELOP"), Some("equals-or-preceeds"))?;
  def_math_sym("\\curlyeqsucc", "\u{22DF}", Some("RELOP"), Some("equals-or-succeeds"))?;
  def_math_sym("\\ncurlyeqprec", "\u{22DE}\u{0338}", Some("RELOP"), Some("not-equals-nor-preceeds"))?;
  def_math_sym("\\ncurlyeqsucc", "\u{22DF}\u{0338}", Some("RELOP"), Some("not-equals-nor-succeeds"))?;
  def_math_sym("\\precsim", "\u{227E}", Some("RELOP"), Some("precedes-or-equivalent-to"))?;
  def_math_sym("\\succsim", "\u{227F}", Some("RELOP"), Some("succeeds-or-equivalent-to"))?;
  def_math_sym("\\nprecsim", "\u{227E}\u{0338}", Some("RELOP"), Some("not-precedes-nor-equivalent-to"))?;
  def_math_sym("\\nsuccsim", "\u{227F}\u{0338}", Some("RELOP"), Some("not-succeeds-nor-equivalent-to"))?;
  def_math_sym("\\precnsim", "\u{22E8}", Some("RELOP"), Some("precedes-and-not-equivalent-to"))?;
  def_math_sym("\\succnsim", "\u{22E9}", Some("RELOP"), Some("succeeds-and-not-equivalent-to"))?;
  def_math_sym("\\precapprox", "\u{2AB7}", Some("RELOP"), Some("precedes-or-approximately-equals"))?;
  def_math_sym("\\succapprox", "\u{2AB8}", Some("RELOP"), Some("succeeds-or-approximately-equals"))?;
  def_math_sym("\\nprecapprox", "\u{2AB7}\u{0338}", Some("RELOP"), Some("not-precedes-nor-approximately-equals"))?;
  def_math_sym("\\nsuccapprox", "\u{2AB8}\u{0338}", Some("RELOP"), Some("not-succeeds-nor-approximately-equals"))?;
  def_math_sym("\\precnapprox", "\u{2AB9}", Some("RELOP"), Some("precedes-and-not-approximately-equals"))?;
  def_math_sym("\\succnapprox", "\u{2ABA}", Some("RELOP"), Some("succeeds-and-not-approximately-equals"))?;
  def_math_sym("\\llcurly", "\u{2ABB}", Some("RELOP"), Some("double-precedes"))?;
  def_math_sym("\\ggcurly", "\u{2ABC}", Some("RELOP"), Some("double-succeeds"))?;

  //======================================================================
  // Arrows and Harpoons (matha)
  //  \leftarrow, \gets \rightarrow, \to
  //  \nwarrow, \nearrow
  //  \swarrow, \searrow
  //  \leftrightarrow
  def_math_sym("\\nleftarrow", "\u{219A}", Some("ARROW"), None)?;
  def_math_sym("\\nrightarrow", "\u{219B}", Some("ARROW"), None)?;
  def_math_sym("\\nleftrightarrow", "\u{21AE}", Some("ARROW"), None)?;    // LEFT RIGHT ARROW WITH STROKE
  //  \relbar
  //  \mapstochar
  def_math_sym("\\mapsfromchar", "|", Some("RELOP"), None)?;
  //  \leftharpoonup
  //  \rightharpoonup, \leftharpoondown
  //  \rightharpoondown,
  def_math_sym("\\upharpoonleft", "\u{21BF}", Some("ARROW"), None)?;
  def_math_sym("\\downharpoonleft", "\u{21C3}", Some("ARROW"), None)?;
  def_math_sym("\\upharpoonright", "\u{21BE}", Some("ARROW"), None)?;
  def_math_sym("\\restriction", "\u{21BE}", Some("ARROW"), None)?;
  def_math_sym("\\downharpoonright", "\u{21C2}", Some("ARROW"), None)?;
  def_math_sym("\\leftrightharpoons", "\u{21CB}", Some("ARROW"), None)?;
  //  \rightleftharpoons
  def_math_sym("\\updownharpoons", "\u{296E}", Some("ARROW"), None)?;
  def_math_sym("\\downupharpoons", "\u{296F}", Some("ARROW"), None)?;
  //  \Leftarrow, \Rightarrow
  //  \Leftrightarrow,
  def_math_sym("\\nLeftarrow", "\u{21CD}", Some("ARROW"), None)?;
  def_math_sym("\\nRightarrow", "\u{21CF}", Some("ARROW"), None)?;
  def_math_sym("\\nLeftrightarrow", "\u{21CE}", Some("ARROW"), None)?;
  //  \Relbar
  def_math_sym("\\Mapstochar", "|", Some("RELOP"), None)?;
  def_math_sym("\\Mapsfromchar", "|", Some("RELOP"), None)?;

  //======================================================================
  // Arrows and Harpoons (mathb)
  def_math_sym("\\leftleftarrows", "\u{21C7}", Some("ARROW"), None)?;
  def_math_sym("\\rightrightarrows", "\u{21C9}", Some("ARROW"), None)?;
  def_math_sym("\\upuparrows", "\u{21C8}", Some("ARROW"), None)?;
  def_math_sym("\\downdownarrows", "\u{21CA}", Some("ARROW"), None)?;
  def_math_sym("\\leftrightarrows", "\u{21C6}", Some("ARROW"), None)?;
  def_math_sym("\\rightleftarrows", "\u{21C4}", Some("ARROW"), None)?;
  def_math_sym("\\updownarrows", "\u{21C5}", Some("ARROW"), None)?;
  def_math_sym("\\downuparrows", "\u{21F5}", Some("ARROW"), None)?;
  def_math_sym("\\leftleftharpoons", "\u{2962}", Some("ARROW"), None)?;
  def_math_sym("\\rightrightharpoons", "\u{2964}", Some("ARROW"), None)?;
  def_math_sym("\\upupharpoons", "\u{2963}", Some("ARROW"), None)?;
  def_math_sym("\\downdownharpoons", "\u{2965}", Some("ARROW"), None)?;
  def_math_sym("\\leftbarharpoon", "\u{296A}", Some("ARROW"), None)?;
  def_math_sym("\\rightbarharpoon", "\u{296C}", Some("ARROW"), None)?;
  def_math_sym("\\barleftharpoon", "\u{296B}", Some("ARROW"), None)?;
  def_math_sym("\\barrightharpoon", "\u{296D}", Some("ARROW"), None)?;
  def_math_sym("\\leftrightharpoon", "\u{294A}", Some("ARROW"), None)?;
  def_math_sym("\\rightleftharpoon", "\u{294B}", Some("ARROW"), None)?;
  //  \rhook, \lhook
  DefMath!("\\diagup",         "\u{2571}");
  DefMath!("\\diagdown",       "\u{2572}");
  def_math_sym("\\Lsh", "\u{21B0}", Some("ARROW"), None)?;
  def_math_sym("\\Rsh", "\u{21B1}", Some("ARROW"), None)?;
  def_math_sym("\\dlsh", "\u{21B2}", Some("ARROW"), None)?;
  def_math_sym("\\drsh", "\u{21B3}", Some("ARROW"), None)?;
  def_math_sym("\\looparrowleft", "\u{21AB}", Some("ARROW"), None)?;
  def_math_sym("\\looparrowright", "\u{21AC}", Some("ARROW"), None)?;
  def_math_sym("\\curvearrowleft", "\u{21B6}", Some("ARROW"), None)?;
  def_math_sym("\\curvearrowright", "\u{21B7}", Some("ARROW"), None)?;
  def_math_sym("\\curvearrowbotright", "\u{293B}", Some("ARROW"), None)?;
  def_math_sym("\\circlearrowleft", "\u{21BA}", Some("ARROW"), None)?;
  def_math_sym("\\circlearrowright", "\u{21BB}", Some("ARROW"), None)?;
  def_math_sym("\\leftsquigarrow", "\u{21DC}", Some("RELOP"), None)?;
  def_math_sym("\\rightsquigarrow", "\u{219D}", Some("ARROW"), None)?;
  def_math_sym("\\leftrightsquigarrow", "\u{21AD}", Some("ARROW"), None)?;
  def_math_sym("\\lefttorightarrow", "\u{2B8E}", Some("ARROW"), None)?;
  def_math_sym("\\righttoleftarrow", "\u{2B8C}", Some("ARROW"), None)?;
  def_math_sym("\\uptodownarrow", "\u{2B8F}", Some("ARROW"), None)?;
  def_math_sym("\\downtouparrow", "\u{2B8D}", Some("ARROW"), None)?;

  //======================================================================
  // Circles (matha)
  //   Using combining circle \x{20DD} for missing cases, but positioning is bad
  //  \oplus, \ominus (\circleddash)
  //  \otimes
  def_math_sym("\\odiv", "\u{00F7}\u{20DD}", Some("ADDOP"), None)?;
  //  \odot
  DefMath!("\\ocirc",      "\u{229A}");
  def_math_sym("\\oasterisk", "\u{229B}", Some("MULOP"), None)?;
  // DefMath('\ocoasterisk',Tokens());
  def_math_sym("\\oleft", "\u{22A3}\u{20DD}", Some("ADDOP"), None)?;
  def_math_sym("\\oright", "\u{22A2}\u{20DD}", Some("ADDOP"), None)?;
  def_math_sym("\\otop", "\u{22A4}\u{20DD}", Some("ADDOP"), None)?;
  DefMath!("\\obot",   "\u{29BA}");
  DefMath!("\\ovoid",  "\u{25CB}");
  //  \oslash
  DefMath!("\\obackslash",  "\u{29B8}");
  def_math_sym("\\otriangleup", "\u{25B3}\u{20DD}", Some("ADDOP"), None)?;

  //======================================================================
  // Boxes (mathb)
  //   Using combining square \x{20DE} for missing cases, but positioning is bad
  def_math_sym("\\boxplus", "\u{229E}", Some("ADDOP"), None)?;
  def_math_sym("\\boxminus", "\u{229F}", Some("ADDOP"), None)?;
  def_math_sym("\\boxtimes", "\u{22A0}", Some("MULOP"), None)?;
  def_math_sym("\\boxdiv", "\u{00F7}\u{20DE}", Some("ADDOP"), None)?;
  def_math_sym("\\boxdot", "\u{22A1}", Some("MULOP"), None)?;
  def_math_sym("\\boxcirc", "\u{2218}\u{20DE}", Some("ADDOP"), None)?;
  def_math_sym("\\boxasterisk", "\u{29C6}", None, None)?;
  // DefMath('\boxcoasterisk',Tokens());
  def_math_sym("\\boxleft", "\u{22A3}\u{20DE}", Some("ADDOP"), None)?;
  def_math_sym("\\boxright", "\u{22A2}\u{20DE}", Some("ADDOP"), None)?;
  def_math_sym("\\boxtop", "\u{22A4}\u{20DE}", Some("ADDOP"), None)?;
  def_math_sym("\\boxbot", "\u{22A5}\u{20DE}", Some("ADDOP"), None)?;
  DefMath!("\\boxvoid",  "\u{25A1}");
  //  \Box
  DefMath!("\\boxslash",      "\u{29C5}");
  DefMath!("\\boxbackslash",  "\u{29C4}");
  def_math_sym("\\boxtriangleup", "\u{25B3}\u{20DE}", Some("ADDOP"), None)?;

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
