use crate::prelude::*;

//**********************************************************************
// C.6 Displayed Paragraphs
//**********************************************************************

/// Perl: setupAligningContext — saves [node, lastChild] for deferred class application.
fn setup_aligning_context(doc: &mut Document) {
  if let Some(node) = doc.get_element() {
    // Save node and its current last child so we only apply to NEW children later
    state::assign_value("ALIGNING_NODE", Stored::Node(node.clone()), None);
    if let Some(last) = node.get_last_child() {
      state::assign_value("ALIGNING_PREV_CHILD", Stored::Node(last), None);
    } else {
      state::assign_value("ALIGNING_PREV_CHILD", Stored::None, None);
    }
  }
}

/// Perl: applyAligningContext — applies align/class to children added AFTER \centering.
fn apply_aligning_context(document: &mut Document, align: &str, class: &str) -> Result<()> {
  let node_opt = state::lookup_value("ALIGNING_NODE");
  if let Some(Stored::Node(node)) = node_opt {
    let previous_opt = match state::lookup_value("ALIGNING_PREV_CHILD") {
      Some(Stored::Node(prev)) => Some(prev),
      _ => None,
    };
    let children = node.get_child_nodes();
    let mut past_previous = previous_opt.is_none(); // if no previous, apply to all
    for mut child in children {
      if !past_previous {
        if let Some(ref prev) = previous_opt {
          if child == *prev {
            past_previous = true;
          }
        }
        continue;
      }
      if child.get_type() == Some(libxml::tree::NodeType::ElementNode) {
        crate::engine::base_functions::set_align_or_class(document, &mut child, align, class)?;
      }
    }
  }
  Ok(())
}

LoadDefinitions!({
  DefEnvironment!("{center}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    // aligning will take care of \\\\ "rows"
    aligning_environment("center", "ltx_centering", document, props)?;
    Ok(())
  });
  // HOWEVER, define a plain \center to act like \centering (?)
  DefMacro!("\\center", "\\centering");
  DefMacro!("\\endcenter", None);
  DefEnvironment!("{flushleft}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    aligning_environment("center", "ltx_align_left", document, props)?;
    Ok(())
  });
  DefEnvironment!("{flushright}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    aligning_environment("center", "ltx_align_right", document, props)?;
    Ok(())
  });

  // # These add an operation to be carried out on the current node & following siblings, when the
  // current group ends. # These operators will add alignment (class) attributes to each "line" in
  // the current block. #DefPrimitiveI('\centering',   undef, sub {
  // UnshiftValue(beforeAfterGroup=>T_CS('\@add@centering')); }); # NOTE: THere's a problem here.
  // The current method seems to work right for these operators # appearing within the typical
  // environments.  HOWEVER, it doesn't work for a simple \bgroup or \begingroup!!! # (they don't
  // create a node! or even a whatsit!)
  // Perl: setupAligningContext saves [node, node.lastChild] to ALIGNING_NODE.
  // applyAligningContext then only applies class to children AFTER the saved lastChild.
  DefConstructor!("\\centering", sub[doc,_args] {
    setup_aligning_context(doc);
  },
  before_digest => {
    unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@centering")]);
  });
  // Perl: latex_constructs.pool.ltxml lines 1299-1302
  DefConstructor!("\\raggedright", sub[doc,_args] {
    setup_aligning_context(doc);
  },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedright")]);
    });
  DefConstructor!("\\raggedleft", sub[doc,_args] {
    setup_aligning_context(doc);
  },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedleft")]);
    });

  DefConstructor!("\\@add@centering", sub[document] {
    apply_aligning_context(document, "center", "ltx_centering")?;
  });
  // Note that \raggedright is essentially align left (undef align, just class)
  DefConstructor!("\\@add@raggedright", sub[document] {
    apply_aligning_context(document, "", "ltx_align_left")?;
  });
  DefConstructor!("\\@add@raggedleft", sub[document] {
    apply_aligning_context(document, "", "ltx_align_right")?;
  });
  DefConstructor!("\\@add@flushright", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "right", "ltx_align_right")?;
      }
    }
  });
  DefConstructor!("\\@add@flushleft", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "left", "ltx_align_left")?;
      }
    }
  });
});
