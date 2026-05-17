use crate::prelude::*;
LoadDefinitions!({
  //**********************************************************************
  // Predefine, then load standard file.

  // Predefine Ogonek — t1enc.def defines it as ugly ooalign fallback
  DefAccent!("\\k", '\u{0328}', "\u{02DB}");

  // t1enc.def's chained load of t1enc.dfu calls `\DeclareUnicodeCharacter`,
  // which we ship in latex_constructs but a few papers manage to undefine
  // (via @onlypreamble cascade or kernel rollback) before t1enc.dfu runs.
  // Re-assert as a defensive no-op-with-mapping-side-effect to ensure the
  // .dfu's `\DeclareUnicodeCharacter{HEX}{expansion}` calls don't crash.
  // Witness 2509.22212 (+3 papers).
  RawTeX!(r"\providecommand\DeclareUnicodeCharacter[2]{}");

  // Now read the rest from the REAL t1enc.
  InputDefinitions!("t1enc", extension => Some("def".into()), noltxml => true);
});
