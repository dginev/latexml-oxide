//! Stub for translations.sty — i18n helper used by acro, fnpct, etc.
//!
//! The raw translations.sty has an \AtBeginDocument block that
//! \let's \@trnslt@loaded@languages based on whether babel or
//! polyglossia is loaded, falling back to \@trnslt@current@language.
//! Our AtBeginDocument execution order doesn't reliably wire this up,
//! so define defensive defaults before the raw load.
use crate::prelude::*;

LoadDefinitions!({
  // Defensive defaults — overridden by the raw load if it gets that far.
  DefMacro!("\\@trnslt@current@language", "english");
  DefMacro!("\\@trnslt@loaded@languages", "english");
  InputDefinitions!("translations", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
