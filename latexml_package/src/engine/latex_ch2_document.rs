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

  // Perl: PushValue('@at@begin@document', $_[1]->unlist)
  // Note: in modern LaTeX with expl3, \AtBeginDocument is redefined to use
  // the L3 hook system (\AddToHook{begindocument}{...}). Our definition here
  // serves as a fallback when expl3 isn't loaded. When expl3 IS loaded, it
  // overrides this with its own version that routes through \hook_gput_code:nnn.
  // Perl 93f875a6: support optional [label] from modern LaTeX hooks system
  DefMacro!("\\AtBeginDocument[]{}", sub[(_label, rules)] {
    state::push_value("@at@begin@document", rules)
  });
  DefMacro!("\\AtEndDocument[]{}", sub[(_label, rules)] {
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
        let id_s = arena::with(id, |s| s.to_string());
        document.set_attribute(&mut docel, "xml:id", &id_s)?;
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
    // Fire the L3 hook system for begindocument.
    // Modern LaTeX (with expl3) uses \hook_use:n{begindocument} instead of
    // \@begindocumenthook. This fires hooks registered via \AtBeginDocument
    // when expl3 has redefined it to use \AddToHook{begindocument}{...}.
    // Includes babel's \lx@babel@activate@mainlang.
    if lookup_definition(&T_CS!("\\hook_use:n"))?.is_some() {
      boxes.push(stomach::digest(Tokenize!(r"\hook_use:n{begindocument}"))?);
    }
    // Fire babel language activation AFTER all hooks (including babel's own
    // \selectlanguage call). This runs even if babel's hook code has errors.
    // Use T_CS! directly since @ is OTHER catcode at \begin{document} time.
    if lookup_definition(&T_CS!("\\lx@babel@activate@mainlang"))?.is_some() {
      boxes.push(stomach::digest(Tokens!(T_CS!("\\lx@babel@activate@mainlang")))?);
    }
    state::assign_value("inPreamble", false, None); // atbegin is still (sorta) preamble
    if let Some(ops) = state::lookup_tokens("@document@preamble@afterend") {
      boxes.push(stomach::digest(ops)?);
    }
    // Safety net: ensure _ has standard catcode at document start.
    // Packages using expl3 internally (mhchem, siunitx, etc.) may leave _ as LETTER
    // if their \ExplSyntaxOff was group-local. Restore _ to SUB globally.
    // Note: we do NOT restore ':' here because French babel makes it ACTIVE for
    // proper spacing before French punctuation (;:!?).
    if state::lookup_catcode('_') != Some(Catcode::SUB) {
      state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
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
      // Perl: pop extra frames until we reach the document frame.
      // Common for unclosed { or environments — compress & avoid cascading errors.
      {
        let mut safety = 200;
        while safety > 0 && get_frame_depth() > 1 {
          let env = state::lookup_string("current_environment");
          if env == "document" { break; }
          let _ = state::pop_frame();
          safety -= 1;
        }
      }
      // Perl: endMode('internal_vertical', 1) — noframe=1
      // End mode without popping stack frame (executes beforeAfterGroup)
      end_mode_opt("internal_vertical", true)?;
      gullet::flush();
      boxes
  });

  // \enddocument is used directly in e.g. standalone.cls
  Let!("\\enddocument", "\\end{document}", Scope::Global);
});
