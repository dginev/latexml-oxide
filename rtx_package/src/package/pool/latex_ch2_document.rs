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

  // DefMacro('\AtBeginDocument{}', sub {
  //     AssignValue('@at@begin@document', []) unless LookupValue('@at@begin@document');
  //     PushValue('@at@begin@document', $_[1]->unlist); });
  // DefMacro('\AtEndDocument{}', sub {
  //     AssignValue('@at@end@document', []) unless LookupValue('@at@end@document');
  //     PushValue('@at@end@document', $_[1]->unlist); });

  DefEnvironment!("{document}", sub[document, args, props, state] {
      let id = prop_str!(props,"id");
      let body = prop_whatsit!(props,"body");
      if let Some(mut docel) = document.findnode("/ltx:document", None, state) { // Already (auto) created?
        if !id.is_empty() {
          document.set_attribute(&mut docel, "xml:id", id)?;
        }
        document.absorb(body, state)?;
      } else {
        let attrib = string_map!("xml:id" => id);
        document.insert_element("ltx:document", vec![body], Some(attrib), state)?;
      }
      Ok(())
    },
    before_digest => { AssignValue!("inPreamble", false); },
    // after_digest_begin => vec![|stomach, whatsit, state| {
    //   whatsit.set_property("id", Expand!(T_CS!("\\thedocument@ID"), state));
    //   if let Some(ops) = LookupValue!("@at@begin@document", state) {
    //     let boxes = Digest!(ops, stomach);
    //     whatsit.set_font(LookupValue!("font")); // Start w/ whatever font was selected.
    //     return boxes
    //   } else {
    //     return Vec::new()
    //   }
    // }],
    before_digest_end => sub[stomach, inner_state] {
      stomach.get_gullet_mut().flush(inner_state);
      if let Some(Stored::Tokens(ops)) = RemoveValue!("@at@end@document") {
        Ok(vec![stomach.digest(ops, inner_state)?]) // TODO: Can we improve to the regular Digest!(ops) syntax?
      } else {
        Ok(Vec::new())
      }
    },
    mode => "text"
  );
});
