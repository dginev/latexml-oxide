//! ifxetex.sty — XeTeX detection (always false in LaTeXML)
//! Perl: ifxetex.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  // ifxetex.sty:4-5 is a legacy wrapper: `\RequirePackage{iftex}` provides
  // `\ifxetex` and the halting `\RequireXeTeX` (the former no-op here let
  // XeTeX-only packages load; batch 56ak).
  RequirePackage!("iftex");
});
