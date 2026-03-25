use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lineno.sty.ltxml — stub (line numbering not meaningful for XML)
  DefEnvironment!("{linenumbers*}[Number]",         "#body");
  DefEnvironment!("{runninglinenumbers*}[Number]",  "#body");
  DefEnvironment!("{pagewiselinenumbers*}[Number]", "#body");
  DefEnvironment!("{linenomath}",                   "#body");
  DefEnvironment!("{linenomath*}",                  "#body");

  DefMacro!("\\linenumbers OptionalMatch:* [Number]",        None);
  DefMacro!("\\nolinenumbers",                               None);
  DefMacro!("\\runninglinenumbers OptionalMatch:* [Number]", None);
  DefMacro!("\\pagewiselinenumbers",                         None);
  DefMacro!("\\realpagewiselinenumbers",                     None);
  DefMacro!("\\runningpagewiselinenumbers",                  None);

  DefMacro!("\\leftlinenumbers  OptionalMatch:*",  None);
  DefMacro!("\\rightlinenumbers OptionalMatch:*",  None);
  DefMacro!("\\switchlinenumbers OptionalMatch:*", None);

  DefMacro!("\\setrunninglinenumbers",  None);
  DefMacro!("\\setpagewiselinenumbers", None);

  DefMacro!("\\resetlinenumber [Number]",   None);
  DefMacro!("\\modulolinenumbers [Number]", None);

  DefMacro!("\\linenumberfont", None);
  DefRegister!("\\linenumbersep", Number(0));

  DefMacro!("\\thelinenumber", None);

  DefMacro!("\\makeLineNumber",        None);
  DefMacro!("\\makeLineNumberRunning", None);
  DefMacro!("\\makeLineNumberOdd",     None);
  DefMacro!("\\makeLineNumberEven",    None);
  DefMacro!("\\makeLineNumberRight",   None);
  DefMacro!("\\makeLineNumberLeft",    None);
  DefMacro!("\\LineNumber",            None);

  DefMacro!("\\numquote",        "\\quote");
  DefMacro!("\\endnumquote",     "\\endquote");
  DefMacro!("\\numquotation",    "\\quote");
  DefMacro!("\\endnumquotation", "\\endquote");

  DefMacro!("\\quotelinenumberfont", None);
  DefRegister!("\\quotelinenumbersep", Number(0));
});
