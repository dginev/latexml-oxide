//! newtxmath.sty — TX math fonts (delegates to other packages)
//! Perl: newtxmath.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("amsmath");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("iftex");
  RequirePackage!("xkeyval");
  RequirePackage!("amssymb");
  RequirePackage!("txfonts");
});
