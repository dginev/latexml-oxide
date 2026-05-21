//! Stub for catoptions.sty — expl3-style option/key-value framework.
//!
//! catoptions is a low-level helper used by xwatermark and a few other
//! "Ahmed Musa" packages. Raw-loading it triggers a `\@latex@error`
//! for `\special_relax already defined` (our gullet pre-registers
//! \special_relax for `\dont_expand`-smuggling). Stub it instead.
//!
//! Witness: 2110.08425, 2111.00068, 2111.04135, etc. (catoptions cascade)
use latexml_package::prelude::*;

LoadDefinitions!({
  // No-op the public API surface most commonly referenced
  def_macro_noop("\\cptnewlength{}")?;
  def_macro_noop("\\cptsetlength{}{}")?;
  def_macro_noop("\\cptaddtolength{}{}")?;
  // catoptions options
  DeclareOption!(None, {});
  ProcessOptions!();
});
