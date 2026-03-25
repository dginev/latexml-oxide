//! aecompl.sty — Almost European Computer Modern font completions
//! Perl: aecompl.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefPrimitive!("\\guilsinglleft",  "\u{2039}");
  DefPrimitive!("\\guilsinglright", "\u{203A}");
  DefPrimitive!("\\guillemotleft",  "\u{00AB}");
  DefPrimitive!("\\guillemotright", "\u{00BB}");
  DefPrimitive!("\\textperthousand", "\u{2030}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textpertenthousand", "\u{2031}",
    bounded => true, font => { encoding => "TS1" });
});
