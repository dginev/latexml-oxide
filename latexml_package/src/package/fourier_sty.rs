//! fourier.sty — Utopia text/math fonts (presentational). fourier.sty:61
//! loads fourier-orns (`\lefthand`, `\decoone` … pgfornament's tikzrput
//! manual); the symbol set is what documents reach.
use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("fourier-orns");
});
