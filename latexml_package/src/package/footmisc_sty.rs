use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  for option in &["perpage", "para", "side", "ragged", "symbol", "symbol*", "bottom",
    "marginal", "flushmargin", "hang", "norule", "splitrule",
    "split", "multiple"] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();

  // could define & use these, but...
  DefMacro!("\\DefineFNsymbols OptionalMatch:* {}{}", None);
  DefMacro!("\\setfnsymbol{}",                        None);

  DefMacro!("\\mpfootnotemark",    "\\footnotemark");
  DefMacro!("\\mpfootnoterule",    "\\footnoterule");
  DefMacro!("\\pagefootnoterule",  "\\footnoterule");
  DefMacro!("\\splitfootnoterule", "\\footnoterule");
  DefMacro!("\\footnotelayout",    "\\@empty");
  DefMacro!("\\footnotehint",      None);

  DefRegister!("\\footnotemargin"       => Dimension!("1.8em"));
  DefRegister!("\\footnotebaselineskip" => Glue!("12pt"));
});
