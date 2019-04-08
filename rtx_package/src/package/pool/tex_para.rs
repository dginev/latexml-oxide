use crate::package::*;

LoadDefinitions!(state, {
  //----------------------------------------------------------------------
  // These determine whether the _next_ paragraph gets indented!
  // thus it needs \par to check whether such indentation has been set.
  DefPrimitive!("\\indent", {
    AssignValue!("next_para_class" => "ltx_indent");
  });
  DefPrimitive!("\\noindent", {
    AssignValue!("next_para_class" => "ltx_noindent");
  });

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
    after_digest => sub[stomach, whatsit, state] {
      let in_preamble = LookupBool!("inPreamble");
      if in_preamble {
        whatsit.set_property("inPreamble", true);
      } else if let Some(c) = RemoveValue!("next_para_class") {
          whatsit.set_property("class", c);
          // TODO
        // Digest!(Tokens!(
        //     T_CS("\\LTX@vadjust@afterpar"),
        //     T_CS("\\LTX@clear@vadjust@afterpar")
        // ));
      }
      Ok(Vec::new())
    },
    properties => skippable_props,
    alias => "\\par"
  );
  Let!("\\par", "\\normal@par");

  // OTOH, sometimes \par is just a minimalistic "start a new line"
  // This should be closer for those cases.
  DefConstructor!("\\inner@par", sub[document, args, props, state] {
    Debug!("inner@par invoked!\n");
    if document.maybe_close_element("ltx:p", state)?.is_some() {
    } else if document.can_contain(document.get_node(), "ltx:break", state) {
      document.insert_element("ltx:break", Vec::new(), None, state)?;
    }
  });

  Tag!("ltx:para", auto_close => true, auto_open => true);
  Tag!("ltx:p", auto_close => true, auto_open => true,
    after_close => sub[document, node, state] {
      document.trim_node_whitespace(node)?;
  });

  // \dump ???
  DefPrimitive!("\\end", sub[stomach, args, state] { stomach.get_gullet_mut().flush(state); });
});
