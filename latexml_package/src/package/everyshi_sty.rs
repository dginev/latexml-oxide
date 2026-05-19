//! everyshi.sty — hooks that fire on every \shipout.
//!
//! Real everyshi.sty exposes `\EveryShipout{<hook>}` to add a callback
//! and `\@EveryShipout@Init` as an init helper. LaTeXML doesn't model
//! page shipout — we render straight to XML — so all hooks are no-ops.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Hook installer — accept and discard the argument.
  def_macro_noop("\\EveryShipout{}")?;
  // Init helper called once at package load by some classes.
  def_macro_noop("\\@EveryShipout@Init")?;
  // Internal token list register.
  DefRegister!("\\@EveryShipout@Hook" => Tokens!());
});
