use crate::package::*;

LoadDefinitions!(state, {
  //**********************************************************************
  // C.2. The Structure of the Document
  //**********************************************************************
  //   prepended files (using filecontents environment)
  //   preamble (starting with \documentclass)
  //   \begin{document}
  //    text
  //   \end{document}

  DefMacro!("\\AtBeginDocument{}", sub[gullet,(rules),state] {
    state.push_value("@at@begin@document", rules.unlist());
  });
  DefMacro!("\\AtEndDocument{}", sub[gullet,(rules),state] {
    state.push_value("@at@end@document", rules.unlist());
  });

  // Like  "<ltx:document xml:id='#id'>#body</ltx:document>",
  // But more complicated due to id, at begin/end document and so forth.
  // AND, lower-level so that we can cope with common errors at document end.
  DefConstructor!(T_CS!("\\begin{document}"), None, sub[document, args, props, state] {
    let id = prop_str!(props,"id");
    if let Some(mut docel) = document.findnode("/ltx:document", None, state) { // Already (auto) created?
      if !id.is_empty() {
        document.set_attribute(&mut docel, "xml:id", id)?;
      }
    } else {
      document.open_element("ltx:document", Some(string_map!("xml:id" => id)), None, state)?;
    }
  },
  after_digest => sub[stomach, whatsit, state] {
    stomach.begin_mode("text", state)?;
    { // we need to re-bind in order to nest calls to the binding macro machinery
      bind_state_mut!(stomach,state);
      DefMacro!("\\@currenvir", "document");
    }
    let mut gullet = stomach.get_gullet_mut();
    state.assign_value("current_environment", "document", None);
    let expanded_id = Expand!(T_CS!("\\thedocument@ID"),gullet,state);
    whatsit.set_property("id", expanded_id);
    let mut boxes = Vec::new();
    if let Some(ops) = state.lookup_value("@document@preamble@atend") {
      unimplemented!();
      //       push(@boxes, $stomach->digest(Tokens(@$ops)));
    }
    if let Some(ops) = state.lookup_value("@at@begin@document") {
      //       push(@boxes, $stomach->digest(Tokens(@$ops)));
      unimplemented!();
    }
    state.assign_value("inPreamble", false, None); // atbegin is still (sorta) preamble
    if let Some(ops) = state.lookup_value("@document@preamble@afterend") {
      unimplemented!();
    //       push(@boxes, $stomach->digest(Tokens(@$ops)));
    }
    whatsit.set_font(state.lookup_font().unwrap()); // Start w/ whatever font was last selected.
    boxes
  });

  // \document is used directly in e.g. expl3.sty
  Let!(&T_CS!("\\document"), T_CS!("\\begin{document}"), Some(Scope::Global));

  DefConstructor!(T_CS!("\\end{document}"), None, sub[document,args,props,state] {
      document.close_element("ltx:document", state)?;
    },
    before_digest => sub[stomach,state] {
      let mut boxes : Vec<Digested> = Vec::new();
      if let Some(ops) = state.lookup_tokens("@at@end@document") {
        boxes.push(stomach.digest(ops,state)?);
      }
      // Should we try to indent the last paragraph? If so, it goes like this:
      // boxes.push(stomach.digest(T_CS!("\\normal@par"), state)?);
      // Now we check whether we're down to the last stack frame.
      // It is common for unclosed { or even environments
      // and we want to at least compress & avoid unnecessary errors & warnings.
      let nframes = state.get_frame_depth();
      //     my $ifstack;
      //     if ($STATE->isValueBound('current_environment', 0)
      //       && ($STATE->valueInFrame('current_environment', 0) eq 'document')
      //       && (!($ifstack = $STATE->lookupValue('if_stack')) || !$$ifstack[0])) { }    # OK!
      //     else {
      //       my @lines = ();
      //       while ((!$STATE->isValueBound('current_environment', 0)
      //           || ($STATE->valueInFrame('current_environment', 0) ne 'document'))
      //         && ($STATE->getFrameDepth > 0)) {
      //         # my $nonbox = $STATE->valueInFrame('groupNonBoxing',0) || 0;
      //         my $tok = $STATE->valueInFrame('groupInitiator',        0) || '<unknown>';
      //         my $loc = $STATE->valueInFrame('groupInitiatorLocator', 0);
      //         $loc = defined $loc ? ToString($loc) : '<unknown>';
      //         my $env = $STATE->isValueBound('current_environment', 0)
      //           && $STATE->valueInFrame('current_environment', 0);
      //         if ($env) {
      //           push(@lines, "Environment $env opened by " . ToString($tok) . ' ' . $loc); }
      //         else {    # but unclosed { is so common and latex itself doesn't Error!
  //           push(@lines, "Group opened by " . ToString($tok) . ' ' . $loc); }
  //         $STATE->popFrame; }
  //       while (($ifstack = $STATE->lookupValue('if_stack')) && $$ifstack[0]) {
  //         my $frame = $STATE->shiftValue('if_stack');
  //         push(@lines, "Conditional " . ToString($$frame{token})
  //             . "started " . ToString($$frame{start})); }
  //       Warn('unexpected', '\end{document}', $stomach,
  //         "Attempt to end document with open groups, environments or conditionals", @lines);
  //     }
      stomach.end_mode("text", state)?;
      stomach.get_gullet_mut().flush(state);
      boxes
  });

  // \enddocument is used directly in e.g. standalone.cls
  Let!(&T_CS!("\\enddocument"), T_CS!("\\end{document}"), Some(Scope::Global));
});
