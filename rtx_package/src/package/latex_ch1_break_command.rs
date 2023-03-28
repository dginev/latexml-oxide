use crate::package::*;

LoadDefinitions!(state, {
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
  DefConstructor!("\\\\ OptionalMatch:* [Glue]",
    "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()),
    // properties => is_break => true
  );

  DefConstructor!("\\newline", "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion  => Tokens!(T_CS!("\\newline"), T_CR!()),
    properties => sub[_whatsit,_args,_state] { Ok(stored_map!("isBreak" => Stored::Bool(true) )) }
  );

  Let!(&T_CS!("\\@normalcr"), T_CS!("\\\\"));
  // NOTE: Activating this binding messes up an \afterassign test,
  //       so it may be best left disabled.
  // PushValue!("TEXT_MODE_BINDINGS" => Tokens!(T_CS!("\\\\"), T_CS!("\\@normalcr")));

  DefMacro!("\\@nolnerr", "");
  DefMacro!(
    "\\@centercr",
    r"\ifhmode\unskip\else\@nolnerr\fi\par\@ifstar{\nobreak\@xcentercr}\@xcentercr"
  );
  DefMacro!("\\@xcentercr", r"\addvspace{-\parskip}\@ifnextchar[\@icentercr\ignorespaces");
  DefMacro!("\\@icentercr[]", "\\vskip #1\\ignorespaces");
});
