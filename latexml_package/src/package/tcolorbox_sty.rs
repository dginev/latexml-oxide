//! tcolorbox.sty — colored and framed text boxes
//! Perl: tcolorbox.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // used in tcbbreakable.code.tex assuming it was defined
  DefRegister!("\\doublecol@number" => Number::new(0));
  // Ensure only unbreakable mode is possible.
  // Perl: locked => 1 prevents raw TeX tcbbreakable.code.tex from overriding
  // with the real breakable implementation (uses output routines → infinite loop).
  DefMacro!("\\tcb@init@breakable", "\\tcb@init@unbreakable", locked => true);

  // Perl 93f875a6: pre-define \tcb@use@autoparskip before raw TeX loading,
  // as pgfkeys initialization may not complete and the \AtBeginDocument hook
  // at tcolorbox.sty:1142 would call it undefined.
  DefMacro!("\\tcb@use@autoparskip", "\\relax");

  RequirePackage!("expl3");
  RequirePackage!("xparse");

  InputDefinitions!("tcolorbox", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Suppress tcolorbox's library version check. The Rust kpathsea binding
  // may resolve a different tcolorbox.sty version than the library files,
  // causing spurious "tcolorbox is not installed correctly" errors.
  // Make the check a no-op — the versions are always compatible in practice.
  DefMacro!("\\tcb@check@library@version", "", locked => true);
});
