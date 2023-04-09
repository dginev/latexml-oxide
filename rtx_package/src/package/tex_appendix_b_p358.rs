use crate::package::*;
// Match negations of many operators
// our %NOTS
static MATH_CHAR_NEGATIONS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
  map!("=" => "\u{2260}", "<" => "\u{226E}", ">" => "\u{226F}",
  "\u{2208}" => "\u{2209}",                              //\in=>\notin
  "\u{2264}" => "\u{2270}", "\u{2265}" => "\u{2271}",    // Less eq, greater eq.
  "\u{227A}" => "\u{2280}", "\u{227B}" => "\u{2281}",    // prec, succ
  "\u{2AAF}" => "\u{22E0}", "\u{2AB0}" => "\u{22E1}",    // preceq, succeq
  "\u{2282}" => "\u{2284}", "\u{2283}" => "\u{2285}",    // subset, supset
  "\u{2286}" => "\u{2288}", "\u{2287}" => "\u{2289}",    // subseteq, supseteq
  "\u{2291}" => "\u{22E2}", "\u{2290}" => "\u{22E3}",    // sqsubseteq, sqsupseteq
  "\u{2261}" => "\u{2262}",                              // equiv
  "\u{224D}" => "\u{226D}", "\u{2248}" => "\u{2249}",    // asymp, approx
  "\u{22B2}" => "\u{22EA}", "\u{22B3}" => "\u{22EB}",    // lhd, rhd
  "\u{22B4}" => "\u{22EC}", "\u{22B5}" => "\u{22ED}",    // unlhd, unrhd
  "\u{2203}" => "\u{2204}"                              // Exists
  )
});

