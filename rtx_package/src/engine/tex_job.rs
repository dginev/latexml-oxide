//! TeX Job
//! 
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // Dumping
  //----------------------------------------------------------------------
  // \dump             c  outputs a format file in INITEX; otherwise it is equivalent to \end.

  DefMacro!("\\dump", {
    Warn!("unexpected", "dump", "Do not know how to \\dump yet, sorry");
  });
});