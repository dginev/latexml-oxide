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
  // footmisc.sty:496-512 `\DefineFNsymbolsTM*{<name>}{<text/math pairs>}`, the text-or-math form of
  // `\DefineFNsymbols` (a named footnote-symbol set for `\setfnsymbol`), read as that one is
  // (run 340: 127 first errors, witnesses 1904.04281, 1904.11899, 1107.5789).
  DefMacro!("\\DefineFNsymbolsTM OptionalMatch:* {}{}", None);

  DefMacro!("\\mpfootnotemark",    "\\footnotemark");
  DefMacro!("\\mpfootnoterule",    "\\footnoterule");
  DefMacro!("\\pagefootnoterule",  "\\footnoterule");
  DefMacro!("\\splitfootnoterule", "\\footnoterule");
  DefMacro!("\\footnotelayout",    "\\@empty");
  DefMacro!("\\footnotehint",      None);

  DefRegister!("\\footnotemargin"       => Dimension!("1.8em"));
  DefRegister!("\\footnotebaselineskip" => Glue!("12pt"));
});
