//! ifxetex.sty — XeTeX detection (always false in LaTeXML)
//! Perl: ifxetex.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  DefConditional!("\\ifxetex");
  DefMacro!("\\RequireXeTeX", None);
});
