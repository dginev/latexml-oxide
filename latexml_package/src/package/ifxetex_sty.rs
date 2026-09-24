//! ifxetex.sty — XeTeX detection (always false in LaTeXML)
//! Perl: ifxetex.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  // ifxetex.sty:4-5 is a legacy wrapper: `\RequirePackage{iftex}` provides
  // `\ifxetex` and `\RequireXeTeX`, which passes and installs XeTeX's
  // inter-character primitives for the requesting package (iftex_sty.rs,
  // OXIDIZED_DESIGN #220 update, batch 56hr).
  RequirePackage!("iftex");
});
