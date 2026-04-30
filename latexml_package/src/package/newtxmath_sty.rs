//! newtxmath.sty — TX math fonts (delegates to other packages)
//! Perl: newtxmath.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  // Perl newtxmath.sty.ltxml L22-31: explicitly require ifxetex+ifluatex
  // (both set identical ifcond false stubs in Rust). Rust previously
  // consolidated to `iftex`, which is a superset — same conditionals
  // available — but drifts from Perl parity. Match the Perl chain exactly.
  RequirePackage!("amsmath");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("ifxetex");
  RequirePackage!("ifluatex");
  RequirePackage!("xkeyval");
  RequirePackage!("amssymb");
  RequirePackage!("txfonts");
});
