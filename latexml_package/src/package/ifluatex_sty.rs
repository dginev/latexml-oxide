//! ifluatex.sty — LuaTeX detection (always false in LaTeXML)
//! Perl: ifluatex.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  DefConditional!("\\ifluatex");
});
