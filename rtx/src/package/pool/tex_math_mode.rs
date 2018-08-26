use package::*;
use rtx_core::document::tag::TagConstructionClosure;
use rtx_core::BoxOps;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //======================================================================
  // Math mode stuff
  // See TeXBook Ch.26
  //======================================================================
  // Decide whether we're going into or out of math, inline or display.
  Tag!("ltx:XMText", auto_open => true, auto_close => true);
  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => tagsub!(document, node, state, {
    generate_id(document, node, "m", state)?;
  }));

  DefPrimitiveII!(
    T_MATH!(),
    None,
    |stomach: &mut Stomach, tokens: Vec<Tokens>, state: &mut State| {
      let mut op = "\\@@BEGININLINEMATH";
      {
        let mut gullet = stomach.get_gullet_mut();
        let mode = LookupString!("MODE", state);
        debug!("T_MATH primitive current mode: {:?}", mode);
        if mode == "display_math" {
          if gullet.if_next(T_MATH!(), state)? {
            gullet.read_token(state);
            op = "\\@@ENDDISPLAYMATH";
          } else {
            // Avoid a Fatal, but we're likely in trouble.
            // Should we switch to text mode? (LaTeX normally wouldn't)
            // Did we miss something and would should have already been in text mode? Possibly...
            error!(target: "expected:$",
          "Missing $ closing display math.\nIgnoring; expect to be in wrong math/text mode.");
            op = ""
          }
        } else if mode == "inline_math" {
          op = "\\@@ENDINLINEMATH";
        } else if gullet.if_next(T_MATH!(), state)? {
          gullet.read_token(state);
          op = "\\@@BEGINDISPLAYMATH";
        }
      }
      if !op.is_empty() {
        // info!(target:"math_op:invoke_token","{:?}", op);
        Ok(stomach.invoke_token(T_CS!(op), state)?)
      } else {
        Ok(Vec::new())
      }
    },
    PrimitiveOptions::default()
  );
  // Let this be the default, conventional $
  LetI!(&T_CS!("\\@dollar@in@normalmode"), T_MATH!());

  // Effectively these are the math hooks, redefine these to do what you want with math?
  DefConstructor!("\\@@BEGINDISPLAYMATH",
  "<ltx:equation>
    <ltx:Math mode=\"display\">
    <ltx:XMath>
    #body
    </ltx:XMath>
    </ltx:Math>
  </ltx:equation>",
    alias         => Some(s!("$$")),
    before_digest => vec!(beforeproc!(stomach, state, { stomach.begin_mode("display_math", state)?; })),
    capture_body  => true
  );

  DefConstructorI!(T_CS!("\\@@ENDDISPLAYMATH"), None, None, alias => Some(s!("$$")),
    before_digest => vec!(beforeproc!(stomach, state, { stomach.end_mode("display_math", state)?; })));

  DefConstructor!("\\@@BEGININLINEMATH",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    alias => Some(s!("$")),
    before_digest => vec![beforeproc!(stomach, state, { 
      stomach.begin_mode("inline_math", state)?; })],
    capture_body => true);

  DefConstructorI!(T_CS!("\\@@ENDINLINEMATH"), None, None, alias => Some(s!("$")),
    before_digest => vec![beforeproc!(stomach, state, { stomach.end_mode("inline_math", state)?; })]);

  // Same as add_TeX, but add the code from the body of the object.
  let add_body_tex_closure: Vec<TagConstructionClosure> = tagsub!(document, node, state, {
    if node.get_attribute("tex").is_none() {
      // only do this once.

      let tex_opt = if let Some(ref tbox) = document.get_node_box(&node) {
        if let Some(body) = tbox.get_body() {
          Some(untex(&body, state))
        // local $LaTeXML::DUAL_BRANCH = 'presentation';
        // let tex = untex(body, state);
        // $LaTeXML::DUAL_BRANCH = 'content';
        // let ctex = untex(body, state);

        // if ctex != tex {
        //   document.set_attribute(node, "content-tex", ctex);
        // }
        } else {
          None
        }
      } else {
        None
      };
      if let Some(tex_string) = tex_opt {
        document.set_attribute(&mut node, "tex", &tex_string)?;
      }
    }
  });

  // Cleanup ltx:Math elements; particularly if they aren't "really" math.
  // But record the oddity with class=ltx_markedasmath
  // let cleanup_math_closure = Rc::new(|document: &mut Document, node: Node, box_opt:
  // Option<Digested>, state: &mut State| { // If the Math ONLY contains XMath/XMText, it
  // apparently isn't math at all!?! let mathy_nodes =
  // document.findnodes("ltx:XMath/ltx:*[local-name() != 'XMText']", node) if (!mathy_nodes.
  // is_empty()) {     // So unwrap down to the contents of the XMText's.
  // let xmtexts = node.get_child_nodes().into_iter().flat_map(|child|
  // child.get_child_nodes()).flat_map(|grandchild| grandchild.get_child_nodes()); let mut
  // texts = vec![];     for text in xmtexts {
  //       if text.get_type() != NodeType::Element {    // Make sure we've got an element
  //         text = document.wrap_nodes("ltx:text", text);
  //       }
  // document.add_class(text, "ltx_markedasmath");   // Now record that it originally was
  // marked as math       texts.push(text);
  //     }
  //     document.replace_node(node, texts); // and replace the whole Math with the pieces
  //   } else {                                                // Cleanup any remaining XMTexts
  //     cleanup_XMText_outer($document, $node);
  //   }
  //   return;
  // }

  Tag!("ltx:Math", after_close => add_body_tex_closure);
  // Tag!("ltx:Math", after_close => vec![cleanup_math_closure]);

  Ok(())
}
