use crate::package::*;

//======================================================================
// TeX Book, Appendix B. p. 358
LoadDefinitions!(state, {
  //----------------------------------------------------------------------
  //  Actually from LaTeX; Table 3.3, Greek, p.41
  //----------------------------------------------------------------------
  DefMathI!("\\alpha", None, "\u{03B1}");
  DefMathI!("\\beta", None, "\u{03B2}");
  DefMathI!("\\gamma", None, "\u{03B3}");
  DefMathI!("\\delta", None, "\u{03B4}");
  DefMathI!("\\epsilon", None, "\u{03F5}");
  DefMathI!("\\varepsilon", None, "\u{03B5}");
  DefMathI!("\\zeta", None, "\u{03B6}");
  DefMathI!("\\eta", None, "\u{03B7}");
  DefMathI!("\\theta", None, "\u{03B8}");
  DefMathI!("\\vartheta", None, "\u{03D1}");
  DefMathI!("\\iota", None, "\u{03B9}");
  DefMathI!("\\kappa", None, "\u{03BA}");
  DefMathI!("\\lambda", None, "\u{03BB}");
  DefMathI!("\\mu", None, "\u{03BC}");
  DefMathI!("\\nu", None, "\u{03BD}");
  DefMathI!("\\xi", None, "\u{03BE}");
  DefMathI!("\\pi", None, "\u{03C0}");
  DefMathI!("\\varpi", None, "\u{03D6}");
  DefMathI!("\\rho", None, "\u{03C1}");
  DefMathI!("\\varrho", None, "\u{03F1}");
  DefMathI!("\\sigma", None, "\u{03C3}");
  DefMathI!("\\varsigma", None, "\u{03C2}");
  DefMathI!("\\tau", None, "\u{03C4}");
  DefMathI!("\\upsilon", None, "\u{03C5}");
  DefMathI!("\\phi", None, "\u{03D5}");
  DefMathI!("\\varphi", None, "\u{03C6}");
  DefMathI!("\\chi", None, "\u{03C7}");
  DefMathI!("\\psi", None, "\u{03C8}");
  DefMathI!("\\omega", None, "\u{03C9}");
  DefMathI!("\\Gamma", None, "\u{0393}");
  DefMathI!("\\Delta", None, "\u{0394}");
  DefMathI!("\\Theta", None, "\u{0398}");
  DefMathI!("\\Lambda", None, "\u{039B}");
  DefMathI!("\\Xi", None, "\u{039E}");
  DefMathI!("\\Pi", None, "\u{03A0}");
  DefMathI!("\\Sigma", None, "\u{03A3}");
  DefMathI!("\\Upsilon", None, "\u{03A5}");
  DefMathI!("\\Phi", None, "\u{03A6}");
  DefMathI!("\\Psi", None, "\u{03A8}");
  DefMathI!("\\Omega", None, "\u{03A9}");

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.7. Miscellaneous Symbols, p.43
  //----------------------------------------------------------------------
  // Some should be differential operators, qualifiers, ...
  DefMathI!("\\aleph", None, "\u{2135}");
  DefMathI!("\\hbar",  None, "\u{210F}", role => "ID", meaning => "Planck-constant-over-2-pi");
  DefMathI!("\\imath", None, "\u{0131}");
  DefMathI!("\\jmath", None, "\u{0237}");
  DefMathI!("\\ell", None, "\u{2113}");
  DefMathI!("\\wp", None, "\u{2118}", meaning => "Weierstrass-p");
  DefMathI!("\\Re", None, "\u{211C}", role    => "OPFUNCTION", meaning => "real-part");
  DefMathI!("\\Im", None, "\u{2111}", role    => "OPFUNCTION", meaning => "imaginary-part");
  DefMathI!("\\mho", None, "\u{2127}");

  DefMathI!("\\prime",    None, "\u{2032}", role => "SUPOP",    locked  => true);
  DefMathI!("\\emptyset", None, "\u{2205}", role => "ID",       meaning => "empty-set");
  DefMathI!("\\nabla",    None, "\u{2207}", role => "OPERATOR");
  DefMathI!("\\surd",     None, "\u{221A}", role => "OPERATOR", meaning => "square-root");
  DefMathI!("\\top",      None, "\u{22A4}", role => "ADDOP",    meaning => "top");
  DefMathI!("\\bot",      None, "\u{22A5}", role => "ADDOP",    meaning => "bottom");
  DefMathI!("\\|", None, "\u{2225}", role => "VERTBAR", name => "||", meaning => "parallel-to");
  DefMathI!("\\angle", None, "\u{2220}");

  // NOTE: This is probably the wrong role.
  // Also, should probably carry info about Binding for OpenMath
  DefMathI!("\\forall", None, "\u{2200}", role => "BIGOP",    meaning => "for-all");
  DefMathI!("\\exists", None, "\u{2203}", role => "BIGOP",    meaning => "exists");
  DefMathI!("\\neg",    None, "\u{00AC}",  role => "FUNCTION", meaning => "not");
  DefMathI!("\\lnot",   None, "\u{00AC}",  role => "FUNCTION", meaning => "not");
  DefMathI!("\\flat", None, "\u{266D}");
  DefMathI!("\\natural", None, "\u{266E}");
  DefMathI!("\\sharp", None, "\u{266F}");
  DefMathI!("\\backslash", None, "\u{005C}", role => "MULOP");
  DefMathI!("\\partial",   None, "\u{2202}", role => "OPERATOR", meaning => "partial-differential");

  DefMathI!("\\infty", None, "\u{221E}", role => "ID", meaning => "infinity");
  DefMathI!("\\Box", None, "\u{25A1}");
  DefMathI!("\\Diamond", None, "\u{25C7}");
  DefMathI!("\\triangle", None, "\u{25B3}");
  DefMathI!("\\clubsuit", None, "\u{2663}");
  DefMathI!("\\diamondsuit", None, "\u{2662}");
  DefMathI!("\\heartsuit", None, "\u{2661}");
  DefMathI!("\\spadesuit", None, "\u{2660}");

  //----------------------------------------------------------------------
  // TODO:
  // DefMathI!("\\smallint", None, "\u{222B}", meaning => "integral", role => "INTOP",
  //   font => { size => 9 }, scriptpos => \&doScriptpos, mathstyle => "text");    // INTEGRAL
  // #----------------------------------------------------------------------
  // # Actually LaTeX; Table 3.8. Variable-sized Symbols, p.44.
  // #----------------------------------------------------------------------
  // sub doScriptpos {
  //   return (LookupValue('font')->getMathstyle eq 'display' ? 'mid' : 'post'); }

  // sub doVariablesizeOp {
  //   return (LookupValue('font')->getMathstyle eq 'display' ? 'display' : 'text'); }

  // DefMathI('\sum', undef, "\x{2211}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'sum',
  //   mathstyle => \&doVariablesizeOp);
  // DefMathI('\prod', undef, "\x{220F}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'product',
  //   mathstyle => \&doVariablesizeOp);
  // DefMathI('\coprod', undef, "\x{2210}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'coproduct',
  //   mathstyle => \&doVariablesizeOp);
  // DefMathI('\int', undef, "\x{222B}",
  //   role      => 'INTOP',
  //   meaning   => 'integral',
  //   mathstyle => \&doVariablesizeOp);
  // DefMathI('\oint', undef, "\x{222E}",
  //   role      => 'INTOP',
  //   meaning   => 'contour-integral',
  //   mathstyle => \&doVariablesizeOp);
  // DefMathI('\bigcap', undef, "\x{22C2}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'intersection',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigcup', undef, "\x{22C3}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'union',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigsqcup', undef, "\x{2294}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'square-union',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigvee', undef, "\x{22C1}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'or',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigwedge', undef, "\x{22C0}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'and',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigodot', undef, "\x{2299}",
  //   role      => 'SUMOP',              #meaning=> ?
  //   scriptpos => \&doScriptpos,
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigotimes', undef, "\x{2297}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'tensor-product',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\bigoplus', undef, "\x{2295}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'direct-sum',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefMathI('\biguplus', undef, "\x{228E}",
  //   role      => 'SUMOP',
  //   scriptpos => \&doScriptpos,
  //   meaning   => 'symmetric-difference',
  //   mathstyle => \&doVariablesizeOp,
  //   font      => { size => 'Big' });
  // DefConstructor('\limits', undef, sub {
  //     my $node = $_[0]->getElement;
  //     $_[0]->setAttribute($_[0]->getLastChildElement($node) || $node, scriptpos => 'mid'); });
  // DefConstructor('\nolimits', undef, sub {
  //     my $node = $_[0]->getElement;
  //     $node = $_[0]->getLastChildElement($node) || $node;
  //     $node->removeAttribute('scriptpos'); });    # default is 'post', so we can just remove the attrib.
  // DefConstructor('\displaylimits', undef, sub {
  //     my ($document, %props) = @_;
  //     my $node = $_[0]->getElement;
  //     $node = $_[0]->getLastChildElement($node) || $node;
  //     if (($props{mathstyle} || 'text') eq 'display') {
  //       $document->setAttribute($node, scriptpos => 'mid'); }
  //     else {
  //       $node->removeAttribute('scriptpos'); } },
  //   properties => sub { (mathstyle => LookupValue('font')->getMathstyle); });

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.4. Binary Operation Symbols, p.42
  //----------------------------------------------------------------------
  DefMathI!("\\pm",    None, "\u{00B1}",  role => "ADDOP", meaning => "plus-or-minus");
  DefMathI!("\\mp",    None, "\u{2213}", role => "ADDOP", meaning => "minus-or-plus");
  DefMathI!("\\times", None, "\u{00D7}",  role => "MULOP", meaning => "times");
  DefMathI!("\\div",   None, "\u{00F7}",  role => "MULOP", meaning => "divide");
  DefMathI!("\\ast",   None, "\u{2217}", role => "MULOP");
  DefMathI!("\\star",  None, "\u{22C6}", role => "MULOP");
  DefMathI!("\\circ",  None, "\u{2218}", role => "MULOP", meaning => "compose");
  DefMathI!("\\bullet", None, "\u{2219}", role => "MULOP");
  DefMathI!("\\cdot",   None, "\u{22C5}", role => "MULOP");
  ////  , meaning=>"inner-product");  that"s pushing it a bit far...

  // Need to classify set operations more carefully....
  DefMathI!("\\cap", None, "\u{2229}", role => "ADDOP", meaning => "intersection");
  DefMathI!("\\cup", None, "\u{222A}", role => "ADDOP", meaning => "union");
  DefMathI!("\\uplus",    None, "\u{228E}", role => "ADDOP");
  DefMathI!("\\sqcap",    None, "\u{2293}", role => "ADDOP", meaning => "square-intersection");
  DefMathI!("\\sqcup",    None, "\u{2294}", role => "ADDOP", meaning => "square-union");
  DefMathI!("\\vee",      None, "\u{2228}", role => "ADDOP", meaning => "or");
  DefMathI!("\\lor",      None, "\u{2228}", role => "ADDOP", meaning => "or");
  DefMathI!("\\wedge",    None, "\u{2227}", role => "ADDOP", meaning => "and");
  DefMathI!("\\land",     None, "\u{2227}", role => "ADDOP", meaning => "and");
  DefMathI!("\\setminus", None, "\u{2216}", role => "ADDOP", meaning => "set-minus");
  DefMathI!("\\wr",       None, "\u{2240}", role => "MULOP");

  // Should this block be ADDOP or something else?
  DefMathI!("\\diamond",         None, "\u{22C4}", role => "ADDOP");
  DefMathI!("\\bigtriangleup",   None, "\u{25B3}", role => "ADDOP");
  DefMathI!("\\bigtriangledown", None, "\u{25BD}", role => "ADDOP");
  DefMathI!("\\triangleleft",    None, "\u{25C1}", role => "ADDOP");
  DefMathI!("\\triangleright",   None, "\u{25B7}", role => "ADDOP");
  DefMathI!("\\lhd",             None, "\u{22B2}", role => "ADDOP", meaning => "subgroup-of");
  DefMathI!("\\rhd",             None, "\u{22B3}", role => "ADDOP", meaning => "contains-as-subgroup");
  DefMathI!("\\unlhd", None, "\u{22B4}", role => "ADDOP", meaning => "subgroup-of-or-equals");
  DefMathI!("\\unrhd", None, "\u{22B5}", role => "ADDOP", meaning => "contains-as-subgroup-or-equals");

  DefMathI!("\\oplus",  None, "\u{2295}", role => "ADDOP", meaning => "direct-sum");
  DefMathI!("\\ominus", None, "\u{2296}", role => "ADDOP", meaning => "symmetric-difference");
  DefMathI!("\\otimes", None, "\u{2297}", role => "MULOP", meaning => "tensor-product");
  DefMathI!("\\oslash", None, "\u{2298}", role => "MULOP");
  DefMathI!("\\odot",   None, "\u{2299}", role => "MULOP", meaning => "direct-product");
  DefMathI!("\\bigcirc", None, "\u{25CB}", role => "MULOP");
  DefMathI!("\\dagger",  None, "\u{2020}", role => "MULOP");
  DefMathI!("\\ddagger", None, "\u{2021}", role => "MULOP");
  DefMathI!("\\amalg",   None, "\u{2210}", role => "MULOP", meaning => "coproduct");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.5. Relation Symbols, p.43
  //----------------------------------------------------------------------
  DefMathI!("\\leq",        None, "\u{2264}", role => "RELOP", meaning => "less-than-or-equals");
  DefMathI!("\\prec",       None, "\u{227A}", role => "RELOP", meaning => "precedes");
  DefMathI!("\\preceq",     None, "\u{2AAF}", role => "RELOP", meaning => "precedes-or-equals");
  DefMathI!("\\ll",         None, "\u{226A}", role => "RELOP", meaning => "much-less-than");
  DefMathI!("\\subset",     None, "\u{2282}", role => "RELOP", meaning => "subset-of");
  DefMathI!("\\subseteq",   None, "\u{2286}", role => "RELOP", meaning => "subset-of-or-equals");
  DefMathI!("\\sqsubset",   None, "\u{228F}", role => "RELOP", meaning => "square-image-of");
  DefMathI!("\\sqsubseteq", None, "\u{2291}", role => "RELOP", meaning => "square-image-of-or-equals");
  DefMathI!("\\in",         None, "\u{2208}", role => "RELOP", meaning => "element-of");
  DefMathI!("\\vdash", None, "\u{22A2}", role => "METARELOP", meaning => "proves");

  DefMathI!("\\geq",      None, "\u{2265}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMathI!("\\succ",     None, "\u{227B}", role => "RELOP", meaning => "succeeds");
  DefMathI!("\\succeq",   None, "\u{2AB0}", role => "RELOP", meaning => "succeeds-or-equals");
  DefMathI!("\\gg",       None, "\u{226B}", role => "RELOP", meaning => "much-greater-than");
  DefMathI!("\\supset",   None, "\u{2283}", role => "RELOP", meaning => "superset-of");
  DefMathI!("\\supseteq", None, "\u{2287}", role => "RELOP", meaning => "superset-of-or-equals");
  DefMathI!("\\sqsupset", None, "\u{2290}", role => "RELOP", meaning => "square-original-of");
  DefMathI!("\\sqsupseteq", None, "\u{2292}", role => "RELOP", meaning => "square-original-of-or-equals");
  DefMathI!("\\ni",    None, "\u{220B}", role => "RELOP",     meaning => "contains");
  DefMathI!("\\dashv", None, "\u{22A3}", role => "METARELOP", meaning => "does-not-prove");

  // I have the impression think that "identical" is a stronger notion than "equivalence"
  // Note that the unicode here is called "Identical To",
  // and that the notion of "equivalent to" usually involves the tilde operator.
  DefMathI!("\\equiv",  None, "\u{2261}", role => "RELOP", meaning => "equivalent-to");
  DefMathI!("\\sim",    None, "\u{223C}", role => "RELOP", meaning => "similar-to");
  DefMathI!("\\simeq",  None, "\u{2243}", role => "RELOP", meaning => "similar-to-or-equals");
  DefMathI!("\\asymp",  None, "\u{224D}", role => "RELOP", meaning => "asymptotically-equals");
  DefMathI!("\\approx", None, "\u{2248}", role => "RELOP", meaning => "approximately-equals");
  DefMathI!("\\cong",   None, "\u{2245}", role => "RELOP", meaning => "approximately-equals");
  DefMathI!("\\neq",    None, "\u{2260}", role => "RELOP", meaning => "not-equals");
  DefMathI!("\\doteq",  None, "\u{2250}", role => "RELOP", meaning => "approaches-limit");
  DefMathI!("\\notin",  None, "\u{2209}", role => "RELOP", meaning => "not-element-of");

  DefMathI!("\\models", None, "\u{22A7}", role => "RELOP", meaning => "models");
  DefMathI!("\\perp",   None, "\u{27C2}", role => "RELOP", meaning => "perpendicular-to");
  DefMathI!("\\mid", None, "\u{2223}", role => "VERTBAR"); // DIVIDES (RELOP?) ?? well, sometimes...
  DefMathI!("\\parallel", None, "\u{2225}", role => "VERTBAR", meaning => "parallel-to");
  DefMathI!("\\bowtie",   None, "\u{22C8}", role => "RELOP"); // BOWTIE
  DefMathI!("\\Join", None, "\u{2A1D}", role => "RELOP", meaning => "join");
  DefMathI!("\\smile",  None, "\u{2323}", role => "RELOP"); // SMILE
  DefMathI!("\\frown",  None, "\u{2322}", role => "RELOP"); // FROWN
  DefMathI!("\\propto", None, "\u{221D}", role => "RELOP", meaning => "proportional-to");

  // TeX defines these as alternate names...
  Let!("\\le", "\\leq");
  Let!("\\ge", "\\geq");
  Let!("\\ne", "\\neq");
  // And it defines some others as alternate names, but they seem to
  // potentially imply slightly different meanings???  Leave them out for now..

  //----------------------------------------------------------------------
  // Not;  (Is fullwidth solidus appropriate for when \not appears in isolation?)
  DefMathI!("\\not", None, "\u{FF0F}", role => "OPFUNCTION", meaning => "not");
});
