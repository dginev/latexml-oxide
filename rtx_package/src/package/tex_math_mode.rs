use crate::package::*;

LoadDefinitions!({
  //======================================================================
  // Math mode stuff
  // See TeXBook Ch.26
  //======================================================================
  // Decide whether we're going into or out of math, inline or display.
  Tag!("ltx:XMText", auto_open => true, auto_close => true);
  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => sub[document, node] {
    document.generate_id(node, "m")?;
  });

  DefPrimitive!(
    T_MATH!(),
    None, {
      let mut op = "\\@@BEGININLINEMATH";
      {
        let mode = state::lookup_string("MODE");
        Debug!("T_MATH primitive current mode: {:?}", mode);
        if mode == "display_math" {
          if gullet::if_next(T_MATH!())? {
            gullet::read_token()?;
            op = "\\@@ENDDISPLAYMATH";
          } else {
            // Avoid a Fatal, but we're likely in trouble.
            // Should we switch to text mode? (LaTeX normally wouldn't)
            // Did we miss something and would should have already been in text mode? Possibly...
            Error!(
              "expected",
              "$",
              "Missing $ closing display math.\nIgnoring; expect to be in wrong math/text mode."
            );
            op = "";
          }
        } else if mode == "inline_math" {
          op = "\\@@ENDINLINEMATH";
        } else if gullet::if_next(T_MATH!())? {
          gullet::read_token()?;
          op = "\\@@BEGINDISPLAYMATH";
        }
      }
      if !op.is_empty() {
        Ok(stomach::invoke_token(&T_CS!(op))?)
      } else {
        Ok(Vec::new())
      }
    }
  );
  // Let this be the default, conventional $
  Let!(T_CS!("\\@dollar@in@normalmode"), T_MATH!());

  // Effectively these are the math hooks
  DefConstructor!("\\@@BEGINDISPLAYMATH",
  "<ltx:equation>\
    <ltx:Math mode=\"display\">\
    <ltx:XMath>\
    #body\
    </ltx:XMath>\
    </ltx:Math>\
  </ltx:equation>",
    reversion         => Tokens!(T_MATH!(),T_MATH!()),
    before_digest => {
      begin_mode("display_math")?;
      // TODO:
      // if let Some(everymath_toks) = lookup_definition(T_CS!("\\everymath")).value_of().unlist() {
      //   gullet::unread(everymath_toks);
      // }
      // if let Some(everydisplay_toks) = lookup_definition(T_CS!("\\everydisplay")).value_of().unlist() {
      //   gullet::unread(everydisplay_toks);
      // }
    },
    capture_body  => true
  );

  DefConstructor!(T_CS!("\\@@ENDDISPLAYMATH"), None, None,
    reversion => Tokens!(T_MATH!(),T_MATH!()),
    before_digest => { end_mode("display_math")?; });

  DefConstructor!("\\@@BEGININLINEMATH",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    reversion    => Tokens!(T_MATH!()),
    before_digest => {
      begin_mode("inline_math")?;
      if let Some(RegisterValue::Tokens(everymath_toks)) = state::lookup_register("\\everymath", Vec::new())? {
        let everymath_toks = everymath_toks.unlist();
        if !everymath_toks.is_empty() {
          gullet::unread(Tokens::new(everymath_toks));
        }
      }
    },
    capture_body => true);

  DefConstructor!(T_CS!("\\@@ENDINLINEMATH"), None, None,
    before_digest => { end_mode("inline_math")?; },
    reversion    => Tokens!(T_MATH!())
  );

  // Same as add_TeX, but add the code from the body of the object.
  Tag!("ltx:Math", after_close => sub[document, node] {
    if !node.has_attribute("tex") {
      // only do this once.

      let tex_opt = if let Some(ref tbox) = document.get_node_box(node) {
        if let Some(body) = tbox.get_body()? {
          set_dual_branch("presentation");
          let tex = body.untex()?;
          expire_dual_branch();
          set_dual_branch("content");
          let ctex = body.untex()?;
          expire_dual_branch();
          if ctex != tex {
            document.set_attribute(node, "content-tex", &ctex)?;
          }
          Some(tex)
        } else {
          None
        }
      } else {
        None
      };
      if let Some(tex_string) = tex_opt {
        document.set_attribute(node, "tex", &tex_string)?;
      }
    }
  });

  Tag!("ltx:Math", after_close => sub[document, node] {
    cleanup_math(document, node.clone())?;
  });
});
