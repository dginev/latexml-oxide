use crate::package::*;
use rtx_core::state::State;

LoadDefinitions!(outer_stomach, state, {
  //**********************************************************************
  // Predefine, then load standard file.

  // TODO:
  // Predefine Ogonek, it's defined in t1enc.def as ugly ooalign
  //DefAccent('\k', "\x{0328}", "\x{02DB}");

  // Now read the rest from the REAL t1enc.
  InputDefinitions!("t1enc", extension => Some("sty"), noltxml => true);
});
