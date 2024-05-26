//! TeX Glue
//! 
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
LoadDefinitions!({
  //======================================================================
  // Lastskip
  //----------------------------------------------------------------------
  // \lastskip         iq is 0.0 pt or the last glue or muglue on the current list.

  DefRegister!("\\lastskip", Glue::new(0), readonly => true);
});