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
  reversion => Some(Tokens!(T_CS!("\\\\"), T_CR!()).into()));

  LetI!(&T_CS!("\\@normalcr"), T_CS!("\\\\"));
  PushValue!("TEXT_MODE_BINDINGS" => Tokens!(T_CS!("\\\\"), T_CS!("\\@normalcr")));

  DefMacro!("\\@nolnerr", "");
  DefMacro!(
    "\\@centercr",
    "\\ifhmode\\unskip\\else\\@nolnerr\\fi\\par\\@ifstar{\\nobreak\\@xcentercr}\\@xcentercr"
  );
  DefMacro!("\\@xcentercr", "\\addvspace{-\\parskip}\\@ifnextchar[\\@icentercr\\ignorespaces");
  DefMacro!("\\@icentercr[]", "\\vskip #1\\ignorespaces");
});
