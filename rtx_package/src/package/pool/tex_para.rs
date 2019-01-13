use crate::package::*;

LoadDefinitions!(state, {
  //----------------------------------------------------------------------
  // These determine whether the _next_ paragraph gets indented!
  // thus it needs \par to check whether such indentation has been set.
  // DefPrimitiveI!("\indent",   None, |state| AssignValue(next_para_class => 'ltx_indent'); });
  // DefPrimitiveI!("\noindent", None, || AssignValue(next_para_class => 'ltx_noindent'); });

  // <ltx:para> represents a Logical Paragraph, whereas <ltx:p> is a `physical paragraph'.
  // A para can contain both p and displayed equations and such.

  // Remember; \par _closes_, not opens, paragraphs!
  // Here, we want to close both an open p and para (if either are open).
  let mut skippable_props: HashMap<String, Stored> = HashMap::new();
  skippable_props.insert(s!("alignmentSkippable"), true.into());

  DefConstructor!("\\normal@par",
    sub[document, args, props, state] {
      let in_preamble = prop_bool!(props, "inPreamble");
      if !in_preamble {
        document.maybe_close_element("ltx:p", state)?;
        let class_str = prop_str!(props,"class");
        if !class_str.is_empty() {
          let element = document.get_element();
          if let Some(mut node) = element {
            if document.get_node_qname(&node, state) == "ltx:para" {  // Only set on the para about to close!
              document.set_attribute(&mut node, "class", &class_str)?;
            }
          }
        }
        document.maybe_close_element("ltx:para", state)?;
      }
    },
    after_digest => aftersub!(stomach, whatsit, state, {
      let in_preamble = state.lookup_bool("inPreamble");
      if in_preamble {
        whatsit.set_property("inPreamble", true);
      } else if let Some(c) = state.remove_value("next_para_class") {
          whatsit.set_property("class", c);
          // TODO
        // Digest!(Tokens!(
        //     T_CS("\\LTX@vadjust@afterpar"),
        //     T_CS("\\LTX@clear@vadjust@afterpar")
        // ));
      }
      Ok(Vec::new())
    }),
    properties => properties!(skippable_props),
    alias => Some(s!("\\par"))
  );
  Let!("\\par", "\\normal@par");

  // OTOH, sometimes \par is just a minimalistic "start a new line"
  // This should be closer for those cases.
  DefConstructor!("\\inner@par", sub[document, args, props, state] {
    debug!("inner@par invoked!\n");
    if document.maybe_close_element("ltx:p", state)?.is_some() {
    } else if document.can_contain(document.get_node(), "ltx:break", state) {
      document.insert_element("ltx:break", Vec::new(), None, state)?;
    }
  });

  //======================================================================
  // Macros
  // See Chapter 24, p.275-276
  // <macro assignment> = <definition> | <prefix><macro assignment>
  // <definition> = <def><control sequence><definition text>
  // <def> = \def | \gdef | \edef | \xdef
  // <definition text> = <register text><left brace><balanced text><right brace>

  fn parse_def_parameters(cs: &Token, params_in: Tokens) -> Option<Parameters> {
    // TODO !!!
    // let mut tokens = params_in.unlist();
    // // Now, recognize parameters and delimiters.
    // let mut params = ();
    // let mut n      = 0;
    // while !tokens.is_empty() {
    //   let t = tokens.pop_front();
    //   if ($t->getCatcode == CC_PARAM) {
    //     if (!@tokens) {    // Special case: lone # NOT following a numbered parameter
    //                       // Note that we require a { to appear next, but do NOT read it!
    //       push(@params, LaTeXML::Core::Parameter->new('RequireBrace', 'RequireBrace')); }
    //     else {
    //       $n++; $t = shift(@tokens);
    //       Fatal('expected', "#$n", $STATE->getStomach,
    //         "Parameters for '" . ToString($cs) . "' not in order in " . ToString($params))
    //         unless (defined $t) && ($n == (ord($t->getString) - ord('0')));
    //       // Check for delimiting text following the parameter #n
    //       my @delim = ();
    //       my ($pc, $cc) = (-1, 0);
    //       while (@tokens && (($cc = $tokens[0]->getCatcode) != CC_PARAM)) {
    //         let d = shift(@tokens);
    //         push(@delim, $d) unless $cc == $pc && $cc == CC_SPACE;    # BUT collapse whitespace!
    //         $pc = $cc; }
    //       // Found text that marks the end of the parameter
    //       if (@delim) {
    //         let expected = Tokens(@delim);
    //         push(@params, LaTeXML::Core::Parameter->new('Until',
    //             'Until:' . ToString($expected),
    //             extra => [$expected])); }
    //       // Special case: trailing sole # => delimited by next opening brace.
    //       elsif ((scalar(@tokens) == 1) && ($tokens[0]->getCatcode == CC_PARAM)) {
    //         shift(@tokens);
    //         push(@params, LaTeXML::Core::Parameter->new('UntilBrace', 'UntilBrace')); }
    //       // Nothing? Just a plain parameter.
    //       else {
    //         push(@params, LaTeXML::Core::Parameter->new('Plain', '{}')); } } }
    //   else {
    //     // Initial delimiting text is required.
    //     my @lit = ($t);
    //     while (@tokens && ($tokens[0]->getCatcode != CC_PARAM)) {
    //       push(@lit, shift(@tokens)); }
    //     let expected = Tokens(@lit);
    //     push(@params, LaTeXML::Core::Parameter->new('Match',
    //         'Match:' . ToString($expected),
    //         extra   => [$expected],
    //         novalue => 1)); }
    // }
    // return (@params ? LaTeXML::Core::Parameters->new(@params) : undef);
    None
  }

  fn do_def(globally: bool, expanded: bool, stomach: &mut Stomach, args: Vec<Tokens>, state: &mut State) -> Result<Vec<Digested>> {
    unpack!(args => cs, params, body);
    let cs: Token = cs.into();
    let paramlist = parse_def_parameters(&cs, params);
    if expanded {
      state.noexpand_the = true;
      let gullet = stomach.get_gullet_mut();
      body = Expand!(body, gullet, state);
    }
    let scope = if globally { Some(Scope::Global) } else { None };
    info!(target:"\\def","defining cs: {:?}, params {:?}, as {:?}", cs, paramlist, body);
    state.install_definition(
      Expandable {
        cs,
        paramlist,
        expansion: SimpleExpansion!(body.clone()),
        ..Expandable::default()
      },
      scope,
    );
    AfterAssignment!(state);
    Ok(Vec::new())
  }

  DefPrimitiveI!("\\def SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, false, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\gdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, false, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\edef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, true, stomach, args, state)
    },
      locked => true
  );
  DefPrimitiveI!("\\xdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, true, stomach, args, state)
    },
    locked => true
  );

  Tag!("ltx:para", auto_close => true, auto_open => true);
  use rtx_core::document::tag::TagConstructionClosure;
  let trim_node_whitespace_closure: Vec<TagConstructionClosure> = tagsub!(document, node, state, {
    document.trim_node_whitespace(node)?;
  });
  Tag!("ltx:p", auto_close => true, auto_open => true, after_close => trim_node_whitespace_closure);

  // \dump ???
  DefPrimitive!("\\end", sub[stomach, args, state] { stomach.get_gullet_mut().flush(state); Ok(vec![]) });

  // TODO: Move to the right place in the pool definitions (maybe split out individual sub-pools by
  // chapter?)
  DefMacroI!(T_CS!("\\space"), None, T_SPACE!());
});
