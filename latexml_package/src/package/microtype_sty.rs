//! microtype.sty — microtypography (no-op in LaTeXML)
//! Perl: microtype.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("etoolbox");

  // All microtypography macros are no-ops
  DefMacro!("\\microtypesetup{}", None);
  DefMacro!("\\DeclareMicrotypeSet OptionalMatch:* {}{}", None);
  DefMacro!("\\DeclareMicrotypeSetDefault{}", None);
  DefMacro!("\\DeclareMicrotypeAlias{}{}", None);
  DefMacro!("\\SetProtrusion OptionalMatch:* {}{}", None);
  DefMacro!("\\SetTracking OptionalMatch:* {}{}", None);
  DefMacro!("\\SetExpansion OptionalMatch:* {}{}", None);
  DefMacro!("\\DisableLigatures OptionalMatch:* {}", None);
  DefMacro!("\\SetExtraKerning OptionalMatch:* {}{}", None);
  DefMacro!("\\SetExtraSpacing OptionalMatch:* {}{}", None);
  // \textls passes through #3 (the body)
  DefMacro!("\\textls OptionalMatch:* []{}", "#3");
  DefMacro!("\\lsstyle", None);
  DefMacro!("\\lslig{}", "#1");

  // Perl L32-46 — additional no-op microtype commands
  DefMacro!("\\UseMicrotypeSet[]{}", None);
  DefMacro!("\\DeclareCharacterInheritance[]{}{}", None);
  DefMacro!("\\DeclareMicrotypeVariants OptionalMatch:* {}", None);
  DefMacro!("\\LoadMicrotypeFile{}", None);
  DefMacro!("\\microtypecontext{}", None);
  DefEnvironment!("{microtypecontext}", "#body");
  DefMacro!("\\textmicrotypecontext{}{}", "#2");
  DefMacro!("\\DeclareMicrotypeBabelHook{}{}", None);
});
