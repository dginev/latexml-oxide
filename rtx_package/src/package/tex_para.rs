use crate::package::*;
use rtx_core::document::helpers::prune_empty_para;

LoadDefinitions!(state, {
  //----------------------------------------------------------------------
  // These determine whether the _next_ paragraph gets indented!
  // thus it needs \par to check whether such indentation has been set.
  DefConstructor!("\\indent", sub[document,whatsit,state] {
    if let Some(mut node) = document.get_element() {
      let tag = document.get_node_qname(&node,state);
      let para_tag = arena::pin_static("ltx:para");
      if tag == para_tag {
        node.set_attribute("class","ltx_indent")?;
      } else if document.sym_can_contain_somehow(tag,para_tag,state) {
        // Used in a position where a paragraph can be started, start
        document.open_element("ltx:para", Some(string_map!("class"=>"ltx_indent")), None, state)?;
      }
      // Otherwise ignore.
    }
  });
  DefConstructor!("\\noindent", sub[document,whatsit,state] {
    if let Some(mut node) = document.get_element() {
      let tag = document.get_node_qname(&node,state);
      let para_tag = arena::pin_static("ltx:para");
      if tag == para_tag {
        node.set_attribute("class","ltx_noindent")?;
      } else if document.sym_can_contain_somehow(tag, para_tag ,state) {
        // Used in a position where a paragraph can be started, start
        document.open_element("ltx:para", Some(string_map!("class"=>"ltx_noindent")), None, state)?;
      }
      // Otherwise ignore.
    }
  });

  // <ltx:para> represents a Logical Paragraph, whereas <ltx:p> is a `physical paragraph'.
  // A para can contain both p and displayed equations and such.

  // Remember; \par _closes_, not opens, paragraphs!
  // Here, we want to close both an open p and para (if either are open).
  let mut skippable_props: HashMap<String, Stored> = HashMap::default();
  skippable_props.insert(s!("alignmentSkippable"), true.into());

  DefConstructor!("\\normal@par",
    sub[document, _args, props, state] {
      if !prop_bool!(props, "inPreamble") {
        document.maybe_close_element("ltx:p", state)?;
        let element = document.get_element();
        if let Some(mut node) = element {
          let qname = document.get_node_qname(&node, state);
          // Only set on the para about to close, if unknown!
          if qname == arena::pin_static("ltx:para") && node.get_attribute("class").is_none() {
            let class_sym = prop_str!(props,"class");
            arena::with(class_sym, |class_str|
              document.set_attribute(&mut node, "class", class_str))?;
          } else if qname == arena::pin_static("ltx:figure") {
            // insert breaks in figures, for vertically separating subfigures
            document.insert_element("ltx:break",Vec::new(), None, state)?;
          }
        }
        document.maybe_close_element("ltx:para", state)?;
      }
    },
    after_digest => sub[stomach, whatsit, state] {
      let in_preamble = LookupBool!("inPreamble");
      if in_preamble {
        whatsit.set_property("inPreamble", true);
        Ok(Vec::new())
      } else {
        if let Some(c) = state.lookup_value("next_para_class") {
          // Check if flags were set by prior \par:
          whatsit.set_property("class", c.clone());
          state.assign_value("next_para_class", Stored::None, None);
        }
        // Fish out flags for next ltx:para, to be used when the next \par closes:
        if state.lookup_register("\\parindent",Vec::new())?.unwrap().value_of() == 0 {
          // respect \parindent if no overrides are given
          state.assign_value("next_para_class", "ltx_noindent", None);
        }
        // Vertical adjustments
        if let Some(Stored::Tokens(vadj)) = RemoveValue!("vAdjust") {
          AssignValue!("vAdjust", Tokens!(), Some(Scope::Global));
          Ok(vec![ Digest!(vadj)? ])
        } else {
          Ok(Vec::new())
        }
      }
    },
    properties => skippable_props,
    alias => "\\par"
  );
  Let!("\\par", "\\normal@par");

  // OTOH, sometimes \par is just a minimalistic "start a new line"
  // This should be closer for those cases.
  DefConstructor!("\\inner@par", sub[document, _args, _props, state] {
    Debug!("inner@par invoked!\n");
    if document.maybe_close_element("ltx:p", state)?.is_some() {
    } else if document.can_contain(document.get_node(), "ltx:break", state) {
      document.insert_element("ltx:break", Vec::new(), None, state)?;
    }
  });

  Tag!("ltx:para", auto_close => true, auto_open => true,
  after_close => sub[document, node, state] {
    prune_empty_para(document, node, state)?;
  });
  Tag!("ltx:p", auto_close => true, auto_open => true,
    after_close => sub[document, node, _state] {
      document.trim_node_whitespace(node)?;
  });

  // \dump ???
  DefPrimitive!("\\end", sub[stomach, (), state] { stomach.get_gullet_mut().flush(state); });
});
