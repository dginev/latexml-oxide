use crate::prelude::*;

LoadDefinitions!({
  // Minimal stub for glossaries to prevent loading raw TeX
  DeclareOption!("acronyms", "");
  DeclareOption!("toc", "");
  DeclareOption!("section", "");
  DeclareOption!("numberedsection", "");
  DeclareOption!("nonumberlist", "");
  DeclareOption!("nopostdot", "");
  DeclareOption!("nomain", "");
  DeclareOption!("style", "");
  ProcessOptions!();

  DefMacro!("\\makenoidxglossaries", "");
  DefMacro!("\\makeglossaries", "");
  DefMacro!("\\newglossaryentry{}{}", "");
  DefMacro!("\\newacronym{}{}{}", "");
  DefMacro!("\\gls Semiverbatim", "#1");
  DefMacro!("\\Gls Semiverbatim", "#1");
  DefMacro!("\\glspl Semiverbatim", "#1");
  DefMacro!("\\Glspl Semiverbatim", "#1");
  DefMacro!("\\acrshort Semiverbatim", "#1");
  DefMacro!("\\acrlong Semiverbatim", "#1");
  DefMacro!("\\acrfull Semiverbatim", "#1");
  DefMacro!("\\glslink{}{}", "#2");
  DefMacro!("\\glsentrytext Semiverbatim", "#1");
  DefMacro!("\\glsentrylong Semiverbatim", "#1");
  DefMacro!("\\glsentryshort Semiverbatim", "#1");
  DefMacro!("\\printglossary OptionalKeyVals", "");
  DefMacro!("\\printglossaries", "");
  DefMacro!("\\printnoidxglossary OptionalKeyVals", "");
  DefMacro!("\\printnoidxglossaries", "");
  DefMacro!("\\newglossary OptionalMatch:* {}{}{}{}", "");
  DefMacro!("\\glsaddall OptionalKeyVals", "");
  DefMacro!("\\glsadd OptionalKeyVals Semiverbatim", "");
});