//======================================================================
// TeX Book, Appendix B. p. 358
LoadDefinitions!(state, {
  //----------------------------------------------------------------------
  //  Actually from LaTeX; Table 3.3, Greek, p.41
  //----------------------------------------------------------------------
  DefMath!("\\alpha", None, "\u{03B1}");
  DefMath!("\\beta", None, "\u{03B2}");
  DefMath!("\\gamma", None, "\u{03B3}");
  DefMath!("\\delta", None, "\u{03B4}");
  DefMath!("\\epsilon", None, "\u{03F5}");
  DefMath!("\\varepsilon", None, "\u{03B5}");
  DefMath!("\\zeta", None, "\u{03B6}");
  DefMath!("\\eta", None, "\u{03B7}");
  DefMath!("\\theta", None, "\u{03B8}");
  DefMath!("\\vartheta", None, "\u{03D1}");
  DefMath!("\\iota", None, "\u{03B9}");
  DefMath!("\\kappa", None, "\u{03BA}");
  DefMath!("\\lambda", None, "\u{03BB}");
  DefMath!("\\mu", None, "\u{03BC}");
  DefMath!("\\nu", None, "\u{03BD}");
  DefMath!("\\xi", None, "\u{03BE}");
  DefMath!("\\pi", None, "\u{03C0}");
  DefMath!("\\varpi", None, "\u{03D6}");
  DefMath!("\\rho", None, "\u{03C1}");
  DefMath!("\\varrho", None, "\u{03F1}");
  DefMath!("\\sigma", None, "\u{03C3}");
  DefMath!("\\varsigma", None, "\u{03C2}");
  DefMath!("\\tau", None, "\u{03C4}");
  DefMath!("\\upsilon", None, "\u{03C5}");
  DefMath!("\\phi", None, "\u{03D5}");
  DefMath!("\\varphi", None, "\u{03C6}");
  DefMath!("\\chi", None, "\u{03C7}");
  DefMath!("\\psi", None, "\u{03C8}");
  DefMath!("\\omega", None, "\u{03C9}");
  DefMath!("\\Gamma", None, "\u{0393}");
  DefMath!("\\Delta", None, "\u{0394}");
  DefMath!("\\Theta", None, "\u{0398}");
  DefMath!("\\Lambda", None, "\u{039B}");
  DefMath!("\\Xi", None, "\u{039E}");
  DefMath!("\\Pi", None, "\u{03A0}");
  DefMath!("\\Sigma", None, "\u{03A3}");
  DefMath!("\\Upsilon", None, "\u{03A5}");
  DefMath!("\\Phi", None, "\u{03A6}");
  DefMath!("\\Psi", None, "\u{03A8}");
  DefMath!("\\Omega", None, "\u{03A9}");

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.7. Miscellaneous Symbols, p.43
  //----------------------------------------------------------------------
  // Some should be differential operators, qualifiers, ...
  DefMath!("\\aleph", None, "\u{2135}");
  DefMath!("\\hbar",  None, "\u{210F}", role => "ID", meaning => "Planck-constant-over-2-pi");
  DefMath!("\\imath", None, "\u{0131}");
  DefMath!("\\jmath", None, "\u{0237}");
  DefMath!("\\ell", None, "\u{2113}");
  DefMath!("\\wp", None, "\u{2118}", meaning => "Weierstrass-p");
  DefMath!("\\Re", None, "\u{211C}", role    => "OPFUNCTION", meaning => "real-part");
  DefMath!("\\Im", None, "\u{2111}", role    => "OPFUNCTION", meaning => "imaginary-part");
  DefMath!("\\mho", None, "\u{2127}");

  DefMath!("\\prime",    None, "\u{2032}", role => "SUPOP",    locked  => true);
  DefMath!("\\emptyset", None, "\u{2205}", role => "ID",       meaning => "empty-set");
  DefMath!("\\nabla",    None, "\u{2207}", role => "OPERATOR");
  DefMath!("\\surd",     None, "\u{221A}", role => "OPERATOR", meaning => "square-root");
  DefMath!("\\top",      None, "\u{22A4}", role => "ADDOP",    meaning => "top");
  DefMath!("\\bot",      None, "\u{22A5}", role => "ADDOP",    meaning => "bottom");
  DefMath!("\\|", None, "\u{2225}", role => "VERTBAR", name => "||");
  // should get meaning => 'parallel-to' when used as infix, but NOT when for OPEN|CLOSE
  DefMath!("\\angle", None, "\u{2220}");

  // NOTE: This is probably the wrong role.
  // Also, should probably carry info about Binding for OpenMath
  DefMath!("\\forall", None, "\u{2200}", role => "BIGOP",    meaning => "for-all");
  DefMath!("\\exists", None, "\u{2203}", role => "BIGOP",    meaning => "exists");
  DefMath!("\\neg",    None, "\u{00AC}",  role => "FUNCTION", meaning => "not");
  DefMath!("\\lnot",   None, "\u{00AC}",  role => "FUNCTION", meaning => "not");
  DefMath!("\\flat", None, "\u{266D}");
  DefMath!("\\natural", None, "\u{266E}");
  DefMath!("\\sharp", None, "\u{266F}");
  DefMath!("\\backslash", None, "\u{005C}", role => "MULOP");
  DefMath!("\\partial",   None, "\u{2202}", role => "OPERATOR", meaning => "partial-differential");

  DefMath!("\\infty", None, "\u{221E}", role => "ID", meaning => "infinity");
  DefMath!("\\Box", None, "\u{25A1}");
  DefMath!("\\Diamond", None, "\u{25C7}");
  DefMath!("\\triangle", None, "\u{25B3}");
  DefMath!("\\clubsuit", None, "\u{2663}");
  DefMath!("\\diamondsuit", None, "\u{2662}");
  DefMath!("\\heartsuit", None, "\u{2661}");
  DefMath!("\\spadesuit", None, "\u{2660}");

  //----------------------------------------------------------------------
  // TODO:
  // DefMath!("\\smallint", None, "\u{222B}", meaning => "integral", role => "INTOP",
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
  //     $node->removeAttribute('scriptpos'); });    # default is 'post', so we can just remove the
  // attrib. DefConstructor('\displaylimits', undef, sub {
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
  DefMath!("\\pm",    None, "\u{00B1}",  role => "ADDOP", meaning => "plus-or-minus");
  DefMath!("\\mp",    None, "\u{2213}", role => "ADDOP", meaning => "minus-or-plus");
  DefMath!("\\times", None, "\u{00D7}",  role => "MULOP", meaning => "times");
  DefMath!("\\div",   None, "\u{00F7}",  role => "MULOP", meaning => "divide");
  DefMath!("\\ast",   None, "\u{2217}", role => "MULOP");
  DefMath!("\\star",  None, "\u{22C6}", role => "MULOP");
  DefMath!("\\circ",  None, "\u{2218}", role => "MULOP", meaning => "compose");
  DefMath!("\\bullet", None, "\u{2219}", role => "MULOP");
  DefMath!("\\cdot",   None, "\u{22C5}", role => "MULOP");
  ////  , meaning=>"inner-product");  that"s pushing it a bit far...

  // Need to classify set operations more carefully....
  DefMath!("\\cap", None, "\u{2229}", role => "ADDOP", meaning => "intersection");
  DefMath!("\\cup", None, "\u{222A}", role => "ADDOP", meaning => "union");
  DefMath!("\\uplus",    None, "\u{228E}", role => "ADDOP");
  DefMath!("\\sqcap",    None, "\u{2293}", role => "ADDOP", meaning => "square-intersection");
  DefMath!("\\sqcup",    None, "\u{2294}", role => "ADDOP", meaning => "square-union");
  DefMath!("\\vee",      None, "\u{2228}", role => "ADDOP", meaning => "or");
  DefMath!("\\lor",      None, "\u{2228}", role => "ADDOP", meaning => "or");
  DefMath!("\\wedge",    None, "\u{2227}", role => "ADDOP", meaning => "and");
  DefMath!("\\land",     None, "\u{2227}", role => "ADDOP", meaning => "and");
  DefMath!("\\setminus", None, "\u{2216}", role => "ADDOP", meaning => "set-minus");
  DefMath!("\\wr",       None, "\u{2240}", role => "MULOP");

  // Should this block be ADDOP or something else?
  DefMath!("\\diamond",         None, "\u{22C4}", role => "ADDOP");
  DefMath!("\\bigtriangleup",   None, "\u{25B3}", role => "ADDOP");
  DefMath!("\\bigtriangledown", None, "\u{25BD}", role => "ADDOP");
  DefMath!("\\triangleleft",    None, "\u{25C1}", role => "ADDOP");
  DefMath!("\\triangleright",   None, "\u{25B7}", role => "ADDOP");
  DefMath!("\\lhd",           None, "\u{22B2}", role => "ADDOP", meaning => "subgroup-of");
  DefMath!("\\rhd",           None, "\u{22B3}", role => "ADDOP", meaning => "contains-as-subgroup");
  DefMath!("\\unlhd", None, "\u{22B4}", role => "ADDOP", meaning => "subgroup-of-or-equals");
  DefMath!("\\unrhd", None, "\u{22B5}", role => "ADDOP",
    meaning => "contains-as-subgroup-or-equals");

  DefMath!("\\oplus",  None, "\u{2295}", role => "ADDOP", meaning => "direct-sum");
  DefMath!("\\ominus", None, "\u{2296}", role => "ADDOP", meaning => "symmetric-difference");
  DefMath!("\\otimes", None, "\u{2297}", role => "MULOP", meaning => "tensor-product");
  DefMath!("\\oslash", None, "\u{2298}", role => "MULOP");
  DefMath!("\\odot",   None, "\u{2299}", role => "MULOP", meaning => "direct-product");
  DefMath!("\\bigcirc", None, "\u{25CB}", role => "MULOP");
  DefMath!("\\dagger",  None, "\u{2020}", role => "MULOP");
  DefMath!("\\ddagger", None, "\u{2021}", role => "MULOP");
  DefMath!("\\amalg",   None, "\u{2210}", role => "MULOP", meaning => "coproduct");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.5. Relation Symbols, p.43
  //----------------------------------------------------------------------
  DefMath!("\\leq",        None, "\u{2264}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\prec",       None, "\u{227A}", role => "RELOP", meaning => "precedes");
  DefMath!("\\preceq",     None, "\u{2AAF}", role => "RELOP", meaning => "precedes-or-equals");
  DefMath!("\\ll",         None, "\u{226A}", role => "RELOP", meaning => "much-less-than");
  DefMath!("\\subset",     None, "\u{2282}", role => "RELOP", meaning => "subset-of");
  DefMath!("\\subseteq",   None, "\u{2286}", role => "RELOP", meaning => "subset-of-or-equals");
  DefMath!("\\sqsubset",   None, "\u{228F}", role => "RELOP", meaning => "square-image-of");
  DefMath!("\\sqsubseteq", None, "\u{2291}", role => "RELOP",
    meaning => "square-image-of-or-equals");
  DefMath!("\\in",         None, "\u{2208}", role => "RELOP", meaning => "element-of");
  DefMath!("\\vdash", None, "\u{22A2}", role => "METARELOP", meaning => "proves");

  DefMath!("\\geq",      None, "\u{2265}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\succ",     None, "\u{227B}", role => "RELOP", meaning => "succeeds");
  DefMath!("\\succeq",   None, "\u{2AB0}", role => "RELOP", meaning => "succeeds-or-equals");
  DefMath!("\\gg",       None, "\u{226B}", role => "RELOP", meaning => "much-greater-than");
  DefMath!("\\supset",   None, "\u{2283}", role => "RELOP", meaning => "superset-of");
  DefMath!("\\supseteq", None, "\u{2287}", role => "RELOP", meaning => "superset-of-or-equals");
  DefMath!("\\sqsupset", None, "\u{2290}", role => "RELOP", meaning => "square-original-of");
  DefMath!("\\sqsupseteq", None, "\u{2292}", role => "RELOP",
    meaning => "square-original-of-or-equals");
  DefMath!("\\ni",    None, "\u{220B}", role => "RELOP",     meaning => "contains");
  DefMath!("\\dashv", None, "\u{22A3}", role => "METARELOP", meaning => "does-not-prove");

  // I have the impression think that "identical" is a stronger notion than "equivalence"
  // Note that the unicode here is called "Identical To",
  // and that the notion of "equivalent to" usually involves the tilde operator.
  DefMath!("\\equiv",  None, "\u{2261}", role => "RELOP", meaning => "equivalent-to");
  DefMath!("\\sim",    None, "\u{223C}", role => "RELOP", meaning => "similar-to");
  DefMath!("\\simeq",  None, "\u{2243}", role => "RELOP", meaning => "similar-to-or-equals");
  DefMath!("\\asymp",  None, "\u{224D}", role => "RELOP", meaning => "asymptotically-equals");
  DefMath!("\\approx", None, "\u{2248}", role => "RELOP", meaning => "approximately-equals");
  DefMath!("\\cong",   None, "\u{2245}", role => "RELOP", meaning => "approximately-equals");
  DefMath!("\\neq",    None, "\u{2260}", role => "RELOP", meaning => "not-equals");
  DefMath!("\\doteq",  None, "\u{2250}", role => "RELOP", meaning => "approaches-limit");
  DefMath!("\\notin",  None, "\u{2209}", role => "RELOP", meaning => "not-element-of");

  DefMath!("\\models", None, "\u{22A7}", role => "RELOP", meaning => "models");
  DefMath!("\\perp",   None, "\u{27C2}", role => "RELOP", meaning => "perpendicular-to");
  DefMath!("\\mid", None, "\u{2223}", role => "VERTBAR"); // DIVIDES (RELOP?) ?? well, sometimes...
  DefMath!("\\parallel", None, "\u{2225}", role => "VERTBAR", meaning => "parallel-to");
  DefMath!("\\bowtie",   None, "\u{22C8}", role => "RELOP"); // BOWTIE
  DefMath!("\\Join", None, "\u{2A1D}", role => "RELOP", meaning => "join");
  DefMath!("\\smile",  None, "\u{2323}", role => "RELOP"); // SMILE
  DefMath!("\\frown",  None, "\u{2322}", role => "RELOP"); // FROWN
  DefMath!("\\propto", None, "\u{221D}", role => "RELOP", meaning => "proportional-to");

  // TeX defines these as alternate names...
  Let!("\\le", "\\leq");
  Let!("\\ge", "\\geq");
  Let!("\\ne", "\\neq");
  // And it defines some others as alternate names, but they seem to
  // potentially imply slightly different meanings???  Leave them out for now..

  //----------------------------------------------------------------------
  // Not;  (Is fullwidth solidus appropriate for when \not appears in isolation?)
  DefMath!("\\not", None, "\u{FF0F}", role => "OPFUNCTION", meaning => "not");

  // For a \not operator that is followed by anything, concoct an appropriate not or cancelation.
  DefRewrite!(select =>
    "descendant-or-self::ltx:XMTok[text()='\u{FF0F}' and @meaning='not'][following-sibling::*]",
  select_count => 2,
  replace =>  sub[document, nodes, state] {
    // TODO: This argument low-level boilerplate is annoying
    // what is a good design pattern to "destructure" a Vec?
    // should it be another datastructure?
    let thing = nodes.pop().unwrap();
    let not_node = nodes.pop().unwrap();
    let text = match state.model.get_node_qname(thing) {
      "ltx:XMTok" => { thing.get_content() },
      _ => String::new()
    };
    if text.len() != 1 { // Not simple char token.
      // Wrap with a cancel op
      document.open_element("ltx:XMApp",
        Some(map!("_box" => not_node.to_hashable().to_string())), None, state)?;
      let mut strike = document.insert_math_token("",
        string_map!("role" => "ENCLOSE", "enclose" => "updiagonalstrike",
        "meaning" => "not", "_box" => not_node.to_hashable()), None, state)?;
      if let Some(id) = not_node.get_attribute_ns("id",XML_NS) {
        not_node.remove_attribute("xml:id")?;
        document.unrecord_id(&id);
        document.set_attribute(&mut strike, "xml:id", &id, state)?;
        document.get_node_mut().add_child(thing)?;
        document.close_element("ltx:XMApp", state)?;
      }
    } else {
      // For simple tokens, we'll modify the relevant content & attributes
      // [children removed, id's presumably ignorable]
      for mut child in thing.get_child_nodes() {
        child.unbind_node();
      }

      if let Some(meaning) = thing.get_attribute("meaning") {
        document.set_attribute(thing, "meaning",  &format!("not-{meaning}"), state)?; }
      if let Some(name) = thing.get_attribute("name") {
        document.set_attribute(thing, "name", &format!("not-{name}"), state)?; }
      else if !text.is_empty() {
        document.set_attribute(thing, "name", &format!("not-{text}"), state)?; }

      let known_c = MATH_CHAR_NEGATIONS.get(&text);
      let new : Cow<'_, str> = match known_c {
        Some(c) => Cow::Borrowed(c),
        None => Cow::Owned(text + "\u{0338}")
      };
      thing.append_text(&new)?;
      // and put the node back in
      document.get_node_mut().add_child(thing)?;
      // Since the <not> element is disappearing, if it had an id that was referenced...!?!?
      if let Some(id) = not_node.get_attribute_ns("id",XML_NS) {
        let idref_xpath = format!("descendant-or-self::ltx:XMRef[@idref='{id}']");
        for mut n in document.findnodes(&idref_xpath, None, state) {
          document.remove_node(&mut n);
        }
      }   // ? Hopefully this is safe.
    }
  });
});
