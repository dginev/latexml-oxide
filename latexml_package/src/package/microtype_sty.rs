//! microtype.sty — microtypography (no-op in LaTeXML)
//! Perl: microtype.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("etoolbox");

  // All microtypography macros are no-ops.
  // Perl uses `[]` optional args on the `Set*`/`DisableLigatures` family
  // (context selectors like `[font name]`); Rust was using `OptionalMatch:*`
  // which consumes a leading `*` not an optional bracket group. Fix to
  // match Perl signatures so user-side `\SetProtrusion[ctx]{set}{list}`
  // parses correctly instead of stranding the `[ctx]` as literal text.
  DefMacro!("\\microtypesetup{}", None);
  DefMacro!("\\DeclareMicrotypeSet OptionalMatch:* []{}{}", None);
  DefMacro!("\\DeclareMicrotypeSetDefault[]{}", None);
  DefMacro!("\\DeclareMicrotypeAlias{}{}", None);
  DefMacro!("\\SetProtrusion[]{}{}", None);
  DefMacro!("\\SetTracking[]{}{}", None);
  DefMacro!("\\SetExpansion[]{}{}", None);
  DefMacro!("\\DisableLigatures[]{}", None);
  DefMacro!("\\SetExtraKerning[]{}{}", None);
  DefMacro!("\\SetExtraSpacing[]{}{}", None);
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
