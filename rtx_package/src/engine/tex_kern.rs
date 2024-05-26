//! TeX Kern
//! 
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  DefRegister!("\\lastkern" => Dimension::new(0), readonly => true);
});