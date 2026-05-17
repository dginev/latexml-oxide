use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lineno.sty.ltxml — stub (line numbering not meaningful for XML)
  DefEnvironment!("{linenumbers*}[Number]",         "#body");
  DefEnvironment!("{runninglinenumbers*}[Number]",  "#body");
  DefEnvironment!("{pagewiselinenumbers*}[Number]", "#body");
  DefEnvironment!("{linenomath}",                   "#body");
  DefEnvironment!("{linenomath*}",                  "#body");
  // \internallinenumbers (lineno.sty) — adds line numbers inside the
  // environment body. Also gets a starred form auto-defined via
  // `\@namedef{internallinenumbers*}{\internallinenumbers*}` at lineno
  // L?? (cf. iclr2025_conference.sty L230 which calls it). Stub as
  // body-passthrough since line numbers are irrelevant in XML output.
  // Witness 52 papers with iclr2025_conference using this env.
  DefEnvironment!("{internallinenumbers}[Number]",  "#body");
  DefEnvironment!("{internallinenumbers*}[Number]", "#body");

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
