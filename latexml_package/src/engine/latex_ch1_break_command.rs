use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // C.1.4 Declarations
  //======================================================================
  // actual implementation later.
  //======================================================================
  // C.1.5 Invisible Commands
  //======================================================================
  // actual implementation later.

  //======================================================================
  // C.1.6 The \\ Command
  //======================================================================
  // In math, \\ is just a formatting hint, unless within an array, cases, .. environment.
  // Perl: DefConstructor('\lx@newline OptionalMatch:* [Glue]', sub { ... });
  // Complex constructor that checks document context:
  //   - in math: insert <ltx:XMHint name='newline'/>
  //   - no context or _CaptureBlock_: skip
  //   - ltx:p with parent _CaptureBlock_: maybeCloseElement('ltx:p')
  //   - can contain ltx:break: insert <ltx:break/>
  DefConstructor!("\\lx@newline OptionalMatch:* [Glue]", sub[document] {
    if lookup_bool("IN_MATH") {
      document.insert_element("ltx:XMHint", Vec::new(), Some(map!("name" => s!("newline"))))?;
    } else {
      if let Some(context) = document.get_element() {
        let tag = document::get_node_qname(&context);
        let capture_block = arena::pin_static("ltx:_CaptureBlock_");
        if tag == capture_block {
          // skip, if in insertBlock
        } else if tag == arena::pin_static("ltx:p") {
          // Close <p> if parent is _CaptureBlock_
          if let Some(parent) = context.get_parent() {
            if document::get_node_qname(&parent) == capture_block {
              document.maybe_close_element("ltx:p")?;
            } else if document::can_contain(&context, "ltx:break") {
              document.insert_element("ltx:break", Vec::new(), None)?;
            }
          }
        } else if document::can_contain(&context, "ltx:break") {
          document.insert_element("ltx:break", Vec::new(), None)?;
        }
      }
      // else: no context => skip
    }
  },
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()),
    properties => { stored_map!("isBreak" => true) },
  );
  Let!("\\\\", "\\lx@newline");

  DefConstructor!("\\newline", "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion  => Tokens!(T_CS!("\\newline"), T_CR!()),
    properties => { Ok(stored_map!("isBreak" => true)) },
  );

  Let!("\\@normalcr", "\\\\");
  Let!("\\@normalnewline", "\\newline");
  // NOTE: Activating this binding messes up an \afterassign test,
  //       so it may be best left disabled.
  // PushValue!("TEXT_MODE_BINDINGS" => Tokens!(T_CS!("\\\\"), T_CS!("\\@normalcr")));

  DefMacro!("\\@nolnerr", "");
  DefMacro!(
    "\\@centercr",
    r"\ifhmode\unskip\else\@nolnerr\fi\par\@ifstar{\nobreak\@xcentercr}\@xcentercr"
  );
  DefMacro!(
    "\\@xcentercr",
    r"\addvspace{-\parskip}\@ifnextchar[\@icentercr\ignorespaces"
  );
  DefMacro!("\\@icentercr[]", "\\vskip #1\\ignorespaces");
});
