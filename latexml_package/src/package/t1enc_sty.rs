use crate::prelude::*;
LoadDefinitions!({
  //**********************************************************************
  // Predefine, then load standard file.

  // TODO:
  // Predefine Ogonek, it's defined in t1enc.def as ugly ooalign
  //DefAccent('\k', "\x{0328}", "\x{02DB}");

  // Now read the rest from the REAL t1enc.
  InputDefinitions!("t1enc", extension => Some("sty".into()), noltxml => true);
});
