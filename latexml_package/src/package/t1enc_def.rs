use crate::prelude::*;
LoadDefinitions!({
  //**********************************************************************
  // Predefine, then load standard file.

  // Predefine Ogonek — t1enc.def defines it as ugly ooalign fallback
  DefAccent!("\\k", '\u{0328}', "\u{02DB}");

  // Now read the rest from the REAL t1enc.
  InputDefinitions!("t1enc", extension => Some("def".into()), noltxml => true);
});
