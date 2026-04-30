use crate::prelude::*;
LoadDefinitions!({
  //**********************************************************************
  // Predefine, then load standard file.

  // Perl t1enc.sty.ltxml L8: predefine Ogonek before loading raw t1enc.def,
  // whose own definition is an ugly ooalign that LaTeXML can't render.
  // U+0328 is COMBINING OGONEK; U+02DB is OGONEK (spacing form). Matches
  // textcomp.sty.ltxml DefAccent entries for other combining accents.
  DefAccent!("\\k", '\u{0328}', "\u{02DB}");

  // Now read the rest from the REAL t1enc.
  InputDefinitions!("t1enc", extension => Some("sty".into()), noltxml => true);
});
