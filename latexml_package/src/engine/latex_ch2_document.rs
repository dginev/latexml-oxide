use crate::prelude::*;

LoadDefinitions!({
  //**********************************************************************
  // C.2. The Structure of the Document
  //**********************************************************************
  //   prepended files (using filecontents environment)
  //   preamble (starting with \documentclass)
  //   \begin{document}
  //    text
  //   \end{document}

  DefMacro!("\\AtBeginDocument{}", sub[(rules)] {
    state::push_value("@at@begin@document", rules)
  });
  DefMacro!("\\AtEndDocument{}", sub[(rules)] {
    state::push_value("@at@end@document", rules)
  });

  // Like  "<ltx:document xml:id='#id'>#body</ltx:document>",
  // But more complicated due to id, at begin/end document and so forth.
  // AND, lower-level so that we can cope with common errors at document end.
  DefConstructor!(T_CS!("\\begin{document}"), None, sub[document, _args, props] {
    let id = prop_str!(props,"id");
    // Already (auto) created?
    if let Some(mut docel) = document.findnode("/ltx:document", None) {
      if id != *EMPTY_SYM {
        arena::with(id, |id_str|
          document.set_attribute(&mut docel, "xml:id", id_str))?;
      }
    } else {
      let props = arena::with(id, |id_str| string_map!("xml:id" => id_str));
      document.open_element("ltx:document", Some(props), None)?;
    }
  },
  after_digest => sub[whatsit] {
    // Perl: beginMode('internal_vertical', 1) — noframe=1
    // Begin internal_vertical mode WITHOUT pushing a stack frame, keeping level=0
    begin_mode_opt("internal_vertical", true)?;
    // we need to re-bind in order to nest calls to the binding macro machinery
    DefMacro!("\\@currenvir", "document");
    state::assign_value("current_environment", "document", None);
    let expanded_id = Expand!(T_CS!("\\thedocument@ID"));
    whatsit.set_property("id", expanded_id);
    Let!("\\@nodocument", "\\relax", Scope::Global);
    // Clear \everypar at document start (Perl parity)
    state::assign_value("\\everypar", Tokens!(), Some(Scope::Global));
    let mut boxes = Vec::new();
    if let Some(ops) = state::lookup_tokens("@document@preamble@atend") {
      boxes.push(stomach::digest(ops)?);
    }
    if let Some(ops) = state::lookup_tokens("@at@begin@document") {
      boxes.push(stomach::digest(ops)?);
    }
    state::assign_value("inPreamble", false, None); // atbegin is still (sorta) preamble
    if let Some(ops) = state::lookup_tokens("@document@preamble@afterend") {
      boxes.push(stomach::digest(ops)?);
    }
    whatsit.set_font(lookup_font().unwrap()); // Start w/ whatever font was last selected.
    leave_horizontal_internal();
    boxes
  });

  // \document is used directly in e.g. expl3.sty
  Let!("\\document", "\\begin{document}", Scope::Global);

  DefConstructor!(T_CS!("\\end{document}"), None, sub[document,_args,_props] {
      document.close_element("ltx:document")?;
    },
    before_digest => {
      let mut boxes : Vec<Digested> = Vec::new();
      if let Some(ops) = state::lookup_tokens("@at@end@document") {
        boxes.push(stomach::digest(ops)?);
      }
      // Should we try to indent the last paragraph? If so, it goes like this:
      boxes.push(stomach::digest(T_CS!("\\lx@normal@par"))?);
      // Now we check whether we're down to the last stack frame.
      // It is common for unclosed { or even environments
      // and we want to at least compress & avoid unnecessary errors & warnings.
      let _nframes = get_frame_depth();
      //     my $ifstack;
      //     if ($state->isValueBound('current_environment', 0)
      //       && ($state->valueInFrame('current_environment', 0) eq 'document')
      //       && (!($ifstack = $state->lookupValue('if_stack')) || !$$ifstack[0])) { }    # OK!
      //     else {
      //       my @lines = ();
      //       while ((!$state->isValueBound('current_environment', 0)
      //           || ($state->valueInFrame('current_environment', 0) ne 'document'))
      //         && ($state->getFrameDepth > 0)) {
      //         # my $nonbox = $state->valueInFrame('groupNonBoxing',0) || 0;
      //         my $tok = $state->valueInFrame('groupInitiator',        0) || '<unknown>';
      //         my $loc = $state->valueInFrame('groupInitiatorLocator', 0);
      //         $loc = defined $loc ? ToString($loc) : '<unknown>';
      //         my $env = $state->isValueBound('current_environment', 0)
      //           && $state->valueInFrame('current_environment', 0);
      //         if ($env) {
      //           push(@lines, "Environment $env opened by " . ToString($tok) . ' ' . $loc); }
      //         else {    # but unclosed { is so common and latex itself doesn't Error!
  //           push(@lines, "Group opened by " . ToString($tok) . ' ' . $loc); }
  //         $state->popFrame; }
  //       while (($ifstack = $state->lookupValue('if_stack')) && $$ifstack[0]) {
  //         my $frame = $state->shiftValue('if_stack');
  //         push(@lines, "Conditional " . ToString($$frame{token})
  //             . "started " . ToString($$frame{start})); }
  //       Warn('unexpected', '\end{document}', $stomach,
  //         "Attempt to end document with open groups, environments or conditionals", @lines);
  //     }
      // Perl: endMode('internal_vertical', 1) — noframe=1
      // End mode without popping stack frame (executes beforeAfterGroup)
      end_mode_opt("internal_vertical", true)?;
      gullet::flush();
      boxes
  });

  // \enddocument is used directly in e.g. standalone.cls
  Let!("\\enddocument", "\\end{document}", Scope::Global);
});
