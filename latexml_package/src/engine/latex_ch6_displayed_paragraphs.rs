use crate::prelude::*;

//**********************************************************************
// C.6 Displayed Paragraphs
//**********************************************************************

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
  DefConstructor!("\\centering", sub[doc,_args] {
  state::assign_value("ALIGNING_NODE", doc.get_element().unwrap(), None); },
  before_digest => {
    unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@centering")]);
  });
  // Perl: latex_constructs.pool.ltxml lines 1299-1302
  DefConstructor!("\\raggedright", sub[doc,_args] {
    state::assign_value("ALIGNING_NODE", doc.get_element().unwrap(), None); },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedright")]);
    });
  DefConstructor!("\\raggedleft", sub[doc,_args] {
    state::assign_value("ALIGNING_NODE", doc.get_element().unwrap(), None); },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedleft")]);
    });

  DefConstructor!("\\@add@centering", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "center", "ltx_centering")?;
      }
    }
  });
  // Note that \raggedright is essentially align left (undef align, just class)
  DefConstructor!("\\@add@raggedright", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "", "ltx_align_left")?;
      }
    }
  });
  DefConstructor!("\\@add@raggedleft", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "", "ltx_align_right")?;
      }
    }
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
