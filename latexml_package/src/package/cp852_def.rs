use crate::prelude::*;
LoadDefinitions!({
  // Perl: cp852.def.ltxml — predefine Ogonek before loading raw cp852.def
  // (the raw file defines it as an ugly ooalign fallback).
  DefAccent!("\\k", '\u{0328}', "\u{02DB}");
  InputDefinitions!("cp852", extension => Some("def".into()), noltxml => true);
});
