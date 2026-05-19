//! chemmacros.sty — chemistry typesetting (expl3-based).
//!
//! chemmacros.sty is a large expl3 package providing \ch{}, \ox{},
//! \state{}, etc. Its raw-load triggers ~1000 cascading errors per
//! paper because our expl3 emulation can't fully resolve namespaced
//! CSes inside `\ProvidesExplPackage` bodies (chemmacros.sty L1240+
//! uses `\cs_new_protected:Npn`).
//!
//! Perl LaTeXML has no chemmacros binding and skips it via
//! INCLUDE_STYLES=false. Match that with a no-op binding. The user
//! paper's `\ch{...}` will end up as ERROR, but the cascade of 1000
//! collateral errors disappears.
//!
//! Witness 2407.06385, 2408.16742, 2408.16711 — and similar.
use crate::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Intentionally empty: chemmacros' expl3 chain doesn't survive
  // our raw-load. Skipping matches Perl LaTeXML's INCLUDE_STYLES=false
  // default behaviour.
  //
  // Minimal user-facing stubs so the paper renders something (rather
  // than just \ch undefined): expose \ch as gobble that emits its arg.
  DefMacro!("\\ch{}", "\\ensuremath{\\mathrm{#1}}");
  DefMacro!("\\Ch{}", "\\ensuremath{\\mathrm{#1}}");
  DefMacro!("\\state{}", "#1");
  def_macro_noop("\\transitionstatesymbol")?;
  def_macro_noop("\\changestate")?;
  def_macro_noop("\\setchemnum{}")?;
});
