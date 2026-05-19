//! microtype.sty — microtypography (no-op in LaTeXML)
//! Perl: microtype.sty.ltxml
use crate::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

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
  def_macro_noop("\\microtypecontext{}")?;
  DefEnvironment!("{microtypecontext}", "#body");
  DefMacro!("\\textmicrotypecontext{}{}", "#2");
  def_macro_noop("\\DeclareMicrotypeBabelHook{}{}")?;
});
