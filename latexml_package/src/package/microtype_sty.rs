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
  def_macro_noop("\\microtypesetup{}")?;
  def_macro_noop("\\DeclareMicrotypeSet OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareMicrotypeSetDefault[]{}")?;
  def_macro_noop("\\DeclareMicrotypeAlias{}{}")?;
  def_macro_noop("\\SetProtrusion[]{}{}")?;
  def_macro_noop("\\SetTracking[]{}{}")?;
  def_macro_noop("\\SetExpansion[]{}{}")?;
  def_macro_noop("\\DisableLigatures[]{}")?;
  def_macro_noop("\\SetExtraKerning[]{}{}")?;
  def_macro_noop("\\SetExtraSpacing[]{}{}")?;
  // \textls passes through #3 (the body)
  DefMacro!("\\textls OptionalMatch:* []{}", "#3");
  def_macro_noop("\\lsstyle")?;
  DefMacro!("\\lslig{}", "#1");

  // Perl L32-46 — additional no-op microtype commands
  def_macro_noop("\\UseMicrotypeSet[]{}")?;
  def_macro_noop("\\DeclareCharacterInheritance[]{}{}")?;
  def_macro_noop("\\DeclareMicrotypeVariants OptionalMatch:* {}")?;
  def_macro_noop("\\LoadMicrotypeFile{}")?;
  // microtype provides BOTH `\microtypecontext{settings}` (a scoped
  // settings *declaration* — no body, applies to the rest of the current
  // group) AND a `{microtypecontext}` environment. Our `DefEnvironment`
  // defines the bare `\microtypecontext` CS as the env-begin, which
  // clobbers the declaration form — so a bare
  // `\begingroup\microtypecontext{expansion=sloppy}…\endgroup` (common
  // around `\bibliography`) treated `\microtypecontext` as an unclosed
  // env-begin, opening a restricted_horizontal mode-switch group that
  // the later `\endgroup` couldn't close (`unexpected:\endgroup`).
  // Define the environment FIRST, then the no-op declaration macro, so
  // `\microtypecontext{…}` resolves to the harmless declaration while
  // `\begin{microtypecontext}` still finds the environment (env lookup
  // is independent of the `\microtypecontext` CS). Witness 2007.06927.
  DefEnvironment!("{microtypecontext}", "#body");
  def_macro_noop("\\microtypecontext{}")?;
  DefMacro!("\\textmicrotypecontext{}{}", "#2");
  def_macro_noop("\\DeclareMicrotypeBabelHook{}{}")?;
});
